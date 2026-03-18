// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
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
use healthspring_barracuda::tolerances::MACHINE_EPSILON;
use healthspring_barracuda::validation::ValidationHarness;
use healthspring_forge::Substrate;
use healthspring_toadstool::pipeline::Pipeline;
use healthspring_toadstool::stage::{Stage, StageOp};

fn main() {
    let mut h = ValidationHarness::new("exp086_toadstool_v16_dispatch");

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
    h.check_bool("MM pipe: success", mm_result.success);
    h.check_exact("MM pipe: 1 stage", mm_result.stage_results.len() as u64, 1);
    h.check_exact(
        "MM pipe: 128 AUCs",
        mm_result.stage_results[0].output_data.len() as u64,
        128,
    );
    h.check_bool(
        "MM pipe: all AUC > 0",
        mm_result.stage_results[0]
            .output_data
            .iter()
            .all(|&v| v > 0.0),
    );
    h.check_bool("MM pipe: timing > 0", mm_result.total_time_us > 0.0);

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
    h.check_bool("SCFA pipe: success", scfa_result.success);
    h.check_exact(
        "SCFA pipe: 600 values (200×3)",
        scfa_result.stage_results[0].output_data.len() as u64,
        600,
    );
    h.check_bool(
        "SCFA pipe: all values > 0",
        scfa_result.stage_results[0]
            .output_data
            .iter()
            .all(|&v| v > 0.0),
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
    h.check_bool("Beat pipe: success", beat_result.success);
    h.check_exact(
        "Beat pipe: 8 values (4 beats × [class, corr])",
        beat_result.stage_results[0].output_data.len() as u64,
        8,
    );

    let beat_data = &beat_result.stage_results[0].output_data;
    h.check_abs(
        "Beat pipe: beat[0] → Normal (0.0)",
        beat_data[0],
        0.0,
        MACHINE_EPSILON,
    );
    h.check_abs(
        "Beat pipe: beat[1] → PVC (1.0)",
        beat_data[2],
        1.0,
        MACHINE_EPSILON,
    );
    h.check_abs(
        "Beat pipe: beat[2] → PAC (2.0)",
        beat_data[4],
        2.0,
        MACHINE_EPSILON,
    );
    h.check_bool(
        "Beat pipe: correlations > 0.99",
        beat_data.chunks(2).all(|pair| pair[1] > 0.99),
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
    h.check_bool("Streaming: success", stream_result.success);
    h.check_exact("Streaming: 2 callbacks fired", callback_count as u64, 2);
    h.check_bool("Streaming: matches CPU result", {
        let cpu = stream_pipe.execute_cpu();
        cpu.stage_results.len() == stream_result.stage_results.len()
            && cpu
                .stage_results
                .iter()
                .zip(stream_result.stage_results.iter())
                .all(|(a, b)| a.output_data == b.output_data)
    });

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
    h.check_bool("MM stage maps to GpuOp", mm_stage.to_gpu_op(None).is_some());

    let scfa_stage = Stage {
        name: "scfa".into(),
        substrate: Substrate::Gpu,
        operation: StageOp::ScfaBatch {
            params: microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: vec![5.0, 10.0, 15.0],
        },
    };
    h.check_bool(
        "SCFA stage maps to GpuOp",
        scfa_stage.to_gpu_op(None).is_some(),
    );

    let beat_stage = Stage {
        name: "beat".into(),
        substrate: Substrate::Gpu,
        operation: StageOp::BeatClassifyBatch {
            beats: vec![classification::generate_normal_template(41)],
            templates,
        },
    };
    h.check_bool(
        "Beat stage maps to GpuOp",
        beat_stage.to_gpu_op(None).is_some(),
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
    h.check_bool("Mixed pipe: success", mixed_result.success);
    h.check_exact(
        "Mixed pipe: 3 stages",
        mixed_result.stage_results.len() as u64,
        3,
    );
    h.check_exact(
        "Mixed pipe: final output = 1 value",
        mixed_result.stage_results[2].output_data.len() as u64,
        1,
    );
    h.check_bool(
        "Mixed pipe: mean > 0",
        mixed_result.stage_results[2].output_data[0] > 0.0,
    );

    h.exit();
}
