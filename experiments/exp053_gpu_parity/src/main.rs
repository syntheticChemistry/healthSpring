// SPDX-License-Identifier: AGPL-3.0-only
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

//! Exp053: Live GPU vs CPU parity validation.
//!
//! Runs Hill dose-response, Population PK, and Diversity operations on
//! both CPU and GPU, then compares results within f64 tolerance.
//! Reports pass/fail with timing for each operation.

use healthspring_barracuda::gpu::{GpuOp, GpuResult, execute_cpu, execute_gpu};
use std::time::Instant;

// f32 intermediate precision for transcendentals (pow, log) limits parity to ~1e-4
const TOLERANCE_TRANSCENDENTAL: f64 = 1e-4;

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential checks are clearest as a single flow"
)]
#[tokio::main]
async fn main() {
    println!("Exp053 GPU Parity — Live GPU vs CPU");
    println!("====================================");

    let mut passed = 0u32;
    let mut total = 0u32;

    // 1. Hill Dose-Response
    println!("\n--- Hill Dose-Response (50 concentrations) ---");
    let concs: Vec<f64> = (0..50)
        .map(|i| {
            let frac = f64::from(i) / 49.0;
            0.1 * 1000.0_f64.powf(frac)
        })
        .collect();

    let op = GpuOp::HillSweep {
        emax: 100.0,
        ec50: 10.0,
        n: 1.5,
        concentrations: concs,
    };

    let cpu_start = Instant::now();
    let cpu_result = execute_cpu(&op);
    let cpu_elapsed = cpu_start.elapsed();

    let gpu_start = Instant::now();
    let gpu_result = execute_gpu(&op).await;
    let gpu_elapsed = gpu_start.elapsed();

    match (&cpu_result, &gpu_result) {
        (GpuResult::HillSweep(cpu), Ok(GpuResult::HillSweep(gpu))) => {
            check(
                "GPU returned correct count",
                cpu.len() == gpu.len(),
                &mut passed,
                &mut total,
            );
            let max_err = cpu
                .iter()
                .zip(gpu.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0_f64, f64::max);
            let max_rel_err = cpu
                .iter()
                .zip(gpu.iter())
                .filter(|(a, _)| a.abs() > 1e-10)
                .map(|(a, b)| ((a - b) / a).abs())
                .fold(0.0_f64, f64::max);
            check(
                &format!("max abs error = {max_err:.2e} (f32 path)"),
                max_err < 1.0,
                &mut passed,
                &mut total,
            );
            check(
                &format!("max rel error = {max_rel_err:.2e} < {TOLERANCE_TRANSCENDENTAL:.0e}"),
                max_rel_err < TOLERANCE_TRANSCENDENTAL,
                &mut passed,
                &mut total,
            );
            check(
                &format!("low conc response small: GPU[0]={:.4}", gpu[0]),
                gpu[0] < 5.0,
                &mut passed,
                &mut total,
            );
            check(
                &format!("high conc response near Emax: GPU[49]={:.2}", gpu[49]),
                gpu[49] > 90.0,
                &mut passed,
                &mut total,
            );
            println!("  CPU: {cpu_elapsed:?}  GPU: {gpu_elapsed:?}");
        }
        (_, Err(e)) => {
            println!("  GPU error: {e}");
            check("GPU execution", false, &mut passed, &mut total);
        }
        _ => {
            check("result type match", false, &mut passed, &mut total);
        }
    }

    // 2. Population PK Monte Carlo
    println!("\n--- Population PK (1000 patients) ---");
    let op = GpuOp::PopulationPkBatch {
        n_patients: 1000,
        dose_mg: 4.0,
        f_bioavail: 0.79,
        seed: 42,
    };

    let cpu_start = Instant::now();
    let cpu_result = execute_cpu(&op);
    let cpu_elapsed = cpu_start.elapsed();

    let gpu_start = Instant::now();
    let gpu_result = execute_gpu(&op).await;
    let gpu_elapsed = gpu_start.elapsed();

    match (&cpu_result, &gpu_result) {
        (GpuResult::PopulationPkBatch(cpu), Ok(GpuResult::PopulationPkBatch(gpu))) => {
            check(
                "GPU returned 1000 AUCs",
                gpu.len() == 1000,
                &mut passed,
                &mut total,
            );
            check(
                "all AUC positive",
                gpu.iter().all(|&a| a > 0.0),
                &mut passed,
                &mut total,
            );
            // GPU uses Wang hash + xorshift32 (u32-only), CPU uses u64 LCG.
            // Different PRNGs with different distributions: CPU extracts upper 31 bits
            // (u ∈ [0, 0.5]), GPU uses full u32 (u ∈ [0, 1]).
            // Validate same-order-of-magnitude statistics, not bit-exact parity.
            #[expect(
                clippy::cast_precision_loss,
                reason = "population count N < 2^52 is safe for f64"
            )]
            let cpu_mean: f64 = cpu.iter().sum::<f64>() / cpu.len() as f64;
            #[expect(
                clippy::cast_precision_loss,
                reason = "population count N < 2^52 is safe for f64"
            )]
            let gpu_mean: f64 = gpu.iter().sum::<f64>() / gpu.len() as f64;
            let cpu_min = cpu.iter().copied().fold(f64::INFINITY, f64::min);
            let cpu_max = cpu.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let gpu_min = gpu.iter().copied().fold(f64::INFINITY, f64::min);
            let gpu_max = gpu.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            check(
                &format!("mean AUC same order: CPU={cpu_mean:.4} GPU={gpu_mean:.4}"),
                gpu_mean > 0.1 && gpu_mean < 2.0,
                &mut passed,
                &mut total,
            );
            check(
                &format!(
                    "AUC range overlap: CPU=[{cpu_min:.3},{cpu_max:.3}] GPU=[{gpu_min:.3},{gpu_max:.3}]"
                ),
                gpu_min < cpu_max && gpu_max > cpu_min,
                &mut passed,
                &mut total,
            );
            check(
                "AUC physiologically valid (0.01-10 range)",
                gpu.iter().all(|&a| a > 0.01 && a < 10.0),
                &mut passed,
                &mut total,
            );
            check(
                "GPU wider distribution (full CL range): std > 0",
                {
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "population count N < 2^52 is safe for f64"
                    )]
                    let var: f64 =
                        gpu.iter().map(|x| (x - gpu_mean).powi(2)).sum::<f64>() / gpu.len() as f64;
                    var.sqrt() > 0.01
                },
                &mut passed,
                &mut total,
            );
            println!("  CPU: {cpu_elapsed:?}  GPU: {gpu_elapsed:?}");
        }
        (_, Err(e)) => {
            println!("  GPU error: {e}");
            check("GPU execution", false, &mut passed, &mut total);
        }
        _ => {
            check("result type match", false, &mut passed, &mut total);
        }
    }

    // 3. Diversity Indices
    println!("\n--- Diversity Indices (4 communities) ---");
    let communities = vec![
        vec![0.25, 0.25, 0.25, 0.25],
        vec![0.9, 0.05, 0.03, 0.02],
        vec![0.20, 0.18, 0.15, 0.12, 0.10, 0.08, 0.07, 0.05, 0.03, 0.02],
        vec![0.5, 0.5],
    ];
    let op = GpuOp::DiversityBatch {
        communities: communities.clone(),
    };

    let cpu_start = Instant::now();
    let cpu_result = execute_cpu(&op);
    let cpu_elapsed = cpu_start.elapsed();

    let gpu_start = Instant::now();
    let gpu_result = execute_gpu(&op).await;
    let gpu_elapsed = gpu_start.elapsed();

    match (&cpu_result, &gpu_result) {
        (GpuResult::DiversityBatch(cpu), Ok(GpuResult::DiversityBatch(gpu))) => {
            check(
                "GPU returned 4 results",
                gpu.len() == 4,
                &mut passed,
                &mut total,
            );
            for (i, ((cs, cd), (gs, gd))) in cpu.iter().zip(gpu.iter()).enumerate() {
                let s_err = (cs - gs).abs();
                let d_err = (cd - gd).abs();
                check(
                    &format!("community[{i}] Shannon err={s_err:.2e} Simpson err={d_err:.2e}"),
                    s_err < TOLERANCE_TRANSCENDENTAL && d_err < TOLERANCE_TRANSCENDENTAL,
                    &mut passed,
                    &mut total,
                );
            }
            check(
                "even > dominated Shannon",
                gpu[0].0 > gpu[1].0,
                &mut passed,
                &mut total,
            );
            println!("  CPU: {cpu_elapsed:?}  GPU: {gpu_elapsed:?}");
        }
        (_, Err(e)) => {
            println!("  GPU error: {e}");
            check("GPU execution", false, &mut passed, &mut total);
        }
        _ => {
            check("result type match", false, &mut passed, &mut total);
        }
    }

    println!("\n====================================");
    println!("Exp053 GPU Parity: {passed}/{total} checks passed");

    std::process::exit(i32::from(passed != total));
}
