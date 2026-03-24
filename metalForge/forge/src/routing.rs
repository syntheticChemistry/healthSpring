// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Lint policy: workspace-level [lints] in root Cargo.toml.
// forbid(unsafe_code), deny(clippy::{all,pedantic,nursery,unwrap_used,expect_used}).

//! Threshold-based substrate selection for workloads.

use crate::discovery::Capabilities;
use crate::types::{DispatchThresholds, Substrate, Workload};

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
    use crate::types::{GpuInfo, NpuInfo, PrecisionRouting};

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
