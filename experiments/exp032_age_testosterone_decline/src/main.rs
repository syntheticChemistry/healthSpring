// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! healthSpring Exp032 — Age-Related Testosterone Decline (Rust validation)

use healthspring_barracuda::endocrine::{self, decline_params as dp};
use healthspring_barracuda::tolerances::{EXPONENTIAL_RESIDUAL, MACHINE_EPSILON};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp032 Age Testosterone Decline");

    // --- Check 1: T(30) = T0 ---
    let t30 = endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_MID, 30.0, 30.0);
    h.check_abs("T(30) = T0", t30, dp::T0_MEAN_NGDL, MACHINE_EPSILON);

    // --- Check 2: Monotonically decreasing ---
    let mono = (31..=90).all(|age| {
        endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_MID, f64::from(age), 30.0)
            <= endocrine::testosterone_decline(
                dp::T0_MEAN_NGDL,
                dp::RATE_MID,
                f64::from(age - 1),
                30.0,
            )
    });
    h.check_bool("Monotonically decreasing", mono);

    // --- Check 3: 1%/yr → expected residual at 90 ---
    let t90_low = endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_LOW, 90.0, 30.0);
    let expected = (-0.01_f64 * 60.0).exp();
    let pct = t90_low / dp::T0_MEAN_NGDL;
    h.check_abs("1%/yr residual at 90", pct, expected, EXPONENTIAL_RESIDUAL);

    // --- Check 4: 3%/yr → expected residual at 90 ---
    let t90_high = endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_HIGH, 90.0, 30.0);
    let expected_h = (-0.03_f64 * 60.0).exp();
    let pct_h = t90_high / dp::T0_MEAN_NGDL;
    h.check_abs(
        "3%/yr residual at 90",
        pct_h,
        expected_h,
        EXPONENTIAL_RESIDUAL,
    );

    // --- Check 5: Age at threshold in range ---
    let age_300 =
        endocrine::age_at_threshold(dp::T0_MEAN_NGDL, dp::RATE_MID, dp::THRESHOLD_CLINICAL, 30.0);
    h.check_bool(
        "Age at 300 ng/dL in [50, 80]",
        (50.0..80.0).contains(&age_300),
    );

    // --- Check 6: Faster decline → earlier threshold ---
    let a_low =
        endocrine::age_at_threshold(dp::T0_MEAN_NGDL, dp::RATE_LOW, dp::THRESHOLD_CLINICAL, 30.0);
    let a_high = endocrine::age_at_threshold(
        dp::T0_MEAN_NGDL,
        dp::RATE_HIGH,
        dp::THRESHOLD_CLINICAL,
        30.0,
    );
    h.check_bool(
        "Rate ordering: a_high < age_300 < a_low",
        a_high < age_300 && age_300 < a_low,
    );

    // --- Check 7: All positive ---
    let all_pos = (30..=100).all(|age| {
        endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_HIGH, f64::from(age), 30.0) > 0.0
    });
    h.check_bool("All T > 0", all_pos);

    // --- Check 8: Threshold age for T0 already below ---
    let age_low_t0 = endocrine::age_at_threshold(250.0, dp::RATE_MID, dp::THRESHOLD_CLINICAL, 30.0);
    h.check_abs(
        "T0 below threshold → onset",
        age_low_t0,
        30.0,
        MACHINE_EPSILON,
    );

    h.exit();
}
