// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

//! Exp054: GPU-native fused pipeline benchmark.
//!
//! Compares three execution modes for the full healthSpring diagnostic workload:
//! 1. CPU-only (pure Rust, sequential)
//! 2. GPU individual (3 separate dispatches, each with device roundtrip)
//! 3. GPU fused pipeline (single command encoder, all ops, one submit)
//!
//! The fused pipeline is the unidirectional pattern required for field
//! deployment (Raspberry Pi + eGPU) where CPU is the bottleneck and
//! GPU-native execution eliminates CPU roundtrips.

use healthspring_barracuda::gpu::{GpuContext, GpuOp, GpuResult, execute_cpu};
use healthspring_forge::Substrate;
use healthspring_toadstool::pipeline::Pipeline;
use healthspring_toadstool::stage::{Stage, StageOp, TransformKind};
use std::time::Instant;

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

fn build_workload() -> Vec<GpuOp> {
    let concs: Vec<f64> = (0..1000)
        .map(|i| {
            let frac = f64::from(i) / 999.0;
            0.01 * 1000.0_f64.powf(frac)
        })
        .collect();

    let communities: Vec<Vec<f64>> = (0..20)
        .map(|seed| {
            let n = 50;
            let mut abundances = Vec::with_capacity(n);
            let mut total = 0.0;
            #[expect(clippy::cast_sign_loss, reason = "seed from 0..20 is non-negative")]
            let mut state = (seed as u64) + 1;
            for _ in 0..n {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                #[expect(clippy::cast_precision_loss)]
                let v = (state >> 33) as f64 / f64::from(u32::MAX) + 0.01;
                abundances.push(v);
                total += v;
            }
            for a in &mut abundances {
                *a /= total;
            }
            abundances
        })
        .collect();

    vec![
        GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 1.5,
            concentrations: concs,
        },
        GpuOp::PopulationPkBatch {
            n_patients: 10_000,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        },
        GpuOp::DiversityBatch { communities },
    ]
}

#[expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential pipeline checks"
)]
#[tokio::main]
async fn main() {
    println!("Exp054 GPU-Native Fused Pipeline");
    println!("=================================");
    println!("Workload: 1K Hill concs + 10K PK patients + 20×50 diversity communities\n");

    let mut passed = 0u32;
    let mut total = 0u32;
    let ops = build_workload();

    // --- Mode 1: CPU ---
    println!("--- Mode 1: CPU (pure Rust) ---");
    let cpu_start = Instant::now();
    let cpu_results: Vec<GpuResult> = ops.iter().map(execute_cpu).collect();
    let cpu_elapsed = cpu_start.elapsed();
    println!("  CPU total: {cpu_elapsed:?}");
    check(
        "CPU produced 3 results",
        cpu_results.len() == 3,
        &mut passed,
        &mut total,
    );

    // --- Mode 2: GPU individual dispatches ---
    println!("\n--- Mode 2: GPU individual (3 dispatches) ---");
    let ctx = GpuContext::new().await;
    match ctx {
        Ok(ctx) => {
            println!("  GPU: {}", ctx.adapter_name());

            let ind_start = Instant::now();
            let mut ind_results = Vec::new();
            for op in &ops {
                match ctx.execute(op).await {
                    Ok(r) => ind_results.push(r),
                    Err(e) => {
                        println!("  [ERROR] {e}");
                        break;
                    }
                }
            }
            let ind_elapsed = ind_start.elapsed();
            println!("  GPU individual total: {ind_elapsed:?}");
            check(
                "individual produced 3 results",
                ind_results.len() == 3,
                &mut passed,
                &mut total,
            );

            // --- Mode 3: GPU fused pipeline ---
            println!("\n--- Mode 3: GPU fused pipeline (1 submission) ---");
            let fused_start = Instant::now();
            let fused_results = ctx.execute_fused(&ops).await;
            let fused_elapsed = fused_start.elapsed();
            println!("  GPU fused total: {fused_elapsed:?}");

            match fused_results {
                Ok(fused) => {
                    check(
                        "fused produced 3 results",
                        fused.len() == 3,
                        &mut passed,
                        &mut total,
                    );

                    // Validate fused vs CPU
                    if let (GpuResult::HillSweep(cpu_h), GpuResult::HillSweep(fused_h)) =
                        (&cpu_results[0], &fused[0])
                    {
                        let max_rel = cpu_h
                            .iter()
                            .zip(fused_h.iter())
                            .filter(|(a, _)| a.abs() > 1e-10)
                            .map(|(a, b)| ((a - b) / a).abs())
                            .fold(0.0_f64, f64::max);
                        check(
                            &format!("Hill fused parity: max_rel={max_rel:.2e}"),
                            max_rel < 1e-4,
                            &mut passed,
                            &mut total,
                        );
                    }
                    if let GpuResult::PopulationPkBatch(fused_pk) = &fused[1] {
                        check(
                            &format!("PK fused: {} patients, all positive", fused_pk.len()),
                            fused_pk.len() == 10_000 && fused_pk.iter().all(|&a| a > 0.0),
                            &mut passed,
                            &mut total,
                        );
                    }
                    if let (GpuResult::DiversityBatch(cpu_d), GpuResult::DiversityBatch(fused_d)) =
                        (&cpu_results[2], &fused[2])
                    {
                        let max_shannon_err = cpu_d
                            .iter()
                            .zip(fused_d.iter())
                            .map(|((cs, _), (fs, _))| (cs - fs).abs())
                            .fold(0.0_f64, f64::max);
                        check(
                            &format!(
                                "Diversity fused parity: max_shannon_err={max_shannon_err:.2e}"
                            ),
                            max_shannon_err < 1e-4,
                            &mut passed,
                            &mut total,
                        );
                    }

                    // Speedup summary
                    println!("\n--- Timing Summary ---");
                    println!(
                        "  CPU:            {:>10.3} ms",
                        cpu_elapsed.as_secs_f64() * 1000.0
                    );
                    println!(
                        "  GPU individual:  {:>10.3} ms",
                        ind_elapsed.as_secs_f64() * 1000.0
                    );
                    println!(
                        "  GPU fused:       {:>10.3} ms",
                        fused_elapsed.as_secs_f64() * 1000.0
                    );

                    let ind_vs_cpu = cpu_elapsed.as_secs_f64() / ind_elapsed.as_secs_f64();
                    let fused_vs_ind = ind_elapsed.as_secs_f64() / fused_elapsed.as_secs_f64();
                    let fused_vs_cpu = cpu_elapsed.as_secs_f64() / fused_elapsed.as_secs_f64();

                    println!("  Individual/CPU:  {ind_vs_cpu:.3}x");
                    println!("  Fused/Individual: {fused_vs_ind:.3}x");
                    println!("  Fused/CPU:        {fused_vs_cpu:.3}x");

                    check(
                        "fused faster than individual",
                        fused_elapsed < ind_elapsed,
                        &mut passed,
                        &mut total,
                    );
                }
                Err(e) => {
                    println!("  Fused pipeline error: {e}");
                    check("fused execution", false, &mut passed, &mut total);
                }
            }
        }
        Err(e) => {
            println!("  No GPU available: {e}");
            println!("  Skipping GPU tests.");
        }
    }

    // --- Mode 4: toadstool Pipeline::execute_gpu ---
    println!("\n--- Mode 4: toadstool Pipeline (GPU dispatch) ---");
    let mut pipe = Pipeline::new("healthSpring_diagnostic");
    pipe.add_stage(Stage {
        name: "generate_concs".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 1000,
            seed: 42,
        },
    });
    pipe.add_stage(Stage {
        name: "hill_dose_response".into(),
        substrate: Substrate::Gpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 100.0,
                ec50: 10.0,
                n: 1.5,
            },
        },
    });

    let cpu_pipe_start = Instant::now();
    let cpu_pipe_result = pipe.execute_cpu();
    let cpu_pipe_elapsed = cpu_pipe_start.elapsed();

    check(
        &format!(
            "toadstool CPU pipeline: {} stages, success={}",
            cpu_pipe_result.stage_results.len(),
            cpu_pipe_result.success
        ),
        cpu_pipe_result.success && cpu_pipe_result.stage_results.len() == 2,
        &mut passed,
        &mut total,
    );
    println!("  CPU pipeline: {cpu_pipe_elapsed:?}");

    match GpuContext::new().await {
        Ok(ctx) => {
            let gpu_pipe_start = Instant::now();
            let gpu_pipe_result = pipe.execute_gpu(&ctx).await;
            let gpu_pipe_elapsed = gpu_pipe_start.elapsed();

            check(
                &format!(
                    "toadstool GPU pipeline: {} stages, success={}",
                    gpu_pipe_result.stage_results.len(),
                    gpu_pipe_result.success
                ),
                gpu_pipe_result.success && gpu_pipe_result.stage_results.len() == 2,
                &mut passed,
                &mut total,
            );

            let gpu_hill_on_gpu = gpu_pipe_result
                .stage_results
                .iter()
                .any(|s| s.stage_name == "hill_dose_response" && s.substrate == Substrate::Gpu);
            check(
                "Hill stage dispatched to GPU",
                gpu_hill_on_gpu,
                &mut passed,
                &mut total,
            );

            let cpu_hill = &cpu_pipe_result.stage_results[1].output_data;
            let gpu_hill = &gpu_pipe_result.stage_results[1].output_data;
            if cpu_hill.len() == gpu_hill.len() {
                let max_rel = cpu_hill
                    .iter()
                    .zip(gpu_hill.iter())
                    .filter(|(a, _)| a.abs() > 1e-10)
                    .map(|(a, b)| ((a - b) / a).abs())
                    .fold(0.0_f64, f64::max);
                check(
                    &format!("toadstool GPU/CPU parity: max_rel={max_rel:.2e}"),
                    max_rel < 1e-4,
                    &mut passed,
                    &mut total,
                );
            } else {
                check(
                    &format!(
                        "toadstool output lengths match: cpu={} gpu={}",
                        cpu_hill.len(),
                        gpu_hill.len()
                    ),
                    false,
                    &mut passed,
                    &mut total,
                );
            }

            println!("  CPU pipeline:  {cpu_pipe_elapsed:?}");
            println!("  GPU pipeline:  {gpu_pipe_elapsed:?}");
            println!(
                "  Substrates:    {:?}",
                gpu_pipe_result
                    .stage_results
                    .iter()
                    .map(|s| format!("{}={:?}", s.stage_name, s.substrate))
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => {
            println!("  No GPU: {e}");
        }
    }

    println!("\n=================================");
    println!("Exp054 GPU Pipeline: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
