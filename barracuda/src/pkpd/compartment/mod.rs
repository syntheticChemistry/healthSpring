// SPDX-License-Identifier: AGPL-3.0-or-later
//! One- and two-compartment PK models (IV bolus, oral Bateman, biexponential).

use core::f64::consts::LN_2;

// ═══════════════════════════════════════════════════════════════════════
// One-compartment PK (Exp002)
// ═══════════════════════════════════════════════════════════════════════

/// IV bolus: `C(t) = (Dose/Vd) * exp(-k_e * t)` where `k_e = ln2 / t½`.
#[must_use]
pub fn pk_iv_bolus(dose_mg: f64, vd_l: f64, half_life_hr: f64, t_hr: f64) -> f64 {
    if half_life_hr <= 0.0 || vd_l <= 0.0 {
        return 0.0;
    }
    let k_e = LN_2 / half_life_hr;
    let c0 = dose_mg / vd_l;
    c0 * (-k_e * t_hr).exp()
}

/// Oral one-compartment (Bateman equation):
/// `C(t) = (F * Dose * k_a) / (Vd * (k_a - k_e)) * (exp(-k_e*t) - exp(-k_a*t))`
#[must_use]
pub fn pk_oral_one_compartment(
    dose_mg: f64,
    f_bioavail: f64,
    vd_l: f64,
    k_a: f64,
    k_e: f64,
    t_hr: f64,
) -> f64 {
    if (k_a - k_e).abs() < 1e-12 || vd_l <= 0.0 {
        return 0.0;
    }
    let coeff = (f_bioavail * dose_mg * k_a) / (vd_l * (k_a - k_e));
    coeff * ((-k_e * t_hr).exp() - (-k_a * t_hr).exp())
}

/// Analytical Tmax for oral dosing: `Tmax = ln(k_a/k_e) / (k_a - k_e)`.
#[must_use]
pub fn oral_tmax(k_a: f64, k_e: f64) -> f64 {
    if (k_a - k_e).abs() < 1e-12 || k_a <= 0.0 || k_e <= 0.0 {
        return 0.0;
    }
    (k_a / k_e).ln() / (k_a - k_e)
}

// ═══════════════════════════════════════════════════════════════════════
// Two-compartment PK (Exp003)
// ═══════════════════════════════════════════════════════════════════════

/// Macro-rate constants from micro-constants.
///
/// `α + β = k10 + k12 + k21`, `α * β = k10 * k21`.
/// Returns `(alpha, beta)` where `alpha > beta`.
#[must_use]
pub fn micro_to_macro(k10: f64, k12: f64, k21: f64) -> (f64, f64) {
    let s = k10 + k12 + k21;
    let p = k10 * k21;
    let disc = (s * s - 4.0 * p).max(0.0);
    let sqrt_d = disc.sqrt();
    (f64::midpoint(s, sqrt_d), s - f64::midpoint(s, sqrt_d))
}

/// Two-compartment IV bolus: `C(t) = A·exp(-α·t) + B·exp(-β·t)`.
#[must_use]
pub fn pk_two_compartment_iv(
    dose_mg: f64,
    v1_l: f64,
    k10: f64,
    k12: f64,
    k21: f64,
    t_hr: f64,
) -> f64 {
    if v1_l <= 0.0 {
        return 0.0;
    }
    let (alpha, beta) = micro_to_macro(k10, k12, k21);
    if (alpha - beta).abs() < 1e-15 {
        let c0 = dose_mg / v1_l;
        return c0 * (-alpha * t_hr).exp();
    }
    let c0 = dose_mg / v1_l;
    let a = c0 * (alpha - k21) / (alpha - beta);
    let b = c0 * (k21 - beta) / (alpha - beta);
    a * (-alpha * t_hr).exp() + b * (-beta * t_hr).exp()
}

/// Macro-coefficients A and B for the two-compartment model.
///
/// `A = C0 * (α - k21) / (α - β)`, `B = C0 * (k21 - β) / (α - β)`.
#[must_use]
pub fn two_compartment_ab(c0: f64, alpha: f64, beta: f64, k21: f64) -> (f64, f64) {
    let denom = alpha - beta;
    if denom.abs() < 1e-15 {
        return (c0, 0.0);
    }
    let a = c0 * (alpha - k21) / denom;
    let b = c0 * (k21 - beta) / denom;
    (a, b)
}

#[cfg(test)]
mod tests;
