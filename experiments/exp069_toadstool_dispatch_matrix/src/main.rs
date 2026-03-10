// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — exercises all stage types sequentially"
)]

//! Exp069: toadStool dispatch matrix — validates all stage types on CPU.
//!
//! Tests each `StageOp` variant (including new `BiosignalFusion`, `AucTrapezoidal`,
//! `BrayCurtis`) through the pipeline, verifying correctness and timing.

use healthspring_forge::Substrate;
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

    println!("Exp069: toadStool Dispatch Matrix");
    println!("==================================");

    // --- Generate → Hill Transform → Sum Reduce pipeline ---
    println!("\n=== Pipeline: Generate → Hill → Sum ===");
    let mut p1 = Pipeline::new("hill_pipeline");
    p1.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 100,
            seed: 42,
        },
    });
    p1.add_stage(Stage {
        name: "hill".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 1.0,
                ec50: 0.5,
                n: 2.0,
            },
        },
    });
    p1.add_stage(Stage {
        name: "sum".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    });
    let r1 = p1.execute_cpu();
    check!("hill_pipeline_success", r1.success);
    check!("hill_pipeline_3_stages", r1.stage_results.len() == 3);
    check!(
        "hill_gen_100_elements",
        r1.stage_results[0].output_data.len() == 100
    );
    check!(
        "hill_transform_100_elements",
        r1.stage_results[1].output_data.len() == 100
    );
    check!(
        "hill_reduce_scalar",
        r1.stage_results[2].output_data.len() == 1
    );
    check!(
        "hill_sum_positive",
        r1.stage_results[2].output_data[0] > 0.0
    );

    // --- Population PK pipeline ---
    println!("\n=== Pipeline: PopulationPk ===");
    let mut p2 = Pipeline::new("pop_pk_pipeline");
    p2.add_stage(Stage {
        name: "pop_pk".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::PopulationPk {
            n_patients: 200,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 123,
        },
    });
    p2.add_stage(Stage {
        name: "mean_auc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Mean,
        },
    });
    let r2 = p2.execute_cpu();
    check!("pop_pk_success", r2.success);
    check!(
        "pop_pk_200_aucs",
        r2.stage_results[0].output_data.len() == 200
    );
    check!(
        "pop_pk_all_positive",
        r2.stage_results[0].output_data.iter().all(|&v| v > 0.0)
    );
    check!(
        "pop_pk_mean_scalar",
        r2.stage_results[1].output_data.len() == 1
    );
    check!(
        "pop_pk_mean_positive",
        r2.stage_results[1].output_data[0] > 0.0
    );

    // --- Diversity pipeline ---
    println!("\n=== Pipeline: DiversityReduce ===");
    let communities = vec![
        vec![0.25, 0.25, 0.25, 0.25],
        vec![0.9, 0.05, 0.03, 0.02],
        vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
    ];
    let mut p3 = Pipeline::new("diversity_pipeline");
    p3.add_stage(Stage {
        name: "diversity".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::DiversityReduce {
            communities: communities.clone(),
        },
    });
    let r3 = p3.execute_cpu();
    check!("diversity_success", r3.success);
    check!(
        "diversity_6_values",
        r3.stage_results[0].output_data.len() == 6
    );
    let even_shannon = r3.stage_results[0].output_data[0];
    let dom_shannon = r3.stage_results[0].output_data[2];
    check!("diversity_even_gt_dominated", even_shannon > dom_shannon);

    // --- BiosignalFusion pipeline ---
    println!("\n=== Pipeline: BiosignalFusion ===");
    let mut p4 = Pipeline::new("biosignal_pipeline");
    p4.add_stage(Stage {
        name: "gen_ecg".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 360,
            seed: 100,
        },
    });
    p4.add_stage(Stage {
        name: "biosignal_fusion".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 3 },
    });
    let r4 = p4.execute_cpu();
    check!("biosignal_fusion_success", r4.success);
    check!(
        "biosignal_fusion_output_len",
        !r4.stage_results[1].output_data.is_empty()
    );

    // --- AUC Trapezoidal pipeline ---
    println!("\n=== Pipeline: AucTrapezoidal ===");
    let mut p5 = Pipeline::new("auc_pipeline");
    p5.add_stage(Stage {
        name: "gen_curve".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 101,
            seed: 77,
        },
    });
    p5.add_stage(Stage {
        name: "auc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::AucTrapezoidal { t_max: 24.0 },
    });
    let r5 = p5.execute_cpu();
    check!("auc_pipeline_success", r5.success);
    check!(
        "auc_scalar_output",
        r5.stage_results[1].output_data.len() == 1
    );
    check!("auc_positive", r5.stage_results[1].output_data[0] > 0.0);

    // --- Bray-Curtis pipeline ---
    println!("\n=== Pipeline: BrayCurtis ===");
    let mut p6 = Pipeline::new("bray_curtis_pipeline");
    p6.add_stage(Stage {
        name: "bray_curtis".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BrayCurtis {
            communities: communities.clone(),
        },
    });
    let r6 = p6.execute_cpu();
    check!("bray_curtis_success", r6.success);
    let n_communities = communities.len();
    let expected_pairs = n_communities * (n_communities - 1) / 2;
    check!(
        &format!("bray_curtis_{expected_pairs}_pairs"),
        r6.stage_results[0].output_data.len() == expected_pairs
    );
    check!(
        "bray_curtis_all_in_range",
        r6.stage_results[0]
            .output_data
            .iter()
            .all(|&v| (0.0..=1.0).contains(&v))
    );

    // --- Filter pipeline ---
    println!("\n=== Pipeline: Filter ===");
    let mut p7 = Pipeline::new("filter_pipeline");
    p7.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 50,
            seed: 7,
        },
    });
    p7.add_stage(Stage {
        name: "filter".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Filter { threshold: 0.5 },
    });
    p7.add_stage(Stage {
        name: "count".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    });
    let r7 = p7.execute_cpu();
    check!("filter_pipeline_success", r7.success);
    check!(
        "filter_all_above_threshold",
        r7.stage_results[1].output_data.iter().all(|&v| v > 0.5)
    );
    check!(
        "filter_reduces_count",
        r7.stage_results[1].output_data.len() < 50
    );

    // --- Variance reduce ---
    println!("\n=== Pipeline: Variance Reduce ===");
    let mut p8 = Pipeline::new("variance_pipeline");
    p8.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 1000,
            seed: 999,
        },
    });
    p8.add_stage(Stage {
        name: "variance".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Variance,
        },
    });
    let r8 = p8.execute_cpu();
    check!("variance_pipeline_success", r8.success);
    check!(
        "variance_scalar",
        r8.stage_results[1].output_data.len() == 1
    );
    check!(
        "variance_positive",
        r8.stage_results[1].output_data[0] > 0.0
    );

    // --- ExpDecay transform ---
    println!("\n=== Pipeline: ExpDecay Transform ===");
    let mut p9 = Pipeline::new("exp_decay_pipeline");
    p9.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 20,
            seed: 11,
        },
    });
    p9.add_stage(Stage {
        name: "decay".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::ExpDecay { k: 0.1, t: 5.0 },
        },
    });
    let r9 = p9.execute_cpu();
    check!("exp_decay_success", r9.success);
    check!(
        "exp_decay_20_elements",
        r9.stage_results[1].output_data.len() == 20
    );
    let decay_factor = (-0.1_f64 * 5.0).exp();
    for i in 0..20 {
        let input = r9.stage_results[0].output_data[i];
        let output = r9.stage_results[1].output_data[i];
        let expected = input * decay_factor;
        check!(
            &format!("exp_decay_element_{i}"),
            (output - expected).abs() < 1e-10
        );
    }

    let total = passed + failed;
    println!("\n==================================");
    println!("Exp069 toadStool Dispatch Matrix: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
