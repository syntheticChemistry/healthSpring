#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! healthSpring Exp034 — TRT Cardiovascular Response (Rust validation)

use healthspring_barracuda::endocrine::{self, cv_params as cv};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp034: TRT Cardiovascular Response (Rust)");
    println!("{}", "=".repeat(72));

    // --- Check 1: Baselines at t=0 ---
    println!("\n--- Check 1: Baselines at t=0 ---");
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
    let ok = (ldl_init - cv::LDL_BASELINE).abs() < 1e-10
        && (hdl_init - cv::HDL_BASELINE).abs() < 1e-10
        && (crp_init - cv::CRP_BASELINE).abs() < 1e-10
        && (sbp_init - cv::SBP_BASELINE).abs() < 1e-10
        && (dbp_init - cv::DBP_BASELINE).abs() < 1e-10;
    if ok {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 2: LDL decreases ---
    println!("\n--- Check 2: LDL decreases ---");
    let ldl_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::LDL_BASELINE, cv::LDL_ENDPOINT, cv::TAU_MONTHS);
    if ldl_at_60mo < cv::LDL_BASELINE {
        println!("  [PASS] LDL: {:.0} -> {ldl_at_60mo:.1}", cv::LDL_BASELINE);
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 3: HDL increases ---
    println!("\n--- Check 3: HDL increases ---");
    let hdl_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::HDL_BASELINE, cv::HDL_ENDPOINT, cv::TAU_MONTHS);
    if hdl_at_60mo > cv::HDL_BASELINE {
        println!("  [PASS] HDL: {:.0} -> {hdl_at_60mo:.1}", cv::HDL_BASELINE);
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 4: CRP decreases ---
    println!("\n--- Check 4: CRP decreases ---");
    let crp_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::CRP_BASELINE, cv::CRP_ENDPOINT, cv::TAU_MONTHS);
    if crp_at_60mo < cv::CRP_BASELINE {
        println!("  [PASS] CRP: {:.2} -> {crp_at_60mo:.2}", cv::CRP_BASELINE);
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 5: Blood pressure decreases ---
    println!("\n--- Check 5: BP decreases ---");
    let sbp_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::SBP_BASELINE, cv::SBP_ENDPOINT, cv::TAU_MONTHS);
    let dbp_at_60mo =
        endocrine::biomarker_trajectory(60.0, cv::DBP_BASELINE, cv::DBP_ENDPOINT, cv::TAU_MONTHS);
    if sbp_at_60mo < cv::SBP_BASELINE && dbp_at_60mo < cv::DBP_BASELINE {
        println!(
            "  [PASS] SBP: {:.0}->{sbp_at_60mo:.1}, DBP: {:.0}->{dbp_at_60mo:.1}",
            cv::SBP_BASELINE,
            cv::DBP_BASELINE
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 6: SBP < 130 ---
    println!("\n--- Check 6: SBP < 130 ---");
    if sbp_at_60mo < 130.0 {
        println!("  [PASS] SBP={sbp_at_60mo:.1}");
        passed += 1;
    } else {
        println!("  [FAIL] SBP={sbp_at_60mo:.1}");
        failed += 1;
    }

    // --- Check 7: Front-loaded ---
    println!("\n--- Check 7: Front-loaded LDL improvement ---");
    let ldl_at_12mo =
        endocrine::biomarker_trajectory(12.0, cv::LDL_BASELINE, cv::LDL_ENDPOINT, cv::TAU_MONTHS);
    let frac = (cv::LDL_BASELINE - ldl_at_12mo) / (cv::LDL_BASELINE - ldl_at_60mo);
    if frac > 0.55 {
        println!("  [PASS] {frac:.1}% by 12 mo");
        passed += 1;
    } else {
        println!("  [FAIL] {frac:.3}");
        failed += 1;
    }

    // --- Check 8: Hazard ratio ordering ---
    println!("\n--- Check 8: HR ordering ---");
    let hr_low = endocrine::hazard_ratio_model(200.0, 300.0, 0.44);
    let hr_mid = endocrine::hazard_ratio_model(300.0, 300.0, 0.44);
    let hr_norm = endocrine::hazard_ratio_model(600.0, 300.0, 0.44);
    if hr_norm <= hr_mid && hr_mid < hr_low {
        println!("  [PASS] HR: {hr_norm:.2} <= {hr_mid:.2} < {hr_low:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] HR: {hr_norm:.2}, {hr_mid:.2}, {hr_low:.2}");
        failed += 1;
    }

    // --- Check 9: HR normalized = 0.44 ---
    println!("\n--- Check 9: HR(normalized) = 0.44 ---");
    if (hr_norm - 0.44).abs() < 1e-10 {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL] {hr_norm}");
        failed += 1;
    }

    // --- Check 10: All smooth ---
    println!("\n--- Check 10: Monotonic trajectories ---");
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
        ) + 1e-12
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
        ) - 1e-12
    });
    if ldl_mono && hdl_mono {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
