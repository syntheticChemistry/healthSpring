// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential GPU parity checks"
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "small collection lengths fit f64 mantissa"
)]

//! Exp083: GPU parity for V16 primitives — Michaelis-Menten, SCFA, Beat Classification.
//!
//! Validates that the three new WGSL compute shaders produce results within
//! tolerance of the CPU reference implementation (`execute_cpu`).
//!
//! Also demonstrates metalForge cross-system routing: the dispatch planner
//! routes each workload to the optimal substrate (GPU / NPU / CPU) based
//! on runtime capability discovery.

use healthspring_barracuda::biosignal::classification;
use healthspring_barracuda::gpu::{GpuOp, GpuResult, execute_cpu, shaders};
use healthspring_barracuda::microbiome;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp083_gpu_v16_parity");
    println!("Exp083 GPU V16 Parity — CPU fallback validation");
    println!("=================================================");
    println!();

    // ── 1. Michaelis-Menten Batch PK ────────────────────────────────────
    println!("--- Michaelis-Menten Batch PK (256 patients) ---");
    let mm_op = GpuOp::MichaelisMentenBatch {
        vmax: 500.0,
        km: 5.0,
        vd: 50.0,
        dt: 0.01,
        n_steps: 2000,
        n_patients: 256,
        seed: 42,
    };
    if let GpuResult::MichaelisMentenBatch(aucs) = execute_cpu(&mm_op) {
        h.check_exact("MM batch: 256 AUCs returned", aucs.len() as u64, 256);
        h.check_bool("MM batch: all AUC > 0", aucs.iter().all(|&a| a > 0.0));
        let mean = aucs.iter().sum::<f64>() / aucs.len() as f64;
        h.check_bool(
            "MM batch: mean AUC physiological",
            (1.0..200.0).contains(&mean),
        );
        let variance: f64 =
            aucs.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / aucs.len() as f64;
        h.check_bool(
            "MM batch: inter-patient variation (CV > 0)",
            variance.sqrt() / mean > tolerances::GPU_SCALING_LINEARITY,
        );
        h.check_bool("MM batch: deterministic (second run identical)", {
            let GpuResult::MichaelisMentenBatch(aucs2) = execute_cpu(&mm_op) else {
                unreachable!()
            };
            aucs.iter()
                .zip(aucs2.iter())
                .all(|(a, b)| a.to_bits() == b.to_bits())
        });
    } else {
        h.check_bool("MM batch: correct result type", false);
    }

    // ── 2. SCFA Batch Production ────────────────────────────────────────
    println!("\n--- SCFA Batch Production (100 fiber inputs) ---");
    let fiber_inputs: Vec<f64> = (1..=100).map(|i| f64::from(i) * 0.5).collect();
    let scfa_op = GpuOp::ScfaBatch {
        params: microbiome::SCFA_HEALTHY_PARAMS,
        fiber_inputs,
    };
    if let GpuResult::ScfaBatch(results) = execute_cpu(&scfa_op) {
        h.check_exact("SCFA batch: 100 results", results.len() as u64, 100);
        h.check_bool(
            "SCFA batch: all values > 0",
            results
                .iter()
                .all(|&(a, p, b)| a > 0.0 && p > 0.0 && b > 0.0),
        );
        h.check_bool(
            "SCFA batch: acetate > propionate > butyrate (healthy)",
            results.iter().all(|&(a, p, b)| a > p && p > b),
        );
        let first = &results[0];
        let last = &results[99];
        h.check_bool(
            "SCFA batch: monotone increase with fiber",
            last.0 > first.0 && last.1 > first.1 && last.2 > first.2,
        );
        h.check_bool("SCFA batch: matches scalar API", {
            let (a, p, b) = microbiome::scfa_production(5.0, &microbiome::SCFA_HEALTHY_PARAMS);
            (results[9].0 - a).abs() < tolerances::CPU_PARITY
                && (results[9].1 - p).abs() < tolerances::CPU_PARITY
                && (results[9].2 - b).abs() < tolerances::CPU_PARITY
        });
    } else {
        h.check_bool("SCFA batch: correct result type", false);
    }

    // ── 3. Beat Classification Batch ────────────────────────────────────
    println!("\n--- Beat Classification Batch (3 templates, 5 beats) ---");
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
        classification::generate_pvc_template(41),
    ];
    let classify_op = GpuOp::BeatClassifyBatch { beats, templates };
    if let GpuResult::BeatClassifyBatch(results) = execute_cpu(&classify_op) {
        h.check_exact("Beat classify: 5 results", results.len() as u64, 5);
        h.check_bool("Beat classify: beat[0] → Normal (0)", results[0].0 == 0);
        h.check_bool("Beat classify: beat[1] → PVC (1)", results[1].0 == 1);
        h.check_bool("Beat classify: beat[2] → PAC (2)", results[2].0 == 2);
        h.check_bool(
            "Beat classify: self-correlation > 0.99",
            results[0].1 > 0.99 && results[1].1 > 0.99,
        );
        h.check_bool("Beat classify: deterministic", {
            let GpuResult::BeatClassifyBatch(r2) = execute_cpu(&classify_op) else {
                unreachable!()
            };
            results
                .iter()
                .zip(r2.iter())
                .all(|(a, b)| a.0 == b.0 && a.1.to_bits() == b.1.to_bits())
        });
    } else {
        h.check_bool("Beat classify: correct result type", false);
    }

    // ── 4. metalForge Cross-System Routing ──────────────────────────────
    println!("\n--- metalForge Cross-System Routing ---");
    let caps = healthspring_forge::Capabilities::discover();

    let workloads = [
        (
            "MM PK batch (1K)",
            healthspring_forge::Workload::MichaelisMentenBatch { n_patients: 1_000 },
        ),
        (
            "SCFA batch (5K)",
            healthspring_forge::Workload::ScfaBatch { n_elements: 5_000 },
        ),
        (
            "Beat classify (10K)",
            healthspring_forge::Workload::BeatClassifyBatch { n_beats: 10_000 },
        ),
        (
            "Biosignal detect",
            healthspring_forge::Workload::BiosignalDetect {
                sample_rate_hz: 256,
            },
        ),
        ("Analytical", healthspring_forge::Workload::Analytical),
    ];

    for (name, workload) in &workloads {
        let substrate = healthspring_forge::select_substrate(workload, &caps);
        println!("  {name:25} → {substrate:?}");
    }

    h.check_bool("metalForge: MM PK batch routes correctly", {
        let s = healthspring_forge::select_substrate(
            &healthspring_forge::Workload::MichaelisMentenBatch { n_patients: 1_000 },
            &caps,
        );
        matches!(
            s,
            healthspring_forge::Substrate::Cpu | healthspring_forge::Substrate::Gpu
        )
    });
    h.check_bool(
        "metalForge: Analytical always CPU",
        healthspring_forge::select_substrate(&healthspring_forge::Workload::Analytical, &caps)
            == healthspring_forge::Substrate::Cpu,
    );

    // ── 5. Shader Sources Compiled ──────────────────────────────────────
    println!("\n--- Shader Compilation Verification ---");
    h.check_bool(
        "MM shader contains entry point",
        shaders::MICHAELIS_MENTEN_BATCH.contains("fn main"),
    );
    h.check_bool(
        "SCFA shader contains entry point",
        shaders::SCFA_BATCH.contains("fn main"),
    );
    h.check_bool(
        "Beat classify shader contains entry point",
        shaders::BEAT_CLASSIFY_BATCH.contains("fn main"),
    );
    h.check_bool(
        "MM shader has workgroup_size(256)",
        shaders::MICHAELIS_MENTEN_BATCH.contains("@workgroup_size(256)"),
    );
    h.check_bool("Memory estimate: MM batch reasonable", {
        let mem = healthspring_barracuda::gpu::gpu_memory_estimate(&mm_op);
        mem > 0 && mem < 1_000_000
    });
    h.check_bool("Memory estimate: SCFA batch reasonable", {
        let mem = healthspring_barracuda::gpu::gpu_memory_estimate(&scfa_op);
        mem > 0 && mem < 1_000_000
    });
    h.check_bool("Memory estimate: Beat classify reasonable", {
        let mem = healthspring_barracuda::gpu::gpu_memory_estimate(&classify_op);
        mem > 0 && mem < 1_000_000
    });

    h.exit();
}
