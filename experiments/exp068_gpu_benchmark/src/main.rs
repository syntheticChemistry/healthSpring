// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "validation binary — benchmark sizing and linear check sequence"
)]

//! Exp068: GPU benchmark with crossover analysis.
//!
//! Measures CPU execution times at various scales to determine the crossover
//! point where GPU would beat CPU. On CPU-only machines, validates CPU timing
//! and writes crossover estimates based on known GPU dispatch overhead.

use healthspring_barracuda::gpu::{GpuOp, execute_cpu};
use healthspring_barracuda::validation::ValidationHarness;
use serde::Serialize;

const GPU_DISPATCH_OVERHEAD_US: f64 = 200.0;

#[derive(Serialize)]
struct ScalePoint {
    n_elements: usize,
    cpu_mean_us: f64,
    estimated_gpu_us: f64,
    gpu_wins: bool,
}

#[derive(Serialize)]
struct CrossoverResult {
    operation: String,
    crossover_n: Option<usize>,
    scale_points: Vec<ScalePoint>,
}

fn bench_cpu_op(op: &GpuOp, n_iter: usize) -> f64 {
    let mut total_us = 0.0;
    for _ in 0..n_iter {
        let start = std::time::Instant::now();
        std::hint::black_box(execute_cpu(op));
        total_us += start.elapsed().as_nanos() as f64 / 1000.0;
    }
    total_us / n_iter as f64
}

fn main() {
    let mut h = ValidationHarness::new("exp068_gpu_benchmark");
    let mut crossovers = Vec::new();

    println!("Exp068: GPU Benchmark — Crossover Analysis");
    println!("============================================");

    // Hill sweep crossover
    println!("\n=== Hill Sweep Crossover ===");
    let scales = [10, 100, 1_000, 10_000, 100_000];
    let mut hill_points = Vec::new();
    let mut hill_crossover = None;

    for &n in &scales {
        let concs: Vec<f64> = (0..n)
            .map(|i| 0.1 * 1000.0_f64.powf(i as f64 / (n as f64 - 1.0).max(1.0)))
            .collect();
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 1.5,
            concentrations: concs,
        };
        let cpu_us = bench_cpu_op(&op, 20);
        let est_gpu_us = (n as f64).mul_add(0.001, GPU_DISPATCH_OVERHEAD_US);
        let gpu_wins = est_gpu_us < cpu_us;
        if gpu_wins && hill_crossover.is_none() {
            hill_crossover = Some(n);
        }
        println!(
            "  N={n:<8} CPU={cpu_us:.1}us  GPU_est={est_gpu_us:.1}us  winner={}",
            if gpu_wins { "GPU" } else { "CPU" }
        );
        hill_points.push(ScalePoint {
            n_elements: n,
            cpu_mean_us: cpu_us,
            estimated_gpu_us: est_gpu_us,
            gpu_wins,
        });
    }
    h.check_bool(
        "hill_cpu_scales_with_n",
        hill_points
            .last()
            .is_some_and(|p| p.cpu_mean_us > hill_points[0].cpu_mean_us),
    );
    crossovers.push(CrossoverResult {
        operation: "HillSweep".to_string(),
        crossover_n: hill_crossover,
        scale_points: hill_points,
    });

    // Population PK crossover
    println!("\n=== Population PK Crossover ===");
    let pk_scales = [10, 100, 1_000, 10_000];
    let mut pk_points = Vec::new();
    let mut pk_crossover = None;

    for &n in &pk_scales {
        let op = GpuOp::PopulationPkBatch {
            n_patients: n,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        };
        let cpu_us = bench_cpu_op(&op, 20);
        let est_gpu_us = (n as f64).mul_add(0.005, GPU_DISPATCH_OVERHEAD_US);
        let gpu_wins = est_gpu_us < cpu_us;
        if gpu_wins && pk_crossover.is_none() {
            pk_crossover = Some(n);
        }
        println!(
            "  N={n:<8} CPU={cpu_us:.1}us  GPU_est={est_gpu_us:.1}us  winner={}",
            if gpu_wins { "GPU" } else { "CPU" }
        );
        pk_points.push(ScalePoint {
            n_elements: n,
            cpu_mean_us: cpu_us,
            estimated_gpu_us: est_gpu_us,
            gpu_wins,
        });
    }
    h.check_bool(
        "pk_cpu_scales_with_n",
        pk_points
            .last()
            .is_some_and(|p| p.cpu_mean_us > pk_points[0].cpu_mean_us),
    );
    crossovers.push(CrossoverResult {
        operation: "PopulationPkBatch".to_string(),
        crossover_n: pk_crossover,
        scale_points: pk_points,
    });

    // Diversity crossover
    println!("\n=== Diversity Batch Crossover ===");
    let div_scales = [10, 100, 1_000, 5_000];
    let mut div_points = Vec::new();
    let mut div_crossover = None;

    for &n in &div_scales {
        let communities: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                let mut a = vec![0.0; 7];
                let mut total = 0.0;
                for (j, val) in a.iter_mut().enumerate() {
                    *val = ((i * 7 + j + 1) as f64).sqrt();
                    total += *val;
                }
                for val in &mut a {
                    *val /= total;
                }
                a
            })
            .collect();
        let op = GpuOp::DiversityBatch { communities };
        let cpu_us = bench_cpu_op(&op, 10);
        let est_gpu_us = (n as f64).mul_add(0.01, GPU_DISPATCH_OVERHEAD_US);
        let gpu_wins = est_gpu_us < cpu_us;
        if gpu_wins && div_crossover.is_none() {
            div_crossover = Some(n);
        }
        println!(
            "  N={n:<8} CPU={cpu_us:.1}us  GPU_est={est_gpu_us:.1}us  winner={}",
            if gpu_wins { "GPU" } else { "CPU" }
        );
        div_points.push(ScalePoint {
            n_elements: n,
            cpu_mean_us: cpu_us,
            estimated_gpu_us: est_gpu_us,
            gpu_wins,
        });
    }
    h.check_bool(
        "div_cpu_scales_with_n",
        div_points
            .last()
            .is_some_and(|p| p.cpu_mean_us > div_points[0].cpu_mean_us),
    );
    crossovers.push(CrossoverResult {
        operation: "DiversityBatch".to_string(),
        crossover_n: div_crossover,
        scale_points: div_points,
    });

    // Write results
    let json = serde_json::to_string_pretty(&crossovers).unwrap_or_default();
    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../control/scripts/bench_results_gpu_crossover.json");
    std::fs::write(&out_path, &json).unwrap_or_else(|_| println!("{json}"));
    println!("\nCrossover results written to {}", out_path.display());

    // Summary
    println!("\n=== Crossover Summary ===");
    for co in &crossovers {
        if let Some(n) = co.crossover_n {
            println!("  {}: GPU wins at N >= {n}", co.operation);
        } else {
            println!("  {}: CPU wins at all tested scales", co.operation);
        }
    }

    h.exit();
}
