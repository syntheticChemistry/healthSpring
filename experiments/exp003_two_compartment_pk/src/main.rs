// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
#![allow(clippy::similar_names)] // macro/micro pairs are standard PK notation

//! healthSpring Exp003 — Rust validation binary
//!
//! Cross-validates two-compartment PK model against the Python control
//! baseline (`control/pkpd/exp003_baseline.json`).

use core::f64::consts::LN_2;
use healthspring_barracuda::pkpd::{
    auc_trapezoidal, micro_to_macro, pk_two_compartment_iv, two_compartment_ab,
};

// Drug parameters from Python baseline (Gentamicin-like)
const DOSE_MG: f64 = 240.0;
const V1_L: f64 = 15.0;
const K10_HR: f64 = 0.35;
const K12_HR: f64 = 0.6;
const K21_HR: f64 = 0.15;

// Expected values from exp003_baseline.json
const EXPECTED_C0: f64 = 16.0;
const EXPECTED_V2_L: f64 = 60.0;
const EXPECTED_VSS_L: f64 = 75.0;
const TOL: f64 = 1e-8;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "n always small")]
            let frac = i as f64 / (n - 1) as f64;
            start + frac * (end - start)
        })
        .collect()
}

/// Peripheral compartment concentration via Euler integration.
fn peripheral_concentration(
    times: &[f64],
    c_central: &[f64],
    v1: f64,
    k12: f64,
    k21: f64,
) -> Vec<f64> {
    let v2 = v1 * k12 / k21;
    let mut a_periph = vec![0.0; times.len()];
    for i in 1..times.len() {
        let dt = times[i] - times[i - 1];
        let da = (k12 * c_central[i - 1]).mul_add(v1, -(k21 * a_periph[i - 1]));
        a_periph[i] = a_periph[i - 1] + da * dt;
    }
    a_periph.iter().map(|&a| a / v2).collect()
}

/// Linear regression slope for log(C) vs t over late times.
fn terminal_slope(times: &[f64], concs: &[f64], t_min: f64) -> Option<f64> {
    let valid: Vec<(f64, f64)> = times
        .iter()
        .copied()
        .zip(concs.iter().copied())
        .filter(|&(t, c)| t > t_min && c > 0.0)
        .map(|(t, c)| (t, c.ln()))
        .collect();
    if valid.len() < 10 {
        return None;
    }
    #[expect(clippy::cast_precision_loss, reason = "n always small")]
    let n = valid.len() as f64;
    let sum_t: f64 = valid.iter().map(|(t, _)| t).sum();
    let sum_ln: f64 = valid.iter().map(|(_, ln)| ln).sum();
    let sum_t2: f64 = valid.iter().map(|(t, _)| t * t).sum();
    let sum_t_ln: f64 = valid.iter().map(|(t, ln)| t * ln).sum();
    let denom = n.mul_add(sum_t2, -(sum_t * sum_t));
    if denom.abs() < 1e-15 {
        return None;
    }
    Some(n.mul_add(sum_t_ln, -(sum_t * sum_ln)) / denom)
}

fn main() {
    let mut passed = 0_u32;
    let mut failed = 0_u32;
    let times = linspace(0.0, 168.0, 2000);

    let c_curve: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_MG, V1_L, K10_HR, K12_HR, K21_HR, t))
        .collect();

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp003 [Rust]: Two-Compartment PK (IV Bolus)");
    println!("{}", "=".repeat(72));

    let (alpha, beta) = micro_to_macro(K10_HR, K12_HR, K21_HR);

    // Check 1: α > β
    print!("\n--- Check 1: α > β --- ");
    if alpha > beta {
        println!("[PASS] α={alpha:.6} > β={beta:.6}");
        passed += 1;
    } else {
        println!("[FAIL] α={alpha:.6} <= β={beta:.6}");
        failed += 1;
    }

    // Check 2: α + β = k10 + k12 + k21
    print!("\n--- Check 2: α + β = k10 + k12 + k21 --- ");
    let sum_macro = alpha + beta;
    let sum_micro = K10_HR + K12_HR + K21_HR;
    if (sum_macro - sum_micro).abs() < TOL {
        println!("[PASS] {sum_macro:.10} == {sum_micro:.10}");
        passed += 1;
    } else {
        println!("[FAIL] {sum_macro:.10} != {sum_micro:.10}");
        failed += 1;
    }

    // Check 3: α * β = k10 * k21
    print!("\n--- Check 3: α * β = k10 * k21 --- ");
    let prod_macro = alpha * beta;
    let prod_micro = K10_HR * K21_HR;
    if (prod_macro - prod_micro).abs() < TOL {
        println!("[PASS] {prod_macro:.10} == {prod_micro:.10}");
        passed += 1;
    } else {
        println!("[FAIL] {prod_macro:.10} != {prod_micro:.10}");
        failed += 1;
    }

    // Check 4: C(0) = Dose / V1
    print!("\n--- Check 4: C(0) = Dose / V1 --- ");
    let c_at_0 = pk_two_compartment_iv(DOSE_MG, V1_L, K10_HR, K12_HR, K21_HR, 0.0);
    if (c_at_0 - EXPECTED_C0).abs() < TOL {
        println!("[PASS] C(0) = {c_at_0:.6}");
        passed += 1;
    } else {
        println!("[FAIL] C(0) = {c_at_0:.6}, expected {EXPECTED_C0}");
        failed += 1;
    }

    // Check 5: All concentrations ≥ 0
    print!("\n--- Check 5: All concentrations ≥ 0 --- ");
    let all_nonneg = c_curve.iter().all(|&c| c >= -1e-12);
    let min_c = c_curve.iter().copied().fold(f64::INFINITY, f64::min);
    if all_nonneg {
        println!("[PASS] min(C) = {min_c:.6e}");
        passed += 1;
    } else {
        println!("[FAIL] min(C) = {min_c:.6e}");
        failed += 1;
    }

    // Check 6: Monotonically decreasing
    print!("\n--- Check 6: Central concentration monotonically decreasing --- ");
    let mono_dec = c_curve.windows(2).all(|w| w[0] >= w[1] - 1e-12);
    if mono_dec {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 7: t½α < t½β
    print!("\n--- Check 7: t½α < t½β --- ");
    let t_half_alpha = LN_2 / alpha;
    let t_half_beta = LN_2 / beta;
    if t_half_alpha < t_half_beta {
        println!("[PASS] t½α = {t_half_alpha:.3} hr < t½β = {t_half_beta:.3} hr");
        passed += 1;
    } else {
        println!("[FAIL] t½α = {t_half_alpha:.3} >= t½β = {t_half_beta:.3}");
        failed += 1;
    }

    // Check 8: AUC analytical
    print!("\n--- Check 8: AUC analytical --- ");
    let auc_numerical = auc_trapezoidal(&times, &c_curve);
    let auc_analytical = DOSE_MG / (V1_L * K10_HR);
    let rel_err = (auc_numerical - auc_analytical).abs() / auc_analytical;
    if rel_err < 0.01 {
        println!("[PASS] num={auc_numerical:.2}, ana={auc_analytical:.2} (err={rel_err:.4})");
        passed += 1;
    } else {
        println!("[FAIL] err={rel_err:.4}");
        failed += 1;
    }

    // Check 9: A + B = C0
    print!("\n--- Check 9: A + B = C0 --- ");
    let c0 = DOSE_MG / V1_L;
    let (a_coeff, b_coeff) = two_compartment_ab(c0, alpha, beta, K21_HR);
    if (a_coeff + b_coeff - c0).abs() < TOL {
        println!("[PASS] A={a_coeff:.6} + B={b_coeff:.6} = C0");
        passed += 1;
    } else {
        println!("[FAIL] A+B = {} != C0={c0:.6}", a_coeff + b_coeff);
        failed += 1;
    }

    // Check 10: Peripheral compartment peaks then declines
    print!("\n--- Check 10: Peripheral compartment peaks then declines --- ");
    let c_periph = peripheral_concentration(&times, &c_curve, V1_L, K12_HR, K21_HR);
    let (idx_peak, _) = c_periph
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap();
    if 0 < idx_peak && idx_peak < times.len() - 1 {
        let peak_time = times[idx_peak];
        let peak_conc = c_periph[idx_peak];
        println!("[PASS] Peripheral peaks at t={peak_time:.2} hr, C={peak_conc:.4}");
        passed += 1;
    } else {
        println!("[FAIL] Peak at boundary (idx={idx_peak})");
        failed += 1;
    }

    // Check 11: Terminal phase log-linearity (slope ≈ -β)
    print!("\n--- Check 11: Terminal phase log-linearity --- ");
    if let Some(slope) = terminal_slope(&times, &c_curve, 8.0) {
        let slope_err = (slope - (-beta)).abs() / beta;
        if slope_err < 0.01 {
            println!(
                "[PASS] slope={slope:.6}, expected={} (err={slope_err:.4})",
                -beta
            );
            passed += 1;
        } else {
            println!("[FAIL] slope err={slope_err:.4}");
            failed += 1;
        }
    } else {
        println!("[FAIL] Insufficient late data");
        failed += 1;
    }

    // Check 12: k12=0 reduces to one-compartment
    print!("\n--- Check 12: k12=0 reduces to one-compartment --- ");
    let c_two: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_MG, V1_L, K10_HR, 0.0, K21_HR, t))
        .collect();
    let c_one: Vec<f64> = times
        .iter()
        .map(|&t| (DOSE_MG / V1_L) * (-K10_HR * t).exp())
        .collect();
    let max_diff: f64 = c_two
        .iter()
        .zip(c_one.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    if max_diff < 1e-10 {
        println!("[PASS] max|diff| = {max_diff:.2e}");
        passed += 1;
    } else {
        println!("[FAIL] max|diff| = {max_diff:.2e}");
        failed += 1;
    }

    // Check 13: V2 = V1 * k12 / k21
    print!("\n--- Check 13: Peripheral volume V2 = V1 * k12 / k21 --- ");
    let v2 = V1_L * K12_HR / K21_HR;
    if (v2 - EXPECTED_V2_L).abs() < TOL && v2 > 0.0 {
        println!("[PASS] V2 = {v2:.2} L");
        passed += 1;
    } else {
        println!("[FAIL] V2 = {v2:.2}");
        failed += 1;
    }

    // Check 14: Vss = V1 + V2
    print!("\n--- Check 14: Vss = V1 + V2 --- ");
    let vss = V1_L + v2;
    if (vss - EXPECTED_VSS_L).abs() < TOL && vss > V1_L {
        println!("[PASS] Vss = {vss:.2} L (> V1={V1_L:.2})");
        passed += 1;
    } else {
        println!("[FAIL] Vss = {vss:.2}");
        failed += 1;
    }

    // Check 15: CL = V1 * k10
    print!("\n--- Check 15: CL = V1 * k10 --- ");
    let cl = V1_L * K10_HR;
    let cl_from_auc = DOSE_MG / auc_analytical;
    if (cl - cl_from_auc).abs() < TOL {
        println!("[PASS] CL = {cl:.4} L/hr (= Dose/AUC)");
        passed += 1;
    } else {
        println!("[FAIL] CL = {cl:.4}, Dose/AUC = {cl_from_auc:.4}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    std::process::exit(i32::from(failed > 0));
}
