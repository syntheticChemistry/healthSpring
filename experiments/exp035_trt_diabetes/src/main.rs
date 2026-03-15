// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! healthSpring Exp035 — TRT and Type 2 Diabetes (Rust validation)

use healthspring_barracuda::endocrine::{self, diabetes_params as dp};
use healthspring_barracuda::tolerances::{
    FRONT_LOADED_HBA1C, LOGNORMAL_RECOVERY, MACHINE_EPSILON, MACHINE_EPSILON_TIGHT,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp035 TRT Diabetes");

    // --- Check 1: HbA1c baseline ---
    let h0 = endocrine::hba1c_trajectory(0.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    h.check_abs(
        "HbA1c(0) = baseline",
        h0,
        dp::HBA1C_BASELINE,
        MACHINE_EPSILON,
    );

    // --- Check 2: HbA1c decreases ---
    let h12 =
        endocrine::hba1c_trajectory(12.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    h.check_bool("HbA1c decreases", h12 < dp::HBA1C_BASELINE);

    // --- Check 3: HbA1c at 3 months (t=τ → 63.2% of change) ---
    let h3 = endocrine::hba1c_trajectory(3.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    let expected_delta = dp::HBA1C_DELTA * (1.0 - (-1.0_f64).exp());
    let delta_3 = h3 - dp::HBA1C_BASELINE;
    h.check_abs(
        "HbA1c delta at 3mo",
        delta_3,
        expected_delta,
        LOGNORMAL_RECOVERY,
    );

    // --- Check 4: HOMA-IR decreases ---
    let homa12 =
        endocrine::biomarker_trajectory(12.0, dp::HOMA_BASELINE, dp::HOMA_ENDPOINT, dp::TAU_MONTHS);
    h.check_bool("HOMA-IR decreases", homa12 < dp::HOMA_BASELINE);

    // --- Check 5: Fasting glucose decreases ---
    let fg12 =
        endocrine::biomarker_trajectory(12.0, dp::FG_BASELINE, dp::FG_ENDPOINT, dp::TAU_MONTHS);
    h.check_bool("FG decreases", fg12 < dp::FG_BASELINE);

    // --- Check 6: HbA1c monotonic ---
    let mono = (1..=12).all(|m| {
        endocrine::hba1c_trajectory(
            f64::from(m),
            dp::HBA1C_BASELINE,
            dp::HBA1C_DELTA,
            dp::TAU_MONTHS,
        ) <= endocrine::hba1c_trajectory(
            f64::from(m - 1),
            dp::HBA1C_BASELINE,
            dp::HBA1C_DELTA,
            dp::TAU_MONTHS,
        ) + MACHINE_EPSILON_TIGHT
    });
    h.check_bool("HbA1c monotonic", mono);

    // --- Check 7: Front-loaded (>80% by 6 months) ---
    let h6 = endocrine::hba1c_trajectory(6.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    let delta_6 = (h6 - dp::HBA1C_BASELINE).abs();
    let delta_12 = (h12 - dp::HBA1C_BASELINE).abs();
    let frac = if delta_12 > MACHINE_EPSILON {
        delta_6 / delta_12
    } else {
        0.0
    };
    h.check_bool("Front-loaded frac > threshold", frac > FRONT_LOADED_HBA1C);

    // --- Check 8: Clinically significant ---
    h.check_bool("Clinically significant (> 0.3%)", delta_12 > 0.30);

    // --- Check 9: Concordant improvement ---
    h.check_bool(
        "All improve concordantly",
        h12 < dp::HBA1C_BASELINE && homa12 < dp::HOMA_BASELINE && fg12 < dp::FG_BASELINE,
    );

    // --- Check 10: HOMA-IR improvement plausible ---
    let homa_pct = (dp::HOMA_BASELINE - homa12) / dp::HOMA_BASELINE;
    h.check_bool("HOMA improvement 15-50%", (0.15..0.50).contains(&homa_pct));

    h.exit();
}
