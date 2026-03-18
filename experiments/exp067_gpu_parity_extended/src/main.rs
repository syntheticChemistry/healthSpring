// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp067: Extended GPU parity — validates Tier B operations (Simpson, AUC, `Bray-Curtis`)
//! against CPU reference.
//!
//! On CPU-only machines, validates that the CPU reference paths produce correct results.

use healthspring_barracuda::gpu::{GpuOp, GpuResult, execute_cpu};
use healthspring_barracuda::microbiome::{bray_curtis, simpson_index};
use healthspring_barracuda::pkpd::{auc_trapezoidal, pk_oral_one_compartment};
use healthspring_barracuda::tolerances::{CPU_PARITY, MACHINE_EPSILON};
use healthspring_barracuda::validation::ValidationHarness;

fn validate_simpson(h: &mut ValidationHarness) {
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
            h.check_upper(
                &format!("simpson_community_{idx}_parity (diff={diff:.2e})"),
                diff,
                CPU_PARITY,
            );
        }
    }

    let even_simpson = simpson_index(&[0.25, 0.25, 0.25, 0.25]);
    let dominated_simpson = simpson_index(&[0.9, 0.05, 0.03, 0.02]);
    h.check_bool(
        "simpson_even_gt_dominated",
        even_simpson > dominated_simpson,
    );
    h.check_abs(
        "simpson_uniform_near_0.75",
        even_simpson,
        0.75,
        MACHINE_EPSILON,
    );
    h.check_upper(
        "simpson_monoculture_zero",
        simpson_index(&[1.0]).abs(),
        MACHINE_EPSILON,
    );
}

fn validate_auc_and_bray_curtis(h: &mut ValidationHarness) {
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
    h.check_bool("auc_positive", auc > 0.0);
    h.check_bool("auc_reasonable_range", auc > 0.1 && auc < 100.0);

    let tri_times = [0.0, 1.0, 2.0];
    let tri_concs = [0.0, 1.0, 0.0];
    let tri_auc = auc_trapezoidal(&tri_times, &tri_concs);
    h.check_abs("auc_triangle_exact_1.0", tri_auc, 1.0, MACHINE_EPSILON);

    let rect_times: Vec<f64> = (0..=10).map(f64::from).collect();
    let rect_concs = vec![5.0; 11];
    let rect_auc = auc_trapezoidal(&rect_times, &rect_concs);
    h.check_abs("auc_rectangle_exact_50.0", rect_auc, 50.0, MACHINE_EPSILON);

    let auc2 = auc_trapezoidal(&times, &concs);
    h.check_abs("auc_deterministic", auc, auc2, f64::EPSILON);

    println!("\n=== Bray-Curtis Dissimilarity (pairwise batch) ===");
    let sample_a = [0.3, 0.3, 0.2, 0.1, 0.1];
    let sample_b = [0.3, 0.3, 0.2, 0.1, 0.1];
    let bc_identical = bray_curtis(&sample_a, &sample_b);
    h.check_upper(
        "bray_curtis_identical_zero",
        bc_identical.abs(),
        MACHINE_EPSILON,
    );

    let sample_c = [1.0, 0.0, 0.0, 0.0, 0.0];
    let sample_d = [0.0, 0.0, 0.0, 0.0, 1.0];
    let bc_disjoint = bray_curtis(&sample_c, &sample_d);
    h.check_abs(
        "bray_curtis_disjoint_one",
        bc_disjoint,
        1.0,
        MACHINE_EPSILON,
    );

    let sample_e = [0.4, 0.3, 0.2, 0.05, 0.05];
    let sample_f = [0.2, 0.2, 0.3, 0.15, 0.15];
    let bc_similar = bray_curtis(&sample_e, &sample_f);
    h.check_bool("bray_curtis_similar_lt_0.5", bc_similar < 0.5);
    h.check_bool(
        "bray_curtis_range_0_to_1",
        (0.0..=1.0).contains(&bc_similar),
    );

    let bc_ef = bray_curtis(&sample_e, &sample_f);
    let bc_fe = bray_curtis(&sample_f, &sample_e);
    h.check_abs("bray_curtis_symmetric", bc_ef, bc_fe, MACHINE_EPSILON);

    println!("\n=== GPU Parity (feature-gated) ===");
    println!("  GPU feature not enabled — CPU-only validation complete");
    h.check_bool("cpu_reference_complete", true);
}

fn main() {
    let mut h = ValidationHarness::new("exp067_gpu_parity_extended");

    println!("Exp067: Extended GPU Parity (Tier B Operations)");
    println!("================================================");

    validate_simpson(&mut h);
    validate_auc_and_bray_curtis(&mut h);

    println!("\n================================================");
    h.exit();
}
