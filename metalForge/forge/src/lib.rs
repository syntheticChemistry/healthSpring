// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

//! metalForge — heterogeneous compute dispatch for healthSpring.
//!
//! Routes workloads to CPU, GPU, or NPU based on runtime capability discovery.
//! Organizes hardware via NUCLEUS atomics (Tower → Node → Nest) and plans
//! inter-device transfers (`PCIe` P2P DMA, host-staged, network IPC).
//!
//! ## ABSORPTION STATUS (barraCuda / toadStool / biomeOS)
//!
//! - `Substrate` + `Workload` enum -> barraCuda workload classification
//! - `select_substrate()` threshold routing -> toadStool dispatcher
//! - `Capabilities::discover()` -> barraCuda hardware probe
//! - `DispatchPlan` + NUCLEUS topology -> biomeOS graph planner
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

pub mod dispatch;
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
    /// Multi-channel biosignal fusion (CPU or NPU).
    BiosignalFusion { channels: u32 },
    /// Endocrine PK computation (CPU-only, analytical).
    EndocrinePk { n_timepoints: u32 },
    /// Batch Michaelis-Menten PK ODE per patient (GPU ideal).
    MichaelisMentenBatch { n_patients: u32 },
    /// Batch SCFA metabolic production per fiber input (GPU ideal).
    ScfaBatch { n_elements: u32 },
    /// Batch beat template-matching classification (GPU ideal).
    BeatClassifyBatch { n_beats: u32 },
    /// Small analytical computation — CPU always.
    Analytical,
}

impl Workload {
    /// The element count that drives the GPU-offload threshold decision.
    #[must_use]
    pub const fn element_count(&self) -> u32 {
        match *self {
            Self::PopulationPk { n_patients } | Self::MichaelisMentenBatch { n_patients } => {
                n_patients
            }
            Self::DoseResponse { n_concentrations } => n_concentrations,
            Self::DiversityIndex { n_samples } => n_samples,
            Self::BiosignalDetect { sample_rate_hz } => sample_rate_hz,
            Self::BiosignalFusion { channels } => channels,
            Self::EndocrinePk { n_timepoints } => n_timepoints,
            Self::ScfaBatch { n_elements } => n_elements,
            Self::BeatClassifyBatch { n_beats } => n_beats,
            Self::Analytical => 0,
        }
    }

    /// Whether this workload benefits from streaming/low-latency NPU.
    #[must_use]
    pub const fn prefers_npu(&self) -> bool {
        matches!(
            self,
            Self::BiosignalDetect { .. } | Self::BiosignalFusion { .. }
        )
    }
}

/// Precision routing advice for f64 GPU workloads.
///
/// Mirrors toadStool's `PrecisionRoutingAdvice` (S128) and barraCuda's
/// `Fp64Strategy`. Determines how shaders should handle f64 arithmetic
/// based on discovered hardware capabilities.
///
/// See also: `GPU_F64_NUMERICAL_STABILITY` in wateringHole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionRouting {
    /// Hardware has native f64 ALUs and reliable shared memory.
    /// Use `enable f64;` WGSL shaders directly.
    F64Native,
    /// Hardware has native f64 compute but shared-memory reduction
    /// returns zeros (naga/SPIR-V bug on some drivers).
    /// Avoid f64 workgroup shared memory; use per-thread accumulators.
    F64NativeNoSharedMem,
    /// No native f64; use double-float emulation (DF64).
    /// coralReef lowers f64 ops to paired f32 arithmetic.
    Df64Only,
    /// f64 compute returns zeros (e.g., NVK Volta).
    /// Fall back to f32 shaders with documented precision loss.
    F32Only,
}

/// Discovered GPU capabilities.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub fp64_native: bool,
    /// Whether f64 shared-memory reductions are reliable on this GPU.
    pub f64_shared_mem_reliable: bool,
    pub max_workgroups: u32,
    /// Routing advice for f64 workloads on this GPU.
    pub precision: PrecisionRouting,
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
            let instance = wgpu::Instance::default();
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    ..Default::default()
                }));
            adapter.ok().map(|a| {
                let info = a.get_info();
                let fp64_native = a.features().contains(wgpu::Features::SHADER_F64);
                let f64_shared_mem_reliable = fp64_native;
                let precision = if !fp64_native {
                    PrecisionRouting::Df64Only
                } else if f64_shared_mem_reliable {
                    PrecisionRouting::F64Native
                } else {
                    PrecisionRouting::F64NativeNoSharedMem
                };
                GpuInfo {
                    name: info.name,
                    fp64_native,
                    f64_shared_mem_reliable,
                    max_workgroups: a.limits().max_compute_workgroups_per_dimension,
                    precision,
                }
            })
        }
        #[cfg(not(feature = "gpu"))]
        {
            None
        }
    }

    /// Probe for neuromorphic accelerator.
    ///
    /// Returns `None` unconditionally — Akida driver integration is
    /// feature-gated behind `npu` and requires the `akida-driver` crate
    /// (which binds to the `BrainChip` `MetaTF` runtime). When the driver is
    /// available, this will query device topology and inference capacity.
    const fn probe_npu() -> Option<NpuInfo> {
        None
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
pub const fn select_substrate_with_thresholds(
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
            Workload::PopulationPk { .. } | Workload::MichaelisMentenBatch { .. } => {
                thresholds.parallel_gpu_min
            }
            Workload::DoseResponse { .. } | Workload::ScfaBatch { .. } => thresholds.sweep_gpu_min,
            Workload::DiversityIndex { .. } | Workload::BeatClassifyBatch { .. } => {
                thresholds.reduce_gpu_min
            }
            Workload::BiosignalDetect { .. }
            | Workload::BiosignalFusion { .. }
            | Workload::EndocrinePk { .. }
            | Workload::Analytical => return Substrate::Cpu,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
                f64_shared_mem_reliable: true,
                max_workgroups: 256,
                precision: PrecisionRouting::F64Native,
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
