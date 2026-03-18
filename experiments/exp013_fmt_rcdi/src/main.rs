// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp013 validation: FMT (Fecal Microbiota Transplant) for rCDI
//!
//! Validates FMT engraftment → diversity restoration pipeline:
//! - `fmt_blend`, `bray_curtis`
//! - Shannon, Pielou, CR improvement with engraftment

use healthspring_barracuda::microbiome::{self, communities};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

const ENGRAFTMENT_LEVELS: [f64; 4] = [0.3, 0.5, 0.7, 0.9];

fn main() {
    let mut h = ValidationHarness::new("Exp013 FMT for rCDI");

    let donor = &communities::HEALTHY_GUT[..];
    let recipient = &communities::DYSBIOTIC_GUT[..];

    let pre_shannon = microbiome::shannon_index(recipient);
    let pre_pielou = microbiome::pielou_evenness(recipient);

    // Check 1: Pre-FMT Shannon < post-FMT Shannon (for engraftment > 0.3)
    let post_03 = microbiome::fmt_blend(donor, recipient, 0.3);
    let shannon_03 = microbiome::shannon_index(&post_03);
    h.check_bool("Shannon improves post-FMT 0.3", shannon_03 > pre_shannon);

    // Check 2: Monotonic improvement with increasing engraftment
    let shannons: Vec<f64> = ENGRAFTMENT_LEVELS
        .iter()
        .map(|&e| microbiome::shannon_index(&microbiome::fmt_blend(donor, recipient, e)))
        .collect();
    let monotonic = shannons.windows(2).all(|w| w[1] > w[0]);
    h.check_bool("Shannon monotonic with engraftment", monotonic);

    // Check 3: Bray-Curtis(post, donor) decreases with engraftment
    let bcs: Vec<f64> = ENGRAFTMENT_LEVELS
        .iter()
        .map(|&e| microbiome::bray_curtis(&microbiome::fmt_blend(donor, recipient, e), donor))
        .collect();
    let bc_decreasing = bcs.windows(2).all(|w| w[1] < w[0]);
    h.check_bool("Bray-Curtis decreases with engraftment", bc_decreasing);

    // Check 4: Bray-Curtis range [0, 1]
    let bc_healthy_dys = microbiome::bray_curtis(donor, recipient);
    let bc_identical = microbiome::bray_curtis(donor, donor);
    let in_range = (0.0..=1.0 + tolerances::MACHINE_EPSILON).contains(&bc_healthy_dys)
        && (0.0..=1.0 + tolerances::MACHINE_EPSILON).contains(&bc_identical);
    h.check_bool("Bray-Curtis range [0, 1]", in_range);

    // Check 5: Bray-Curtis symmetry: BC(a,b) = BC(b,a)
    let bc_ab = microbiome::bray_curtis(donor, recipient);
    let bc_ba = microbiome::bray_curtis(recipient, donor);
    h.check_abs(
        "Bray-Curtis symmetry",
        bc_ab,
        bc_ba,
        tolerances::MACHINE_EPSILON,
    );

    // Check 6: 100% engraftment = donor community
    let blended_100 = microbiome::fmt_blend(donor, recipient, 1.0);
    let match_donor = blended_100
        .iter()
        .zip(donor.iter())
        .all(|(a, b)| (a - b).abs() < tolerances::MACHINE_EPSILON)
        && blended_100.len() == donor.len();
    h.check_bool("100% engraftment = donor", match_donor);

    // Check 7: 0% engraftment = recipient
    let blended_0 = microbiome::fmt_blend(donor, recipient, 0.0);
    let match_recipient = blended_0
        .iter()
        .zip(recipient.iter())
        .all(|(a, b)| (a - b).abs() < tolerances::MACHINE_EPSILON)
        && blended_0.len() == recipient.len();
    h.check_bool("0% engraftment = recipient", match_recipient);

    // Check 8: All abundances sum to 1.0 (within tolerance)
    let all_sum_one = ENGRAFTMENT_LEVELS.iter().all(|&e| {
        let b = microbiome::fmt_blend(donor, recipient, e);
        (b.iter().sum::<f64>() - 1.0).abs() < tolerances::MACHINE_EPSILON
    });
    h.check_bool("All abundances sum to 1.0", all_sum_one);

    // Check 9: CR improves post-FMT (Pielou ↑ → W ↑ → ξ ↓ → CR ↑ in model)
    let post_07 = microbiome::fmt_blend(donor, recipient, 0.7);
    let post_pielou_07 = microbiome::pielou_evenness(&post_07);
    h.check_bool("CR improves post-FMT", post_pielou_07 > pre_pielou);

    // Check 10: Pielou improves post-FMT
    let pielous: Vec<f64> = ENGRAFTMENT_LEVELS
        .iter()
        .map(|&e| microbiome::pielou_evenness(&microbiome::fmt_blend(donor, recipient, e)))
        .collect();
    let pielou_improves = pielous.iter().all(|&j| j > pre_pielou);
    h.check_bool("Pielou improves post-FMT", pielou_improves);

    // Check 11: Bray-Curtis(identical) = 0
    let bc_id = microbiome::bray_curtis(donor, donor);
    h.check_abs(
        "Bray-Curtis(identical)",
        bc_id,
        0.0,
        tolerances::MACHINE_EPSILON,
    );

    // Check 12: Post-FMT community non-negative
    let all_nonneg = ENGRAFTMENT_LEVELS.iter().all(|&e| {
        microbiome::fmt_blend(donor, recipient, e)
            .iter()
            .all(|&v| v >= -tolerances::MACHINE_EPSILON)
    });
    h.check_bool("Post-FMT non-negative", all_nonneg);

    h.exit();
}
