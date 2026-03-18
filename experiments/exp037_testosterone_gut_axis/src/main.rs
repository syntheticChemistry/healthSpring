// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! healthSpring Exp037 — Testosterone-Gut Axis (Rust validation)
//!
//! Cross-track (Track 2 × Track 4) validation. Uses deterministic
//! synthetic communities to verify the Pielou → Anderson → ξ → response
//! pipeline without RNG dependency.

use healthspring_barracuda::endocrine::{self, gut_axis_params as gap};
use healthspring_barracuda::microbiome;
use healthspring_barracuda::tolerances::MACHINE_EPSILON;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp037_testosterone_gut_axis");

    // Synthetic gut communities (deterministic)
    let n_species = 50;

    // Perfectly even community
    #[expect(clippy::cast_precision_loss, reason = "n_species = 50")]
    let n_species_f64 = n_species as f64;
    let even: Vec<f64> = vec![1.0 / n_species_f64; n_species];
    // Dominated community (one species at 90%)
    #[expect(clippy::cast_precision_loss, reason = "n_species - 1 = 49")]
    let dominated_share = 0.1 / (n_species - 1) as f64;
    let mut dominated = vec![dominated_share; n_species];
    dominated[0] = 0.9;
    // Moderately diverse
    #[expect(clippy::cast_precision_loss, reason = "values ≤ 50")]
    let moderate: Vec<f64> = (0..n_species)
        .map(|i| (n_species - i) as f64)
        .collect::<Vec<_>>();
    let mod_sum: f64 = moderate.iter().sum();
    let moderate: Vec<f64> = moderate.iter().map(|&w| w / mod_sum).collect();

    let j_even = microbiome::pielou_evenness(&even);
    let j_dom = microbiome::pielou_evenness(&dominated);
    let j_mod = microbiome::pielou_evenness(&moderate);

    let w_even = endocrine::evenness_to_disorder(j_even, gap::DISORDER_SCALE);
    let w_dom = endocrine::evenness_to_disorder(j_dom, gap::DISORDER_SCALE);
    let w_mod = endocrine::evenness_to_disorder(j_mod, gap::DISORDER_SCALE);

    let xi_even = endocrine::anderson_localization_length(w_even, gap::LATTICE_SIZE);
    let xi_dom = endocrine::anderson_localization_length(w_dom, gap::LATTICE_SIZE);
    let xi_mod = endocrine::anderson_localization_length(w_mod, gap::LATTICE_SIZE);

    let xi_max = xi_even.max(xi_dom).max(xi_mod);
    let resp_even = endocrine::gut_metabolic_response(xi_even, xi_max, gap::BASE_RESPONSE_KG);
    let resp_dom = endocrine::gut_metabolic_response(xi_dom, xi_max, gap::BASE_RESPONSE_KG);
    let resp_mod = endocrine::gut_metabolic_response(xi_mod, xi_max, gap::BASE_RESPONSE_KG);

    // --- Check 1: Pielou ordering ---
    h.check_bool("pielou_ordering", j_even > j_mod && j_mod > j_dom);

    // --- Check 2: Pielou in [0, 1] ---
    h.check_bool(
        "pielou_in_bounds",
        (0.0..=1.001).contains(&j_even)
            && (0.0..=1.001).contains(&j_dom)
            && (0.0..=1.001).contains(&j_mod),
    );

    // --- Check 3: Shannon > 0 ---
    let h_even = microbiome::shannon_index(&even);
    let h_dom = microbiome::shannon_index(&dominated);
    h.check_bool("shannon_positive", h_even > 0.0 && h_dom > 0.0);

    // --- Check 4: Disorder scales with Pielou ---
    h.check_bool(
        "disorder_scales_with_pielou",
        gap::DISORDER_SCALE.mul_add(-j_even, w_even).abs() < MACHINE_EPSILON
            && gap::DISORDER_SCALE.mul_add(-j_dom, w_dom).abs() < MACHINE_EPSILON,
    );

    // --- Check 5: ξ ordering ---
    h.check_bool("xi_ordering", xi_even > xi_mod && xi_mod > xi_dom);

    // --- Check 6: ξ > 0 ---
    h.check_bool(
        "all_xi_positive",
        xi_even > 0.0 && xi_mod > 0.0 && xi_dom > 0.0,
    );

    // --- Check 7: Even gut → more weight loss (more negative) ---
    h.check_bool("even_gut_more_weight_loss", resp_even < resp_dom);

    // --- Check 8: Response ordering ---
    h.check_bool(
        "response_ordering",
        resp_even < resp_mod && resp_mod < resp_dom,
    );

    // --- Check 9: Response magnitude plausible ---
    h.check_bool(
        "response_magnitude_plausible",
        resp_even < 0.0 && resp_even > -20.0 && resp_dom < 0.0,
    );

    // --- Check 10: Zero disorder → ξ = 1 ---
    let xi_zero = endocrine::anderson_localization_length(0.0, gap::LATTICE_SIZE);
    h.check_abs("xi_zero_disorder", xi_zero, 1.0, MACHINE_EPSILON);

    h.exit();
}
