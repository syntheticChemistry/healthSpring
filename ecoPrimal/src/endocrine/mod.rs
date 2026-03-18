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

mod testosterone;
mod trt_outcomes;

// Re-exports: testosterone PK, pellet, age decline, population
pub use testosterone::{
    ImRegimen, age_adjusted_t0, age_at_threshold, decline_params, im_steady_state_metrics,
    lognormal_params, pellet_concentration, pellet_params, pk_im_depot, pop_trt,
    testosterone_cypionate, testosterone_decline,
};

// Re-exports: TRT outcomes (weight, CV, diabetes, gut axis, HRV)
pub use trt_outcomes::{
    anderson_localization_length, anderson_scaling, biomarker_trajectory, cardiac_risk_composite,
    cardiac_risk_thresholds, cv_params, diabetes_params, evenness_to_disorder, gut_axis_params,
    gut_metabolic_response, hazard_ratio_model, hba1c_trajectory, hrv_trt_response, weight_params,
    weight_trajectory,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkpd;
    use crate::tolerances;

    // ── IM Depot PK (Exp030) ───────────────────────────────────────────

    #[test]
    fn im_c0_is_zero() {
        let c = pk_im_depot(100.0, 1.0, 70.0, 0.46, 0.087, 0.0);
        assert!(
            c.abs() < tolerances::TEST_ASSERTION_TIGHT,
            "C(0) = 0 for IM depot"
        );
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
        assert!(
            concs
                .iter()
                .all(|&c| c >= -tolerances::MACHINE_EPSILON_TIGHT)
        );
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
        assert!(
            c.abs() < tolerances::TEST_ASSERTION_TIGHT,
            "Pellet C(0) = 0"
        );
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
            assert!(
                c >= -tolerances::MACHINE_EPSILON_TIGHT,
                "non-negative at t={t}"
            );
        }
    }

    // ── Age Decline (Exp032) ───────────────────────────────────────────

    #[test]
    fn decline_at_onset_equals_t0() {
        let t = testosterone_decline(600.0, 0.017, 30.0, 30.0);
        assert!((t - 600.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
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
        assert!(dw.abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn weight_at_endpoint_matches() {
        let dw = weight_trajectory(60.0, -16.0, 6.0, 60.0);
        assert!((dw - (-16.0)).abs() < tolerances::DIVERSITY_CROSS_VALIDATE);
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
        assert!((v - 165.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn biomarker_approaches_endpoint() {
        let v = biomarker_trajectory(120.0, 165.0, 130.0, 12.0);
        assert!(
            (v - 130.0).abs() < tolerances::BIOMARKER_ENDPOINT,
            "approaches endpoint at 10τ"
        );
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
        assert!((hr - 0.44).abs() < tolerances::TEST_ASSERTION_TIGHT);
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
        assert!((v - 7.60).abs() < tolerances::TEST_ASSERTION_TIGHT);
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
            assert!(
                w[0] >= w[1] - tolerances::MACHINE_EPSILON_TIGHT,
                "monotonically decreasing"
            );
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
            (recovered_mean - 70.0).abs() < tolerances::TEST_ASSERTION_LOOSE,
            "mean ≈ typical: got {recovered_mean}"
        );
        assert!(sigma > 0.0);
    }

    #[test]
    fn age_adjusted_t0_at_30() {
        let t = age_adjusted_t0(600.0, 30.0, 0.017);
        assert!((t - 600.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
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
        assert!(
            (xi - 1.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "zero disorder → localized (ξ=1)"
        );
    }

    #[test]
    fn evenness_to_disorder_linear() {
        let w1 = evenness_to_disorder(0.5, 5.0);
        let w2 = evenness_to_disorder(1.0, 5.0);
        assert!((w1 - 2.5).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert!((w2 - 5.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
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
        assert!(
            (s - 40.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "at t=0, SDNN=base"
        );
    }

    #[test]
    fn hrv_trt_response_improves() {
        let s0 = hrv_trt_response(40.0, 20.0, 6.0, 0.0);
        let s12 = hrv_trt_response(40.0, 20.0, 6.0, 12.0);
        let s_inf = hrv_trt_response(40.0, 20.0, 6.0, 120.0);
        assert!(s12 > s0, "SDNN improves with TRT");
        assert!(
            (s_inf - 60.0).abs() < tolerances::LOKIVETMAB_DURATION,
            "approaches base + delta"
        );
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
