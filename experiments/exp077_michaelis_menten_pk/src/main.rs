// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp077: Michaelis-Menten Nonlinear Pharmacokinetics
//!
//! Validates capacity-limited elimination (phenytoin-like).
//! Key property: dose-dependent half-life and supralinear AUC.
//!
//! Reference: Rowland & Tozer Ch. 20, Ludden et al. 1977.

use healthspring_barracuda::pkpd;
use healthspring_barracuda::tolerances::{
    ANDERSON_IDENTITY, EXPONENTIAL_RESIDUAL, MACHINE_EPSILON, TEST_ASSERTION_2_PERCENT,
    WASHOUT_HALF_LIFE,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp077 Michaelis-Menten PK");

    let params = &pkpd::PHENYTOIN_PARAMS;

    // Check 1: Initial concentration = dose/Vd
    let (_, concs) = pkpd::mm_pk_simulate(params, 300.0, 1.0, 0.001);
    let c0_expected = 300.0 / params.vd;
    h.check_abs("C0 = dose/Vd", concs[0], c0_expected, MACHINE_EPSILON);

    // Check 2: Monotone decline
    let (_, concs10) = pkpd::mm_pk_simulate(params, 300.0, 10.0, 0.001);
    let monotone = concs10.windows(2).all(|w| w[1] <= w[0] + ANDERSON_IDENTITY);
    h.check_bool("Concentration declines monotonically", monotone);

    // Check 3: Dose-dependent half-life
    let t_half_low = pkpd::mm_apparent_half_life(params, 1.0);
    let t_half_mid = pkpd::mm_apparent_half_life(params, 5.0);
    let t_half_high = pkpd::mm_apparent_half_life(params, 20.0);
    h.check_bool(
        "t½: low < mid < high",
        t_half_low < t_half_mid && t_half_mid < t_half_high,
    );

    // Check 4: At low C, approaches first-order
    let ratio_low = pkpd::mm_nonlinearity_ratio(params, 10.0, 20.0);
    h.check_abs(
        "Nonlinearity ratio at low dose ≈ 1.0",
        ratio_low,
        1.0,
        WASHOUT_HALF_LIFE,
    );

    // Check 5: At high C, supralinear AUC
    let ratio_high = pkpd::mm_nonlinearity_ratio(params, 200.0, 400.0);
    h.check_bool("Nonlinearity ratio at high dose > 1.0", ratio_high > 1.0);

    // Check 6: Css exists for rate < Vmax
    let css = pkpd::mm_css_infusion(params, 250.0);
    h.check_bool("Css for infusion rate < Vmax", css.is_some_and(|c| c > 0.0));

    // Check 7: No Css for rate >= Vmax
    let no_css = pkpd::mm_css_infusion(params, 500.0);
    h.check_bool("No Css for rate >= Vmax (returns None)", no_css.is_none());

    // Check 8: Css increases steeply near Vmax
    let Some(css_low) = pkpd::mm_css_infusion(params, 100.0) else {
        eprintln!("FAIL: Css for rate 100");
        std::process::exit(1);
    };
    let Some(css_high) = pkpd::mm_css_infusion(params, 400.0) else {
        eprintln!("FAIL: Css for rate 400");
        std::process::exit(1);
    };
    h.check_bool("Css(400) / Css(100) > 4.0", css_high / css_low > 4.0);

    // Check 9: Numerical AUC ≈ analytical AUC
    let (_, concs_long) = pkpd::mm_pk_simulate(params, 300.0, 20.0, 0.0001);
    let num_auc = pkpd::mm_auc(&concs_long, 0.0001);
    let anal_auc = pkpd::mm_auc_analytical(params, 300.0);
    let rel_err = (num_auc - anal_auc).abs() / anal_auc;
    h.check_upper(
        "AUC numerical vs analytical rel_err",
        rel_err,
        TEST_ASSERTION_2_PERCENT,
    );

    // Check 10: AUC(2D)/AUC(D) > 2 (supralinear)
    let auc_200 = pkpd::mm_auc_analytical(params, 200.0);
    let auc_400 = pkpd::mm_auc_analytical(params, 400.0);
    let auc_ratio = auc_400 / auc_200;
    h.check_bool("AUC(400)/AUC(200) > 2.0", auc_ratio > 2.0);

    // Check 11: Concentration reaches near-zero
    let Some(c_last) = concs_long.last() else {
        eprintln!("FAIL: concentration curve has at least one point");
        std::process::exit(1);
    };
    let c_final = *c_last;
    h.check_upper("C(20 days) ≈ 0", c_final, EXPONENTIAL_RESIDUAL);

    // Check 12: Css formula matches steady-state Michaelis-Menten
    let rate = 250.0;
    let Some(css_val) = pkpd::mm_css_infusion(params, rate) else {
        eprintln!("FAIL: Css for rate 250");
        std::process::exit(1);
    };
    let elim_at_css = params.vmax * css_val / (params.km + css_val);
    h.check_abs(
        "Elim at Css ≈ rate",
        elim_at_css,
        rate,
        EXPONENTIAL_RESIDUAL,
    );

    h.exit();
}
