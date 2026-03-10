// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endocrine modeling: testosterone PK, age-related decline, TRT outcomes.
//!
//! ## Tier 1 (CPU)
//!
//! - [`pk_im_depot`]: First-order absorption from IM injection site
//! - [`pellet_concentration`]: Zero-order release depot (pellet implant)
//! - [`testosterone_decline`]: Exponential age-related decline
//! - [`age_at_threshold`]: Age when T crosses a clinical threshold
//! - [`biomarker_trajectory`]: Exponential approach to new setpoint
//! - [`weight_trajectory`]: Logarithmic weight change under TRT
//! - [`hba1c_trajectory`]: `HbA1c` response to TRT
//! - [`hazard_ratio_model`]: Cardiovascular hazard ratio from T level

use crate::pkpd;

// ═══════════════════════════════════════════════════════════════════════
// Testosterone IM Injection PK (Exp030)
// ═══════════════════════════════════════════════════════════════════════

/// Testosterone cypionate published PK parameters.
pub mod testosterone_cypionate {
    use core::f64::consts::LN_2;
    pub const T_HALF_DAYS: f64 = 8.0;
    pub const K_E: f64 = LN_2 / T_HALF_DAYS;
    pub const K_A_IM: f64 = LN_2 / 1.5;
    pub const VD_L: f64 = 70.0;
    pub const F_IM: f64 = 1.0;
    pub const DOSE_WEEKLY_MG: f64 = 100.0;
    pub const DOSE_BIWEEKLY_MG: f64 = 200.0;
    pub const INTERVAL_WEEKLY: f64 = 7.0;
    pub const INTERVAL_BIWEEKLY: f64 = 14.0;
}

/// First-order absorption from IM depot (Bateman equation).
///
/// Same math as [`pkpd::pk_oral_one_compartment`] — IM depot absorption
/// follows the same first-order kinetics as oral absorption.
#[must_use]
pub fn pk_im_depot(dose_mg: f64, f: f64, vd: f64, ka: f64, ke: f64, t: f64) -> f64 {
    pkpd::pk_oral_one_compartment(dose_mg, f, vd, ka, ke, t)
}

/// Compute IM steady-state metrics for a repeated dosing regimen.
///
/// Returns `(ss_cmax, ss_trough)` estimated from the last interval of
/// `n_doses` repeated injections.
/// IM dosing regimen for [`im_steady_state_metrics`].
pub struct ImRegimen {
    pub dose_mg: f64,
    pub f: f64,
    pub vd: f64,
    pub ka: f64,
    pub ke: f64,
    pub interval: f64,
    pub n_doses: usize,
}

#[must_use]
pub fn im_steady_state_metrics(regimen: &ImRegimen, times: &[f64]) -> (f64, f64) {
    let concs = pkpd::pk_multiple_dose(
        |t| {
            pk_im_depot(
                regimen.dose_mg,
                regimen.f,
                regimen.vd,
                regimen.ka,
                regimen.ke,
                t,
            )
        },
        regimen.interval,
        regimen.n_doses,
        times,
    );
    #[expect(clippy::cast_precision_loss, reason = "n_doses always small")]
    let last_start = (regimen.n_doses - 1) as f64 * regimen.interval;
    let mut cmax = 0.0_f64;
    let mut trough = f64::INFINITY;
    for (&t, &c) in times.iter().zip(concs.iter()) {
        if t >= last_start {
            cmax = cmax.max(c);
            trough = trough.min(c);
        }
    }
    if trough == f64::INFINITY {
        trough = 0.0;
    }
    (cmax, trough)
}

// ═══════════════════════════════════════════════════════════════════════
// Testosterone Pellet PK (Exp031)
// ═══════════════════════════════════════════════════════════════════════

/// Pellet PK parameters.
pub mod pellet_params {
    pub const BODY_WEIGHT_LB: f64 = 200.0;
    pub const DOSE_MG: f64 = 10.0 * BODY_WEIGHT_LB;
    pub const DURATION_DAYS: f64 = 150.0;
    pub const RELEASE_RATE: f64 = DOSE_MG / DURATION_DAYS;
}

/// Pellet concentration: zero-order input for `duration` days, then washout.
///
/// During infusion (t <= duration):
///   C(t) = R0/(Vd*ke) * (1 - exp(-ke*t))
///
/// After infusion (t > duration):
///   `C(t) = C_plateau * exp(-ke*(t - duration))`
#[must_use]
pub fn pellet_concentration(t: f64, release_rate: f64, ke: f64, vd: f64, duration: f64) -> f64 {
    let c_ss = release_rate / (vd * ke);
    if t <= duration {
        c_ss * (1.0 - (-ke * t).exp())
    } else {
        c_ss * (1.0 - (-ke * duration).exp()) * (-ke * (t - duration)).exp()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Age-Related Testosterone Decline (Exp032)
// ═══════════════════════════════════════════════════════════════════════

/// Published decline parameters.
pub mod decline_params {
    pub const T0_MEAN_NGDL: f64 = 600.0;
    pub const T0_CV: f64 = 0.25;
    pub const RATE_LOW: f64 = 0.01;
    pub const RATE_MID: f64 = 0.017;
    pub const RATE_HIGH: f64 = 0.03;
    pub const THRESHOLD_CLINICAL: f64 = 300.0;
}

/// Exponential testosterone decline: T(age) = T0 * exp(-rate * (age - onset)).
#[must_use]
pub fn testosterone_decline(t0: f64, rate: f64, age: f64, onset: f64) -> f64 {
    t0 * (-rate * (age - onset)).exp()
}

/// Age when T crosses below a threshold.
///
/// Returns `onset` if T0 is already below threshold.
#[must_use]
pub fn age_at_threshold(t0: f64, rate: f64, threshold: f64, onset: f64) -> f64 {
    if t0 <= threshold {
        return onset;
    }
    onset + (t0 / threshold).ln() / rate
}

// ═══════════════════════════════════════════════════════════════════════
// TRT Outcome Trajectories (Exp033–035)
// ═══════════════════════════════════════════════════════════════════════

/// Logarithmic weight trajectory: `ΔW(t) = delta_final * ln(1+t/τ) / ln(1+T/τ)`.
///
/// Models decelerating weight loss under TRT (Saad 2013 registry).
#[must_use]
pub fn weight_trajectory(month: f64, delta_final: f64, tau: f64, total_months: f64) -> f64 {
    let norm = (total_months / tau).ln_1p();
    if norm.abs() < 1e-15 {
        return 0.0;
    }
    delta_final * (month / tau).ln_1p() / norm
}

/// Biomarker trajectory: exponential approach to new setpoint.
///
/// `value(t) = baseline + (endpoint - baseline) * (1 - exp(-t/τ))`
///
/// Used for LDL, HDL, CRP, SBP, DBP, `HOMA-IR`, fasting glucose, `HbA1c`.
#[must_use]
pub fn biomarker_trajectory(month: f64, baseline: f64, endpoint: f64, tau: f64) -> f64 {
    let delta = endpoint - baseline;
    delta.mul_add(1.0 - (-month / tau).exp(), baseline)
}

/// `HbA1c` trajectory (alias for biomarker trajectory with specific semantics).
#[must_use]
pub fn hba1c_trajectory(month: f64, baseline: f64, delta: f64, tau: f64) -> f64 {
    delta.mul_add(1.0 - (-month / tau).exp(), baseline)
}

/// Cardiovascular hazard ratio model (Sharma 2015).
///
/// If T >= threshold: HR = `hr_normalized` (0.44 in Sharma).
/// If T < threshold: linear interpolation from 1.0 to `hr_normalized`.
#[must_use]
pub fn hazard_ratio_model(t_level: f64, threshold: f64, hr_normalized: f64) -> f64 {
    if t_level >= threshold {
        return hr_normalized;
    }
    let ratio = t_level / threshold;
    (1.0 - hr_normalized).mul_add(-ratio, 1.0)
}

/// Saad 2016 cardiovascular parameters.
pub mod cv_params {
    pub const LDL_BASELINE: f64 = 165.0;
    pub const LDL_ENDPOINT: f64 = 130.0;
    pub const HDL_BASELINE: f64 = 38.0;
    pub const HDL_ENDPOINT: f64 = 55.0;
    pub const CRP_BASELINE: f64 = 1.40;
    pub const CRP_ENDPOINT: f64 = 0.90;
    pub const SBP_BASELINE: f64 = 135.0;
    pub const SBP_ENDPOINT: f64 = 123.0;
    pub const DBP_BASELINE: f64 = 82.0;
    pub const DBP_ENDPOINT: f64 = 76.0;
    pub const TAU_MONTHS: f64 = 12.0;
}

/// Diabetes TRT parameters.
pub mod diabetes_params {
    pub const HBA1C_BASELINE: f64 = 7.60;
    pub const HBA1C_DELTA: f64 = -0.37;
    pub const HOMA_BASELINE: f64 = 4.5;
    pub const HOMA_ENDPOINT: f64 = 3.2;
    pub const FG_BASELINE: f64 = 140.0;
    pub const FG_ENDPOINT: f64 = 120.0;
    pub const TAU_MONTHS: f64 = 3.0;
}

/// Weight/waist TRT parameters (Saad 2013).
pub mod weight_params {
    pub const WEIGHT_LOSS_5YR_KG: f64 = -16.0;
    pub const WAIST_LOSS_5YR_CM: f64 = -12.0;
    pub const BMI_LOSS_5YR: f64 = -5.6;
    pub const TAU_MONTHS: f64 = 6.0;
    pub const TOTAL_MONTHS: f64 = 60.0;
}

// ═══════════════════════════════════════════════════════════════════════
// Population TRT Monte Carlo (Exp036)
// ═══════════════════════════════════════════════════════════════════════

/// Population PK parameters for testosterone cypionate IM.
pub mod pop_trt {
    pub const VD_TYPICAL: f64 = 70.0;
    pub const VD_CV: f64 = 0.25;
    pub const KA_TYPICAL: f64 = 0.462;
    pub const KA_CV: f64 = 0.30;
    pub const KE_TYPICAL: f64 = 0.0866;
    pub const KE_CV: f64 = 0.20;
    pub const T0_TYPICAL: f64 = 600.0;
    pub const T0_CV: f64 = 0.25;
    pub const DECLINE_RATE: f64 = 0.017;
}

/// Lognormal underlying parameters from typical value and CV.
///
/// Returns (mu, sigma) for the normal distribution underlying the lognormal.
#[must_use]
pub fn lognormal_params(typical: f64, cv: f64) -> (f64, f64) {
    crate::pkpd::LognormalParam { typical, cv }.to_normal_params()
}

/// Compute age-adjusted baseline testosterone.
#[must_use]
pub fn age_adjusted_t0(t0: f64, age: f64, decline_rate: f64) -> f64 {
    t0 * (-decline_rate * (age - 30.0)).exp()
}

// ═══════════════════════════════════════════════════════════════════════
// Testosterone-Gut Axis (Exp037)
// ═══════════════════════════════════════════════════════════════════════

/// Anderson localization length from disorder strength (power-law scaling).
///
/// `ξ = ξ_0 * (W / W_ref)^ν` where ν = 1.5.
/// Maintains discrimination across the clinical Pielou range.
#[must_use]
pub fn anderson_localization_length(disorder_w: f64, lattice_size: f64) -> f64 {
    if disorder_w <= 0.0 {
        return 1.0;
    }
    let w_ref = 5.0;
    let xi_0 = lattice_size * 0.5;
    let nu = 1.5;
    xi_0 * (disorder_w / w_ref).powf(nu)
}

/// Pielou evenness → Anderson disorder (linear mapping).
///
/// Delegates to [`crate::microbiome::evenness_to_disorder`] — single
/// source of truth for the `W = J × scale` relationship.
#[must_use]
pub fn evenness_to_disorder(pielou_j: f64, scale: f64) -> f64 {
    crate::microbiome::evenness_to_disorder(pielou_j, scale)
}

/// Metabolic response model: higher ξ → better TRT response.
///
/// Returns weight change (negative = loss).
#[must_use]
pub fn gut_metabolic_response(xi: f64, xi_max: f64, base_response: f64) -> f64 {
    if xi_max <= 0.0 {
        return 0.0;
    }
    base_response * xi / xi_max
}

/// Gut axis parameters.
pub mod gut_axis_params {
    pub const DISORDER_SCALE: f64 = 5.0;
    pub const LATTICE_SIZE: f64 = 100.0;
    pub const BASE_RESPONSE_KG: f64 = -16.0;
}

// ═══════════════════════════════════════════════════════════════════════
// Cross-Track: HRV × TRT Cardiovascular (Exp038 — Mok D3)
// ═══════════════════════════════════════════════════════════════════════

/// Model HRV improvement from TRT-induced cardiovascular benefit.
///
/// As TRT normalizes testosterone → reduced inflammation → improved
/// autonomic balance → increased HRV.
///
/// `SDNN(months) = sdnn_base + delta_sdnn * (1 - exp(-months / tau))`
///
/// Published correlation: SDNN < 50ms associated with ~5× cardiac mortality risk
/// (Kleiger 1987, NEJM). TRT normalization → expected +10-20ms SDNN improvement.
#[must_use]
pub fn hrv_trt_response(
    sdnn_base_ms: f64,
    delta_sdnn_ms: f64,
    tau_months: f64,
    months: f64,
) -> f64 {
    delta_sdnn_ms.mul_add(1.0 - (-months / tau_months).exp(), sdnn_base_ms)
}

/// Composite cardiac risk score from HRV + testosterone level.
///
/// Risk = `baseline_risk` * `hrv_factor` * `testosterone_factor`
/// - HRV factor: SDNN < 50ms doubles risk, SDNN > 100ms halves risk
/// - T factor: T < 300 ng/dL doubles risk, T > 500 ng/dL halves risk
#[must_use]
pub fn cardiac_risk_composite(sdnn_ms: f64, testosterone_ng_dl: f64, baseline_risk: f64) -> f64 {
    let hrv_factor = if sdnn_ms < 50.0 {
        2.0 - sdnn_ms / 50.0
    } else if sdnn_ms > 100.0 {
        0.5
    } else {
        1.0 - 0.5 * (sdnn_ms - 50.0) / 50.0
    };

    let t_factor = if testosterone_ng_dl < 300.0 {
        2.0 - testosterone_ng_dl / 300.0
    } else if testosterone_ng_dl > 500.0 {
        0.5
    } else {
        1.0 - 0.5 * (testosterone_ng_dl - 300.0) / 200.0
    };

    baseline_risk * hrv_factor * t_factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkpd;

    const TOL: f64 = 1e-10;

    // ── IM Depot PK (Exp030) ───────────────────────────────────────────

    #[test]
    fn im_c0_is_zero() {
        let c = pk_im_depot(100.0, 1.0, 70.0, 0.46, 0.087, 0.0);
        assert!(c.abs() < TOL, "C(0) = 0 for IM depot");
    }

    #[test]
    fn im_cmax_positive() {
        use testosterone_cypionate as tc;
        let times: Vec<f64> = (0..2000).map(|i| 56.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| {
                pk_im_depot(
                    tc::DOSE_WEEKLY_MG,
                    tc::F_IM,
                    tc::VD_L,
                    tc::K_A_IM,
                    tc::K_E,
                    t,
                )
            })
            .collect();
        let (cmax, tmax) = pkpd::find_cmax_tmax(&times, &concs);
        assert!(cmax > 0.0, "Cmax > 0");
        assert!(tmax > 0.0, "Tmax > 0");
    }

    #[test]
    fn im_nonneg() {
        use testosterone_cypionate as tc;
        let times: Vec<f64> = (0..2000).map(|i| 56.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| {
                pk_im_depot(
                    tc::DOSE_WEEKLY_MG,
                    tc::F_IM,
                    tc::VD_L,
                    tc::K_A_IM,
                    tc::K_E,
                    t,
                )
            })
            .collect();
        assert!(concs.iter().all(|&c| c >= -1e-12));
    }

    #[test]
    fn im_weekly_accumulates() {
        use testosterone_cypionate as tc;
        let times: Vec<f64> = (0..2000).map(|i| 56.0 * f64::from(i) / 1999.0).collect();
        let concs_single: Vec<f64> = times
            .iter()
            .map(|&t| {
                pk_im_depot(
                    tc::DOSE_WEEKLY_MG,
                    tc::F_IM,
                    tc::VD_L,
                    tc::K_A_IM,
                    tc::K_E,
                    t,
                )
            })
            .collect();
        let (cmax_single, _) = pkpd::find_cmax_tmax(&times, &concs_single);

        let concs_multi = pkpd::pk_multiple_dose(
            |t| {
                pk_im_depot(
                    tc::DOSE_WEEKLY_MG,
                    tc::F_IM,
                    tc::VD_L,
                    tc::K_A_IM,
                    tc::K_E,
                    t,
                )
            },
            tc::INTERVAL_WEEKLY,
            8,
            &times,
        );
        let threshold_t = 5.0 * tc::INTERVAL_WEEKLY;
        let cmax_ss = times
            .iter()
            .zip(concs_multi.iter())
            .filter(|(t, _)| **t >= threshold_t)
            .map(|(_, c)| *c)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            cmax_ss > cmax_single,
            "steady-state Cmax > single-dose Cmax"
        );
    }

    // ── Pellet PK (Exp031) ─────────────────────────────────────────────

    #[test]
    fn pellet_c0_is_zero() {
        use testosterone_cypionate as tc;
        let c = pellet_concentration(
            0.0,
            pellet_params::RELEASE_RATE,
            tc::K_E,
            tc::VD_L,
            pellet_params::DURATION_DAYS,
        );
        assert!(c.abs() < TOL, "Pellet C(0) = 0");
    }

    #[test]
    fn pellet_approaches_steady_state() {
        use testosterone_cypionate as tc;
        let c_ss = pellet_params::RELEASE_RATE / (tc::VD_L * tc::K_E);
        let c_5hl = pellet_concentration(
            5.0 * tc::T_HALF_DAYS,
            pellet_params::RELEASE_RATE,
            tc::K_E,
            tc::VD_L,
            pellet_params::DURATION_DAYS,
        );
        assert!(c_5hl / c_ss > 0.95, "reaches 95% of C_ss by 5 half-lives");
    }

    #[test]
    fn pellet_washout_decays() {
        use testosterone_cypionate as tc;
        let c_end = pellet_concentration(
            pellet_params::DURATION_DAYS,
            pellet_params::RELEASE_RATE,
            tc::K_E,
            tc::VD_L,
            pellet_params::DURATION_DAYS,
        );
        let c_post = pellet_concentration(
            pellet_params::DURATION_DAYS + 30.0,
            pellet_params::RELEASE_RATE,
            tc::K_E,
            tc::VD_L,
            pellet_params::DURATION_DAYS,
        );
        assert!(
            c_post < c_end,
            "washout: C decreases after pellet exhaustion"
        );
    }

    #[test]
    fn pellet_nonneg() {
        use testosterone_cypionate as tc;
        for i in 0..3000 {
            let t = 180.0 * f64::from(i) / 2999.0;
            let c = pellet_concentration(
                t,
                pellet_params::RELEASE_RATE,
                tc::K_E,
                tc::VD_L,
                pellet_params::DURATION_DAYS,
            );
            assert!(c >= -1e-12, "non-negative at t={t}");
        }
    }

    // ── Age Decline (Exp032) ───────────────────────────────────────────

    #[test]
    fn decline_at_onset_equals_t0() {
        let t = testosterone_decline(600.0, 0.017, 30.0, 30.0);
        assert!((t - 600.0).abs() < TOL);
    }

    #[test]
    fn decline_monotonic() {
        for age in 31..=90 {
            let t_prev = testosterone_decline(600.0, 0.017, f64::from(age - 1), 30.0);
            let t_curr = testosterone_decline(600.0, 0.017, f64::from(age), 30.0);
            assert!(t_curr <= t_prev, "monotonically decreasing at age {age}");
        }
    }

    #[test]
    fn decline_always_positive() {
        for age in 30..=100 {
            let t = testosterone_decline(600.0, 0.03, f64::from(age), 30.0);
            assert!(t > 0.0, "always positive at age {age}");
        }
    }

    #[test]
    fn age_threshold_ordering() {
        let age_low = age_at_threshold(600.0, 0.01, 300.0, 30.0);
        let age_mid = age_at_threshold(600.0, 0.017, 300.0, 30.0);
        let age_high = age_at_threshold(600.0, 0.03, 300.0, 30.0);
        assert!(age_high < age_mid, "faster decline → earlier threshold");
        assert!(age_mid < age_low, "slower decline → later threshold");
    }

    // ── Weight Trajectory (Exp033) ─────────────────────────────────────

    #[test]
    fn weight_at_zero_is_zero() {
        let dw = weight_trajectory(0.0, -16.0, 6.0, 60.0);
        assert!(dw.abs() < TOL);
    }

    #[test]
    fn weight_at_endpoint_matches() {
        let dw = weight_trajectory(60.0, -16.0, 6.0, 60.0);
        assert!((dw - (-16.0)).abs() < 1e-8);
    }

    #[test]
    fn weight_front_loaded() {
        let dw_24 = weight_trajectory(24.0, -16.0, 6.0, 60.0);
        let dw_60 = weight_trajectory(60.0, -16.0, 6.0, 60.0);
        let frac = dw_24 / dw_60;
        assert!(frac > 0.60, "front-loaded: {frac:.2} of loss by 24 months");
    }

    // ── Biomarker Trajectory (Exp034) ──────────────────────────────────

    #[test]
    fn biomarker_baseline_at_t0() {
        let v = biomarker_trajectory(0.0, 165.0, 130.0, 12.0);
        assert!((v - 165.0).abs() < TOL);
    }

    #[test]
    fn biomarker_approaches_endpoint() {
        let v = biomarker_trajectory(120.0, 165.0, 130.0, 12.0);
        assert!((v - 130.0).abs() < 0.5, "approaches endpoint at 10τ");
    }

    #[test]
    fn ldl_decreases() {
        let v60 = biomarker_trajectory(60.0, 165.0, 130.0, 12.0);
        assert!(v60 < 165.0, "LDL decreases");
    }

    #[test]
    fn hdl_increases() {
        let v60 = biomarker_trajectory(60.0, 38.0, 55.0, 12.0);
        assert!(v60 > 38.0, "HDL increases");
    }

    // ── Hazard Ratio (Exp034) ──────────────────────────────────────────

    #[test]
    fn hr_at_normalization() {
        let hr = hazard_ratio_model(600.0, 300.0, 0.44);
        assert!((hr - 0.44).abs() < TOL);
    }

    #[test]
    fn hr_ordering() {
        let hr_low = hazard_ratio_model(200.0, 300.0, 0.44);
        let hr_norm = hazard_ratio_model(600.0, 300.0, 0.44);
        assert!(hr_norm <= hr_low, "normalized HR <= low-T HR");
    }

    // ── HbA1c / Diabetes (Exp035) ──────────────────────────────────────

    #[test]
    fn hba1c_baseline_correct() {
        let v = hba1c_trajectory(0.0, 7.60, -0.37, 3.0);
        assert!((v - 7.60).abs() < TOL);
    }

    #[test]
    fn hba1c_decreases() {
        let v12 = hba1c_trajectory(12.0, 7.60, -0.37, 3.0);
        assert!(v12 < 7.60, "HbA1c decreases with TRT");
    }

    #[test]
    fn hba1c_monotonic() {
        let vals: Vec<f64> = (0..=12)
            .map(|m| hba1c_trajectory(f64::from(m), 7.60, -0.37, 3.0))
            .collect();
        for w in vals.windows(2) {
            assert!(w[0] >= w[1] - 1e-12, "monotonically decreasing");
        }
    }

    #[test]
    fn homa_improves() {
        let v12 = biomarker_trajectory(12.0, 4.5, 3.2, 3.0);
        assert!(v12 < 4.5, "HOMA-IR decreases");
    }

    // ── Population TRT (Exp036) ────────────────────────────────────────

    #[test]
    fn lognormal_params_mean() {
        let (mu, sigma) = lognormal_params(70.0, 0.25);
        // Lognormal mean = exp(mu + σ²/2) should equal typical
        let recovered_mean = (mu + sigma * sigma / 2.0).exp();
        assert!(
            (recovered_mean - 70.0).abs() < 0.01,
            "mean ≈ typical: got {recovered_mean}"
        );
        assert!(sigma > 0.0);
    }

    #[test]
    fn age_adjusted_t0_at_30() {
        let t = age_adjusted_t0(600.0, 30.0, 0.017);
        assert!((t - 600.0).abs() < TOL);
    }

    #[test]
    fn age_adjusted_t0_declines() {
        let t30 = age_adjusted_t0(600.0, 30.0, 0.017);
        let t60 = age_adjusted_t0(600.0, 60.0, 0.017);
        assert!(t60 < t30, "T declines with age");
    }

    // ── Gut Axis (Exp037) ──────────────────────────────────────────────

    #[test]
    fn anderson_xi_positive() {
        let xi = anderson_localization_length(3.0, 100.0);
        assert!(xi > 0.0);
    }

    #[test]
    fn anderson_xi_increases_with_disorder() {
        let xi_low = anderson_localization_length(1.0, 100.0);
        let xi_high = anderson_localization_length(4.0, 100.0);
        assert!(xi_high > xi_low, "higher disorder → higher ξ");
    }

    #[test]
    fn anderson_xi_zero_disorder() {
        let xi = anderson_localization_length(0.0, 100.0);
        assert!((xi - 1.0).abs() < TOL, "zero disorder → localized (ξ=1)");
    }

    #[test]
    fn evenness_to_disorder_linear() {
        let w1 = evenness_to_disorder(0.5, 5.0);
        let w2 = evenness_to_disorder(1.0, 5.0);
        assert!((w1 - 2.5).abs() < TOL);
        assert!((w2 - 5.0).abs() < TOL);
    }

    #[test]
    fn gut_response_scales_with_xi() {
        let r_low = gut_metabolic_response(10.0, 50.0, -16.0);
        let r_high = gut_metabolic_response(40.0, 50.0, -16.0);
        assert!(
            r_high < r_low,
            "higher ξ → more weight loss (more negative)"
        );
    }

    #[test]
    fn im_pk_deterministic() {
        let times: Vec<f64> = (0..100).map(|i| 56.0 * f64::from(i) / 99.0).collect();
        let c1 = pkpd::pk_multiple_dose(
            |t| pk_im_depot(100.0, 1.0, 70.0, 0.462, 0.0866, t),
            7.0,
            8,
            &times,
        );
        let c2 = pkpd::pk_multiple_dose(
            |t| pk_im_depot(100.0, 1.0, 70.0, 0.462, 0.0866, t),
            7.0,
            8,
            &times,
        );
        for (a, b) in c1.iter().zip(c2.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "IM PK must be bit-identical");
        }
    }

    // ── HRV × TRT Cardiovascular (Exp038) ────────────────────────────────

    #[test]
    fn hrv_trt_response_at_zero() {
        let s = hrv_trt_response(40.0, 20.0, 6.0, 0.0);
        assert!((s - 40.0).abs() < 1e-10, "at t=0, SDNN=base");
    }

    #[test]
    fn hrv_trt_response_improves() {
        let s0 = hrv_trt_response(40.0, 20.0, 6.0, 0.0);
        let s12 = hrv_trt_response(40.0, 20.0, 6.0, 12.0);
        let s_inf = hrv_trt_response(40.0, 20.0, 6.0, 120.0);
        assert!(s12 > s0, "SDNN improves with TRT");
        assert!((s_inf - 60.0).abs() < 1.0, "approaches base + delta");
    }

    #[test]
    fn cardiac_risk_low_sdnn_high() {
        let risk_low = cardiac_risk_composite(30.0, 400.0, 1.0);
        let risk_high = cardiac_risk_composite(120.0, 400.0, 1.0);
        assert!(risk_low > risk_high, "low SDNN → higher risk");
    }

    #[test]
    fn cardiac_risk_low_testosterone_high() {
        let risk_low_t = cardiac_risk_composite(80.0, 200.0, 1.0);
        let risk_high_t = cardiac_risk_composite(80.0, 600.0, 1.0);
        assert!(risk_low_t > risk_high_t, "low T → higher risk");
    }
}
