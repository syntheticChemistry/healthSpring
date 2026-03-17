// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
#![expect(
    clippy::similar_names,
    reason = "macro/micro pairs are standard PK notation"
)]

//! healthSpring Exp003 — Rust validation binary
//!
//! Cross-validates two-compartment PK model against the Python control
//! baseline (`control/pkpd/exp003_baseline.json`).

use core::f64::consts::LN_2;
use healthspring_barracuda::pkpd::{
    auc_trapezoidal, micro_to_macro, pk_two_compartment_iv, two_compartment_ab,
};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::{OrExit, ValidationHarness};

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
    if denom.abs() < tolerances::MACHINE_EPSILON_STRICT {
        return None;
    }
    Some(n.mul_add(sum_t_ln, -(sum_t * sum_ln)) / denom)
}

fn main() {
    let mut h = ValidationHarness::new("Exp003 Two-Compartment PK");
    let times = linspace(0.0, 168.0, 2000);

    let c_curve: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_MG, V1_L, K10_HR, K12_HR, K21_HR, t))
        .collect();

    let (alpha, beta) = micro_to_macro(K10_HR, K12_HR, K21_HR);

    // Check 1: α > β
    h.check_bool("α > β", alpha > beta);

    // Check 2: α + β = k10 + k12 + k21
    let sum_macro = alpha + beta;
    let sum_micro = K10_HR + K12_HR + K21_HR;
    h.check_abs(
        "α + β = k10 + k12 + k21",
        sum_macro,
        sum_micro,
        tolerances::TWO_COMPARTMENT_IDENTITY,
    );

    // Check 3: α * β = k10 * k21
    let prod_macro = alpha * beta;
    let prod_micro = K10_HR * K21_HR;
    h.check_abs(
        "α * β = k10 * k21",
        prod_macro,
        prod_micro,
        tolerances::TWO_COMPARTMENT_IDENTITY,
    );

    // Check 4: C(0) = Dose / V1
    let c_at_0 = pk_two_compartment_iv(DOSE_MG, V1_L, K10_HR, K12_HR, K21_HR, 0.0);
    h.check_abs(
        "C(0) = Dose / V1",
        c_at_0,
        EXPECTED_C0,
        tolerances::TWO_COMPARTMENT_IDENTITY,
    );

    // Check 5: All concentrations ≥ 0
    let all_nonneg = c_curve
        .iter()
        .all(|&c| c >= -tolerances::MACHINE_EPSILON_TIGHT);
    h.check_bool("All concentrations ≥ 0", all_nonneg);

    // Check 6: Monotonically decreasing
    let mono_dec = c_curve
        .windows(2)
        .all(|w| w[0] >= w[1] - tolerances::MACHINE_EPSILON_TIGHT);
    h.check_bool("Central concentration monotonically decreasing", mono_dec);

    // Check 7: t½α < t½β
    let t_half_alpha = LN_2 / alpha;
    let t_half_beta = LN_2 / beta;
    h.check_bool("t½α < t½β", t_half_alpha < t_half_beta);

    // Check 8: AUC analytical
    let auc_numerical = auc_trapezoidal(&times, &c_curve);
    let auc_analytical = DOSE_MG / (V1_L * K10_HR);
    h.check_rel(
        "AUC analytical",
        auc_numerical,
        auc_analytical,
        tolerances::AUC_TRAPEZOIDAL,
    );

    // Check 9: A + B = C0
    let c0 = DOSE_MG / V1_L;
    let (a_coeff, b_coeff) = two_compartment_ab(c0, alpha, beta, K21_HR);
    h.check_abs(
        "A + B = C0",
        a_coeff + b_coeff,
        c0,
        tolerances::TWO_COMPARTMENT_IDENTITY,
    );

    // Check 10: Peripheral compartment peaks then declines
    let c_periph = peripheral_concentration(&times, &c_curve, V1_L, K12_HR, K21_HR);
    let (idx_peak, _) = c_periph
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(core::cmp::Ordering::Equal))
        .or_exit("peripheral concentration curve has at least one point");
    h.check_bool(
        "Peripheral compartment peaks then declines",
        0 < idx_peak && idx_peak < times.len() - 1,
    );

    // Check 11: Terminal phase log-linearity (slope ≈ -β)
    if let Some(slope) = terminal_slope(&times, &c_curve, 8.0) {
        let slope_err = (slope - (-beta)).abs() / beta;
        h.check_bool(
            "Terminal phase log-linearity",
            slope_err < tolerances::TERMINAL_SLOPE,
        );
    } else {
        h.check_bool("Terminal phase log-linearity (sufficient data)", false);
    }

    // Check 12: k12=0 reduces to one-compartment
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
    h.check_upper(
        "k12=0 reduces to one-compartment",
        max_diff,
        tolerances::CPU_PARITY,
    );

    // Check 13: V2 = V1 * k12 / k21
    let v2 = V1_L * K12_HR / K21_HR;
    h.check_bool(
        "Peripheral volume V2 = V1 * k12 / k21",
        (v2 - EXPECTED_V2_L).abs() < tolerances::TWO_COMPARTMENT_IDENTITY && v2 > 0.0,
    );

    // Check 14: Vss = V1 + V2
    let vss = V1_L + v2;
    h.check_bool(
        "Vss = V1 + V2",
        (vss - EXPECTED_VSS_L).abs() < tolerances::TWO_COMPARTMENT_IDENTITY && vss > V1_L,
    );

    // Check 15: CL = V1 * k10
    let cl = V1_L * K10_HR;
    let cl_from_auc = DOSE_MG / auc_analytical;
    h.check_abs(
        "CL = V1 * k10",
        cl,
        cl_from_auc,
        tolerances::TWO_COMPARTMENT_IDENTITY,
    );

    h.exit();
}
