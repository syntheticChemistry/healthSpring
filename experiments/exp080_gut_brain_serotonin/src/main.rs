// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp080: Gut-Brain Serotonin Pathway
//!
//! ~90% of body serotonin is gut-derived. Microbiome diversity modulates
//! tryptophan availability for enterochromaffin cell synthesis.
//! Tests the diversity → tryptophan → serotonin causal chain.
//!
//! Reference: Yano et al. 2015 (Cell), Clarke et al. 2013,
//!            Cryan & Dinan 2012 (Nat Rev Neurosci).

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
    println!("healthSpring Exp080 — Gut-Brain Serotonin Pathway");
    println!("{}", "=".repeat(72));

    let dietary_trp = 200.0;
    let k_synth = 0.8;
    let scale = 0.1;

    // Community Shannon H' values
    let h_healthy = microbiome::shannon_index(&microbiome::communities::HEALTHY_GUT);
    let h_dysbiotic = microbiome::shannon_index(&microbiome::communities::DYSBIOTIC_GUT);
    let h_cdiff = microbiome::shannon_index(&microbiome::communities::CDIFF_COLONIZED);

    println!("\n  Shannon: healthy={h_healthy:.3}, cdiff={h_cdiff:.3}, dysbiotic={h_dysbiotic:.3}");

    // Check 1: Higher diversity → higher tryptophan availability
    println!("\n--- Check 1: Diversity → tryptophan availability ---");
    let trp_healthy = microbiome::tryptophan_availability(dietary_trp, h_healthy);
    let trp_dysbiotic = microbiome::tryptophan_availability(dietary_trp, h_dysbiotic);
    check!(
        passed,
        failed,
        format!("trp: healthy={trp_healthy:.1} > dysbiotic={trp_dysbiotic:.1}"),
        trp_healthy > trp_dysbiotic
    );

    // Check 2: Tryptophan availability in physiological range
    println!("\n--- Check 2: Tryptophan in range ---");
    check!(
        passed,
        failed,
        format!("trp_healthy={trp_healthy:.1} µmol/L in [80, 180]"),
        trp_healthy > 80.0 && trp_healthy < 180.0
    );

    // Check 3: Higher diversity → more serotonin production
    println!("\n--- Check 3: Diversity → serotonin ---");
    let ser_healthy = microbiome::gut_serotonin_production(trp_healthy, h_healthy, k_synth, scale);
    let ser_dysbiotic =
        microbiome::gut_serotonin_production(trp_dysbiotic, h_dysbiotic, k_synth, scale);
    check!(
        passed,
        failed,
        format!("5-HT: healthy={ser_healthy:.1} > dysbiotic={ser_dysbiotic:.1}"),
        ser_healthy > ser_dysbiotic
    );

    // Check 4: Serotonin positive for all states
    println!("\n--- Check 4: Serotonin > 0 ---");
    let ser_cdiff = microbiome::gut_serotonin_production(
        microbiome::tryptophan_availability(dietary_trp, h_cdiff),
        h_cdiff,
        k_synth,
        scale,
    );
    check!(
        passed,
        failed,
        "all serotonin values positive",
        ser_healthy > 0.0 && ser_dysbiotic > 0.0 && ser_cdiff > 0.0
    );

    // Check 5: Serotonin ordering matches diversity ordering
    println!("\n--- Check 5: Serotonin ordering ---");
    check!(
        passed,
        failed,
        format!(
            "5-HT: healthy({ser_healthy:.1}) > cdiff({ser_cdiff:.1}) > dysbiotic({ser_dysbiotic:.1})"
        ),
        ser_healthy > ser_cdiff && ser_cdiff > ser_dysbiotic
    );

    // Check 6: Diversity factor sigmoid shape
    println!("\n--- Check 6: Sigmoid diversity factor ---");
    let low_div = microbiome::gut_serotonin_production(100.0, 0.5, 1.0, 0.1);
    let mid_div = microbiome::gut_serotonin_production(100.0, 1.5, 1.0, 0.1);
    let high_div = microbiome::gut_serotonin_production(100.0, 2.5, 1.0, 0.1);
    check!(
        passed,
        failed,
        format!("sigmoid: low={low_div:.2} < mid={mid_div:.2} < high={high_div:.2}"),
        low_div < mid_div && mid_div < high_div
    );

    // Check 7: Mid-point of sigmoid near H_ref=1.5
    println!("\n--- Check 7: Sigmoid midpoint at H'=1.5 ---");
    let at_midpoint = microbiome::gut_serotonin_production(100.0, 1.5, 1.0, 0.1);
    check!(
        passed,
        failed,
        format!("5-HT at H'=1.5 ≈ 50 (midpoint): {at_midpoint:.1}"),
        at_midpoint > 40.0 && at_midpoint < 60.0
    );

    // Check 8: Tryptophan availability monotone with diversity
    println!("\n--- Check 8: Tryptophan monotone ---");
    let steps: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.15).collect();
    let trps: Vec<f64> = steps
        .iter()
        .map(|&h| microbiome::tryptophan_availability(200.0, h))
        .collect();
    let monotone = trps.windows(2).all(|w| w[1] >= w[0] - 1e-10);
    check!(
        passed,
        failed,
        "tryptophan increases monotonically with H'",
        monotone
    );

    // Check 9: Cross-track: testosterone-gut link
    println!("\n--- Check 9: Testosterone-gut cross-track ---");
    let h_post_fmt = 2.1;
    let ser_post_fmt = microbiome::gut_serotonin_production(
        microbiome::tryptophan_availability(dietary_trp, h_post_fmt),
        h_post_fmt,
        k_synth,
        scale,
    );
    check!(
        passed,
        failed,
        format!("FMT recovery: 5-HT={ser_post_fmt:.1} > cdiff={ser_cdiff:.1}"),
        ser_post_fmt > ser_cdiff
    );

    // Check 10: Zero tryptophan → zero serotonin
    println!("\n--- Check 10: Zero tryptophan ---");
    let ser_zero = microbiome::gut_serotonin_production(0.0, h_healthy, k_synth, scale);
    check!(
        passed,
        failed,
        format!("5-HT at trp=0: {ser_zero:.6} ≈ 0"),
        ser_zero.abs() < 1e-10
    );

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("Exp080 Gut-Brain Serotonin: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
