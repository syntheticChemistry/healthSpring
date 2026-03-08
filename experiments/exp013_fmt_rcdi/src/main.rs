#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp013 validation: FMT (Fecal Microbiota Transplant) for rCDI
//!
//! Validates FMT engraftment → diversity restoration pipeline:
//! - `fmt_blend`, `bray_curtis`
//! - Shannon, Pielou, CR improvement with engraftment

use healthspring_barracuda::microbiome::{self, communities};

const TOL: f64 = 1e-10;
const ENGRAFTMENT_LEVELS: [f64; 4] = [0.3, 0.5, 0.7, 0.9];

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp013 — Rust CPU Validation: FMT for rCDI");
    println!("{}", "=".repeat(72));

    let donor = &communities::HEALTHY_GUT[..];
    let recipient = &communities::DYSBIOTIC_GUT[..];

    let pre_shannon = microbiome::shannon_index(recipient);
    let pre_pielou = microbiome::pielou_evenness(recipient);

    println!("\n  Pre-FMT: H'={pre_shannon:.4}  J={pre_pielou:.4}");

    // Check 1: Pre-FMT Shannon < post-FMT Shannon (for engraftment > 0.3)
    println!("\n--- Check 1: Pre-FMT Shannon < post-FMT Shannon (engraftment > 0.3) ---");
    let post_03 = microbiome::fmt_blend(donor, recipient, 0.3);
    let shannon_03 = microbiome::shannon_index(&post_03);
    if shannon_03 > pre_shannon {
        println!("  [PASS] H'(post 0.3)={shannon_03:.4} > H'(pre)={pre_shannon:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 2: Monotonic improvement with increasing engraftment
    println!("\n--- Check 2: Monotonic improvement with engraftment ---");
    let shannons: Vec<f64> = ENGRAFTMENT_LEVELS
        .iter()
        .map(|&e| microbiome::shannon_index(&microbiome::fmt_blend(donor, recipient, e)))
        .collect();
    let monotonic = shannons.windows(2).all(|w| w[1] > w[0]);
    if monotonic {
        println!("  [PASS] Shannon: {shannons:?} strictly increasing");
        passed += 1;
    } else {
        println!("  [FAIL] Shannon: {shannons:?}");
        failed += 1;
    }

    // Check 3: Bray-Curtis(post, donor) decreases with engraftment
    println!("\n--- Check 3: Bray-Curtis(post, donor) decreases with engraftment ---");
    let bcs: Vec<f64> = ENGRAFTMENT_LEVELS
        .iter()
        .map(|&e| microbiome::bray_curtis(&microbiome::fmt_blend(donor, recipient, e), donor))
        .collect();
    let bc_decreasing = bcs.windows(2).all(|w| w[1] < w[0]);
    if bc_decreasing {
        println!("  [PASS] BC(post, donor): {bcs:?} strictly decreasing");
        passed += 1;
    } else {
        println!("  [FAIL] BC: {bcs:?}");
        failed += 1;
    }

    // Check 4: Bray-Curtis range [0, 1]
    println!("\n--- Check 4: Bray-Curtis range [0, 1] ---");
    let bc_healthy_dys = microbiome::bray_curtis(donor, recipient);
    let bc_identical = microbiome::bray_curtis(donor, donor);
    let in_range =
        (0.0..=1.0 + TOL).contains(&bc_healthy_dys) && (0.0..=1.0 + TOL).contains(&bc_identical);
    if in_range {
        println!(
            "  [PASS] BC(healthy,dysbiotic)={bc_healthy_dys:.4}, BC(identical)={bc_identical:.4}"
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 5: Bray-Curtis symmetry: BC(a,b) = BC(b,a)
    println!("\n--- Check 5: Bray-Curtis symmetry BC(a,b) = BC(b,a) ---");
    let bc_ab = microbiome::bray_curtis(donor, recipient);
    let bc_ba = microbiome::bray_curtis(recipient, donor);
    if (bc_ab - bc_ba).abs() < TOL {
        println!("  [PASS] BC(donor,recipient)={bc_ab:.4} = BC(recipient,donor)={bc_ba:.4}");
        passed += 1;
    } else {
        println!("  [FAIL] BC(a,b)={bc_ab:.4} != BC(b,a)={bc_ba:.4}");
        failed += 1;
    }

    // Check 6: 100% engraftment = donor community
    println!("\n--- Check 6: 100% engraftment = donor ---");
    let blended_100 = microbiome::fmt_blend(donor, recipient, 1.0);
    let match_donor = blended_100
        .iter()
        .zip(donor.iter())
        .all(|(a, b)| (a - b).abs() < TOL)
        && blended_100.len() == donor.len();
    if match_donor {
        println!("  [PASS] fmt_blend(., ., 1.0) == donor");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 7: 0% engraftment = recipient
    println!("\n--- Check 7: 0% engraftment = recipient ---");
    let blended_0 = microbiome::fmt_blend(donor, recipient, 0.0);
    let match_recipient = blended_0
        .iter()
        .zip(recipient.iter())
        .all(|(a, b)| (a - b).abs() < TOL)
        && blended_0.len() == recipient.len();
    if match_recipient {
        println!("  [PASS] fmt_blend(., ., 0.0) == recipient");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 8: All abundances sum to 1.0 (within tolerance)
    println!("\n--- Check 8: All abundances sum to 1.0 ---");
    let all_sum_one = ENGRAFTMENT_LEVELS.iter().all(|&e| {
        let b = microbiome::fmt_blend(donor, recipient, e);
        (b.iter().sum::<f64>() - 1.0).abs() < TOL
    });
    if all_sum_one {
        println!("  [PASS] All blended communities sum to 1.0");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 9: CR improves post-FMT (Pielou ↑ → W ↑ → ξ ↓ → CR ↑ in model)
    println!("\n--- Check 9: CR improves post-FMT ---");
    let post_07 = microbiome::fmt_blend(donor, recipient, 0.7);
    let post_pielou_07 = microbiome::pielou_evenness(&post_07);
    let cr_improves = post_pielou_07 > pre_pielou;
    if cr_improves {
        println!(
            "  [PASS] Pielou post-FMT(0.7)={post_pielou_07:.4} > pre={pre_pielou:.4} → CR improves"
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 10: Pielou improves post-FMT
    println!("\n--- Check 10: Pielou improves post-FMT ---");
    let pielous: Vec<f64> = ENGRAFTMENT_LEVELS
        .iter()
        .map(|&e| microbiome::pielou_evenness(&microbiome::fmt_blend(donor, recipient, e)))
        .collect();
    let pielou_improves = pielous.iter().all(|&j| j > pre_pielou);
    if pielou_improves {
        println!("  [PASS] All post-FMT Pielou > pre-FMT: {pielous:?}");
        passed += 1;
    } else {
        println!("  [FAIL] pre={pre_pielou:.4}, post={pielous:?}");
        failed += 1;
    }

    // Check 11: Bray-Curtis(identical) = 0
    println!("\n--- Check 11: Bray-Curtis(identical) = 0 ---");
    let bc_id = microbiome::bray_curtis(donor, donor);
    if bc_id.abs() < TOL {
        println!("  [PASS] BC(donor, donor)={bc_id:.6} ≈ 0");
        passed += 1;
    } else {
        println!("  [FAIL] BC(identical)={bc_id:.6}");
        failed += 1;
    }

    // Check 12: Post-FMT community non-negative
    println!("\n--- Check 12: Post-FMT community non-negative ---");
    let all_nonneg = ENGRAFTMENT_LEVELS.iter().all(|&e| {
        microbiome::fmt_blend(donor, recipient, e)
            .iter()
            .all(|&v| v >= -TOL)
    });
    if all_nonneg {
        println!("  [PASS] All abundances ≥ 0");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
