// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! Exp105: Canine gut microbiome Anderson lattice (CM-006)
//!
//! Validates `Shannon`, `Pielou`, `evenness_to_disorder` on synthetic canine gut communities.

use healthspring_barracuda::microbiome::{
    anderson_diagonalize, evenness_to_disorder, inverse_participation_ratio,
    localization_length_from_ipr, pielou_evenness, shannon_index,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{DETERMINISM, DIVERSITY_CROSS_VALIDATE, MACHINE_EPSILON};
use healthspring_barracuda::validation::ValidationHarness;

const W_SCALE: f64 = 10.0;
const LATTICE_L: usize = 50;
const T_HOP: f64 = 1.0;

/// Synthetic healthy dog gut: 6 species, high evenness.
fn healthy_dog() -> Vec<f64> {
    vec![0.20, 0.18, 0.17, 0.16, 0.15, 0.14]
}

/// Synthetic AD dog gut: 6 species, low evenness.
fn ad_dog() -> Vec<f64> {
    vec![0.60, 0.15, 0.10, 0.08, 0.05, 0.02]
}

/// Synthetic post-treatment dog: intermediate evenness.
fn post_treatment_dog() -> Vec<f64> {
    vec![0.35, 0.25, 0.18, 0.12, 0.07, 0.03]
}

/// Build disorder vector from W for Anderson lattice.
#[expect(clippy::cast_precision_loss, reason = "lattice size < 2^52")]
fn disorder_from_w(w: f64) -> Vec<f64> {
    let l_f64 = LATTICE_L as f64;
    (0..LATTICE_L)
        .map(|i| w * ((i as f64) / l_f64 - 0.5))
        .collect()
}

/// Compute localization length ξ from Pielou evenness via Anderson lattice.
fn xi_from_evenness(evenness: f64) -> f64 {
    let w = evenness_to_disorder(evenness, W_SCALE);
    let disorder = disorder_from_w(w);
    let (_eigs, evecs) = anderson_diagonalize(&disorder, T_HOP);
    let mid = LATTICE_L / 2;
    let psi: Vec<f64> = (0..LATTICE_L).map(|j| evecs[mid * LATTICE_L + j]).collect();
    let ipr = inverse_participation_ratio(&psi);
    localization_length_from_ipr(ipr)
}

fn main() {
    let mut h = ValidationHarness::new("exp105_canine_gut_anderson");

    log_analytical(&AnalyticalProvenance {
        formula: "H = -Σ p_i ln(p_i); J = H/ln(N); W = W_max × J",
        reference: "Shannon 1948 + Anderson 1958",
        doi: None,
    });

    let healthy = healthy_dog();
    let ad = ad_dog();
    let treated = post_treatment_dog();

    let shannon_healthy = shannon_index(&healthy);
    let shannon_ad = shannon_index(&ad);
    let shannon_treated = shannon_index(&treated);

    let pielou_healthy = pielou_evenness(&healthy);
    let pielou_ad = pielou_evenness(&ad);
    let pielou_treated = pielou_evenness(&treated);

    let w_healthy = evenness_to_disorder(pielou_healthy, W_SCALE);
    let w_ad = evenness_to_disorder(pielou_ad, W_SCALE);
    let w_treated = evenness_to_disorder(pielou_treated, W_SCALE);

    // 1. Shannon index: healthy > AD (higher diversity)
    h.check_bool(
        "Shannon index: healthy > AD (higher diversity)",
        shannon_healthy > shannon_ad,
    );

    // 2. Shannon index: treated > AD (treatment restores some diversity)
    h.check_bool(
        "Shannon index: treated > AD (treatment restores some diversity)",
        shannon_treated > shannon_ad,
    );

    // 3. Pielou evenness: healthy > AD
    h.check_bool("Pielou evenness: healthy > AD", pielou_healthy > pielou_ad);

    // 4. Pielou evenness: treated > AD
    h.check_bool("Pielou evenness: treated > AD", pielou_treated > pielou_ad);

    // 5. evenness_to_disorder: healthy W > AD W
    h.check_bool("evenness_to_disorder: healthy W > AD W", w_healthy > w_ad);

    // 6. evenness_to_disorder: higher W → shorter ξ (better colonization resistance)
    let xi_healthy = xi_from_evenness(pielou_healthy);
    let xi_ad = xi_from_evenness(pielou_ad);
    h.check_bool(
        "evenness_to_disorder: higher W → shorter ξ (better colonization resistance)",
        xi_healthy < xi_ad,
    );

    // 7. Shannon of uniform distribution = ln(N) (analytical identity)
    let uniform_n6 = vec![1.0 / 6.0; 6];
    let shannon_uniform = shannon_index(&uniform_n6);
    let expected_ln_n = 6.0_f64.ln();
    h.check_abs(
        "Shannon of uniform distribution = ln(N)",
        shannon_uniform,
        expected_ln_n,
        DIVERSITY_CROSS_VALIDATE,
    );

    // 8. Pielou of uniform distribution = 1.0 (maximum evenness)
    let pielou_uniform = pielou_evenness(&uniform_n6);
    h.check_abs(
        "Pielou of uniform distribution = 1.0",
        pielou_uniform,
        1.0,
        MACHINE_EPSILON,
    );

    // 9. Shannon non-negative for all communities
    h.check_bool(
        "Shannon non-negative for all communities",
        shannon_healthy >= 0.0 && shannon_ad >= 0.0 && shannon_treated >= 0.0,
    );

    // 10. Cross-species: same abundances → same Shannon (species-agnostic math)
    let same_abundances = vec![0.25, 0.25, 0.25, 0.25];
    let shannon_a = shannon_index(&same_abundances);
    let shannon_b = shannon_index(&same_abundances);
    h.check_abs(
        "Cross-species: same abundances → same Shannon",
        shannon_a,
        shannon_b,
        DETERMINISM,
    );

    // 11. Disorder parameter monotonic with evenness
    h.check_bool(
        "Disorder parameter monotonic with evenness",
        w_healthy > w_treated && w_treated > w_ad,
    );

    // 12. AD dog has longest localization length (most vulnerable)
    h.check_bool(
        "AD dog has longest localization length (most vulnerable)",
        xi_ad > xi_healthy && xi_ad > xi_from_evenness(pielou_treated),
    );

    // 13. Determinism
    let run1 = shannon_index(&healthy);
    let run2 = shannon_index(&healthy);
    h.check_abs("Determinism", run1, run2, DETERMINISM);

    h.exit();
}
