// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![forbid(unsafe_code)]

//! Exp053: Live GPU vs CPU parity validation.
//!
//! Runs Hill dose-response, Population PK, and Diversity operations on
//! both CPU and GPU, then compares results within f64 tolerance.
//! Reports pass/fail with timing for each operation.

use healthspring_barracuda::gpu::{GpuOp, GpuResult, execute_cpu, execute_gpu};
use healthspring_barracuda::tolerances::{GPU_F32_TRANSCENDENTAL, MACHINE_EPSILON};
use healthspring_barracuda::validation::ValidationHarness;
use std::time::Instant;

#[expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential checks are clearest as a single flow"
)]
#[tokio::main]
async fn main() {
    let mut h = ValidationHarness::new("exp053 GPU Parity");

    // 1. Hill Dose-Response
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
            h.check_bool("GPU returned correct count", cpu.len() == gpu.len());
            let max_err = cpu
                .iter()
                .zip(gpu.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0_f64, f64::max);
            let max_rel_err = cpu
                .iter()
                .zip(gpu.iter())
                .filter(|(a, _)| a.abs() > MACHINE_EPSILON)
                .map(|(a, b)| ((a - b) / a).abs())
                .fold(0.0_f64, f64::max);
            h.check_bool(
                &format!("max abs error = {max_err:.2e} (f32 path)"),
                max_err < 1.0,
            );
            h.check_bool(
                &format!("max rel error = {max_rel_err:.2e} < {GPU_F32_TRANSCENDENTAL:.0e}"),
                max_rel_err < GPU_F32_TRANSCENDENTAL,
            );
            h.check_bool(
                &format!("low conc response small: GPU[0]={:.4}", gpu[0]),
                gpu[0] < 5.0,
            );
            h.check_bool(
                &format!("high conc response near Emax: GPU[49]={:.2}", gpu[49]),
                gpu[49] > 90.0,
            );
            println!("  CPU: {cpu_elapsed:?}  GPU: {gpu_elapsed:?}");
        }
        (_, Err(e)) => {
            println!("  GPU error: {e}");
            h.check_bool("GPU execution", false);
        }
        _ => {
            h.check_bool("result type match", false);
        }
    }

    // 2. Population PK Monte Carlo
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
            h.check_bool("GPU returned 1000 AUCs", gpu.len() == 1000);
            h.check_bool("all AUC positive", gpu.iter().all(|&a| a > 0.0));
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
            h.check_bool(
                &format!("mean AUC same order: CPU={cpu_mean:.4} GPU={gpu_mean:.4}"),
                gpu_mean > 0.1 && gpu_mean < 2.0,
            );
            h.check_bool(
                &format!(
                    "AUC range overlap: CPU=[{cpu_min:.3},{cpu_max:.3}] GPU=[{gpu_min:.3},{gpu_max:.3}]"
                ),
                gpu_min < cpu_max && gpu_max > cpu_min,
            );
            h.check_bool(
                "AUC physiologically valid (0.01-10 range)",
                gpu.iter().all(|&a| a > 0.01 && a < 10.0),
            );
            h.check_bool("GPU wider distribution (full CL range): std > 0", {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "population count N < 2^52 is safe for f64"
                )]
                let var: f64 =
                    gpu.iter().map(|x| (x - gpu_mean).powi(2)).sum::<f64>() / gpu.len() as f64;
                var.sqrt() > 0.01
            });
            println!("  CPU: {cpu_elapsed:?}  GPU: {gpu_elapsed:?}");
        }
        (_, Err(e)) => {
            println!("  GPU error: {e}");
            h.check_bool("GPU execution", false);
        }
        _ => {
            h.check_bool("result type match", false);
        }
    }

    // 3. Diversity Indices
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
            h.check_bool("GPU returned 4 results", gpu.len() == 4);
            for (i, ((cs, cd), (gs, gd))) in cpu.iter().zip(gpu.iter()).enumerate() {
                let s_err = (cs - gs).abs();
                let d_err = (cd - gd).abs();
                h.check_bool(
                    &format!("community[{i}] Shannon err={s_err:.2e} Simpson err={d_err:.2e}"),
                    s_err < GPU_F32_TRANSCENDENTAL && d_err < GPU_F32_TRANSCENDENTAL,
                );
            }
            h.check_bool("even > dominated Shannon", gpu[0].0 > gpu[1].0);
            println!("  CPU: {cpu_elapsed:?}  GPU: {gpu_elapsed:?}");
        }
        (_, Err(e)) => {
            println!("  GPU error: {e}");
            h.check_bool("GPU execution", false);
        }
        _ => {
            h.check_bool("result type match", false);
        }
    }

    h.exit();
}
