// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential pipeline dispatch checks"
)]

//! Exp086: toadStool V16 Streaming Dispatch
//!
//! Validates that all V16 `StageOp` variants execute correctly through
//! toadStool's pipeline infrastructure: `execute_cpu`, `execute_streaming`,
//! and multi-stage data flow between V16 ops.
//!
//! Proves the dispatch path is ready for GPU promotion: each V16 stage
//! maps to a `GpuOp` via `to_gpu_op()`, confirming the WGSL shader path.

use healthspring_barracuda::biosignal::classification;
use healthspring_barracuda::microbiome;
use healthspring_forge::Substrate;
use healthspring_toadstool::pipeline::Pipeline;
use healthspring_toadstool::stage::{Stage, StageOp};

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

fn main() {
    let mut passed = 0u32;
    let mut total = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp086 — toadStool V16 Streaming Dispatch");
    println!("{}", "=".repeat(72));

    // ── 1. Michaelis-Menten Batch Pipeline ──────────────────────────────
    println!("\n── 1. MM Batch Pipeline (generate → MM → reduce) ────────────");

    let mut mm_pipe = Pipeline::new("mm_pk_pipeline");
    mm_pipe.add_stage(Stage {
        name: "mm_batch".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 2000,
            n_patients: 128,
            seed: 42,
        },
    });

    let mm_result = mm_pipe.execute_cpu();
    check(
        "MM pipe: success",
        mm_result.success,
        &mut passed,
        &mut total,
    );
    check(
        "MM pipe: 1 stage",
        mm_result.stage_results.len() == 1,
        &mut passed,
        &mut total,
    );
    check(
        "MM pipe: 128 AUCs",
        mm_result.stage_results[0].output_data.len() == 128,
        &mut passed,
        &mut total,
    );
    check(
        "MM pipe: all AUC > 0",
        mm_result.stage_results[0]
            .output_data
            .iter()
            .all(|&v| v > 0.0),
        &mut passed,
        &mut total,
    );
    check(
        "MM pipe: timing > 0",
        mm_result.total_time_us > 0.0,
        &mut passed,
        &mut total,
    );

    // ── 2. SCFA Batch Pipeline ──────────────────────────────────────────
    println!("\n── 2. SCFA Batch Pipeline ─────────────────────────────────────");

    let fiber_inputs: Vec<f64> = (1..=200).map(|i| f64::from(i) * 0.25).collect();
    let mut scfa_pipe = Pipeline::new("scfa_pipeline");
    scfa_pipe.add_stage(Stage {
        name: "scfa_batch".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ScfaBatch {
            params: microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs,
        },
    });

    let scfa_result = scfa_pipe.execute_cpu();
    check(
        "SCFA pipe: success",
        scfa_result.success,
        &mut passed,
        &mut total,
    );
    check(
        "SCFA pipe: 600 values (200×3)",
        scfa_result.stage_results[0].output_data.len() == 600,
        &mut passed,
        &mut total,
    );
    check(
        "SCFA pipe: all values > 0",
        scfa_result.stage_results[0]
            .output_data
            .iter()
            .all(|&v| v > 0.0),
        &mut passed,
        &mut total,
    );

    // ── 3. Beat Classification Batch Pipeline ───────────────────────────
    println!("\n── 3. Beat Classification Batch Pipeline ──────────────────────");

    let templates = vec![
        classification::generate_normal_template(41),
        classification::generate_pvc_template(41),
        classification::generate_pac_template(41),
    ];
    let beats = vec![
        classification::generate_normal_template(41),
        classification::generate_pvc_template(41),
        classification::generate_pac_template(41),
        classification::generate_normal_template(41),
    ];

    let mut beat_pipe = Pipeline::new("beat_classify_pipeline");
    beat_pipe.add_stage(Stage {
        name: "beat_classify".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BeatClassifyBatch {
            beats,
            templates: templates.clone(),
        },
    });

    let beat_result = beat_pipe.execute_cpu();
    check(
        "Beat pipe: success",
        beat_result.success,
        &mut passed,
        &mut total,
    );
    check(
        "Beat pipe: 8 values (4 beats × [class, corr])",
        beat_result.stage_results[0].output_data.len() == 8,
        &mut passed,
        &mut total,
    );

    let beat_data = &beat_result.stage_results[0].output_data;
    check(
        "Beat pipe: beat[0] → Normal (0.0)",
        beat_data[0].abs() < 1e-10,
        &mut passed,
        &mut total,
    );
    check(
        "Beat pipe: beat[1] → PVC (1.0)",
        (beat_data[2] - 1.0).abs() < 1e-10,
        &mut passed,
        &mut total,
    );
    check(
        "Beat pipe: beat[2] → PAC (2.0)",
        (beat_data[4] - 2.0).abs() < 1e-10,
        &mut passed,
        &mut total,
    );
    check(
        "Beat pipe: correlations > 0.99",
        beat_data.chunks(2).all(|pair| pair[1] > 0.99),
        &mut passed,
        &mut total,
    );

    // ── 4. Streaming Dispatch (callbacks) ───────────────────────────────
    println!("\n── 4. Streaming Dispatch (per-stage callbacks) ────────────────");

    let mut stream_pipe = Pipeline::new("v16_streaming");
    stream_pipe.add_stage(Stage {
        name: "mm_stream".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 1000,
            n_patients: 64,
            seed: 7,
        },
    });
    stream_pipe.add_stage(Stage {
        name: "scfa_stream".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ScfaBatch {
            params: microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: (1..=64).map(|i| f64::from(i) * 0.5).collect(),
        },
    });

    let mut callback_count = 0usize;
    let stream_result = stream_pipe.execute_streaming(|_stage_idx, _total, _result| {
        callback_count += 1;
    });
    check(
        "Streaming: success",
        stream_result.success,
        &mut passed,
        &mut total,
    );
    check(
        "Streaming: 2 callbacks fired",
        callback_count == 2,
        &mut passed,
        &mut total,
    );
    check(
        "Streaming: matches CPU result",
        {
            let cpu = stream_pipe.execute_cpu();
            cpu.stage_results.len() == stream_result.stage_results.len()
                && cpu
                    .stage_results
                    .iter()
                    .zip(stream_result.stage_results.iter())
                    .all(|(a, b)| a.output_data == b.output_data)
        },
        &mut passed,
        &mut total,
    );

    // ── 5. GPU-mappability of all V16 stages ────────────────────────────
    println!("\n── 5. GPU-Mappability (to_gpu_op) ──────────────────────────────");

    let mm_stage = Stage {
        name: "mm".into(),
        substrate: Substrate::Gpu,
        operation: StageOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 1000,
            n_patients: 32,
            seed: 1,
        },
    };
    check(
        "MM stage maps to GpuOp",
        mm_stage.to_gpu_op(None).is_some(),
        &mut passed,
        &mut total,
    );

    let scfa_stage = Stage {
        name: "scfa".into(),
        substrate: Substrate::Gpu,
        operation: StageOp::ScfaBatch {
            params: microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: vec![5.0, 10.0, 15.0],
        },
    };
    check(
        "SCFA stage maps to GpuOp",
        scfa_stage.to_gpu_op(None).is_some(),
        &mut passed,
        &mut total,
    );

    let beat_stage = Stage {
        name: "beat".into(),
        substrate: Substrate::Gpu,
        operation: StageOp::BeatClassifyBatch {
            beats: vec![classification::generate_normal_template(41)],
            templates,
        },
    };
    check(
        "Beat stage maps to GpuOp",
        beat_stage.to_gpu_op(None).is_some(),
        &mut passed,
        &mut total,
    );

    // ── 6. Multi-stage mixed pipeline ───────────────────────────────────
    println!("\n── 6. Mixed V15+V16 Pipeline ───────────────────────────────────");

    let mut mixed = Pipeline::new("mixed_v15_v16");
    mixed.add_stage(Stage {
        name: "generate_100".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 100,
            seed: 42,
        },
    });
    mixed.add_stage(Stage {
        name: "hill_transform".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: healthspring_toadstool::stage::TransformKind::Hill {
                emax: 100.0,
                ec50: 10.0,
                n: 1.5,
            },
        },
    });
    mixed.add_stage(Stage {
        name: "reduce_mean".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: healthspring_toadstool::stage::ReduceKind::Mean,
        },
    });

    let mixed_result = mixed.execute_cpu();
    check(
        "Mixed pipe: success",
        mixed_result.success,
        &mut passed,
        &mut total,
    );
    check(
        "Mixed pipe: 3 stages",
        mixed_result.stage_results.len() == 3,
        &mut passed,
        &mut total,
    );
    check(
        "Mixed pipe: final output = 1 value",
        mixed_result.stage_results[2].output_data.len() == 1,
        &mut passed,
        &mut total,
    );
    check(
        "Mixed pipe: mean > 0",
        mixed_result.stage_results[2].output_data[0] > 0.0,
        &mut passed,
        &mut total,
    );

    // ── Summary ─────────────────────────────────────────────────────────
    println!("\n{}", "=".repeat(72));
    println!("Exp086 toadStool V16 Dispatch: {passed}/{total} PASS");
    println!("{}", "=".repeat(72));

    if passed != total {
        std::process::exit(1);
    }
}
