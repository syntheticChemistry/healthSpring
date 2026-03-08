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

/// Workload categories that determine dispatch routing.
#[derive(Debug, Clone, Copy)]
pub enum Workload {
    /// Population PK Monte Carlo — embarrassingly parallel (GPU ideal).
    PopulationPk { n_patients: u32 },
    /// Hill dose-response sweep — element-wise (GPU ideal).
    DoseResponse { n_concentrations: u32 },
    /// Diversity indices — fused map-reduce (GPU possible).
    DiversityIndex { n_samples: u32 },
    /// Biosignal detection — streaming pipeline (NPU ideal).
    BiosignalDetect { sample_rate_hz: u32 },
    /// Small analytical computation — CPU always.
    Analytical,
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

impl Capabilities {
    /// Discover available compute substrates.
    /// CPU is always available; GPU/NPU are probed at runtime.
    #[must_use]
    pub fn discover() -> Self {
        Self {
            cpu: true,
            gpu: Self::probe_gpu(),
            npu: Self::probe_npu(),
        }
    }

    fn probe_gpu() -> Option<GpuInfo> {
        // Tier 1: no GPU dispatch yet. Returns None.
        // Tier 2: will use wgpu adapter enumeration.
        None
    }

    fn probe_npu() -> Option<NpuInfo> {
        // Future: probe Akida via akida-driver.
        None
    }
}

/// Select the optimal substrate for a workload given capabilities.
#[must_use]
pub fn select_substrate(workload: &Workload, caps: &Capabilities) -> Substrate {
    match workload {
        Workload::PopulationPk { n_patients } if *n_patients > 100 && caps.gpu.is_some() => {
            Substrate::Gpu
        }
        Workload::DoseResponse { n_concentrations }
            if *n_concentrations > 1000 && caps.gpu.is_some() =>
        {
            Substrate::Gpu
        }
        Workload::DiversityIndex { n_samples } if *n_samples > 500 && caps.gpu.is_some() => {
            Substrate::Gpu
        }
        Workload::BiosignalDetect { .. } if caps.npu.is_some() => Substrate::Npu,
        _ => Substrate::Cpu,
    }
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
