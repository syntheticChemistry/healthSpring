// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! Exp055: GPU scaling benchmark — single GPU at population scale.
//!
//! Sweeps problem sizes from 1K to 10M to show:
//! - CPU/GPU crossover point for each operation
//! - GPU throughput scaling (elements/second)
//! - Fused pipeline advantage at scale
//!
//! The thesis: a single GPU where people are (Pi + eGPU, edge node, clinic
//! laptop) handles population-scale health computations without infrastructure.

use healthspring_barracuda::gpu::{GpuContext, GpuOp, GpuResult, execute_cpu};
use std::time::Instant;

const HILL_SIZES: &[usize] = &[1_000, 10_000, 100_000, 1_000_000, 5_000_000, 10_000_000];
const PK_SIZES: &[usize] = &[1_000, 10_000, 100_000, 1_000_000, 5_000_000, 10_000_000];
const DIV_COMMUNITY_SIZES: &[usize] = &[10, 100, 1_000, 5_000, 10_000];
const DIV_SPECIES: usize = 50;

fn make_concentrations(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss)]
            let frac = i as f64 / (n.max(1) - 1).max(1) as f64;
            0.01 * 1000.0_f64.powf(frac)
        })
        .collect()
}

#[expect(clippy::cast_precision_loss)]
fn make_communities(n_communities: usize) -> Vec<Vec<f64>> {
    (0..n_communities)
        .map(|seed| {
            let mut abundances = Vec::with_capacity(DIV_SPECIES);
            let mut total = 0.0;
            let mut state = (seed as u64) + 1;
            for _ in 0..DIV_SPECIES {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let v = (state >> 33) as f64 / f64::from(u32::MAX) + 0.01;
                abundances.push(v);
                total += v;
            }
            for a in &mut abundances {
                *a /= total;
            }
            abundances
        })
        .collect()
}

fn format_count(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{n}")
    }
}

fn format_rate(n: usize, secs: f64) -> String {
    #[expect(clippy::cast_precision_loss)]
    let rate = n as f64 / secs;
    if rate >= 1e9 {
        format!("{:.1} G/s", rate / 1e9)
    } else if rate >= 1e6 {
        format!("{:.1} M/s", rate / 1e6)
    } else if rate >= 1e3 {
        format!("{:.1} K/s", rate / 1e3)
    } else {
        format!("{rate:.1} /s")
    }
}

struct BenchResult {
    n: usize,
    speedup: f64,
}

#[tokio::main]
async fn main() {
    println!("Exp055 GPU Scaling — Single GPU at Population Scale");
    println!("====================================================\n");

    let ctx = match GpuContext::new().await {
        Ok(ctx) => {
            println!("GPU: {}", ctx.adapter_name());
            ctx
        }
        Err(e) => {
            println!("No GPU available: {e}");
            println!("Cannot run scaling benchmark without GPU.");
            std::process::exit(1);
        }
    };

    // Warmup: one small dispatch to JIT compile shaders
    let warmup_op = GpuOp::HillSweep {
        emax: 100.0,
        ec50: 10.0,
        n: 1.5,
        concentrations: vec![1.0; 64],
    };
    let _ = ctx.execute(&warmup_op).await;

    // ─── Hill Dose-Response Scaling ───
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Hill Dose-Response: E(c) = Emax·c^n / (c^n + EC50^n)         ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:>8} │ {:>12} │ {:>12} │ {:>8} │ {:>12} ║",
        "N", "CPU", "GPU", "Speedup", "GPU Rate"
    );
    println!("╟──────────┼──────────────┼──────────────┼──────────┼──────────────╢");

    let mut hill_results: Vec<BenchResult> = Vec::new();
    for &n in HILL_SIZES {
        let concs = make_concentrations(n);
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 1.5,
            concentrations: concs,
        };

        let cpu_start = Instant::now();
        let _cpu = execute_cpu(&op);
        let cpu_ms = cpu_start.elapsed().as_secs_f64() * 1000.0;

        let gpu_start = Instant::now();
        let gpu_result = ctx.execute(&op).await;
        let gpu_ms = gpu_start.elapsed().as_secs_f64() * 1000.0;

        let speedup = cpu_ms / gpu_ms;
        let marker = if speedup >= 1.0 { "▲" } else { " " };

        if let Ok(GpuResult::HillSweep(ref v)) = gpu_result {
            println!(
                "║  {:>8} │ {:>9.3} ms │ {:>9.3} ms │ {:>6.2}x{} │ {:>12} ║",
                format_count(n),
                cpu_ms,
                gpu_ms,
                speedup,
                marker,
                format_rate(v.len(), gpu_ms / 1000.0)
            );
        }

        hill_results.push(BenchResult { n, speedup });
    }
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // ─── Population PK Scaling ───
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Population PK Monte Carlo: AUC = F·Dose / CL(random)         ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:>8} │ {:>12} │ {:>12} │ {:>8} │ {:>12} ║",
        "Patients", "CPU", "GPU", "Speedup", "GPU Rate"
    );
    println!("╟──────────┼──────────────┼──────────────┼──────────┼──────────────╢");

    let mut pk_results: Vec<BenchResult> = Vec::new();
    for &n in PK_SIZES {
        let op = GpuOp::PopulationPkBatch {
            n_patients: n,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        };

        let cpu_start = Instant::now();
        let _cpu = execute_cpu(&op);
        let cpu_ms = cpu_start.elapsed().as_secs_f64() * 1000.0;

        let gpu_start = Instant::now();
        let gpu_result = ctx.execute(&op).await;
        let gpu_ms = gpu_start.elapsed().as_secs_f64() * 1000.0;

        let speedup = cpu_ms / gpu_ms;
        let marker = if speedup >= 1.0 { "▲" } else { " " };

        if let Ok(GpuResult::PopulationPkBatch(ref v)) = gpu_result {
            println!(
                "║  {:>8} │ {:>9.3} ms │ {:>9.3} ms │ {:>6.2}x{} │ {:>12} ║",
                format_count(n),
                cpu_ms,
                gpu_ms,
                speedup,
                marker,
                format_rate(v.len(), gpu_ms / 1000.0)
            );
        }

        pk_results.push(BenchResult { n, speedup });
    }
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // ─── Diversity Index Scaling ───
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Diversity Indices: Shannon + Simpson, {DIV_SPECIES} species/community      ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:>8} │ {:>12} │ {:>12} │ {:>8} │ {:>12} ║",
        "Commun.", "CPU", "GPU", "Speedup", "GPU Rate"
    );
    println!("╟──────────┼──────────────┼──────────────┼──────────┼──────────────╢");

    let mut div_results: Vec<BenchResult> = Vec::new();
    for &n in DIV_COMMUNITY_SIZES {
        let communities = make_communities(n);
        let op = GpuOp::DiversityBatch { communities };

        let cpu_start = Instant::now();
        let _cpu = execute_cpu(&op);
        let cpu_ms = cpu_start.elapsed().as_secs_f64() * 1000.0;

        let gpu_start = Instant::now();
        let gpu_result = ctx.execute(&op).await;
        let gpu_ms = gpu_start.elapsed().as_secs_f64() * 1000.0;

        let speedup = cpu_ms / gpu_ms;
        let marker = if speedup >= 1.0 { "▲" } else { " " };

        if gpu_result.is_ok() {
            println!(
                "║  {:>8} │ {:>9.3} ms │ {:>9.3} ms │ {:>6.2}x{} │ {:>12} ║",
                format_count(n),
                cpu_ms,
                gpu_ms,
                speedup,
                marker,
                format_rate(n * DIV_SPECIES, gpu_ms / 1000.0)
            );
        }

        div_results.push(BenchResult { n, speedup });
    }
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // ─── Fused Pipeline Scaling ───
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Fused Pipeline: Hill + PK + Diversity in ONE submission        ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  {:>8} │ {:>12} │ {:>12} │ {:>12} │ {:>8} ║",
        "Scale", "CPU", "GPU Indiv.", "GPU Fused", "Speedup"
    );
    println!("╟──────────┼──────────────┼──────────────┼──────────────┼──────────╢");

    let fused_scales: &[(usize, usize, usize)] = &[
        (1_000, 1_000, 10),
        (10_000, 10_000, 100),
        (100_000, 100_000, 1_000),
        (1_000_000, 1_000_000, 5_000),
        (5_000_000, 5_000_000, 10_000),
    ];

    for &(n_hill, n_pk, n_div) in fused_scales {
        let ops = vec![
            GpuOp::HillSweep {
                emax: 100.0,
                ec50: 10.0,
                n: 1.5,
                concentrations: make_concentrations(n_hill),
            },
            GpuOp::PopulationPkBatch {
                n_patients: n_pk,
                dose_mg: 4.0,
                f_bioavail: 0.79,
                seed: 42,
            },
            GpuOp::DiversityBatch {
                communities: make_communities(n_div),
            },
        ];

        let cpu_start = Instant::now();
        for op in &ops {
            let _ = execute_cpu(op);
        }
        let cpu_ms = cpu_start.elapsed().as_secs_f64() * 1000.0;

        let ind_start = Instant::now();
        for op in &ops {
            let _ = ctx.execute(op).await;
        }
        let ind_ms = ind_start.elapsed().as_secs_f64() * 1000.0;

        let fused_start = Instant::now();
        let _ = ctx.execute_fused(&ops).await;
        let fused_ms = fused_start.elapsed().as_secs_f64() * 1000.0;

        let speedup = cpu_ms / fused_ms;
        let label = format!(
            "{}+{}+{}",
            format_count(n_hill),
            format_count(n_pk),
            format_count(n_div)
        );
        let marker = if speedup >= 1.0 { "▲" } else { " " };

        println!(
            "║ {:>8} │ {:>9.3} ms │ {:>9.3} ms │ {:>9.3} ms │ {:>6.2}x{} ║",
            label, cpu_ms, ind_ms, fused_ms, speedup, marker
        );
    }
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // ─── Field Deployment Summary ───
    println!("\n┌──────────────────────────────────────────────────────────────────┐");
    println!("│                  FIELD DEPLOYMENT SUMMARY                       │");
    println!("├──────────────────────────────────────────────────────────────────┤");

    let hill_crossover = hill_results.iter().find(|r| r.speedup >= 1.0).map(|r| r.n);
    let pk_crossover = pk_results.iter().find(|r| r.speedup >= 1.0).map(|r| r.n);
    let div_crossover = div_results.iter().find(|r| r.speedup >= 1.0).map(|r| r.n);

    if let Some(n) = hill_crossover {
        println!(
            "│  Hill crossover:      GPU wins at ≥ {:>8}                   │",
            format_count(n)
        );
    } else {
        println!("│  Hill crossover:      CPU faster at all tested sizes          │");
    }
    if let Some(n) = pk_crossover {
        println!(
            "│  PopPK crossover:     GPU wins at ≥ {:>8}                   │",
            format_count(n)
        );
    } else {
        println!("│  PopPK crossover:     CPU faster at all tested sizes          │");
    }
    if let Some(n) = div_crossover {
        println!(
            "│  Diversity crossover: GPU wins at ≥ {:>8}                   │",
            format_count(n)
        );
    } else {
        println!("│  Diversity crossover: CPU faster at all tested sizes          │");
    }

    let peak_hill = hill_results
        .iter()
        .max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        .unwrap();
    let peak_pk = pk_results
        .iter()
        .max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        .unwrap();

    println!("│                                                                │");
    println!(
        "│  Peak Hill speedup:   {:.1}x at {}                              │",
        peak_hill.speedup,
        format_count(peak_hill.n)
    );
    println!(
        "│  Peak PK speedup:     {:.1}x at {}                              │",
        peak_pk.speedup,
        format_count(peak_pk.n)
    );
    println!("│                                                                │");
    println!("│  Thesis: one GPU where people are — not infrastructure.        │");
    println!("│  Same pipeline maps to TPU/NPU when hardware becomes native.   │");
    println!("└──────────────────────────────────────────────────────────────────┘");
}
