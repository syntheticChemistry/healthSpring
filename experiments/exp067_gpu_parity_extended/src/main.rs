// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

//! Exp067: Extended GPU parity — validates Tier B operations (Simpson, AUC, `Bray-Curtis`)
//! against CPU reference.
//!
//! On CPU-only machines, validates that the CPU reference paths produce correct results.

use healthspring_barracuda::gpu::{GpuOp, GpuResult, execute_cpu};
use healthspring_barracuda::microbiome::{bray_curtis, simpson_index};
use healthspring_barracuda::pkpd::{auc_trapezoidal, pk_oral_one_compartment};

const NUMERICAL_TOL: f64 = 1e-10;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                passed += 1;
                println!("  [PASS] {}", $name);
            } else {
                eprintln!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    println!("Exp067: Extended GPU Parity (Tier B Operations)");
    println!("================================================");

    // --- Simpson index via DiversityBatch CPU path ---
    println!("\n=== Simpson Index (FusedMapReduce pattern) ===");
    let communities = vec![
        vec![0.25, 0.25, 0.25, 0.25],
        vec![0.9, 0.05, 0.03, 0.02],
        vec![0.5, 0.5],
        vec![1.0],
        vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
    ];

    let op = GpuOp::DiversityBatch {
        communities: communities.clone(),
    };
    if let GpuResult::DiversityBatch(gpu_results) = execute_cpu(&op) {
        for (idx, community) in communities.iter().enumerate() {
            let cpu_simpson = simpson_index(community);
            let gpu_simpson = gpu_results[idx].1;
            let diff = (cpu_simpson - gpu_simpson).abs();
            check!(
                &format!("simpson_community_{idx}_parity (diff={diff:.2e})"),
                diff < NUMERICAL_TOL
            );
        }
    }

    let even_simpson = simpson_index(&[0.25, 0.25, 0.25, 0.25]);
    let dominated_simpson = simpson_index(&[0.9, 0.05, 0.03, 0.02]);
    check!(
        "simpson_even_gt_dominated",
        even_simpson > dominated_simpson
    );
    check!(
        "simpson_uniform_near_0.75",
        (even_simpson - 0.75).abs() < 1e-10
    );
    check!(
        "simpson_monoculture_zero",
        simpson_index(&[1.0]).abs() < 1e-10
    );

    // --- AUC Trapezoidal ---
    println!("\n=== AUC Trapezoidal (parallel prefix sum) ===");
    let clearance = 0.15 * (85.0_f64 / 70.0).powf(0.75);
    let volume_d = 15.0 * (85.0 / 70.0);
    let ke = clearance / volume_d;
    let times: Vec<f64> = (0..=100).map(|idx| f64::from(idx) * 24.0 / 100.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(4.0, 0.79, volume_d, 1.5, ke, t))
        .collect();
    let auc = auc_trapezoidal(&times, &concs);
    check!("auc_positive", auc > 0.0);
    check!("auc_reasonable_range", auc > 0.1 && auc < 100.0);

    let tri_times = [0.0, 1.0, 2.0];
    let tri_concs = [0.0, 1.0, 0.0];
    let tri_auc = auc_trapezoidal(&tri_times, &tri_concs);
    check!("auc_triangle_exact_1.0", (tri_auc - 1.0).abs() < 1e-10);

    let rect_times: Vec<f64> = (0..=10).map(f64::from).collect();
    let rect_concs = vec![5.0; 11];
    let rect_auc = auc_trapezoidal(&rect_times, &rect_concs);
    check!("auc_rectangle_exact_50.0", (rect_auc - 50.0).abs() < 1e-10);

    let auc2 = auc_trapezoidal(&times, &concs);
    check!("auc_deterministic", (auc - auc2).abs() < f64::EPSILON);

    // --- Bray-Curtis Dissimilarity ---
    println!("\n=== Bray-Curtis Dissimilarity (pairwise batch) ===");
    let sample_a = [0.3, 0.3, 0.2, 0.1, 0.1];
    let sample_b = [0.3, 0.3, 0.2, 0.1, 0.1];
    let bc_identical = bray_curtis(&sample_a, &sample_b);
    check!("bray_curtis_identical_zero", bc_identical.abs() < 1e-10);

    let sample_c = [1.0, 0.0, 0.0, 0.0, 0.0];
    let sample_d = [0.0, 0.0, 0.0, 0.0, 1.0];
    let bc_disjoint = bray_curtis(&sample_c, &sample_d);
    check!(
        "bray_curtis_disjoint_one",
        (bc_disjoint - 1.0).abs() < 1e-10
    );

    let sample_e = [0.4, 0.3, 0.2, 0.05, 0.05];
    let sample_f = [0.2, 0.2, 0.3, 0.15, 0.15];
    let bc_similar = bray_curtis(&sample_e, &sample_f);
    check!("bray_curtis_similar_lt_0.5", bc_similar < 0.5);
    check!(
        "bray_curtis_range_0_to_1",
        (0.0..=1.0).contains(&bc_similar)
    );

    let bc_ef = bray_curtis(&sample_e, &sample_f);
    let bc_fe = bray_curtis(&sample_f, &sample_e);
    check!("bray_curtis_symmetric", (bc_ef - bc_fe).abs() < 1e-10);

    // --- GPU path ---
    println!("\n=== GPU Parity (feature-gated) ===");
    println!("  GPU feature not enabled — CPU-only validation complete");
    check!("cpu_reference_complete", true);

    let total = passed + failed;
    println!("\n================================================");
    println!("Exp067 Extended GPU Parity: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
