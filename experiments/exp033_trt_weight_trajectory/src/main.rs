// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp033 — TRT Weight/Waist Trajectory (Rust validation)

use healthspring_barracuda::endocrine::{self, weight_params as wp};
use healthspring_barracuda::tolerances::{
    FRONT_LOADED_WEIGHT, MACHINE_EPSILON, MACHINE_EPSILON_TIGHT, TWO_COMPARTMENT_IDENTITY,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp033 TRT Weight Trajectory");

    // --- Check 1: ΔW(0) = 0 ---
    let dw_init = endocrine::weight_trajectory(
        0.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    h.check_abs("DW(0) = 0", dw_init, 0.0, MACHINE_EPSILON);

    // --- Check 2: ΔW(60) matches target ---
    let dw_at_60mo = endocrine::weight_trajectory(
        60.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    h.check_abs(
        "DW(60) = -16 kg",
        dw_at_60mo,
        wp::WEIGHT_LOSS_5YR_KG,
        TWO_COMPARTMENT_IDENTITY,
    );

    // --- Check 3: Monotonically decreasing ---
    let mono = (1..=60).all(|m| {
        endocrine::weight_trajectory(
            f64::from(m),
            wp::WEIGHT_LOSS_5YR_KG,
            wp::TAU_MONTHS,
            wp::TOTAL_MONTHS,
        ) <= endocrine::weight_trajectory(
            f64::from(m - 1),
            wp::WEIGHT_LOSS_5YR_KG,
            wp::TAU_MONTHS,
            wp::TOTAL_MONTHS,
        ) + MACHINE_EPSILON_TIGHT
    });
    h.check_bool("Monotonically decreasing", mono);

    // --- Check 4: Front-loaded (>60% by 24 months) ---
    let dw24 = endocrine::weight_trajectory(
        24.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    let frac = dw24 / dw_at_60mo;
    h.check_lower("Front-loaded frac > 0.6", frac, FRONT_LOADED_WEIGHT);

    // --- Check 5: Waist parallels weight ---
    let dwc60 = endocrine::weight_trajectory(
        60.0,
        wp::WAIST_LOSS_5YR_CM,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    h.check_abs(
        "DWaist(60)",
        dwc60,
        wp::WAIST_LOSS_5YR_CM,
        TWO_COMPARTMENT_IDENTITY,
    );

    // --- Check 6: BMI trajectory ---
    let dbmi =
        endocrine::weight_trajectory(60.0, wp::BMI_LOSS_5YR, wp::TAU_MONTHS, wp::TOTAL_MONTHS);
    h.check_abs("DBMI(60)", dbmi, wp::BMI_LOSS_5YR, TWO_COMPARTMENT_IDENTITY);

    // --- Check 7: Decelerating rate ---
    let rate_yr1 = endocrine::weight_trajectory(
        12.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    ) - endocrine::weight_trajectory(
        0.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    let rate_yr5 = endocrine::weight_trajectory(
        60.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    ) - endocrine::weight_trajectory(
        48.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    h.check_bool(
        "Decelerating: |rate_yr1| > |rate_yr5|",
        rate_yr1.abs() > rate_yr5.abs(),
    );

    h.exit();
}
