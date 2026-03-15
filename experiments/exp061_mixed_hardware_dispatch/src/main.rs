// SPDX-License-Identifier: AGPL-3.0-only
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

//! Exp061: Mixed hardware dispatch via `metalForge` NUCLEUS topology.
//!
//! Validates that `DispatchPlan` correctly assigns stages to CPU/GPU/NPU
//! based on workload type and element count, and generates correct transfer
//! plans between devices (`PCIe` P2P, host-staged, network IPC).

use healthspring_forge::dispatch::{DispatchPlan, plan_dispatch};
use healthspring_forge::nucleus::{
    DeviceStatus, Nest, NestId, Node, NodeId, PcieGeneration, Tower,
};
use healthspring_forge::transfer::TransferMethod;
use healthspring_forge::{Capabilities, GpuInfo, NpuInfo, PrecisionRouting, Substrate, Workload};

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

fn workstation_tower() -> Tower {
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

fn cluster_tower() -> Tower {
    Tower {
        id: 0,
        nodes: vec![
            Node {
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
                ],
                pcie_gen: PcieGeneration::Gen4,
            },
            Node {
                id: NodeId { tower: 0, node: 1 },
                nests: vec![
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 1,
                            device: 0,
                        },
                        substrate: Substrate::Cpu,
                        memory_bytes: 128 * 1024 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 1,
                            device: 1,
                        },
                        substrate: Substrate::Gpu,
                        memory_bytes: 80 * 1024 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                ],
                pcie_gen: PcieGeneration::Gen5,
            },
        ],
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

const fn cpu_only_caps() -> Capabilities {
    Capabilities::with_known(None, None)
}

fn print_plan(plan: &DispatchPlan) {
    for a in &plan.assignments {
        let transfer_str = a.transfer.as_ref().map_or_else(
            || "none".into(),
            |t| format!("{:?} ({:.1} us)", t.method, t.estimated_time_us()),
        );
        println!(
            "    stage {}: {:?} -> {:?} @ {}  transfer: {}",
            a.stage_index, a.workload, a.substrate, a.nest_id, transfer_str,
        );
    }
    println!(
        "    transitions: {}  total transfer: {} bytes ({:.2} us)\n",
        plan.n_substrate_transitions,
        plan.total_transfer_bytes,
        plan.total_transfer_time_us(),
    );
}

#[expect(clippy::too_many_lines)]
fn main() {
    println!("Exp061 Mixed Hardware Dispatch — metalForge NUCLEUS");
    println!("====================================================\n");

    let mut passed = 0u32;
    let mut total = 0u32;

    // -----------------------------------------------------------------------
    // Scenario 1: Full healthSpring pipeline on workstation (CPU+GPU+NPU)
    // -----------------------------------------------------------------------
    println!("--- Scenario 1: Full pipeline on workstation (CPU+GPU+NPU) ---");
    let caps = full_caps();
    let tower = workstation_tower();

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
    print_plan(&plan);

    check(
        "s1: 5 stages assigned",
        plan.assignments.len() == 5,
        &mut passed,
        &mut total,
    );
    check(
        "s1: Hill -> GPU",
        plan.assignments[0].substrate == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "s1: PopPK -> GPU",
        plan.assignments[1].substrate == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "s1: Diversity -> GPU",
        plan.assignments[2].substrate == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "s1: Biosignal -> NPU",
        plan.assignments[3].substrate == Substrate::Npu,
        &mut passed,
        &mut total,
    );
    check(
        "s1: Endocrine -> CPU",
        plan.assignments[4].substrate == Substrate::Cpu,
        &mut passed,
        &mut total,
    );

    check(
        "s1: GPU->NPU via PCIe P2P",
        plan.assignments[3]
            .transfer
            .as_ref()
            .is_some_and(|t| t.method == TransferMethod::PcieP2p),
        &mut passed,
        &mut total,
    );
    check(
        "s1: NPU->CPU via PCIe P2P",
        plan.assignments[4]
            .transfer
            .as_ref()
            .is_some_and(|t| t.method == TransferMethod::PcieP2p),
        &mut passed,
        &mut total,
    );
    check(
        "s1: 2 transitions",
        plan.n_substrate_transitions == 2,
        &mut passed,
        &mut total,
    );
    check(
        "s1: transfer time > 0",
        plan.total_transfer_time_us() > 0.0,
        &mut passed,
        &mut total,
    );
    check(
        "s1: 3 substrates used",
        plan.substrates_used() == vec![Substrate::Gpu, Substrate::Npu, Substrate::Cpu],
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Scenario 2: CPU-only fallback (no GPU, no NPU)
    // -----------------------------------------------------------------------
    println!("--- Scenario 2: CPU-only fallback ---");
    let caps_cpu = cpu_only_caps();

    let plan_cpu = plan_dispatch(&workloads, &caps_cpu, &tower);
    print_plan(&plan_cpu);

    check(
        "s2: all stages on CPU",
        plan_cpu
            .assignments
            .iter()
            .all(|a| a.substrate == Substrate::Cpu),
        &mut passed,
        &mut total,
    );
    check(
        "s2: 0 transitions",
        plan_cpu.n_substrate_transitions == 0,
        &mut passed,
        &mut total,
    );
    check(
        "s2: 0 transfer bytes",
        plan_cpu.total_transfer_bytes == 0,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Scenario 3: Biosignal fusion pipeline (NPU-centric)
    // -----------------------------------------------------------------------
    println!("--- Scenario 3: Biosignal NPU-centric pipeline ---");
    let workloads_bio = vec![
        (
            0,
            Workload::BiosignalDetect {
                sample_rate_hz: 256,
            },
            2048,
        ),
        (1, Workload::BiosignalFusion { channels: 4 }, 256),
        (2, Workload::Analytical, 64),
    ];

    let plan_bio = plan_dispatch(&workloads_bio, &caps, &tower);
    print_plan(&plan_bio);

    check(
        "s3: detect -> NPU",
        plan_bio.assignments[0].substrate == Substrate::Npu,
        &mut passed,
        &mut total,
    );
    check(
        "s3: fusion -> NPU",
        plan_bio.assignments[1].substrate == Substrate::Npu,
        &mut passed,
        &mut total,
    );
    check(
        "s3: analytical -> CPU",
        plan_bio.assignments[2].substrate == Substrate::Cpu,
        &mut passed,
        &mut total,
    );
    check(
        "s3: NPU->CPU transition",
        plan_bio.n_substrate_transitions == 1,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Scenario 4: Small workloads stay CPU (below threshold)
    // -----------------------------------------------------------------------
    println!("--- Scenario 4: Small workloads (below GPU threshold) ---");
    let workloads_small = vec![
        (
            0,
            Workload::DoseResponse {
                n_concentrations: 50,
            },
            400,
        ),
        (1, Workload::PopulationPk { n_patients: 10 }, 80),
        (2, Workload::DiversityIndex { n_samples: 5 }, 80),
    ];

    let plan_small = plan_dispatch(&workloads_small, &caps, &tower);
    print_plan(&plan_small);

    check(
        "s4: all small -> CPU",
        plan_small
            .assignments
            .iter()
            .all(|a| a.substrate == Substrate::Cpu),
        &mut passed,
        &mut total,
    );
    check(
        "s4: 0 transitions",
        plan_small.n_substrate_transitions == 0,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Scenario 5: Cluster topology (multi-node)
    // -----------------------------------------------------------------------
    println!("--- Scenario 5: Cluster topology ---");
    let cluster = cluster_tower();

    let plan_cluster = plan_dispatch(&workloads, &caps, &cluster);
    print_plan(&plan_cluster);

    check(
        "s5: GPU stages use local node",
        plan_cluster.assignments[0].nest_id.node == 0,
        &mut passed,
        &mut total,
    );
    check(
        "s5: cluster has 2 nodes 5 nests",
        cluster.total_nests() == 4,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Summary
    // -----------------------------------------------------------------------
    println!("====================================================");
    println!("Exp061 Mixed Hardware Dispatch: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
