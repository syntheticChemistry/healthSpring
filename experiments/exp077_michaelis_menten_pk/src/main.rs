// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp077: Michaelis-Menten Nonlinear Pharmacokinetics
//!
//! Validates capacity-limited elimination (phenytoin-like).
//! Key property: dose-dependent half-life and supralinear AUC.
//!
//! Reference: Rowland & Tozer Ch. 20, Ludden et al. 1977.

use healthspring_barracuda::pkpd;

macro_rules! check {
    ($p:expr, $f:expr, $name:expr, $cond:expr) => {
        if $cond {
            $p += 1;
            println!("  [PASS] {}", $name);
        } else {
            $f += 1;
            println!("  [FAIL] {}", $name);
        }
    };
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp077 — Michaelis-Menten Nonlinear PK");
    println!("{}", "=".repeat(72));

    let params = &pkpd::PHENYTOIN_PARAMS;

    // Check 1: Initial concentration = dose/Vd
    println!("\n--- Check 1: C0 = dose/Vd ---");
    let (_, concs) = pkpd::mm_pk_simulate(params, 300.0, 1.0, 0.001);
    let c0_expected = 300.0 / params.vd;
    check!(
        passed,
        failed,
        format!("C0={:.2} ≈ {c0_expected:.2}", concs[0]),
        (concs[0] - c0_expected).abs() < 1e-10
    );

    // Check 2: Monotone decline
    println!("\n--- Check 2: Monotone decline ---");
    let (_, concs10) = pkpd::mm_pk_simulate(params, 300.0, 10.0, 0.001);
    let monotone = concs10.windows(2).all(|w| w[1] <= w[0] + 1e-14);
    check!(
        passed,
        failed,
        "concentration declines monotonically",
        monotone
    );

    // Check 3: Dose-dependent half-life
    println!("\n--- Check 3: Half-life increases with concentration ---");
    let t_half_low = pkpd::mm_apparent_half_life(params, 1.0);
    let t_half_mid = pkpd::mm_apparent_half_life(params, 5.0);
    let t_half_high = pkpd::mm_apparent_half_life(params, 20.0);
    check!(
        passed,
        failed,
        format!("t½: {t_half_low:.2} < {t_half_mid:.2} < {t_half_high:.2}"),
        t_half_low < t_half_mid && t_half_mid < t_half_high
    );

    // Check 4: At low C, approaches first-order
    println!("\n--- Check 4: Low-dose linearity ---");
    let ratio_low = pkpd::mm_nonlinearity_ratio(params, 10.0, 20.0);
    check!(
        passed,
        failed,
        format!("nonlinearity ratio at low dose = {ratio_low:.3} ≈ 1.0"),
        (ratio_low - 1.0).abs() < 0.15
    );

    // Check 5: At high C, supralinear AUC
    println!("\n--- Check 5: High-dose supralinearity ---");
    let ratio_high = pkpd::mm_nonlinearity_ratio(params, 200.0, 400.0);
    check!(
        passed,
        failed,
        format!("nonlinearity ratio at high dose = {ratio_high:.3} > 1.0"),
        ratio_high > 1.0
    );

    // Check 6: Css exists for rate < Vmax
    println!("\n--- Check 6: Css for infusion rate < Vmax ---");
    let css = pkpd::mm_css_infusion(params, 250.0);
    check!(
        passed,
        failed,
        format!("Css = {:.2} mg/L", css.unwrap_or(0.0)),
        css.is_some() && css.unwrap() > 0.0
    );

    // Check 7: No Css for rate >= Vmax
    println!("\n--- Check 7: No Css for rate >= Vmax ---");
    let no_css = pkpd::mm_css_infusion(params, 500.0);
    check!(
        passed,
        failed,
        "returns None (accumulates)",
        no_css.is_none()
    );

    // Check 8: Css increases steeply near Vmax
    println!("\n--- Check 8: Css steep near Vmax ---");
    let css_low = pkpd::mm_css_infusion(params, 100.0).unwrap();
    let css_high = pkpd::mm_css_infusion(params, 400.0).unwrap();
    check!(
        passed,
        failed,
        format!("Css(400) / Css(100) = {:.1} > 4.0", css_high / css_low),
        css_high / css_low > 4.0
    );

    // Check 9: Numerical AUC ≈ analytical AUC
    println!("\n--- Check 9: Numerical vs analytical AUC ---");
    let (_, concs_long) = pkpd::mm_pk_simulate(params, 300.0, 20.0, 0.0001);
    let num_auc = pkpd::mm_auc(&concs_long, 0.0001);
    let anal_auc = pkpd::mm_auc_analytical(params, 300.0);
    let rel_err = (num_auc - anal_auc).abs() / anal_auc;
    check!(
        passed,
        failed,
        format!("AUC numerical={num_auc:.2}, analytical={anal_auc:.2}, err={rel_err:.4}"),
        rel_err < 0.02
    );

    // Check 10: AUC(2D)/AUC(D) > 2 (supralinear)
    println!("\n--- Check 10: AUC supralinearity ---");
    let auc_200 = pkpd::mm_auc_analytical(params, 200.0);
    let auc_400 = pkpd::mm_auc_analytical(params, 400.0);
    let auc_ratio = auc_400 / auc_200;
    check!(
        passed,
        failed,
        format!("AUC(400)/AUC(200) = {auc_ratio:.3} > 2.0"),
        auc_ratio > 2.0
    );

    // Check 11: Concentration reaches near-zero
    println!("\n--- Check 11: Elimination completes ---");
    let c_final = *concs_long.last().unwrap();
    check!(
        passed,
        failed,
        format!("C(20 days) = {c_final:.4} ≈ 0"),
        c_final < 0.01
    );

    // Check 12: Css formula matches steady-state Michaelis-Menten
    println!("\n--- Check 12: Css formula validation ---");
    let rate = 250.0;
    let css_val = pkpd::mm_css_infusion(params, rate).unwrap();
    let elim_at_css = params.vmax * css_val / (params.km + css_val);
    check!(
        passed,
        failed,
        format!("elim at Css = {elim_at_css:.2} ≈ rate = {rate:.0}"),
        (elim_at_css - rate).abs() < 0.01
    );

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("Exp077 Michaelis-Menten PK: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
