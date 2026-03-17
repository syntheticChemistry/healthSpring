// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

//! Exp060: CPU vs GPU parity matrix through `toadStool` pipeline.
//!
//! Validates all three GPU-backed kernels (Hill, `PopPK`, Diversity) through
//! the toadStool `Pipeline` abstraction at multiple scales, comparing
//! `execute_cpu()`, `execute_gpu()`, and `execute_auto()`.

use healthspring_barracuda::gpu::GpuContext;
use healthspring_barracuda::tolerances::{GPU_F32_TRANSCENDENTAL, GPU_STATISTICAL_PARITY};
use healthspring_forge::{Capabilities, Substrate};
use healthspring_toadstool::pipeline::Pipeline;
use healthspring_toadstool::stage::{Stage, StageOp, TransformKind};

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

#[tokio::main]
#[expect(
    clippy::too_many_lines,
    reason = "sequential GPU parity matrix across 3 kernels × 3 scales"
)]
async fn main() {
    println!("Exp060 CPU vs GPU Parity Matrix — toadStool Pipeline");
    println!("=====================================================\n");

    let ctx = match GpuContext::new().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("FAIL: GPU required for exp060: {e}");
            std::process::exit(1);
        }
    };
    let caps = Capabilities::discover();
    let mut passed = 0u32;
    let mut total = 0u32;

    // -----------------------------------------------------------------------
    // Kernel 1: Hill dose-response at 3 scales
    // -----------------------------------------------------------------------
    for n_concs in [50, 500, 5000] {
        println!("--- Hill dose-response ({n_concs} concentrations) ---");

        let mut cpu_pipe = Pipeline::new(format!("hill_cpu_{n_concs}"));
        cpu_pipe.add_stage(Stage {
            name: "gen_concs".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: n_concs,
                seed: 42,
            },
        });
        cpu_pipe.add_stage(Stage {
            name: "hill".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Hill {
                    emax: 100.0,
                    ec50: 10.0,
                    n: 1.5,
                },
            },
        });

        let mut gpu_pipe = Pipeline::new(format!("hill_gpu_{n_concs}"));
        gpu_pipe.add_stage(Stage {
            name: "gen_concs".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: n_concs,
                seed: 42,
            },
        });
        gpu_pipe.add_stage(Stage {
            name: "hill".into(),
            substrate: Substrate::Gpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Hill {
                    emax: 100.0,
                    ec50: 10.0,
                    n: 1.5,
                },
            },
        });

        let cpu_result = cpu_pipe.execute_cpu();
        let gpu_result = gpu_pipe.execute_gpu(&ctx).await;
        let auto_result = gpu_pipe.execute_auto(&ctx, &caps).await;

        let cpu_data = if let Some(s) = cpu_result.stage_results.last() {
            &s.output_data
        } else {
            eprintln!("FAIL: CPU pipeline produced no stage results");
            std::process::exit(1);
        };
        let gpu_data = if let Some(s) = gpu_result.stage_results.last() {
            &s.output_data
        } else {
            eprintln!("FAIL: GPU pipeline produced no stage results");
            std::process::exit(1);
        };
        let auto_data = if let Some(s) = auto_result.stage_results.last() {
            &s.output_data
        } else {
            eprintln!("FAIL: Auto pipeline produced no stage results");
            std::process::exit(1);
        };

        check(
            &format!("hill_{n_concs}: CPU vs GPU length"),
            cpu_data.len() == gpu_data.len(),
            &mut passed,
            &mut total,
        );

        let max_err: f64 = cpu_data
            .iter()
            .zip(gpu_data.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        check(
            &format!("hill_{n_concs}: max error {max_err:.2e} < {GPU_F32_TRANSCENDENTAL:.0e}"),
            max_err < GPU_F32_TRANSCENDENTAL,
            &mut passed,
            &mut total,
        );

        let auto_max: f64 = cpu_data
            .iter()
            .zip(auto_data.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        check(
            &format!("hill_{n_concs}: auto max error {auto_max:.2e}"),
            auto_max < GPU_F32_TRANSCENDENTAL,
            &mut passed,
            &mut total,
        );

        println!(
            "  CPU: {:.1}us  GPU: {:.1}us  Auto: {:.1}us  Speedup: {:.1}x\n",
            cpu_result.total_time_us,
            gpu_result.total_time_us,
            auto_result.total_time_us,
            cpu_result.total_time_us / gpu_result.total_time_us.max(0.1),
        );
    }

    // -----------------------------------------------------------------------
    // Kernel 2: Population PK Monte Carlo at 3 scales
    // -----------------------------------------------------------------------
    for n_patients in [100, 1000, 10_000] {
        println!("--- Population PK ({n_patients} patients) ---");

        let mut cpu_pipe = Pipeline::new(format!("pop_pk_cpu_{n_patients}"));
        cpu_pipe.add_stage(Stage {
            name: "pop_pk".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::PopulationPk {
                n_patients,
                dose_mg: 4.0,
                f_bioavail: 0.79,
                seed: 42,
            },
        });

        let mut gpu_pipe = Pipeline::new(format!("pop_pk_gpu_{n_patients}"));
        gpu_pipe.add_stage(Stage {
            name: "pop_pk".into(),
            substrate: Substrate::Gpu,
            operation: StageOp::PopulationPk {
                n_patients,
                dose_mg: 4.0,
                f_bioavail: 0.79,
                seed: 42,
            },
        });

        let cpu_result = cpu_pipe.execute_cpu();
        let gpu_result = gpu_pipe.execute_gpu(&ctx).await;
        let auto_result = gpu_pipe.execute_auto(&ctx, &caps).await;

        let cpu_data = &cpu_result.stage_results[0].output_data;
        let gpu_data = &gpu_result.stage_results[0].output_data;
        let auto_data = &auto_result.stage_results[0].output_data;

        check(
            &format!(
                "pop_pk_{n_patients}: CPU len={} GPU len={}",
                cpu_data.len(),
                gpu_data.len()
            ),
            cpu_data.len() == gpu_data.len(),
            &mut passed,
            &mut total,
        );

        // PRNGs differ (CPU uses LCG, GPU uses xorshift32+Wang hash) —
        // compare statistical properties, not element-wise values.
        #[expect(
            clippy::cast_precision_loss,
            reason = "usize len fits f64 for mean calculation"
        )]
        let cpu_mean: f64 = cpu_data.iter().sum::<f64>() / cpu_data.len() as f64;
        #[expect(
            clippy::cast_precision_loss,
            reason = "usize len fits f64 for mean calculation"
        )]
        let gpu_mean: f64 = gpu_data.iter().sum::<f64>() / gpu_data.len() as f64;
        let rel_err = (cpu_mean - gpu_mean).abs() / cpu_mean;
        check(
            &format!(
                "pop_pk_{n_patients}: mean AUC rel err {rel_err:.4} < {GPU_STATISTICAL_PARITY}"
            ),
            rel_err < GPU_STATISTICAL_PARITY,
            &mut passed,
            &mut total,
        );

        #[expect(
            clippy::cast_precision_loss,
            reason = "usize len fits f64 for mean calculation"
        )]
        let auto_mean: f64 = auto_data.iter().sum::<f64>() / auto_data.len() as f64;
        let auto_rel = (cpu_mean - auto_mean).abs() / cpu_mean;
        check(
            &format!("pop_pk_{n_patients}: auto mean rel err {auto_rel:.4}"),
            auto_rel < GPU_STATISTICAL_PARITY,
            &mut passed,
            &mut total,
        );

        println!(
            "  CPU: {:.1}us  GPU: {:.1}us  Auto: {:.1}us  Speedup: {:.1}x\n",
            cpu_result.total_time_us,
            gpu_result.total_time_us,
            auto_result.total_time_us,
            cpu_result.total_time_us / gpu_result.total_time_us.max(0.1),
        );
    }

    // -----------------------------------------------------------------------
    // Kernel 3: Diversity indices at 3 scales
    // -----------------------------------------------------------------------
    for n_communities in [10, 100, 1000] {
        println!("--- Diversity indices ({n_communities} communities) ---");

        let communities: Vec<Vec<f64>> = (0..n_communities)
            .map(|i| {
                let bias = f64::from(i) / f64::from(n_communities);
                vec![
                    0.3f64.mul_add(-bias, 0.4),
                    0.1f64.mul_add(-bias, 0.3),
                    0.2,
                    0.4f64.mul_add(bias, 0.1),
                ]
            })
            .collect();

        let mut cpu_pipe = Pipeline::new(format!("div_cpu_{n_communities}"));
        cpu_pipe.add_stage(Stage {
            name: "diversity".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::DiversityReduce {
                communities: communities.clone(),
            },
        });

        let mut gpu_pipe = Pipeline::new(format!("div_gpu_{n_communities}"));
        gpu_pipe.add_stage(Stage {
            name: "diversity".into(),
            substrate: Substrate::Gpu,
            operation: StageOp::DiversityReduce {
                communities: communities.clone(),
            },
        });

        let cpu_result = cpu_pipe.execute_cpu();
        let gpu_result = gpu_pipe.execute_gpu(&ctx).await;
        let auto_result = gpu_pipe.execute_auto(&ctx, &caps).await;

        let cpu_data = &cpu_result.stage_results[0].output_data;
        let gpu_data = &gpu_result.stage_results[0].output_data;
        let auto_data = &auto_result.stage_results[0].output_data;

        check(
            &format!(
                "div_{n_communities}: length CPU={} GPU={}",
                cpu_data.len(),
                gpu_data.len()
            ),
            cpu_data.len() == gpu_data.len(),
            &mut passed,
            &mut total,
        );

        let max_err: f64 = cpu_data
            .iter()
            .zip(gpu_data.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        check(
            &format!("div_{n_communities}: max error {max_err:.2e} < {GPU_F32_TRANSCENDENTAL:.0e}"),
            max_err < GPU_F32_TRANSCENDENTAL,
            &mut passed,
            &mut total,
        );

        let auto_max: f64 = cpu_data
            .iter()
            .zip(auto_data.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        check(
            &format!("div_{n_communities}: auto max error {auto_max:.2e}"),
            auto_max < GPU_F32_TRANSCENDENTAL,
            &mut passed,
            &mut total,
        );

        println!(
            "  CPU: {:.1}us  GPU: {:.1}us  Auto: {:.1}us  Speedup: {:.1}x\n",
            cpu_result.total_time_us,
            gpu_result.total_time_us,
            auto_result.total_time_us,
            cpu_result.total_time_us / gpu_result.total_time_us.max(0.1),
        );
    }

    // -----------------------------------------------------------------------
    // Summary
    // -----------------------------------------------------------------------
    println!("=================================================");
    println!("Exp060 CPU vs GPU Pipeline: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
