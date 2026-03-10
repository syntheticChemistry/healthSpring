// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp032 — Age-Related Testosterone Decline (Rust validation)

use healthspring_barracuda::endocrine::{self, decline_params as dp};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp032: Age-Related Testosterone Decline (Rust)");
    println!("{}", "=".repeat(72));

    // --- Check 1: T(30) = T0 ---
    println!("\n--- Check 1: T(30) = T0 ---");
    let t30 = endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_MID, 30.0, 30.0);
    if (t30 - dp::T0_MEAN_NGDL).abs() < 1e-10 {
        println!("  [PASS] T(30) = {t30:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] T(30) = {t30:.2}");
        failed += 1;
    }

    // --- Check 2: Monotonically decreasing ---
    println!("\n--- Check 2: Monotonically decreasing ---");
    let mono = (31..=90).all(|age| {
        endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_MID, f64::from(age), 30.0)
            <= endocrine::testosterone_decline(
                dp::T0_MEAN_NGDL,
                dp::RATE_MID,
                f64::from(age - 1),
                30.0,
            )
    });
    if mono {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 3: 1%/yr → expected residual at 90 ---
    println!("\n--- Check 3: 1%/yr residual at 90 ---");
    let t90_low = endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_LOW, 90.0, 30.0);
    let expected = (-0.01_f64 * 60.0).exp();
    let pct = t90_low / dp::T0_MEAN_NGDL;
    if (pct - expected).abs() < 0.01 {
        println!("  [PASS] {pct:.3} ≈ {expected:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] {pct:.3} vs {expected:.3}");
        failed += 1;
    }

    // --- Check 4: 3%/yr → expected residual at 90 ---
    println!("\n--- Check 4: 3%/yr residual at 90 ---");
    let t90_high = endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_HIGH, 90.0, 30.0);
    let expected_h = (-0.03_f64 * 60.0).exp();
    let pct_h = t90_high / dp::T0_MEAN_NGDL;
    if (pct_h - expected_h).abs() < 0.01 {
        println!("  [PASS] {pct_h:.3} ≈ {expected_h:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] {pct_h:.3} vs {expected_h:.3}");
        failed += 1;
    }

    // --- Check 5: Age at threshold in range ---
    println!("\n--- Check 5: Age at 300 ng/dL in [50, 80] ---");
    let age_300 =
        endocrine::age_at_threshold(dp::T0_MEAN_NGDL, dp::RATE_MID, dp::THRESHOLD_CLINICAL, 30.0);
    if (50.0..80.0).contains(&age_300) {
        println!("  [PASS] age = {age_300:.1}");
        passed += 1;
    } else {
        println!("  [FAIL] age = {age_300:.1}");
        failed += 1;
    }

    // --- Check 6: Faster decline → earlier threshold ---
    println!("\n--- Check 6: Rate ordering ---");
    let a_low =
        endocrine::age_at_threshold(dp::T0_MEAN_NGDL, dp::RATE_LOW, dp::THRESHOLD_CLINICAL, 30.0);
    let a_high = endocrine::age_at_threshold(
        dp::T0_MEAN_NGDL,
        dp::RATE_HIGH,
        dp::THRESHOLD_CLINICAL,
        30.0,
    );
    if a_high < age_300 && age_300 < a_low {
        println!("  [PASS] {a_high:.1} < {age_300:.1} < {a_low:.1}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 7: All positive ---
    println!("\n--- Check 7: All T > 0 ---");
    let all_pos = (30..=100).all(|age| {
        endocrine::testosterone_decline(dp::T0_MEAN_NGDL, dp::RATE_HIGH, f64::from(age), 30.0) > 0.0
    });
    if all_pos {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 8: Threshold age for T0 already below ---
    println!("\n--- Check 8: T0 below threshold → onset ---");
    let age_low_t0 = endocrine::age_at_threshold(250.0, dp::RATE_MID, dp::THRESHOLD_CLINICAL, 30.0);
    if (age_low_t0 - 30.0).abs() < 1e-10 {
        println!("  [PASS] age = onset");
        passed += 1;
    } else {
        println!("  [FAIL] age = {age_low_t0:.1}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
