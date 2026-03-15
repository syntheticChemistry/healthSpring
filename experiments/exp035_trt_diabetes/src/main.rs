// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp035 — TRT and Type 2 Diabetes (Rust validation)

use healthspring_barracuda::endocrine::{self, diabetes_params as dp};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp035: TRT and Type 2 Diabetes (Rust)");
    println!("{}", "=".repeat(72));

    // --- Check 1: HbA1c baseline ---
    println!("\n--- Check 1: HbA1c(0) = baseline ---");
    let h0 = endocrine::hba1c_trajectory(0.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    if (h0 - dp::HBA1C_BASELINE).abs() < 1e-10 {
        println!("  [PASS] HbA1c(0) = {h0:.2}%");
        passed += 1;
    } else {
        println!("  [FAIL] {h0}");
        failed += 1;
    }

    // --- Check 2: HbA1c decreases ---
    println!("\n--- Check 2: HbA1c decreases ---");
    let h12 =
        endocrine::hba1c_trajectory(12.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    if h12 < dp::HBA1C_BASELINE {
        println!("  [PASS] HbA1c: {:.2} -> {h12:.2}", dp::HBA1C_BASELINE);
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 3: HbA1c at 3 months (t=τ → 63.2% of change) ---
    println!("\n--- Check 3: HbA1c at 3 months ---");
    let h3 = endocrine::hba1c_trajectory(3.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    let expected_delta = dp::HBA1C_DELTA * (1.0 - (-1.0_f64).exp());
    let delta_3 = h3 - dp::HBA1C_BASELINE;
    if (delta_3 - expected_delta).abs() < 0.05 {
        println!("  [PASS] delta={delta_3:.3}, expected={expected_delta:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] delta={delta_3:.3}");
        failed += 1;
    }

    // --- Check 4: HOMA-IR decreases ---
    println!("\n--- Check 4: HOMA-IR decreases ---");
    let homa12 =
        endocrine::biomarker_trajectory(12.0, dp::HOMA_BASELINE, dp::HOMA_ENDPOINT, dp::TAU_MONTHS);
    if homa12 < dp::HOMA_BASELINE {
        println!("  [PASS] HOMA: {:.1} -> {homa12:.2}", dp::HOMA_BASELINE);
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 5: Fasting glucose decreases ---
    println!("\n--- Check 5: FG decreases ---");
    let fg12 =
        endocrine::biomarker_trajectory(12.0, dp::FG_BASELINE, dp::FG_ENDPOINT, dp::TAU_MONTHS);
    if fg12 < dp::FG_BASELINE {
        println!("  [PASS] FG: {:.0} -> {fg12:.1}", dp::FG_BASELINE);
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 6: HbA1c monotonic ---
    println!("\n--- Check 6: HbA1c monotonic ---");
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
        ) + 1e-12
    });
    if mono {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 7: Front-loaded (>80% by 6 months) ---
    println!("\n--- Check 7: Front-loaded ---");
    let h6 = endocrine::hba1c_trajectory(6.0, dp::HBA1C_BASELINE, dp::HBA1C_DELTA, dp::TAU_MONTHS);
    let delta_6 = (h6 - dp::HBA1C_BASELINE).abs();
    let delta_12 = (h12 - dp::HBA1C_BASELINE).abs();
    let frac = if delta_12 > 1e-10 {
        delta_6 / delta_12
    } else {
        0.0
    };
    if frac > 0.80 {
        println!("  [PASS] {frac:.3} by 6mo");
        passed += 1;
    } else {
        println!("  [FAIL] {frac:.3}");
        failed += 1;
    }

    // --- Check 8: Clinically significant ---
    println!("\n--- Check 8: Clinically significant (> 0.3%) ---");
    if delta_12 > 0.30 {
        println!("  [PASS] delta = {delta_12:.3}%");
        passed += 1;
    } else {
        println!("  [FAIL] {delta_12:.3}");
        failed += 1;
    }

    // --- Check 9: Concordant improvement ---
    println!("\n--- Check 9: All improve concordantly ---");
    if h12 < dp::HBA1C_BASELINE && homa12 < dp::HOMA_BASELINE && fg12 < dp::FG_BASELINE {
        println!("  [PASS] HbA1c↓ HOMA↓ FG↓");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 10: HOMA-IR improvement plausible ---
    println!("\n--- Check 10: HOMA improvement 15-50% ---");
    let homa_pct = (dp::HOMA_BASELINE - homa12) / dp::HOMA_BASELINE;
    if (0.15..0.50).contains(&homa_pct) {
        println!("  [PASS] {homa_pct:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] {homa_pct:.3}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
