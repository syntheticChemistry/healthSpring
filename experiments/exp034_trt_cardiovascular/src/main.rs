// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! healthSpring Exp034 — TRT Cardiovascular Response (Rust validation)

use healthspring_barracuda::endocrine::{self, cv_params as cv};
use healthspring_barracuda::tolerances::{
    FRONT_LOADED_LDL, MACHINE_EPSILON, MACHINE_EPSILON_TIGHT,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp034 TRT Cardiovascular");

    // --- Check 1: Baselines at t=0 ---
    let ldl_init =
        endocrine::biomarker_trajectory(0.0, cv::LDL_BASELINE, cv::LDL_ENDPOINT, cv::TAU_MONTHS);
    let hdl_init =
        endocrine::biomarker_trajectory(0.0, cv::HDL_BASELINE, cv::HDL_ENDPOINT, cv::TAU_MONTHS);
    let crp_init =
        endocrine::biomarker_trajectory(0.0, cv::CRP_BASELINE, cv::CRP_ENDPOINT, cv::TAU_MONTHS);
    let sbp_init =
        endocrine::biomarker_trajectory(0.0, cv::SBP_BASELINE, cv::SBP_ENDPOINT, cv::TAU_MONTHS);
    let dbp_init =
        endocrine::biomarker_trajectory(0.0, cv::DBP_BASELINE, cv::DBP_ENDPOINT, cv::TAU_MONTHS);
    h.check_abs("LDL baseline", ldl_init, cv::LDL_BASELINE, MACHINE_EPSILON);
    h.check_abs("HDL baseline", hdl_init, cv::HDL_BASELINE, MACHINE_EPSILON);
    h.check_abs("CRP baseline", crp_init, cv::CRP_BASELINE, MACHINE_EPSILON);
    h.check_abs("SBP baseline", sbp_init, cv::SBP_BASELINE, MACHINE_EPSILON);
    h.check_abs("DBP baseline", dbp_init, cv::DBP_BASELINE, MACHINE_EPSILON);

    // --- Check 2: LDL decreases ---
    let ldl_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::LDL_BASELINE, cv::LDL_ENDPOINT, cv::TAU_MONTHS);
    h.check_bool("LDL decreases", ldl_at_60mo < cv::LDL_BASELINE);

    // --- Check 3: HDL increases ---
    let hdl_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::HDL_BASELINE, cv::HDL_ENDPOINT, cv::TAU_MONTHS);
    h.check_bool("HDL increases", hdl_at_60mo > cv::HDL_BASELINE);

    // --- Check 4: CRP decreases ---
    let crp_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::CRP_BASELINE, cv::CRP_ENDPOINT, cv::TAU_MONTHS);
    h.check_bool("CRP decreases", crp_at_60mo < cv::CRP_BASELINE);

    // --- Check 5: Blood pressure decreases ---
    let sbp_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::SBP_BASELINE, cv::SBP_ENDPOINT, cv::TAU_MONTHS);
    let dbp_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::DBP_BASELINE, cv::DBP_ENDPOINT, cv::TAU_MONTHS);
    h.check_bool(
        "SBP and DBP decrease",
        sbp_at_60mo < cv::SBP_BASELINE && dbp_at_60mo < cv::DBP_BASELINE,
    );

    // --- Check 6: SBP < 130 ---
    h.check_bool("SBP at 60mo < 130", sbp_at_60mo < 130.0);

    // --- Check 7: Front-loaded ---
    let ldl_at_12mo =
        endocrine::biomarker_trajectory(12.0, cv::LDL_BASELINE, cv::LDL_ENDPOINT, cv::TAU_MONTHS);
    let frac = (cv::LDL_BASELINE - ldl_at_12mo) / (cv::LDL_BASELINE - ldl_at_60mo);
    h.check_bool("Front-loaded LDL frac > threshold", frac > FRONT_LOADED_LDL);

    // --- Check 8: Hazard ratio ordering ---
    let hr_low = endocrine::hazard_ratio_model(200.0, 300.0, 0.44);
    let hr_mid = endocrine::hazard_ratio_model(300.0, 300.0, 0.44);
    let hr_norm = endocrine::hazard_ratio_model(600.0, 300.0, 0.44);
    h.check_bool(
        "HR ordering: norm <= mid < low",
        hr_norm <= hr_mid && hr_mid < hr_low,
    );

    // --- Check 9: HR normalized = 0.44 ---
    h.check_abs("HR(normalized) = 0.44", hr_norm, 0.44, MACHINE_EPSILON);

    // --- Check 10: All smooth ---
    let ldl_mono = (1..=60).all(|m| {
        endocrine::biomarker_trajectory(
            f64::from(m),
            cv::LDL_BASELINE,
            cv::LDL_ENDPOINT,
            cv::TAU_MONTHS,
        ) <= endocrine::biomarker_trajectory(
            f64::from(m - 1),
            cv::LDL_BASELINE,
            cv::LDL_ENDPOINT,
            cv::TAU_MONTHS,
        ) + MACHINE_EPSILON_TIGHT
    });
    let hdl_mono = (1..=60).all(|m| {
        endocrine::biomarker_trajectory(
            f64::from(m),
            cv::HDL_BASELINE,
            cv::HDL_ENDPOINT,
            cv::TAU_MONTHS,
        ) >= endocrine::biomarker_trajectory(
            f64::from(m - 1),
            cv::HDL_BASELINE,
            cv::HDL_ENDPOINT,
            cv::TAU_MONTHS,
        ) - MACHINE_EPSILON_TIGHT
    });
    h.check_bool("Monotonic trajectories", ldl_mono && hdl_mono);

    h.exit();
}
