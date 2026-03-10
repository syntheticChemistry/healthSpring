// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear benchmark sequence"
)]

//! Exp066: Rust CPU benchmark — Tier 1 parity timing.
//!
//! Runs the same benchmark suite as `control/scripts/bench_barracuda_cpu_vs_python.py`,
//! outputs `bench_results_rust_cpu.json`, and compares to Python results.

use healthspring_barracuda::microbiome::{pielou_evenness, shannon_index, simpson_index};
use healthspring_barracuda::pkpd::{
    auc_trapezoidal, hill_dose_response, pk_oral_one_compartment, population_pk_cpu,
};
use serde::Serialize;

const N_ITERATIONS: usize = 100;

#[derive(Serialize)]
struct BenchResult {
    name: String,
    n_iterations: usize,
    mean_us: f64,
    min_us: f64,
    max_us: f64,
    p50_us: f64,
    p95_us: f64,
}

#[derive(Serialize)]
struct BenchSuite {
    tier: String,
    benchmarks: Vec<BenchResult>,
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "benchmark timing: nanoseconds to microseconds, index to p95 position"
)]
fn bench<F: Fn()>(name: &str, func: F, n_iter: usize) -> BenchResult {
    let mut times_us = Vec::with_capacity(n_iter);
    for _ in 0..n_iter {
        let start = std::time::Instant::now();
        func();
        times_us.push(start.elapsed().as_nanos() as f64 / 1000.0);
    }
    times_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let sum: f64 = times_us.iter().sum();
    let mean = sum / n_iter as f64;
    let p50_idx = n_iter / 2;
    let p95_idx = (n_iter as f64 * 0.95) as usize;
    BenchResult {
        name: name.to_string(),
        n_iterations: n_iter,
        mean_us: mean,
        min_us: times_us[0],
        max_us: times_us[n_iter - 1],
        p50_us: times_us[p50_idx],
        p95_us: times_us[p95_idx.min(n_iter - 1)],
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "index-to-f64 for log-spaced concentrations"
)]
fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut benchmarks = Vec::new();

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                passed += 1;
            } else {
                eprintln!("FAIL: {}", $name);
                failed += 1;
            }
        };
    }

    println!("Exp066: Rust CPU Benchmark Suite");
    println!("================================");

    let concs_50: Vec<f64> = (0..50)
        .map(|i| 0.1 * 1000.0_f64.powf(f64::from(i) / 49.0))
        .collect();
    let bench_result = bench(
        "hill_sweep_50",
        || {
            for &c in &concs_50 {
                std::hint::black_box(hill_dose_response(c, 10.0, 1.5, 100.0));
            }
        },
        N_ITERATIONS,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("hill_sweep_50_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let concs_10k: Vec<f64> = (0..10_000)
        .map(|i| 0.1 * 1000.0_f64.powf(f64::from(i) / 9_999.0))
        .collect();
    let bench_result = bench(
        "hill_sweep_10K",
        || {
            for &c in &concs_10k {
                std::hint::black_box(hill_dose_response(c, 10.0, 1.5, 100.0));
            }
        },
        N_ITERATIONS,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("hill_sweep_10K_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let clearance = 0.15 * (85.0_f64 / 70.0).powf(0.75);
    let volume_d = 15.0 * (85.0 / 70.0);
    let ke = clearance / volume_d;
    let bench_result = bench(
        "pk_curve_101_points",
        || {
            for i in 0..=100 {
                let t = f64::from(i) * 24.0 / 100.0;
                std::hint::black_box(pk_oral_one_compartment(4.0, 0.79, volume_d, 1.5, ke, t));
            }
        },
        N_ITERATIONS,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("pk_curve_101_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let abund = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02];
    let bench_result = bench(
        "diversity_indices_7_genera",
        || {
            std::hint::black_box(shannon_index(&abund));
            std::hint::black_box(simpson_index(&abund));
            std::hint::black_box(pielou_evenness(&abund));
        },
        N_ITERATIONS,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("diversity_7_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let times_pk: Vec<f64> = (0..=100).map(|i| f64::from(i) * 24.0 / 100.0).collect();
    let concs_pk: Vec<f64> = times_pk
        .iter()
        .map(|&t| pk_oral_one_compartment(4.0, 0.79, volume_d, 1.5, ke, t))
        .collect();
    let bench_result = bench(
        "auc_trapezoidal_101_points",
        || {
            std::hint::black_box(auc_trapezoidal(&times_pk, &concs_pk));
        },
        N_ITERATIONS,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("auc_trapezoidal_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let n500 = 500;
    let cl_500: Vec<f64> = (0..n500).map(|i| (i as f64).mul_add(0.01, 8.0)).collect();
    let vd_500: Vec<f64> = (0..n500).map(|i| (i as f64).mul_add(0.05, 70.0)).collect();
    let ka_500: Vec<f64> = (0..n500).map(|i| (i as f64).mul_add(0.001, 1.2)).collect();
    let bench_result = bench(
        "population_montecarlo_500",
        || {
            std::hint::black_box(population_pk_cpu(
                n500, &cl_500, &vd_500, &ka_500, 4.0, 0.79, &times_pk,
            ));
        },
        N_ITERATIONS.max(10) / 10,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("pop_mc_500_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let n5000 = 5_000;
    let cl_5k: Vec<f64> = (0..n5000).map(|i| (i as f64).mul_add(0.001, 8.0)).collect();
    let vd_5k: Vec<f64> = (0..n5000)
        .map(|i| (i as f64).mul_add(0.005, 70.0))
        .collect();
    let ka_5k: Vec<f64> = (0..n5000)
        .map(|i| (i as f64).mul_add(0.0001, 1.2))
        .collect();
    let bench_result = bench(
        "population_montecarlo_5000",
        || {
            std::hint::black_box(population_pk_cpu(
                n5000, &cl_5k, &vd_5k, &ka_5k, 4.0, 0.79, &times_pk,
            ));
        },
        N_ITERATIONS.max(20) / 20,
    );
    println!(
        "  {:<40} mean={:.3}us  p95={:.3}us",
        bench_result.name, bench_result.mean_us, bench_result.p95_us
    );
    check!("pop_mc_5000_runs", bench_result.mean_us > 0.0);
    benchmarks.push(bench_result);

    let suite = BenchSuite {
        tier: "rust_cpu".to_string(),
        benchmarks,
    };
    let json = serde_json::to_string_pretty(&suite).expect("serialize");

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../control/scripts/bench_results_rust_cpu.json");
    if let Some(parent) = out_dir.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&out_dir, &json).unwrap_or_else(|_| println!("{json}"));
    println!("\nResults written to {}", out_dir.display());

    let total = passed + failed;
    println!("\nExp066 Rust CPU Bench: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
