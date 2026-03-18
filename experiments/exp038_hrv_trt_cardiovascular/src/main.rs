// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! healthSpring Exp038 — HRV × TRT Cardiovascular Cross-Track (Mok D3)
//!
//! Cross-validates `endocrine::hrv_trt_response` and `cardiac_risk_composite`.

use healthspring_barracuda::endocrine;
use healthspring_barracuda::tolerances::{MACHINE_EPSILON, MACHINE_EPSILON_TIGHT};
use healthspring_barracuda::validation::ValidationHarness;

const SDNN_BASE_MS: f64 = 35.0;
const DELTA_SDNN_MS: f64 = 20.0;
const TAU_MONTHS: f64 = 6.0;

fn main() {
    let mut h = ValidationHarness::new("exp038_hrv_trt_cardiovascular");

    // --- Check 1: HRV at t=0 equals baseline SDNN ---
    let sdnn_0 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 0.0);
    h.check_abs(
        "hrv_at_t0_equals_baseline",
        sdnn_0,
        SDNN_BASE_MS,
        MACHINE_EPSILON,
    );

    // --- Check 2: HRV improves monotonically with TRT ---
    let mut monotonic = true;
    for m in 1..=24 {
        let prev =
            endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, f64::from(m - 1));
        let curr =
            endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, f64::from(m));
        if curr < prev - MACHINE_EPSILON_TIGHT {
            monotonic = false;
            break;
        }
    }
    h.check_bool("hrv_monotonic_with_trt", monotonic);

    // --- Check 3: HRV approaches base + delta asymptotically ---
    let sdnn_120 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 120.0);
    let asymptote = SDNN_BASE_MS + DELTA_SDNN_MS;
    h.check_abs("hrv_asymptote_120mo", sdnn_120, asymptote, 1.0);

    // --- Check 4: Cardiac risk decreases with TRT (pre > post) ---
    let risk_pre = endocrine::cardiac_risk_composite(35.0, 250.0, 1.0);
    let risk_post = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    h.check_bool("cardiac_risk_decreases_with_trt", risk_pre > risk_post);

    // --- Check 5: Low SDNN (<50ms) → risk factor > 1.0 ---
    let risk_low_sdnn = endocrine::cardiac_risk_composite(30.0, 400.0, 1.0);
    let risk_high_sdnn = endocrine::cardiac_risk_composite(120.0, 400.0, 1.0);
    h.check_bool(
        "low_sdnn_higher_risk",
        risk_low_sdnn > risk_high_sdnn && risk_low_sdnn > 1.0,
    );

    // --- Check 6: High SDNN (>100ms) → risk factor = 0.5 ---
    let risk_sdnn_120_t500 = endocrine::cardiac_risk_composite(120.0, 500.0, 1.0);
    h.check_abs(
        "high_sdnn_risk_factor",
        risk_sdnn_120_t500,
        0.25,
        MACHINE_EPSILON,
    );

    // --- Check 7: Low T (<300) → risk factor > 1.0 ---
    let risk_low_t = endocrine::cardiac_risk_composite(80.0, 100.0, 1.0);
    h.check_bool("low_t_higher_risk", risk_low_t > 1.0);

    // --- Check 8: High T (>500) → risk factor = 0.5 ---
    let risk_high_t = endocrine::cardiac_risk_composite(80.0, 600.0, 1.0);
    h.check_abs("high_t_risk_factor", risk_high_t, 0.35, MACHINE_EPSILON);

    // --- Check 9: Combined improvement → risk reduction > 50% ---
    let risk_pre_combined = endocrine::cardiac_risk_composite(35.0, 250.0, 1.0);
    let risk_post_combined = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    let reduction = (risk_pre_combined - risk_post_combined) / risk_pre_combined;
    h.check_bool("risk_reduction_gt_50pct", reduction > 0.5);

    // --- Check 10: Determinism (bit-identical) ---
    let r1 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 12.0);
    let r2 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 12.0);
    let c1 = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    let c2 = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    h.check_bool(
        "determinism_bit_identical",
        r1.to_bits() == r2.to_bits() && c1.to_bits() == c2.to_bits(),
    );

    h.exit();
}
