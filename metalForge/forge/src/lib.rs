// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! metalForge — heterogeneous compute dispatch for healthSpring.
//!
//! Routes workloads to CPU, GPU, or NPU based on runtime capability discovery.
//! Organizes hardware via NUCLEUS atomics (Tower → Node → Nest) and plans
//! inter-device transfers (`PCIe` P2P DMA, host-staged, network IPC).
//!
//! ## Architecture
//!
//! ```text
//! biomeOS graph (DAG of pipeline stages)
//!     │
//!     ▼
//! metalForge dispatch ── selects substrate per stage
//!     │
//!     ▼
//! NUCLEUS topology ── Tower → Node → Nest
//!     │
//!     ├── `PCIe` P2P DMA (GPU↔NPU, bypass CPU)
//!     ├── Host-staged (CPU mediates)
//!     └── Network IPC (cross-node via biomeOS)
//! ```

pub mod nucleus;
pub mod transfer;

/// Available compute substrates for healthSpring workloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Substrate {
    /// Pure Rust CPU — always available.
    Cpu,
    /// barraCuda GPU via WGSL shaders — requires `gpu` feature.
    Gpu,
    /// Neuromorphic NPU (Akida AKD1000) — requires `npu` feature.
    Npu,
}

/// Workload descriptor for capability-based dispatch.
///
/// Primals describe their workload semantics; metalForge maps them to
/// the best available substrate without the caller knowing what hardware
/// exists. The `element_count` field drives the CPU/GPU threshold decision.
#[derive(Debug, Clone, Copy)]
pub enum Workload {
    /// Embarrassingly parallel — no inter-element dependencies (GPU ideal).
    PopulationPk { n_patients: u32 },
    /// Element-wise sweep — independent per element (GPU ideal).
    DoseResponse { n_concentrations: u32 },
    /// Fused map-reduce over a collection (GPU possible above threshold).
    DiversityIndex { n_samples: u32 },
    /// Streaming time-series pipeline (NPU ideal, latency-critical).
    BiosignalDetect { sample_rate_hz: u32 },
    /// Small analytical computation — CPU always.
    Analytical,
}

impl Workload {
    /// The element count that drives the GPU-offload threshold decision.
    #[must_use]
    pub const fn element_count(&self) -> u32 {
        match *self {
            Self::PopulationPk { n_patients } => n_patients,
            Self::DoseResponse { n_concentrations } => n_concentrations,
            Self::DiversityIndex { n_samples } => n_samples,
            Self::BiosignalDetect { sample_rate_hz } => sample_rate_hz,
            Self::Analytical => 0,
        }
    }

    /// Whether this workload benefits from streaming/low-latency NPU.
    #[must_use]
    pub const fn prefers_npu(&self) -> bool {
        matches!(self, Self::BiosignalDetect { .. })
    }
}

/// Discovered GPU capabilities.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub fp64_native: bool,
    pub max_workgroups: u32,
}

/// Discovered NPU capabilities.
#[derive(Debug, Clone)]
pub struct NpuInfo {
    pub name: String,
    pub max_inference_rate_hz: u32,
}

/// Discovered compute capabilities at runtime.
#[derive(Debug, Default)]
pub struct Capabilities {
    pub cpu: bool,
    pub gpu: Option<GpuInfo>,
    pub npu: Option<NpuInfo>,
}

/// Configurable thresholds for GPU offload decisions.
///
/// Below these thresholds, CPU is faster due to dispatch overhead.
/// Callers (or biomeOS) can tune these based on profiled hardware.
#[derive(Debug, Clone)]
pub struct DispatchThresholds {
    pub parallel_gpu_min: u32,
    pub sweep_gpu_min: u32,
    pub reduce_gpu_min: u32,
}

impl Default for DispatchThresholds {
    fn default() -> Self {
        Self {
            parallel_gpu_min: 100,
            sweep_gpu_min: 1000,
            reduce_gpu_min: 500,
        }
    }
}

impl Capabilities {
    /// Discover available compute substrates at runtime.
    ///
    /// CPU is always available. GPU discovery attempts wgpu adapter
    /// enumeration when the `gpu` feature is enabled; otherwise returns
    /// None. NPU discovery is feature-gated behind `npu`.
    #[must_use]
    pub fn discover() -> Self {
        Self {
            cpu: true,
            gpu: Self::probe_gpu(),
            npu: Self::probe_npu(),
        }
    }

    /// Construct with explicitly injected capabilities (for testing or
    /// when biomeOS provides hardware topology).
    #[must_use]
    pub const fn with_known(gpu: Option<GpuInfo>, npu: Option<NpuInfo>) -> Self {
        Self {
            cpu: true,
            gpu,
            npu,
        }
    }

    fn probe_gpu() -> Option<GpuInfo> {
        #[cfg(feature = "gpu")]
        {
            // wgpu adapter enumeration — runtime GPU discovery
            let instance = wgpu::Instance::default();
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    ..Default::default()
                }));
            adapter.map(|a| {
                let info = a.get_info();
                GpuInfo {
                    name: info.name.clone(),
                    fp64_native: false,
                    max_workgroups: a.limits().max_compute_workgroups_per_dimension,
                }
            })
        }
        #[cfg(not(feature = "gpu"))]
        {
            None
        }
    }

    fn probe_npu() -> Option<NpuInfo> {
        #[cfg(feature = "npu")]
        {
            // akida-driver probe would go here
            None
        }
        #[cfg(not(feature = "npu"))]
        {
            None
        }
    }
}

/// Select the optimal substrate for a workload using default thresholds.
#[must_use]
pub fn select_substrate(workload: &Workload, caps: &Capabilities) -> Substrate {
    select_substrate_with_thresholds(workload, caps, &DispatchThresholds::default())
}

/// Select the optimal substrate with custom dispatch thresholds.
///
/// biomeOS or callers can supply profiled thresholds for their specific
/// hardware topology rather than relying on compiled-in defaults.
#[must_use]
pub fn select_substrate_with_thresholds(
    workload: &Workload,
    caps: &Capabilities,
    thresholds: &DispatchThresholds,
) -> Substrate {
    if workload.prefers_npu() && caps.npu.is_some() {
        return Substrate::Npu;
    }

    if let Some(ref _gpu) = caps.gpu {
        let n = workload.element_count();
        let threshold = match workload {
            Workload::PopulationPk { .. } => thresholds.parallel_gpu_min,
            Workload::DoseResponse { .. } => thresholds.sweep_gpu_min,
            Workload::DiversityIndex { .. } => thresholds.reduce_gpu_min,
            Workload::BiosignalDetect { .. } | Workload::Analytical => return Substrate::Cpu,
        };
        if n > threshold {
            return Substrate::Gpu;
        }
    }

    Substrate::Cpu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_always_available() {
        let caps = Capabilities::discover();
        assert!(caps.cpu, "CPU must always be available");
    }

    #[test]
    fn capability_discovery_returns_cpu_true() {
        let caps = Capabilities::discover();
        assert!(caps.cpu);
    }

    #[test]
    fn no_gpu_falls_back_to_cpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: None,
            npu: None,
        };
        let workload = Workload::PopulationPk { n_patients: 1000 };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Cpu);
    }

    #[test]
    fn workload_routing_population_pk_small_cpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: None,
        };
        let workload = Workload::PopulationPk { n_patients: 50 };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Cpu);
    }

    #[test]
    fn workload_routing_population_pk_large_gpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: None,
        };
        let workload = Workload::PopulationPk { n_patients: 500 };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Gpu);
    }

    #[test]
    fn workload_routing_dose_response_small_cpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: None,
        };
        let workload = Workload::DoseResponse {
            n_concentrations: 100,
        };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Cpu);
    }

    #[test]
    fn workload_routing_dose_response_large_gpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: None,
        };
        let workload = Workload::DoseResponse {
            n_concentrations: 2000,
        };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Gpu);
    }

    #[test]
    fn workload_routing_diversity_index_small_cpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: None,
        };
        let workload = Workload::DiversityIndex { n_samples: 100 };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Cpu);
    }

    #[test]
    fn workload_routing_diversity_index_large_gpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: None,
        };
        let workload = Workload::DiversityIndex { n_samples: 1000 };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Gpu);
    }

    #[test]
    fn workload_routing_biosignal_npu() {
        let caps = Capabilities {
            cpu: true,
            gpu: None,
            npu: Some(NpuInfo {
                name: "Akida AKD1000".into(),
                max_inference_rate_hz: 10_000,
            }),
        };
        let workload = Workload::BiosignalDetect {
            sample_rate_hz: 256,
        };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Npu);
    }

    #[test]
    fn workload_routing_biosignal_no_npu_falls_back_cpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: None,
            npu: None,
        };
        let workload = Workload::BiosignalDetect {
            sample_rate_hz: 256,
        };
        assert_eq!(select_substrate(&workload, &caps), Substrate::Cpu);
    }

    #[test]
    fn workload_routing_analytical_always_cpu() {
        let caps = Capabilities {
            cpu: true,
            gpu: Some(GpuInfo {
                name: "test".into(),
                fp64_native: true,
                max_workgroups: 256,
            }),
            npu: Some(NpuInfo {
                name: "Akida".into(),
                max_inference_rate_hz: 10_000,
            }),
        };
        let workload = Workload::Analytical;
        assert_eq!(select_substrate(&workload, &caps), Substrate::Cpu);
    }
}
