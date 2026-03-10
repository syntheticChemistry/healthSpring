// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp079: Short-Chain Fatty Acid (SCFA) Production Model
//!
//! Michaelis-Menten fermentation kinetics: fiber → acetate, propionate, butyrate.
//! Validates the canonical Cummings 1987 ratio (60:20:15) and saturation behavior.
//!
//! Reference: den Besten et al. 2013, Cummings 1987.

use healthspring_barracuda::microbiome;

macro_rules! check {
    ($p:expr, $f:expr, $name:expr, $cond:expr) => {
        if $cond {
            $p += 1;
            println!("  [PASS] {}", $name);
        } else {
            $f += 1;
            println!("  [FAIL] {}", $name);
        }
    };
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp079 — SCFA Production Model");
    println!("{}", "=".repeat(72));

    let healthy = &microbiome::SCFA_HEALTHY_PARAMS;
    let dysbiotic = &microbiome::SCFA_DYSBIOTIC_PARAMS;

    // Check 1: Healthy SCFA ratio at moderate fiber
    println!("\n--- Check 1: Healthy 60:20:15 ratio ---");
    let (a, p, b) = microbiome::scfa_production(20.0, healthy);
    let total = a + p + b;
    let a_frac = a / total;
    let p_frac = p / total;
    let b_frac = b / total;
    check!(
        passed,
        failed,
        format!("acetate:{a_frac:.2} propionate:{p_frac:.2} butyrate:{b_frac:.2}"),
        a_frac > 0.50 && a_frac < 0.70 && p_frac > 0.15 && p_frac < 0.30
    );

    // Check 2: Acetate dominant
    println!("\n--- Check 2: Acetate dominant ---");
    check!(
        passed,
        failed,
        format!("acetate ({a:.1}) > propionate ({p:.1}) > butyrate ({b:.1})"),
        a > p && p > b
    );

    // Check 3: Saturation at high fiber
    println!("\n--- Check 3: Michaelis-Menten saturation ---");
    let (a_low, _, _) = microbiome::scfa_production(5.0, healthy);
    let (a_high, _, _) = microbiome::scfa_production(100.0, healthy);
    let fold = a_high / a_low;
    check!(
        passed,
        failed,
        format!("100g/5g fiber → {fold:.1}× acetate (< 20×)"),
        fold < 20.0 && fold > 1.0
    );

    // Check 4: Zero fiber → zero SCFA
    println!("\n--- Check 4: Zero fiber = zero SCFA ---");
    let (a0, p0, b0) = microbiome::scfa_production(0.0, healthy);
    check!(
        passed,
        failed,
        format!("at 0 fiber: A={a0:.4}, P={p0:.4}, B={b0:.4}"),
        a0.abs() < 1e-10 && p0.abs() < 1e-10 && b0.abs() < 1e-10
    );

    // Check 5: Dysbiotic gut has less butyrate
    println!("\n--- Check 5: Dysbiotic → reduced butyrate ---");
    let (_, _, b_healthy) = microbiome::scfa_production(20.0, healthy);
    let (_, _, b_dys) = microbiome::scfa_production(20.0, dysbiotic);
    check!(
        passed,
        failed,
        format!("butyrate: healthy={b_healthy:.2} > dysbiotic={b_dys:.2}"),
        b_healthy > b_dys
    );

    // Check 6: Dysbiotic butyrate fraction lower
    println!("\n--- Check 6: Dysbiotic butyrate fraction lower ---");
    let (a_d, p_d, b_d) = microbiome::scfa_production(20.0, dysbiotic);
    let total_d = a_d + p_d + b_d;
    let b_frac_d = b_d / total_d;
    check!(
        passed,
        failed,
        format!("butyrate fraction: healthy={b_frac:.2} > dysbiotic={b_frac_d:.2}"),
        b_frac > b_frac_d
    );

    // Check 7: Total SCFA increases with fiber
    println!("\n--- Check 7: More fiber → more total SCFA ---");
    let total_low = {
        let (a, p, b) = microbiome::scfa_production(5.0, healthy);
        a + p + b
    };
    let total_high = {
        let (a, p, b) = microbiome::scfa_production(30.0, healthy);
        a + p + b
    };
    check!(
        passed,
        failed,
        format!("total: 30g={total_high:.1} > 5g={total_low:.1}"),
        total_high > total_low
    );

    // Check 8: All values positive at nonzero fiber
    println!("\n--- Check 8: All SCFA positive ---");
    check!(
        passed,
        failed,
        "all positive",
        a > 0.0 && p > 0.0 && b > 0.0
    );

    // Check 9: Propionate fraction in expected range
    println!("\n--- Check 9: Propionate fraction ~20% ---");
    check!(
        passed,
        failed,
        format!("propionate fraction = {p_frac:.2} in [0.15, 0.30]"),
        (0.15..=0.30).contains(&p_frac)
    );

    // Check 10: Butyrate fraction in expected range
    println!("\n--- Check 10: Butyrate fraction ~15% ---");
    check!(
        passed,
        failed,
        format!("butyrate fraction = {b_frac:.2} in [0.10, 0.25]"),
        (0.10..=0.25).contains(&b_frac)
    );

    let total_checks = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("Exp079 SCFA Production: {passed}/{total_checks} PASS, {failed}/{total_checks} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
