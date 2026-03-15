// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp038 — HRV × TRT Cardiovascular Cross-Track (Mok D3)
//!
//! Cross-validates `endocrine::hrv_trt_response` and `cardiac_risk_composite`.

use healthspring_barracuda::endocrine;

const SDNN_BASE_MS: f64 = 35.0;
const DELTA_SDNN_MS: f64 = 20.0;
const TAU_MONTHS: f64 = 6.0;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp038: HRV × TRT Cardiovascular (Mok D3)");
    println!("{}", "=".repeat(72));

    // --- Check 1: HRV at t=0 equals baseline SDNN ---
    println!("\n--- Check 1: HRV at t=0 equals baseline SDNN ---");
    let sdnn_0 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 0.0);
    if (sdnn_0 - SDNN_BASE_MS).abs() < 1e-10 {
        println!("  [PASS] SDNN(0) = {sdnn_0:.1} ms");
        passed += 1;
    } else {
        println!("  [FAIL] SDNN(0) = {sdnn_0}, expected {SDNN_BASE_MS}");
        failed += 1;
    }

    // --- Check 2: HRV improves monotonically with TRT ---
    println!("\n--- Check 2: HRV improves monotonically with TRT ---");
    let mut monotonic = true;
    for m in 1..=24 {
        let prev =
            endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, f64::from(m - 1));
        let curr =
            endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, f64::from(m));
        if curr < prev - 1e-12 {
            monotonic = false;
            break;
        }
    }
    if monotonic {
        println!("  [PASS] SDNN increases monotonically over 24 months");
        passed += 1;
    } else {
        println!("  [FAIL] Non-monotonic SDNN trajectory");
        failed += 1;
    }

    // --- Check 3: HRV approaches base + delta asymptotically ---
    println!("\n--- Check 3: HRV approaches base + delta asymptotically ---");
    let sdnn_120 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 120.0);
    let asymptote = SDNN_BASE_MS + DELTA_SDNN_MS;
    if (sdnn_120 - asymptote).abs() < 1.0 {
        println!("  [PASS] SDNN(120mo) = {sdnn_120:.2} ≈ {asymptote:.0} ms");
        passed += 1;
    } else {
        println!("  [FAIL] SDNN(120) = {sdnn_120}, asymptote = {asymptote}");
        failed += 1;
    }

    // --- Check 4: Cardiac risk decreases with TRT (pre > post) ---
    println!("\n--- Check 4: Cardiac risk decreases with TRT ---");
    let risk_pre = endocrine::cardiac_risk_composite(35.0, 250.0, 1.0);
    let risk_post = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    if risk_pre > risk_post {
        println!("  [PASS] Risk: {risk_pre:.3} → {risk_post:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] Risk: pre={risk_pre:.3}, post={risk_post:.3}");
        failed += 1;
    }

    // --- Check 5: Low SDNN (<50ms) → risk factor > 1.0 ---
    println!("\n--- Check 5: Low SDNN (<50ms) → risk factor > 1.0 ---");
    let risk_low_sdnn = endocrine::cardiac_risk_composite(30.0, 400.0, 1.0);
    let risk_high_sdnn = endocrine::cardiac_risk_composite(120.0, 400.0, 1.0);
    if risk_low_sdnn > risk_high_sdnn && risk_low_sdnn > 1.0 {
        println!("  [PASS] Low SDNN risk={risk_low_sdnn:.2} > high SDNN risk={risk_high_sdnn:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] Low SDNN risk={risk_low_sdnn:.2}, high SDNN risk={risk_high_sdnn:.2}");
        failed += 1;
    }

    // --- Check 6: High SDNN (>100ms) → risk factor = 0.5 ---
    println!("\n--- Check 6: High SDNN (>100ms) → risk factor = 0.5 ---");
    let risk_sdnn_120_t500 = endocrine::cardiac_risk_composite(120.0, 500.0, 1.0);
    if (risk_sdnn_120_t500 - 0.25).abs() < 1e-10 {
        println!("  [PASS] risk(120ms, 500ng/dL) = 0.25 (hrv_factor=0.5)");
        passed += 1;
    } else {
        println!("  [FAIL] risk(120ms, 500ng/dL) = {risk_sdnn_120_t500}, expected 0.25");
        failed += 1;
    }

    // --- Check 7: Low T (<300) → risk factor > 1.0 ---
    println!("\n--- Check 7: Low T (<300) → risk factor > 1.0 ---");
    let risk_low_t = endocrine::cardiac_risk_composite(80.0, 100.0, 1.0);
    if risk_low_t > 1.0 {
        println!("  [PASS] Low T risk={risk_low_t:.2} > 1.0");
        passed += 1;
    } else {
        println!("  [FAIL] Low T risk={risk_low_t:.2}");
        failed += 1;
    }

    // --- Check 8: High T (>500) → risk factor = 0.5 ---
    println!("\n--- Check 8: High T (>500) → risk factor = 0.5 ---");
    let risk_high_t = endocrine::cardiac_risk_composite(80.0, 600.0, 1.0);
    if (risk_high_t - 0.35).abs() < 1e-10 {
        println!("  [PASS] risk(80ms, 600ng/dL) = 0.35 (t_factor=0.5)");
        passed += 1;
    } else {
        println!("  [FAIL] risk(80ms, 600ng/dL) = {risk_high_t}, expected 0.35");
        failed += 1;
    }

    // --- Check 9: Combined improvement → risk reduction > 50% ---
    println!("\n--- Check 9: Combined improvement → risk reduction > 50% ---");
    let risk_pre_combined = endocrine::cardiac_risk_composite(35.0, 250.0, 1.0);
    let risk_post_combined = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    let reduction = (risk_pre_combined - risk_post_combined) / risk_pre_combined;
    if reduction > 0.5 {
        println!(
            "  [PASS] Risk reduction = {:.1}% (> 50%)",
            reduction * 100.0
        );
        passed += 1;
    } else {
        println!("  [FAIL] Risk reduction = {:.1}%", reduction * 100.0);
        failed += 1;
    }

    // --- Check 10: Determinism (bit-identical) ---
    println!("\n--- Check 10: Determinism (bit-identical) ---");
    let r1 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 12.0);
    let r2 = endocrine::hrv_trt_response(SDNN_BASE_MS, DELTA_SDNN_MS, TAU_MONTHS, 12.0);
    let c1 = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    let c2 = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    if r1.to_bits() == r2.to_bits() && c1.to_bits() == c2.to_bits() {
        println!("  [PASS] Bit-identical across repeated calls");
        passed += 1;
    } else {
        println!("  [FAIL] Non-deterministic output");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
