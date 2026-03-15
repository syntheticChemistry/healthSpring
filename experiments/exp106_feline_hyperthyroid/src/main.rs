// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! Exp106: Feline hyperthyroidism methimazole PK (Trepanier 2006, CM-007)
//!
//! Validates methimazole PK simulation, half-life, Css, and T4 response.

use healthspring_barracuda::comparative::feline::{
    methimazole_apparent_half_life, methimazole_css, methimazole_simulate, t4_response,
    FELINE_METHIMAZOLE, HUMAN_METHIMAZOLE,
};
use healthspring_barracuda::provenance::{log_analytical, AnalyticalProvenance};
use healthspring_barracuda::tolerances::{
    DETERMINISM, FELINE_MM_PK, FELINE_T4_RESPONSE, MACHINE_EPSILON,
};
use healthspring_barracuda::validation::ValidationHarness;

const DOSE_MG: f64 = 2.5;
const T_END_HR: f64 = 48.0;
const DT_HR: f64 = 0.5;
const T4_BASELINE: f64 = 8.0;

fn main() {
    let mut h = ValidationHarness::new("exp106_feline_hyperthyroid");

    log_analytical(&AnalyticalProvenance {
        formula: "dC/dt = -Vmax×C/(Vd×(Km+C))",
        reference: "Trepanier 2006, JVIM 20:18",
        doi: None,
    });

    // 1. methimazole_simulate: C decays from C0
    let (_times, concs) = methimazole_simulate(&FELINE_METHIMAZOLE, DOSE_MG, T_END_HR, DT_HR);
    let c0 = concs.first().copied().unwrap_or(0.0);
    let c_end = concs.last().copied().unwrap_or(0.0);
    h.check_bool(
        "methimazole_simulate: C decays from C0",
        c_end < c0,
    );

    // 2. methimazole_simulate: all concentrations ≥ 0
    h.check_bool(
        "methimazole_simulate: all concentrations ≥ 0",
        concs.iter().all(|&c| c >= 0.0),
    );

    // 3. methimazole_simulate: returns correct number of time points
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "T_END_HR/DT_HR is small positive, no truncation in practice")]
    let expected_steps = (T_END_HR / DT_HR).ceil() as usize;
    let expected_count = expected_steps + 1;
    h.check_exact(
        "methimazole_simulate: returns correct number of time points",
        concs.len() as u64,
        expected_count as u64,
    );

    // 4. methimazole_apparent_half_life: increases with concentration (nonlinear PK hallmark)
    let t_half_low = methimazole_apparent_half_life(&FELINE_METHIMAZOLE, 0.5);
    let t_half_high = methimazole_apparent_half_life(&FELINE_METHIMAZOLE, 5.0);
    h.check_bool(
        "methimazole_apparent_half_life: increases with concentration",
        t_half_high > t_half_low,
    );

    // 5. methimazole_apparent_half_life: at C=0, t½ ≈ 0.693 × Km × Vd / Vmax (first-order limit)
    let t_half_at_0 = methimazole_apparent_half_life(&FELINE_METHIMAZOLE, 0.0);
    let expected_first_order = 0.693
        * FELINE_METHIMAZOLE.km
        * FELINE_METHIMAZOLE.vd
        / FELINE_METHIMAZOLE.vmax;
    h.check_abs(
        "methimazole_apparent_half_life: at C=0, first-order limit",
        t_half_at_0,
        expected_first_order,
        FELINE_MM_PK,
    );

    // 6. methimazole_css: exists for rate < Vmax
    let rate_low = 1.0;
    let css_opt = methimazole_css(&FELINE_METHIMAZOLE, rate_low);
    h.check_bool(
        "methimazole_css: exists for rate < Vmax",
        css_opt.is_some() && css_opt.unwrap_or(0.0) > 0.0,
    );

    // 7. methimazole_css: None for rate ≥ Vmax
    let css_at_vmax = methimazole_css(&FELINE_METHIMAZOLE, FELINE_METHIMAZOLE.vmax);
    let css_above_vmax = methimazole_css(&FELINE_METHIMAZOLE, FELINE_METHIMAZOLE.vmax + 1.0);
    h.check_bool(
        "methimazole_css: None for rate ≥ Vmax",
        css_at_vmax.is_none() && css_above_vmax.is_none(),
    );

    // 8. methimazole_css: increases with infusion rate
    let css_1 = methimazole_css(&FELINE_METHIMAZOLE, 1.0).unwrap_or(0.0);
    let css_2 = methimazole_css(&FELINE_METHIMAZOLE, 2.0).unwrap_or(0.0);
    h.check_bool(
        "methimazole_css: increases with infusion rate",
        css_2 > css_1,
    );

    // 9. t4_response at t=0: T4 = baseline
    let t4_baseline_check = t4_response(T4_BASELINE, 2.0, 0.0);
    h.check_abs(
        "t4_response at t=0: T4 = baseline",
        t4_baseline_check,
        T4_BASELINE,
        MACHINE_EPSILON,
    );

    // 10. t4_response at large t: T4 → target (~2.5)
    let t4_at_large = t4_response(T4_BASELINE, 2.0, 100.0);
    h.check_abs(
        "t4_response at large t: T4 → target (~2.5)",
        t4_at_large,
        2.5,
        FELINE_T4_RESPONSE,
    );

    // 11. t4_response monotonically decreasing (treatment normalizes T4)
    let t4_at_start = t4_response(T4_BASELINE, 2.0, 0.0);
    let t4_at_7 = t4_response(T4_BASELINE, 2.0, 7.0);
    let t4_at_30 = t4_response(T4_BASELINE, 2.0, 30.0);
    h.check_bool(
        "t4_response monotonically decreasing",
        t4_at_start > t4_at_7 && t4_at_7 > t4_at_30,
    );

    // 12. Cross-species: human Vmax > feline Vmax (allometric scaling)
    h.check_bool(
        "Cross-species: human Vmax > feline Vmax",
        HUMAN_METHIMAZOLE.vmax > FELINE_METHIMAZOLE.vmax,
    );

    // 13. Cross-species: human Vd > feline Vd
    h.check_bool(
        "Cross-species: human Vd > feline Vd",
        HUMAN_METHIMAZOLE.vd > FELINE_METHIMAZOLE.vd,
    );

    // 14. Determinism
    let (_, concs_run1) = methimazole_simulate(&FELINE_METHIMAZOLE, DOSE_MG, T_END_HR, DT_HR);
    let (_, concs_run2) = methimazole_simulate(&FELINE_METHIMAZOLE, DOSE_MG, T_END_HR, DT_HR);
    let max_diff = concs_run1
        .iter()
        .zip(concs_run2.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    h.check_abs("Determinism", max_diff, 0.0, DETERMINISM);

    h.exit();
}
