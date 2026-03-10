// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — exercises mixed hardware pipeline sequentially"
)]

//! Exp071: Mixed system pipeline — runs a clinical pipeline across mixed substrates
//! (NPU biosignal → GPU `PopPK` → CPU diagnostic fusion) using NUCLEUS topology,
//! validates stage assignments, transfer plans, and end-to-end correctness.

use healthspring_forge::dispatch::plan_dispatch;
use healthspring_forge::nucleus::{
    DeviceStatus, Nest, NestId, Node, NodeId, PcieGeneration, Tower,
};
use healthspring_forge::transfer::TransferMethod;
use healthspring_forge::{Capabilities, GpuInfo, NpuInfo, PrecisionRouting, Substrate, Workload};
use healthspring_toadstool::pipeline::Pipeline;
use healthspring_toadstool::stage::{ReduceKind, Stage, StageOp, TransformKind};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                passed += 1;
                println!("  [PASS] {}", $name);
            } else {
                eprintln!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    println!("Exp071: Mixed System Pipeline");
    println!("==============================");

    // --- Build Tower topology ---
    let tower = Tower {
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
    };

    let caps = Capabilities::with_known(
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
    );

    // --- Clinical pipeline stages ---
    println!("\n=== Stage 1: Biosignal (NPU-targeted) ===");
    let biosignal_workload = Workload::BiosignalDetect {
        sample_rate_hz: 360,
    };
    let biosignal_sub = healthspring_forge::select_substrate(&biosignal_workload, &caps);
    check!("biosignal_routes_to_npu", biosignal_sub == Substrate::Npu);

    println!("\n=== Stage 2: Population PK (GPU-targeted) ===");
    let pk_workload = Workload::PopulationPk { n_patients: 5000 };
    let pk_sub = healthspring_forge::select_substrate(&pk_workload, &caps);
    check!("pop_pk_routes_to_gpu", pk_sub == Substrate::Gpu);

    println!("\n=== Stage 3: Diagnostic Fusion (CPU) ===");
    let diag_workload = Workload::Analytical;
    let diag_sub = healthspring_forge::select_substrate(&diag_workload, &caps);
    check!("diagnostic_routes_to_cpu", diag_sub == Substrate::Cpu);

    // --- Dispatch plan ---
    println!("\n=== Dispatch Plan ===");
    let workloads = vec![
        (0, biosignal_workload, 2880_u64),
        (1, pk_workload, 40_000),
        (2, diag_workload, 800),
    ];
    let plan = plan_dispatch(&workloads, &caps, &tower);

    check!("plan_3_assignments", plan.assignments.len() == 3);
    check!(
        "assignment_0_npu",
        plan.assignments[0].substrate == Substrate::Npu
    );
    check!(
        "assignment_1_gpu",
        plan.assignments[1].substrate == Substrate::Gpu
    );
    check!(
        "assignment_2_cpu",
        plan.assignments[2].substrate == Substrate::Cpu
    );
    check!("plan_2_transitions", plan.n_substrate_transitions == 2);

    // NPU→GPU should be P2P
    if let Some(ref transfer) = plan.assignments[1].transfer {
        check!("npu_to_gpu_p2p", transfer.method == TransferMethod::PcieP2p);
        check!(
            "npu_to_gpu_bandwidth_gt_20gbps",
            transfer.estimated_bandwidth_gbps > 20.0
        );
    } else {
        check!("npu_to_gpu_has_transfer", false);
    }

    // GPU→CPU should be P2P
    if let Some(ref transfer) = plan.assignments[2].transfer {
        check!("gpu_to_cpu_p2p", transfer.method == TransferMethod::PcieP2p);
    } else {
        check!("gpu_to_cpu_has_transfer", false);
    }

    // Substrates used
    let subs = plan.substrates_used();
    check!("uses_3_substrates", subs.len() == 3);
    check!(
        "substrates_npu_gpu_cpu",
        subs == vec![Substrate::Npu, Substrate::Gpu, Substrate::Cpu]
    );

    // Transfer overhead
    let overhead = plan.total_transfer_time_us();
    check!(
        &format!("transfer_overhead_{overhead:.1}us"),
        overhead > 0.0
    );

    // --- Execute toadStool pipeline on CPU (reference) ---
    println!("\n=== toadStool Pipeline Execution (CPU reference) ===");
    let mut pipeline = Pipeline::new("mixed_clinical");

    // Biosignal fusion stage
    pipeline.add_stage(Stage {
        name: "ecg_stream".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 360,
            seed: 42,
        },
    });
    pipeline.add_stage(Stage {
        name: "biosignal_fusion".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 3 },
    });

    // Population PK stage
    pipeline.add_stage(Stage {
        name: "population_pk".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::PopulationPk {
            n_patients: 200,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 123,
        },
    });

    // Diagnostic fusion (reduce to summary)
    pipeline.add_stage(Stage {
        name: "diagnostic_mean".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Mean,
        },
    });

    let result = pipeline.execute_cpu();
    check!("pipeline_success", result.success);
    check!("pipeline_4_stages", result.stage_results.len() == 4);
    check!(
        "ecg_360_samples",
        result.stage_results[0].output_data.len() == 360
    );
    check!(
        "biosignal_fused_output",
        !result.stage_results[1].output_data.is_empty()
    );
    check!(
        "pop_pk_200_patients",
        result.stage_results[2].output_data.len() == 200
    );
    check!(
        "pop_pk_all_positive",
        result.stage_results[2].output_data.iter().all(|&v| v > 0.0)
    );
    check!(
        "diagnostic_scalar",
        result.stage_results[3].output_data.len() == 1
    );
    check!(
        "diagnostic_positive",
        result.stage_results[3].output_data[0] > 0.0
    );
    check!("pipeline_time_positive", result.total_time_us > 0.0);

    // --- Extended pipeline with new stage types ---
    println!("\n=== Extended Pipeline (AUC + Bray-Curtis) ===");
    let mut ext_pipeline = Pipeline::new("extended_mixed");
    ext_pipeline.add_stage(Stage {
        name: "gen_curve".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 101,
            seed: 77,
        },
    });
    ext_pipeline.add_stage(Stage {
        name: "hill_transform".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 1.0,
                ec50: 0.5,
                n: 2.0,
            },
        },
    });
    ext_pipeline.add_stage(Stage {
        name: "auc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::AucTrapezoidal { t_max: 24.0 },
    });

    let ext_result = ext_pipeline.execute_cpu();
    check!("ext_pipeline_success", ext_result.success);
    check!(
        "ext_auc_scalar",
        ext_result.stage_results[2].output_data.len() == 1
    );
    check!(
        "ext_auc_positive",
        ext_result.stage_results[2].output_data[0] > 0.0
    );

    // Bray-Curtis standalone
    let mut bc_pipeline = Pipeline::new("bray_curtis");
    bc_pipeline.add_stage(Stage {
        name: "bc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BrayCurtis {
            communities: vec![
                vec![0.4, 0.3, 0.2, 0.1],
                vec![0.1, 0.2, 0.3, 0.4],
                vec![0.25, 0.25, 0.25, 0.25],
            ],
        },
    });
    let bc_result = bc_pipeline.execute_cpu();
    check!("bc_pipeline_success", bc_result.success);
    check!(
        "bc_3_pairs",
        bc_result.stage_results[0].output_data.len() == 3
    );
    check!(
        "bc_all_in_range",
        bc_result.stage_results[0]
            .output_data
            .iter()
            .all(|&v| (0.0..=1.0).contains(&v))
    );

    let total = passed + failed;
    println!("\n==============================");
    println!("Exp071 Mixed System Pipeline: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
