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

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp033: TRT Weight/Waist Trajectory (Rust)");
    println!("{}", "=".repeat(72));

    // --- Check 1: ΔW(0) = 0 ---
    println!("\n--- Check 1: DW(0) = 0 ---");
    let dw_init = endocrine::weight_trajectory(
        0.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    if dw_init.abs() < 1e-10 {
        println!("  [PASS] DW(0) = {dw_init:.10}");
        passed += 1;
    } else {
        println!("  [FAIL] DW(0) = {dw_init}");
        failed += 1;
    }

    // --- Check 2: ΔW(60) matches target ---
    println!("\n--- Check 2: DW(60) = -16 kg ---");
    let dw_at_60mo = endocrine::weight_trajectory(
        60.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    if (dw_at_60mo - wp::WEIGHT_LOSS_5YR_KG).abs() < 1e-8 {
        println!("  [PASS] DW(60) = {dw_at_60mo:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] DW(60) = {dw_at_60mo:.2}");
        failed += 1;
    }

    // --- Check 3: Monotonically decreasing ---
    println!("\n--- Check 3: Monotonically decreasing ---");
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
        ) + 1e-12
    });
    if mono {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 4: Front-loaded (>60% by 24 months) ---
    println!("\n--- Check 4: Front-loaded weight loss ---");
    let dw24 = endocrine::weight_trajectory(
        24.0,
        wp::WEIGHT_LOSS_5YR_KG,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    let frac = dw24 / dw_at_60mo;
    if frac > 0.60 {
        println!("  [PASS] {frac:.1}% by 24 months");
        passed += 1;
    } else {
        println!("  [FAIL] {frac:.3}");
        failed += 1;
    }

    // --- Check 5: Waist parallels weight ---
    println!("\n--- Check 5: Waist trajectory ---");
    let dwc60 = endocrine::weight_trajectory(
        60.0,
        wp::WAIST_LOSS_5YR_CM,
        wp::TAU_MONTHS,
        wp::TOTAL_MONTHS,
    );
    if (dwc60 - wp::WAIST_LOSS_5YR_CM).abs() < 1e-8 {
        println!("  [PASS] DWaist(60) = {dwc60:.2} cm");
        passed += 1;
    } else {
        println!("  [FAIL] DWaist(60) = {dwc60:.2}");
        failed += 1;
    }

    // --- Check 6: BMI trajectory ---
    println!("\n--- Check 6: BMI trajectory ---");
    let dbmi =
        endocrine::weight_trajectory(60.0, wp::BMI_LOSS_5YR, wp::TAU_MONTHS, wp::TOTAL_MONTHS);
    if (dbmi - wp::BMI_LOSS_5YR).abs() < 1e-8 {
        println!("  [PASS] DBMI(60) = {dbmi:.2}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 7: Decelerating rate ---
    println!("\n--- Check 7: Decelerating ---");
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
    if rate_yr1.abs() > rate_yr5.abs() {
        println!("  [PASS] yr1={rate_yr1:.2}, yr5={rate_yr5:.2}");
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
