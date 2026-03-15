// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp037 — Testosterone-Gut Axis (Rust validation)
//!
//! Cross-track (Track 2 × Track 4) validation. Uses deterministic
//! synthetic communities to verify the Pielou → Anderson → ξ → response
//! pipeline without RNG dependency.

use healthspring_barracuda::endocrine::{self, gut_axis_params as gap};
use healthspring_barracuda::microbiome;
use healthspring_barracuda::tolerances::MACHINE_EPSILON;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp037: Testosterone-Gut Axis (Rust)");
    println!("{}", "=".repeat(72));

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
    println!("\n--- Check 1: Pielou ordering J(even) > J(moderate) > J(dominated) ---");
    if j_even > j_mod && j_mod > j_dom {
        println!("  [PASS] J: even={j_even:.3}, mod={j_mod:.3}, dom={j_dom:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] J: {j_even:.3}, {j_mod:.3}, {j_dom:.3}");
        failed += 1;
    }

    // --- Check 2: Pielou in [0, 1] ---
    println!("\n--- Check 2: Pielou in [0, 1] ---");
    if (0.0..=1.001).contains(&j_even)
        && (0.0..=1.001).contains(&j_dom)
        && (0.0..=1.001).contains(&j_mod)
    {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 3: Shannon > 0 ---
    println!("\n--- Check 3: Shannon H' > 0 ---");
    let h_even = microbiome::shannon_index(&even);
    let h_dom = microbiome::shannon_index(&dominated);
    if h_even > 0.0 && h_dom > 0.0 {
        println!("  [PASS] H'(even)={h_even:.3}, H'(dom)={h_dom:.3}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 4: Disorder scales with Pielou ---
    println!("\n--- Check 4: W scales linearly with J ---");
    if gap::DISORDER_SCALE.mul_add(-j_even, w_even).abs() < MACHINE_EPSILON
        && gap::DISORDER_SCALE.mul_add(-j_dom, w_dom).abs() < MACHINE_EPSILON
    {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 5: ξ ordering ---
    println!("\n--- Check 5: ξ(even) > ξ(mod) > ξ(dom) ---");
    if xi_even > xi_mod && xi_mod > xi_dom {
        println!("  [PASS] ξ: even={xi_even:.2}, mod={xi_mod:.2}, dom={xi_dom:.2}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 6: ξ > 0 ---
    println!("\n--- Check 6: All ξ > 0 ---");
    if xi_even > 0.0 && xi_mod > 0.0 && xi_dom > 0.0 {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 7: Even gut → more weight loss (more negative) ---
    println!("\n--- Check 7: Even gut → more weight loss ---");
    if resp_even < resp_dom {
        println!("  [PASS] even={resp_even:.2} < dom={resp_dom:.2} kg");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 8: Response ordering ---
    println!("\n--- Check 8: Response ordering ---");
    if resp_even < resp_mod && resp_mod < resp_dom {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL] {resp_even:.2}, {resp_mod:.2}, {resp_dom:.2}");
        failed += 1;
    }

    // --- Check 9: Response magnitude plausible ---
    println!("\n--- Check 9: Response magnitude plausible ---");
    if resp_even < 0.0 && resp_even > -20.0 && resp_dom < 0.0 {
        println!("  [PASS] range: [{resp_even:.2}, {resp_dom:.2}] kg");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 10: Zero disorder → ξ = 1 ---
    println!("\n--- Check 10: Zero disorder → minimal ξ ---");
    let xi_zero = endocrine::anderson_localization_length(0.0, gap::LATTICE_SIZE);
    if (xi_zero - 1.0).abs() < MACHINE_EPSILON {
        println!("  [PASS] ξ(W=0) = {xi_zero}");
        passed += 1;
    } else {
        println!("  [FAIL] ξ(W=0) = {xi_zero}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
