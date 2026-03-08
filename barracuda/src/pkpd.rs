// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pharmacokinetic / pharmacodynamic modeling pipelines.
//!
//! Extends neuralSpring nS-601–605 (veterinary PK/PD) to human therapeutics.
//!
//! ## Tier 1 (CPU)
//!
//! - [`hill_dose_response`]: Generalized Hill equation for dose-response
//! - [`pk_iv_bolus`]: One-compartment IV bolus decay
//! - [`pk_oral_one_compartment`]: Bateman equation for oral absorption
//! - [`auc_trapezoidal`]: AUC by trapezoidal rule
//! - [`find_cmax_tmax`]: Peak concentration and time from PK curve
//! - [`pk_multiple_dose`]: Superposition for repeated dosing
//! - [`compute_ec_values`]: EC10/EC50/EC90 from Hill parameters
//!
//! ## Human JAK Inhibitor Reference Data
//!
//! Published IC50 values from Phase III trial literature:
//! - Baricitinib (Olumiant): JAK1/JAK2, IC50 ≈ 5.9 nM
//! - Upadacitinib (Rinvoq): JAK1, IC50 ≈ 8 nM
//! - Abrocitinib (Cibinqo): JAK1, IC50 ≈ 29 nM
//! - Oclacitinib (Apoquel): JAK1, IC50 = 10 nM (canine reference, Gonzales 2014)

use core::f64::consts::LN_2;

// ═══════════════════════════════════════════════════════════════════════
// Hill dose-response (Exp001)
// ═══════════════════════════════════════════════════════════════════════

/// Generalized Hill equation: `E = E_max * C^n / (C^n + IC50^n)`.
///
/// When `n = 1` this reduces to Michaelis-Menten. The Hill coefficient
/// captures cooperativity: `n > 1` = positive (steeper), `n < 1` = negative.
#[must_use]
pub fn hill_dose_response(concentration: f64, ic50: f64, hill_n: f64, e_max: f64) -> f64 {
    if ic50 <= 0.0 || concentration < 0.0 {
        return 0.0;
    }
    let c_n = concentration.powf(hill_n);
    let ic50_n = ic50.powf(hill_n);
    e_max * c_n / (c_n + ic50_n)
}

/// Sweep Hill dose-response across a concentration array.
#[must_use]
pub fn hill_sweep(ic50: f64, hill_n: f64, e_max: f64, concentrations: &[f64]) -> Vec<f64> {
    concentrations
        .iter()
        .map(|&c| hill_dose_response(c, ic50, hill_n, e_max))
        .collect()
}

/// Compute EC10, EC50, EC90 from Hill parameters.
///
/// `EC_x = IC50 * (x / (1 - x))^(1/n)`
#[must_use]
pub fn compute_ec_values(ic50: f64, hill_n: f64) -> EcValues {
    let ec50 = ic50;
    let ec10 = ic50 * (0.1_f64 / 0.9).powf(1.0 / hill_n);
    let ec90 = ic50 * (0.9_f64 / 0.1).powf(1.0 / hill_n);
    EcValues { ec10, ec50, ec90 }
}

/// EC10 / EC50 / EC90 triplet.
#[derive(Debug, Clone, Copy)]
pub struct EcValues {
    pub ec10: f64,
    pub ec50: f64,
    pub ec90: f64,
}

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

/// AUC by trapezoidal rule over `(time, concentration)` pairs.
///
/// # Panics
///
/// Panics if `times` and `concentrations` have different lengths.
#[must_use]
pub fn auc_trapezoidal(times: &[f64], concentrations: &[f64]) -> f64 {
    assert_eq!(times.len(), concentrations.len());
    if times.len() < 2 {
        return 0.0;
    }
    let mut auc = 0.0;
    for i in 1..times.len() {
        let dt = times[i] - times[i - 1];
        auc += 0.5 * (concentrations[i - 1] + concentrations[i]) * dt;
    }
    auc
}

/// Find Cmax and Tmax from discrete concentration-time data.
///
/// Returns `(cmax, tmax)`. If the slice is empty, returns `(0.0, 0.0)`.
///
/// # Panics
///
/// Panics if `times` and `concentrations` have different lengths.
#[must_use]
pub fn find_cmax_tmax(times: &[f64], concentrations: &[f64]) -> (f64, f64) {
    assert_eq!(times.len(), concentrations.len());
    if concentrations.is_empty() {
        return (0.0, 0.0);
    }
    let (idx, &cmax) = concentrations
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap();
    (cmax, times[idx])
}

/// Multiple dosing via superposition of a single-dose model.
///
/// Evaluates the single-dose function at each `t - n*interval` for `n_doses`
/// and sums the contributions.
pub fn pk_multiple_dose<F>(single_dose: F, interval_hr: f64, n_doses: usize, times: &[f64]) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    times
        .iter()
        .map(|&t| {
            (0..n_doses)
                .map(|i| {
                    #[expect(clippy::cast_precision_loss, reason = "n_doses always small")]
                    let t_shifted = t - (i as f64) * interval_hr;
                    if t_shifted >= 0.0 {
                        single_dose(t_shifted)
                    } else {
                        0.0
                    }
                })
                .sum()
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// Human JAK inhibitor reference data
// ═══════════════════════════════════════════════════════════════════════

/// Human JAK inhibitor drug profile.
#[derive(Debug, Clone)]
pub struct JakInhibitor {
    pub name: &'static str,
    pub ic50_jak1_nm: f64,
    pub hill_n: f64,
    pub selectivity: &'static str,
}

pub const BARICITINIB: JakInhibitor = JakInhibitor {
    name: "baricitinib",
    ic50_jak1_nm: 5.9,
    hill_n: 1.0,
    selectivity: "JAK1/JAK2",
};

pub const UPADACITINIB: JakInhibitor = JakInhibitor {
    name: "upadacitinib",
    ic50_jak1_nm: 8.0,
    hill_n: 1.0,
    selectivity: "JAK1",
};

pub const ABROCITINIB: JakInhibitor = JakInhibitor {
    name: "abrocitinib",
    ic50_jak1_nm: 29.0,
    hill_n: 1.0,
    selectivity: "JAK1",
};

pub const OCLACITINIB: JakInhibitor = JakInhibitor {
    name: "oclacitinib",
    ic50_jak1_nm: 10.0,
    hill_n: 1.0,
    selectivity: "JAK1 (canine)",
};

pub const ALL_INHIBITORS: [&JakInhibitor; 4] = [&BARICITINIB, &UPADACITINIB, &ABROCITINIB, &OCLACITINIB];

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
pub fn pk_two_compartment_iv(dose_mg: f64, v1_l: f64, k10: f64, k12: f64, k21: f64, t_hr: f64) -> f64 {
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

// ═══════════════════════════════════════════════════════════════════════
// Allometric scaling (Exp004)
// ═══════════════════════════════════════════════════════════════════════

/// Allometric scaling: `P_human = P_animal * (BW_human / BW_animal)^b`.
#[must_use]
pub fn allometric_scale(param_animal: f64, bw_animal: f64, bw_human: f64, exponent: f64) -> f64 {
    param_animal * (bw_human / bw_animal).powf(exponent)
}

/// mAb PK curve with SC absorption (Bateman-like for subcutaneous mAb).
///
/// Models typical mAb SC absorption: `k_a = ln2 / 2 days` (Tmax ≈ 3-8 days).
#[must_use]
pub fn mab_pk_sc(dose_mg: f64, vd_l: f64, half_life_days: f64, t_days: f64) -> f64 {
    if half_life_days <= 0.0 || vd_l <= 0.0 {
        return 0.0;
    }
    let k_e = LN_2 / half_life_days;
    let k_a = LN_2 / 2.0;
    if (k_a - k_e).abs() < 1e-12 {
        return 0.0;
    }
    let coeff = (dose_mg / vd_l) * k_a / (k_a - k_e);
    coeff * ((-k_e * t_days).exp() - (-k_a * t_days).exp())
}

/// Lokivetmab canine reference constants.
pub mod lokivetmab_canine {
    pub const BW_KG: f64 = 15.0;
    pub const HALF_LIFE_DAYS: f64 = 14.0;
    pub const VD_L_KG: f64 = 0.07;
    pub const CL_ML_DAY_KG: f64 = 3.5;
}

/// Standard allometric exponents for mAb PK.
pub mod allometric_exp {
    pub const CLEARANCE: f64 = 0.75;
    pub const VOLUME: f64 = 1.0;
    pub const HALF_LIFE: f64 = 0.25;
}

// ═══════════════════════════════════════════════════════════════════════
// Population PK (Exp005)
// ═══════════════════════════════════════════════════════════════════════

/// Lognormal sampling parameters.
#[derive(Debug, Clone, Copy)]
pub struct LognormalParam {
    pub typical: f64,
    pub cv: f64,
}

impl LognormalParam {
    /// Compute underlying normal μ and σ for lognormal with given typical (median) and CV.
    #[must_use]
    pub fn to_normal_params(self) -> (f64, f64) {
        let omega_sq = (1.0 + self.cv * self.cv).ln();
        let mu = self.typical.ln() - omega_sq / 2.0;
        let sigma = omega_sq.sqrt();
        (mu, sigma)
    }
}

/// Population PK parameters for baricitinib-like oral dosing.
pub mod pop_baricitinib {
    use super::LognormalParam;
    pub const CL: LognormalParam = LognormalParam { typical: 10.0, cv: 0.30 };
    pub const VD: LognormalParam = LognormalParam { typical: 80.0, cv: 0.25 };
    pub const KA: LognormalParam = LognormalParam { typical: 1.5, cv: 0.40 };
    pub const F_BIOAVAIL: f64 = 0.79;
    pub const DOSE_MG: f64 = 4.0;
}

/// Per-patient PK exposure metrics.
#[derive(Debug, Clone, Copy)]
pub struct PatientExposure {
    pub cmax: f64,
    pub tmax: f64,
    pub auc: f64,
}

/// Compute population PK for a cohort of patients (CPU, sequential).
///
/// For each patient, uses provided PK parameters and computes the oral
/// Bateman equation concentration-time curve.
///
/// # Panics
///
/// Panics if `cl_params`, `vd_params`, or `ka_params` length differs
/// from `n_patients`.
#[must_use]
pub fn population_pk_cpu(
    n_patients: usize,
    cl_params: &[f64],
    vd_params: &[f64],
    ka_params: &[f64],
    dose_mg: f64,
    f_bioavail: f64,
    times: &[f64],
) -> Vec<PatientExposure> {
    assert_eq!(cl_params.len(), n_patients);
    assert_eq!(vd_params.len(), n_patients);
    assert_eq!(ka_params.len(), n_patients);

    (0..n_patients)
        .map(|i| {
            let cl = cl_params[i];
            let vd = vd_params[i];
            let ka = ka_params[i];
            let ke = cl / vd;

            let concs: Vec<f64> = times
                .iter()
                .map(|&t| pk_oral_one_compartment(dose_mg, f_bioavail, vd, ka, ke, t))
                .collect();

            let (cmax, tmax) = find_cmax_tmax(times, &concs);
            let auc = auc_trapezoidal(times, &concs);

            PatientExposure { cmax, tmax, auc }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-10;

    // ── Hill dose-response (Exp001 parity) ──────────────────────────

    #[test]
    fn hill_at_ic50_is_half() {
        for drug in ALL_INHIBITORS {
            let r = hill_dose_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);
            assert!(
                (r - 0.5).abs() < TOL,
                "{}: at IC50 got {r}, expected 0.5",
                drug.name
            );
        }
    }

    #[test]
    fn hill_monotonic_per_drug() {
        let concs: Vec<f64> = (0..100).map(|i| 10.0_f64.powf(-1.0 + 5.0 * f64::from(i) / 99.0)).collect();
        for drug in ALL_INHIBITORS {
            let responses = hill_sweep(drug.ic50_jak1_nm, drug.hill_n, 1.0, &concs);
            for w in responses.windows(2) {
                assert!(
                    w[0] <= w[1] + TOL,
                    "{}: not monotonic",
                    drug.name
                );
            }
        }
    }

    #[test]
    fn potency_ordering_at_10nm() {
        let r_bari = hill_dose_response(10.0, BARICITINIB.ic50_jak1_nm, 1.0, 1.0);
        let r_upa = hill_dose_response(10.0, UPADACITINIB.ic50_jak1_nm, 1.0, 1.0);
        let r_ocla = hill_dose_response(10.0, OCLACITINIB.ic50_jak1_nm, 1.0, 1.0);
        let r_abro = hill_dose_response(10.0, ABROCITINIB.ic50_jak1_nm, 1.0, 1.0);
        assert!(r_bari > r_upa, "baricitinib > upadacitinib");
        assert!(r_upa > r_ocla, "upadacitinib > oclacitinib");
        assert!(r_ocla > r_abro, "oclacitinib > abrocitinib");
    }

    #[test]
    fn ec_values_ordered() {
        for drug in ALL_INHIBITORS {
            let ec = compute_ec_values(drug.ic50_jak1_nm, drug.hill_n);
            assert!(ec.ec10 < ec.ec50, "{}: EC10 < EC50", drug.name);
            assert!(ec.ec50 < ec.ec90, "{}: EC50 < EC90", drug.name);
        }
    }

    #[test]
    fn hill_cooperativity_below_ic50() {
        let r_n1 = hill_dose_response(5.0, 10.0, 1.0, 1.0);
        let r_n2 = hill_dose_response(5.0, 10.0, 2.0, 1.0);
        assert!(r_n2 < r_n1, "n=2 steeper below IC50");
    }

    #[test]
    fn hill_cooperativity_above_ic50() {
        let r_n1 = hill_dose_response(20.0, 10.0, 1.0, 1.0);
        let r_n2 = hill_dose_response(20.0, 10.0, 2.0, 1.0);
        assert!(r_n2 > r_n1, "n=2 higher above IC50");
    }

    #[test]
    fn saturation_at_100x() {
        for drug in ALL_INHIBITORS {
            let conc = drug.ic50_jak1_nm * 100.0;
            let r = hill_dose_response(conc, drug.ic50_jak1_nm, drug.hill_n, 1.0);
            assert!(r > 0.99, "{}: saturation {r} at 100x IC50", drug.name);
        }
    }

    // ── One-compartment PK (Exp002 parity) ──────────────────────────

    const DOSE_IV: f64 = 500.0;
    const VD_IV: f64 = 50.0;
    const HL_IV: f64 = 6.0;

    const DOSE_ORAL: f64 = 250.0;
    const F_ORAL: f64 = 0.8;
    const VD_ORAL: f64 = 35.0;
    const HL_ORAL: f64 = 4.0;
    const KA_ORAL: f64 = 1.5;

    #[test]
    fn iv_c0_equals_dose_over_vd() {
        let c0 = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, 0.0);
        assert!((c0 - DOSE_IV / VD_IV).abs() < TOL);
    }

    #[test]
    fn iv_at_half_life_is_half() {
        let c = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, HL_IV);
        let expected = (DOSE_IV / VD_IV) / 2.0;
        assert!((c - expected).abs() < 1e-6);
    }

    #[test]
    fn iv_monotonically_decreasing() {
        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
        let concs: Vec<f64> = times.iter().map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t)).collect();
        for w in concs.windows(2) {
            assert!(w[0] >= w[1] - TOL, "IV not monotonically decreasing");
        }
    }

    #[test]
    fn iv_auc_matches_analytical() {
        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
        let concs: Vec<f64> = times.iter().map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t)).collect();
        let auc_num = auc_trapezoidal(&times, &concs);
        let k_e = LN_2 / HL_IV;
        let auc_ana = DOSE_IV / (VD_IV * k_e);
        let rel_err = (auc_num - auc_ana).abs() / auc_ana;
        assert!(rel_err < 0.01, "AUC rel err {rel_err}");
    }

    #[test]
    fn oral_c0_is_zero() {
        let k_e = LN_2 / HL_ORAL;
        let c = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, 0.0);
        assert!(c.abs() < TOL);
    }

    #[test]
    fn oral_cmax_at_positive_tmax() {
        let k_e = LN_2 / HL_ORAL;
        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
            .collect();
        let (cmax, tmax) = find_cmax_tmax(&times, &concs);
        assert!(cmax > 0.0, "Cmax > 0");
        assert!(tmax > 0.0, "Tmax > 0");
    }

    #[test]
    fn oral_tmax_matches_analytical() {
        let k_e = LN_2 / HL_ORAL;
        let tmax_ana = oral_tmax(KA_ORAL, k_e);

        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
            .collect();
        let (_, tmax_num) = find_cmax_tmax(&times, &concs);
        assert!(
            (tmax_num - tmax_ana).abs() < 0.1,
            "Tmax numerical={tmax_num}, analytical={tmax_ana}"
        );
    }

    #[test]
    fn oral_decays_by_48hr() {
        let k_e = LN_2 / HL_ORAL;
        let c = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, 48.0);
        assert!(c < 0.01, "C(48hr) = {c}");
    }

    #[test]
    fn oral_auc_matches_analytical() {
        let k_e = LN_2 / HL_ORAL;
        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
            .collect();
        let auc_num = auc_trapezoidal(&times, &concs);
        let auc_ana = (F_ORAL * DOSE_ORAL) / (VD_ORAL * k_e);
        let rel_err = (auc_num - auc_ana).abs() / auc_ana;
        assert!(rel_err < 0.01, "Oral AUC rel err {rel_err}");
    }

    #[test]
    fn multiple_iv_doses_accumulate() {
        let c0 = DOSE_IV / VD_IV;
        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
        let c_multi = pk_multiple_dose(
            |t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t),
            8.0,
            6,
            &times,
        );
        let peak_after_first = times
            .iter()
            .zip(c_multi.iter())
            .filter_map(|(t, c)| if *t >= 8.0 { Some(*c) } else { None })
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            peak_after_first > c0,
            "multi-dose peak {peak_after_first} > single C0 {c0}"
        );
    }

    #[test]
    fn all_concentrations_nonneg() {
        let k_e = LN_2 / HL_ORAL;
        let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();

        let c_iv: Vec<f64> = times.iter().map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t)).collect();
        let c_oral: Vec<f64> = times
            .iter()
            .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
            .collect();
        let c_multi = pk_multiple_dose(
            |t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t),
            8.0,
            6,
            &times,
        );

        assert!(c_iv.iter().all(|&c| c >= 0.0), "IV non-negative");
        assert!(c_oral.iter().all(|&c| c >= 0.0), "Oral non-negative");
        assert!(c_multi.iter().all(|&c| c >= -1e-12), "Multi non-negative");
    }

    // ── Two-compartment PK (Exp003 parity) ──────────────────────────

    const V1_2C: f64 = 15.0;
    const K10_2C: f64 = 0.35;
    const K12_2C: f64 = 0.6;
    const K21_2C: f64 = 0.15;
    const DOSE_2C: f64 = 240.0;

    #[test]
    fn two_comp_alpha_gt_beta() {
        let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
        assert!(alpha > beta, "α={alpha} should > β={beta}");
    }

    #[test]
    fn two_comp_sum_identity() {
        let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
        let sum_macro = alpha + beta;
        let sum_micro = K10_2C + K12_2C + K21_2C;
        assert!((sum_macro - sum_micro).abs() < TOL, "α+β = k10+k12+k21");
    }

    #[test]
    fn two_comp_product_identity() {
        let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
        let prod_macro = alpha * beta;
        let prod_micro = K10_2C * K21_2C;
        assert!((prod_macro - prod_micro).abs() < TOL, "α·β = k10·k21");
    }

    #[test]
    fn two_comp_c0() {
        let c = pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, 0.0);
        assert!((c - DOSE_2C / V1_2C).abs() < TOL, "C(0) = Dose/V1");
    }

    #[test]
    fn two_comp_nonneg() {
        let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, t))
            .collect();
        assert!(concs.iter().all(|&c| c >= -1e-12), "all non-negative");
    }

    #[test]
    fn two_comp_monotonic_dec() {
        let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, t))
            .collect();
        for w in concs.windows(2) {
            assert!(w[0] >= w[1] - 1e-12, "central monotonic decreasing");
        }
    }

    #[test]
    fn two_comp_half_lives_ordered() {
        let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
        let t_half_alpha = LN_2 / alpha;
        let t_half_beta = LN_2 / beta;
        assert!(t_half_alpha < t_half_beta, "t½α < t½β");
    }

    #[test]
    fn two_comp_auc_analytical() {
        let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, t))
            .collect();
        let auc_num = auc_trapezoidal(&times, &concs);
        let auc_ana = DOSE_2C / (V1_2C * K10_2C);
        let rel_err = (auc_num - auc_ana).abs() / auc_ana;
        assert!(rel_err < 0.01, "AUC rel err {rel_err}");
    }

    #[test]
    fn two_comp_a_plus_b_eq_c0() {
        let c0 = DOSE_2C / V1_2C;
        let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
        let (a, b) = two_compartment_ab(c0, alpha, beta, K21_2C);
        assert!((a + b - c0).abs() < TOL, "A+B = C0");
    }

    #[test]
    fn two_comp_reduces_to_one() {
        let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
        let c_two: Vec<f64> = times
            .iter()
            .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, 0.0, K21_2C, t))
            .collect();
        let c_one: Vec<f64> = times
            .iter()
            .map(|&t| (DOSE_2C / V1_2C) * (-K10_2C * t).exp())
            .collect();
        let max_diff: f64 = c_two
            .iter()
            .zip(c_one.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        assert!(max_diff < 1e-10, "k12=0 → one-compartment, diff={max_diff}");
    }

    // ── Allometric scaling (Exp004 parity) ──────────────────────────

    #[test]
    fn allometric_identity_at_b0() {
        let val = allometric_scale(100.0, 15.0, 70.0, 0.0);
        assert!((val - 100.0).abs() < TOL, "b=0 → identity");
    }

    #[test]
    fn allometric_hl_scaling() {
        let hl = allometric_scale(
            lokivetmab_canine::HALF_LIFE_DAYS,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::HALF_LIFE,
        );
        assert!(hl > lokivetmab_canine::HALF_LIFE_DAYS, "human t½ > canine");
        assert!((14.0..=28.0).contains(&hl), "scaled t½ in expected range");
    }

    #[test]
    fn allometric_vd_scaling() {
        let vd_animal = lokivetmab_canine::VD_L_KG * lokivetmab_canine::BW_KG;
        let vd_human = allometric_scale(vd_animal, lokivetmab_canine::BW_KG, 70.0, allometric_exp::VOLUME);
        assert!(vd_human > vd_animal, "human Vd > canine");
    }

    #[test]
    fn allometric_cl_ratio() {
        let bw_dog = lokivetmab_canine::BW_KG;
        let bw_human = 70.0_f64;
        let cl_animal = lokivetmab_canine::CL_ML_DAY_KG * bw_dog;
        let cl_human = allometric_scale(cl_animal, bw_dog, bw_human, allometric_exp::CLEARANCE);
        let expected_ratio = (bw_human / bw_dog).powf(0.75);
        let actual_ratio = cl_human / cl_animal;
        assert!((actual_ratio - expected_ratio).abs() < 1e-6);
    }

    #[test]
    fn mab_pk_sc_shape() {
        let vd = allometric_scale(
            lokivetmab_canine::VD_L_KG * lokivetmab_canine::BW_KG,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::VOLUME,
        );
        let hl = allometric_scale(
            lokivetmab_canine::HALF_LIFE_DAYS,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::HALF_LIFE,
        );
        let times: Vec<f64> = (0..2000).map(|i| 56.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times.iter().map(|&t| mab_pk_sc(30.0, vd, hl, t)).collect();
        assert!(concs.iter().all(|&c| c >= -1e-12), "non-negative");
        let (cmax, tmax) = find_cmax_tmax(&times, &concs);
        assert!(cmax > 0.0, "Cmax > 0");
        assert!(tmax > 0.0, "Tmax > 0");
    }

    #[test]
    fn mab_pk_sc_zero_at_t0() {
        let c = mab_pk_sc(30.0, 5.0, 20.0, 0.0);
        assert!(c.abs() < TOL, "C(0) = 0 for SC dosing");
    }

    // ── Population PK (Exp005 parity) ─────────────────────────────────

    #[test]
    fn lognormal_params_roundtrip() {
        let p = LognormalParam { typical: 10.0, cv: 0.30 };
        let (mu, sigma) = p.to_normal_params();
        let recovered_median = mu.exp();
        assert!((recovered_median - 10.0).abs() < 0.5, "median ~ typical");
        assert!(sigma > 0.0);
    }

    #[test]
    fn population_pk_cpu_basic() {
        let n = 5;
        let cl = vec![10.0, 12.0, 8.0, 11.0, 9.0];
        let vd = vec![80.0, 85.0, 75.0, 82.0, 78.0];
        let ka = vec![1.5, 1.8, 1.2, 1.6, 1.4];
        let times: Vec<f64> = (0..200).map(|i| 24.0 * f64::from(i) / 199.0).collect();
        let results = population_pk_cpu(n, &cl, &vd, &ka, 4.0, 0.79, &times);
        assert_eq!(results.len(), n);
        for r in &results {
            assert!(r.auc > 0.0, "AUC > 0");
            assert!(r.cmax > 0.0, "Cmax > 0");
            assert!(r.tmax >= 0.0, "Tmax ≥ 0");
        }
    }

    #[test]
    fn population_pk_higher_cl_lower_auc() {
        let times: Vec<f64> = (0..500).map(|i| 24.0 * f64::from(i) / 499.0).collect();
        let r_low = population_pk_cpu(
            1, &[5.0], &[80.0], &[1.5], 4.0, 0.79, &times,
        );
        let r_high = population_pk_cpu(
            1, &[20.0], &[80.0], &[1.5], 4.0, 0.79, &times,
        );
        assert!(r_low[0].auc > r_high[0].auc, "lower CL → higher AUC");
    }

    #[test]
    fn population_pk_c_zero_at_t0() {
        let times = vec![0.0, 1.0, 2.0];
        let results = population_pk_cpu(
            1, &[10.0], &[80.0], &[1.5], 4.0, 0.79, &times,
        );
        let c0 = pk_oral_one_compartment(4.0, 0.79, 80.0, 1.5, 10.0 / 80.0, 0.0);
        assert!(c0.abs() < TOL, "C(0) = 0 for oral");
        assert!(results[0].cmax > 0.0);
    }
}
