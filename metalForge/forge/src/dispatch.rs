// SPDX-License-Identifier: AGPL-3.0-only
//! Pipeline dispatch planning — maps stages to NUCLEUS Nests with transfer plans.
//!
//! Given a sequence of workloads and a hardware topology (`Tower`), produces a
//! `DispatchPlan` that assigns each stage to a specific `Nest` and plans
//! inter-device data transfers between consecutive stages.
//!
//! ## ABSORPTION STATUS (toadStool S142 / biomeOS)
//!
//! toadStool S139–S142 has evolved:
//! - `StreamingDispatchContext` with `StageProgress`/`ProgressCallback` (S140)
//! - `PipelineGraph` DAG with Kahn sort and cycle detection (S139)
//! - `PcieTransport` GPU-to-GPU topology discovery (S142)
//! - `ResourceOrchestrator` multi-tenant GPU allocation (S142)
//!
//! **Pending** (local to healthSpring until next absorption):
//! - `DispatchPlan` + `plan_dispatch()` → toadStool planner
//! - `StageAssignment` with `NestId` → biomeOS graph node annotations
//! - Transfer overhead analysis → toadStool scheduling heuristics

use crate::nucleus::{NestId, Node, Tower};
use crate::transfer::{TransferPlan, plan_transfer};
use crate::{Capabilities, Substrate, Workload, select_substrate};

/// A single stage assignment within a dispatch plan.
#[derive(Debug, Clone)]
pub struct StageAssignment {
    pub stage_index: usize,
    pub workload: Workload,
    pub substrate: Substrate,
    pub nest_id: NestId,
    pub transfer: Option<TransferPlan>,
}

/// Complete dispatch plan for a pipeline across heterogeneous hardware.
#[derive(Debug)]
pub struct DispatchPlan {
    pub assignments: Vec<StageAssignment>,
    pub total_transfer_bytes: u64,
    pub n_substrate_transitions: usize,
}

impl DispatchPlan {
    /// Estimated total transfer overhead in microseconds.
    #[must_use]
    pub fn total_transfer_time_us(&self) -> f64 {
        self.assignments
            .iter()
            .filter_map(|a| a.transfer.as_ref())
            .map(TransferPlan::estimated_time_us)
            .sum()
    }

    /// Substrates used in this plan (deduplicated, in order of first use).
    #[must_use]
    pub fn substrates_used(&self) -> Vec<Substrate> {
        let mut seen = Vec::new();
        for a in &self.assignments {
            if !seen.contains(&a.substrate) {
                seen.push(a.substrate);
            }
        }
        seen
    }
}

/// Plan dispatch for a sequence of workloads across a Tower topology.
///
/// Each workload is routed to a substrate via `select_substrate()`, then
/// mapped to the first available `Nest` of that type on the local node.
/// When consecutive stages land on different devices, a `TransferPlan`
/// is generated for the inter-device data movement.
#[must_use]
pub fn plan_dispatch(
    workloads: &[(usize, Workload, u64)],
    caps: &Capabilities,
    tower: &Tower,
) -> DispatchPlan {
    let local_node = tower.nodes.first();
    let mut assignments = Vec::with_capacity(workloads.len());
    let mut total_bytes = 0u64;
    let mut transitions = 0usize;
    let mut prev_nest: Option<NestId> = None;

    for &(stage_index, workload, output_bytes) in workloads {
        let substrate = select_substrate(&workload, caps);
        let nest_id = find_nest(local_node, substrate);

        let transfer = prev_nest.filter(|&prev| prev != nest_id).map(|prev| {
            transitions += 1;
            total_bytes += output_bytes;
            plan_transfer(prev, nest_id, output_bytes, local_node)
        });

        prev_nest = Some(nest_id);
        assignments.push(StageAssignment {
            stage_index,
            workload,
            substrate,
            nest_id,
            transfer,
        });
    }

    DispatchPlan {
        assignments,
        total_transfer_bytes: total_bytes,
        n_substrate_transitions: transitions,
    }
}

fn find_nest(node: Option<&Node>, substrate: Substrate) -> NestId {
    let Some(n) = node else {
        return NestId {
            tower: 0,
            node: 0,
            device: 0,
        };
    };
    // Prefer exact substrate match, fall back to CPU nest on same node.
    let exact = n.nests.iter().find(|nest| nest.substrate == substrate);
    let fallback = n.nests.iter().find(|nest| nest.substrate == Substrate::Cpu);
    exact.or(fallback).map_or(
        NestId {
            tower: 0,
            node: 0,
            device: 0,
        },
        |nest| nest.id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleus::{DeviceStatus, Nest, Node, NodeId, PcieGeneration};
    use crate::{GpuInfo, NpuInfo, PrecisionRouting};

    fn test_tower() -> Tower {
        Tower {
            id: 0,
            nodes: vec![Node {
                id: NodeId { tower: 0, node: 0 },
                nests: vec![
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 0,
                            device: 0,
                        },
                        substrate: Substrate::Cpu,
                        memory_bytes: 64 * 1024 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 0,
                            device: 1,
                        },
                        substrate: Substrate::Gpu,
                        memory_bytes: 24 * 1024 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 0,
                            device: 2,
                        },
                        substrate: Substrate::Npu,
                        memory_bytes: 256 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                ],
                pcie_gen: PcieGeneration::Gen4,
            }],
        }
    }

    fn full_caps() -> Capabilities {
        Capabilities::with_known(
            Some(GpuInfo {
                name: "RTX 4090".into(),
                fp64_native: false,
                f64_shared_mem_reliable: false,
                max_workgroups: 65535,
                precision: PrecisionRouting::Df64Only,
            }),
            Some(NpuInfo {
                name: "Akida AKD1000".into(),
                max_inference_rate_hz: 10_000,
            }),
        )
    }

    #[test]
    fn mixed_pipeline_assigns_correct_substrates() {
        let caps = full_caps();
        let tower = test_tower();

        let workloads = vec![
            (
                0,
                Workload::DoseResponse {
                    n_concentrations: 5000,
                },
                40_000,
            ),
            (1, Workload::PopulationPk { n_patients: 10_000 }, 80_000),
            (2, Workload::DiversityIndex { n_samples: 1000 }, 16_000),
            (
                3,
                Workload::BiosignalDetect {
                    sample_rate_hz: 256,
                },
                2048,
            ),
            (4, Workload::EndocrinePk { n_timepoints: 100 }, 800),
        ];

        let plan = plan_dispatch(&workloads, &caps, &tower);

        assert_eq!(plan.assignments.len(), 5);
        assert_eq!(plan.assignments[0].substrate, Substrate::Gpu);
        assert_eq!(plan.assignments[1].substrate, Substrate::Gpu);
        assert_eq!(plan.assignments[2].substrate, Substrate::Gpu);
        assert_eq!(plan.assignments[3].substrate, Substrate::Npu);
        assert_eq!(plan.assignments[4].substrate, Substrate::Cpu);
    }

    #[test]
    fn transitions_tracked_correctly() {
        let caps = full_caps();
        let tower = test_tower();

        let workloads = vec![
            (
                0,
                Workload::DoseResponse {
                    n_concentrations: 5000,
                },
                40_000,
            ),
            (
                1,
                Workload::BiosignalDetect {
                    sample_rate_hz: 256,
                },
                2048,
            ),
            (2, Workload::EndocrinePk { n_timepoints: 100 }, 800),
        ];

        let plan = plan_dispatch(&workloads, &caps, &tower);
        assert_eq!(plan.n_substrate_transitions, 2);
        assert!(plan.total_transfer_bytes > 0);
    }

    #[test]
    fn same_substrate_no_transfer() {
        let caps = full_caps();
        let tower = test_tower();

        let workloads = vec![
            (
                0,
                Workload::DoseResponse {
                    n_concentrations: 5000,
                },
                40_000,
            ),
            (1, Workload::PopulationPk { n_patients: 10_000 }, 80_000),
        ];

        let plan = plan_dispatch(&workloads, &caps, &tower);
        assert_eq!(plan.n_substrate_transitions, 0);
        assert!(plan.assignments[1].transfer.is_none());
    }

    #[test]
    fn gpu_to_npu_generates_p2p_transfer() {
        let caps = full_caps();
        let tower = test_tower();

        let workloads = vec![
            (
                0,
                Workload::DoseResponse {
                    n_concentrations: 5000,
                },
                40_000,
            ),
            (
                1,
                Workload::BiosignalDetect {
                    sample_rate_hz: 256,
                },
                2048,
            ),
        ];

        let plan = plan_dispatch(&workloads, &caps, &tower);
        let transfer = plan.assignments[1].transfer.as_ref().unwrap();
        assert_eq!(transfer.method, crate::transfer::TransferMethod::PcieP2p,);
    }

    #[test]
    fn substrates_used_deduplicates() {
        let caps = full_caps();
        let tower = test_tower();

        let workloads = vec![
            (
                0,
                Workload::DoseResponse {
                    n_concentrations: 5000,
                },
                40_000,
            ),
            (1, Workload::PopulationPk { n_patients: 10_000 }, 80_000),
            (
                2,
                Workload::BiosignalDetect {
                    sample_rate_hz: 256,
                },
                2048,
            ),
            (3, Workload::EndocrinePk { n_timepoints: 100 }, 800),
        ];

        let plan = plan_dispatch(&workloads, &caps, &tower);
        let subs = plan.substrates_used();
        assert_eq!(subs, vec![Substrate::Gpu, Substrate::Npu, Substrate::Cpu]);
    }

    #[test]
    fn transfer_time_is_positive() {
        let caps = full_caps();
        let tower = test_tower();

        let workloads = vec![
            (
                0,
                Workload::DoseResponse {
                    n_concentrations: 5000,
                },
                40_000,
            ),
            (
                1,
                Workload::BiosignalDetect {
                    sample_rate_hz: 256,
                },
                2048,
            ),
            (2, Workload::EndocrinePk { n_timepoints: 100 }, 800),
        ];

        let plan = plan_dispatch(&workloads, &caps, &tower);
        assert!(plan.total_transfer_time_us() > 0.0);
    }
}
