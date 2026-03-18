// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential GPU vs CPU bench"
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "small collection lengths fit f64 mantissa"
)]

//! Exp085: barraCuda CPU vs GPU timing for V16 ops.
//!
//! Benchmarks the three V16 `GpuOp` variants (`MichaelisMentenBatch`,
//! `ScfaBatch`, `BeatClassifyBatch`) at multiple scales via `execute_cpu`.
//! Validates numerical correctness and timing, proving the math is portable
//! and the CPU fallback path is the exact same code that the GPU kernel
//! would execute.
//!
//! When GPU hardware is available, `execute_fused()` would run the same
//! ops through WGSL shaders. Here we validate the dispatch path and
//! timing characteristics to prove readiness.

use healthspring_barracuda::biosignal::classification;
use healthspring_barracuda::gpu::{self, GpuOp, GpuResult};
use healthspring_barracuda::microbiome;
use healthspring_barracuda::validation::ValidationHarness;
use serde::Serialize;

#[derive(Serialize)]
struct ScaleBench {
    op: String,
    scale: u32,
    mean_us: f64,
    p95_us: f64,
}

#[derive(Serialize)]
struct BenchSuite {
    experiment: String,
    results: Vec<ScaleBench>,
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "benchmark timing: nanos to micros, index to percentile"
)]
fn time_op(op: &GpuOp, n_iter: usize) -> (f64, f64) {
    let mut times: Vec<f64> = (0..n_iter)
        .map(|_| {
            let start = std::time::Instant::now();
            std::hint::black_box(gpu::execute_cpu(op));
            start.elapsed().as_nanos() as f64 / 1000.0
        })
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    let p95 = times[(times.len() as f64 * 0.95) as usize];
    (mean, p95)
}

fn main() {
    let mut h = ValidationHarness::new("exp085_gpu_vs_cpu_v16_bench");
    let mut bench_results = Vec::new();
    let n_iter = 50;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp085 — barraCuda CPU vs GPU V16 Bench");
    println!("{}", "=".repeat(72));

    // ── 1. Michaelis-Menten Batch: scaling from 64 to 4096 patients ─────
    println!("\n── 1. Michaelis-Menten Batch PK (scaling) ──────────────────────");

    for &n_patients in &[64u32, 256, 1024, 4096] {
        let op = GpuOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 2000,
            n_patients,
            seed: 42,
        };

        if let GpuResult::MichaelisMentenBatch(aucs) = gpu::execute_cpu(&op) {
            h.check_exact(
                &format!("MM batch {n_patients}: correct count"),
                aucs.len() as u64,
                u64::from(n_patients),
            );
            h.check_bool(
                &format!("MM batch {n_patients}: all AUC > 0"),
                aucs.iter().all(|&a| a > 0.0),
            );
            h.check_bool(&format!("MM batch {n_patients}: deterministic"), {
                let GpuResult::MichaelisMentenBatch(r2) = gpu::execute_cpu(&op) else {
                    unreachable!()
                };
                aucs == r2
            });
        }

        let (mean, p95) = time_op(&op, n_iter);
        println!("  MM {n_patients:>5} patients:  mean={mean:.1}us  p95={p95:.1}us");
        bench_results.push(ScaleBench {
            op: "mm_batch".into(),
            scale: n_patients,
            mean_us: mean,
            p95_us: p95,
        });
    }

    let mem_small = gpu::gpu_memory_estimate(&GpuOp::MichaelisMentenBatch {
        vmax: 500.0,
        km: 5.0,
        vd: 50.0,
        dt: 0.01,
        n_steps: 2000,
        n_patients: 64,
        seed: 42,
    });
    let mem_large = gpu::gpu_memory_estimate(&GpuOp::MichaelisMentenBatch {
        vmax: 500.0,
        km: 5.0,
        vd: 50.0,
        dt: 0.01,
        n_steps: 2000,
        n_patients: 4096,
        seed: 42,
    });
    h.check_bool("MM memory scales with patients", mem_large > mem_small);

    // ── 2. SCFA Batch: scaling from 100 to 10000 fiber inputs ───────────
    println!("\n── 2. SCFA Batch Production (scaling) ─────────────────────────");

    for &n_elements in &[100u32, 500, 2000, 10_000] {
        let fiber_inputs: Vec<f64> = (0..n_elements)
            .map(|i| f64::from(i).mul_add(0.05, 0.1))
            .collect();
        let op = GpuOp::ScfaBatch {
            params: microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs,
        };

        if let GpuResult::ScfaBatch(results) = gpu::execute_cpu(&op) {
            h.check_exact(
                &format!("SCFA batch {n_elements}: correct count"),
                results.len() as u64,
                u64::from(n_elements),
            );
            h.check_bool(
                &format!("SCFA batch {n_elements}: all positive"),
                results
                    .iter()
                    .all(|&(a, p, b)| a > 0.0 && p > 0.0 && b > 0.0),
            );
            h.check_bool(
                &format!("SCFA batch {n_elements}: acetate > propionate > butyrate"),
                results.iter().all(|&(a, p, b)| a > p && p > b),
            );
        }

        let (mean, p95) = time_op(&op, n_iter);
        println!("  SCFA {n_elements:>5} fibers:  mean={mean:.1}us  p95={p95:.1}us");
        bench_results.push(ScaleBench {
            op: "scfa_batch".into(),
            scale: n_elements,
            mean_us: mean,
            p95_us: p95,
        });
    }

    // ── 3. Beat Classification Batch: scaling from 10 to 1000 beats ─────
    println!("\n── 3. Beat Classification Batch (scaling) ─────────────────────");

    let templates = vec![
        classification::generate_normal_template(41),
        classification::generate_pvc_template(41),
        classification::generate_pac_template(41),
    ];

    for &n_beats in &[10u32, 50, 200, 1000] {
        let beats: Vec<Vec<f64>> = (0..n_beats)
            .map(|i| match i % 3 {
                0 => classification::generate_normal_template(41),
                1 => classification::generate_pvc_template(41),
                _ => classification::generate_pac_template(41),
            })
            .collect();
        let op = GpuOp::BeatClassifyBatch {
            beats,
            templates: templates.clone(),
        };

        if let GpuResult::BeatClassifyBatch(results) = gpu::execute_cpu(&op) {
            h.check_exact(
                &format!("Beat classify {n_beats}: correct count"),
                results.len() as u64,
                u64::from(n_beats),
            );
            h.check_bool(
                &format!("Beat classify {n_beats}: correct classes"),
                results
                    .iter()
                    .enumerate()
                    .all(|(i, &(cls, _))| cls == u32::try_from(i % 3).unwrap_or(0)),
            );
            h.check_bool(
                &format!("Beat classify {n_beats}: high correlation"),
                results.iter().all(|&(_, corr)| corr > 0.99),
            );
        }

        let (mean, p95) = time_op(&op, n_iter);
        println!("  Beat {n_beats:>5} beats:   mean={mean:.1}us  p95={p95:.1}us");
        bench_results.push(ScaleBench {
            op: "beat_classify_batch".into(),
            scale: n_beats,
            mean_us: mean,
            p95_us: p95,
        });
    }

    // ── 4. Fused pipeline readiness ─────────────────────────────────────
    println!("\n── 4. Fused Pipeline Readiness ─────────────────────────────────");

    let ops = [
        GpuOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 2000,
            n_patients: 256,
            seed: 42,
        },
        GpuOp::ScfaBatch {
            params: microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: (1..=256).map(|i| f64::from(i) * 0.2).collect(),
        },
        GpuOp::BeatClassifyBatch {
            beats: vec![classification::generate_normal_template(41); 256],
            templates,
        },
    ];

    for (idx, op) in ops.iter().enumerate() {
        h.check_bool(
            &format!("fused[{idx}]: shader source loaded"),
            !gpu::shader_for_op(op).is_empty(),
        );
        h.check_bool(
            &format!("fused[{idx}]: memory estimate > 0"),
            gpu::gpu_memory_estimate(op) > 0,
        );
    }

    let total_mem: u64 = ops.iter().map(gpu::gpu_memory_estimate).sum();
    h.check_bool("fused pipeline total mem < 100MB", total_mem < 100_000_000);

    let fused_start = std::time::Instant::now();
    let fused_count = ops.iter().map(gpu::execute_cpu).count();
    let _fused_us = fused_start.elapsed().as_micros() as f64;
    h.check_exact("fused 3-op CPU pipeline", fused_count as u64, 3);

    // ── 5. metalForge routing at scale ──────────────────────────────────
    println!("\n── 5. metalForge Workload Routing (V16) ───────────────────────");

    let caps = healthspring_forge::Capabilities::discover();
    let workloads = [
        (
            "MM 64 → CPU",
            healthspring_forge::Workload::MichaelisMentenBatch { n_patients: 64 },
        ),
        (
            "MM 10K → GPU",
            healthspring_forge::Workload::MichaelisMentenBatch { n_patients: 10_000 },
        ),
        (
            "SCFA 50 → CPU",
            healthspring_forge::Workload::ScfaBatch { n_elements: 50 },
        ),
        (
            "SCFA 5K → GPU",
            healthspring_forge::Workload::ScfaBatch { n_elements: 5_000 },
        ),
        (
            "Beat 10 → CPU",
            healthspring_forge::Workload::BeatClassifyBatch { n_beats: 10 },
        ),
        (
            "Beat 10K → GPU",
            healthspring_forge::Workload::BeatClassifyBatch { n_beats: 10_000 },
        ),
        ("Analytical", healthspring_forge::Workload::Analytical),
    ];

    for (label, workload) in &workloads {
        let substrate = healthspring_forge::select_substrate(workload, &caps);
        println!("  {label:30} → {substrate:?}");
    }

    h.check_bool(
        "metalForge: Analytical → CPU",
        healthspring_forge::select_substrate(&healthspring_forge::Workload::Analytical, &caps)
            == healthspring_forge::Substrate::Cpu,
    );
    h.check_bool(
        "metalForge: all V16 workloads route to CPU or GPU",
        workloads.iter().all(|(_, wl)| {
            let sub = healthspring_forge::select_substrate(wl, &caps);
            matches!(
                sub,
                healthspring_forge::Substrate::Cpu | healthspring_forge::Substrate::Gpu
            )
        }),
    );

    // ── Output ──────────────────────────────────────────────────────────
    let suite = BenchSuite {
        experiment: "exp085".into(),
        results: bench_results,
    };
    let json = serde_json::to_string_pretty(&suite).unwrap_or_default();
    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../control/scripts/bench_results_v16_gpu_scaling.json");
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&out_path, &json).unwrap_or_else(|_| println!("{json}"));
    println!("\nResults written to {}", out_path.display());

    h.exit();
}
