#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp011 validation: Anderson Localization in Gut Lattice
//!
//! Cross-validates `healthspring_barracuda::microbiome` Anderson lattice
//! functions against the Python control (`exp011_anderson_gut_lattice.py`).

use healthspring_barracuda::microbiome;

const L: usize = 50;
const T_HOP: f64 = 1.0;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp011 — Rust CPU Validation: Anderson Gut Lattice");
    println!("  L={L}");
    println!("{}", "=".repeat(72));

    // Build Hamiltonian with known disorder
    #[expect(clippy::cast_precision_loss, reason = "i < 50")]
    let disorder: Vec<f64> = (0..L).map(|i| (i as f64 - 25.0) * 0.2).collect();
    let h = microbiome::anderson_hamiltonian_1d(&disorder, T_HOP);

    // Check 1: Matrix size
    println!("\n--- Check 1: Matrix size ---");
    if h.len() == L * L {
        println!("  [PASS] {L}×{L} = {} elements", h.len());
        passed += 1;
    } else {
        println!("  [FAIL] {} elements", h.len());
        failed += 1;
    }

    // Check 2: Symmetric
    println!("\n--- Check 2: Symmetric ---");
    let mut symmetric = true;
    for i in 0..L {
        for j in 0..L {
            if (h[i * L + j] - h[j * L + i]).abs() > 1e-14 {
                symmetric = false;
            }
        }
    }
    if symmetric {
        println!("  [PASS] H = H^T");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 3: Diagonal = disorder
    println!("\n--- Check 3: Diagonal = disorder ---");
    let diag_ok = (0..L).all(|i| (h[i * L + i] - disorder[i]).abs() < 1e-14);
    if diag_ok {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 4: Off-diagonal hopping
    println!("\n--- Check 4: Nearest-neighbor hopping ---");
    let hop_ok = (0..L - 1).all(|i| (h[i * L + (i + 1)] - T_HOP).abs() < 1e-14);
    if hop_ok {
        println!("  [PASS] t = {T_HOP}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 5: No long-range hopping
    println!("\n--- Check 5: No long-range hopping ---");
    let mut no_lr = true;
    for i in 0..L {
        for j in 0..L {
            if i != j && i.abs_diff(j) > 1 && h[i * L + j].abs() > 1e-14 {
                no_lr = false;
            }
        }
    }
    if no_lr {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 6: IPR of uniform state = 1/L
    println!("\n--- Check 6: IPR(uniform) = 1/L ---");
    #[expect(clippy::cast_precision_loss, reason = "L = 50")]
    let l_f64 = L as f64;
    let val = 1.0 / l_f64.sqrt();
    let uniform: Vec<f64> = vec![val; L];
    let ipr = microbiome::inverse_participation_ratio(&uniform);
    let expected = 1.0 / l_f64;
    if (ipr - expected).abs() < 1e-10 {
        println!("  [PASS] IPR = {ipr:.8}, expected = {expected:.8}");
        passed += 1;
    } else {
        println!("  [FAIL] IPR = {ipr:.8}");
        failed += 1;
    }

    // Check 7: IPR of delta state = 1.0
    println!("\n--- Check 7: IPR(delta) = 1.0 ---");
    let mut delta = vec![0.0; L];
    delta[L / 2] = 1.0;
    let ipr_d = microbiome::inverse_participation_ratio(&delta);
    if (ipr_d - 1.0).abs() < 1e-14 {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL] IPR = {ipr_d}");
        failed += 1;
    }

    // Check 8: ξ = 1/IPR
    println!("\n--- Check 8: ξ = 1/IPR ---");
    let xi = microbiome::localization_length_from_ipr(0.25);
    if (xi - 4.0).abs() < 1e-14 {
        println!("  [PASS] ξ(0.25) = {xi}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 9: ξ(0) = inf
    println!("\n--- Check 9: ξ(0) = ∞ ---");
    let xi_0 = microbiome::localization_length_from_ipr(0.0);
    if xi_0.is_infinite() {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 10: Level spacing ratio with few values
    println!("\n--- Check 10: Level spacing ratio (few) ---");
    let r = microbiome::level_spacing_ratio(&[1.0, 2.0]);
    if r == 0.0 {
        println!("  [PASS] <r> = 0 for < 3 values");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 11: Uniform spacing → r ≈ 1
    println!("\n--- Check 11: Uniform spacing → r ≈ 1 ---");
    let uniform_eigs: Vec<f64> = (0..50).map(f64::from).collect();
    let r_u = microbiome::level_spacing_ratio(&uniform_eigs);
    if (r_u - 1.0).abs() < 0.02 {
        println!("  [PASS] <r> = {r_u:.4}");
        passed += 1;
    } else {
        println!("  [FAIL] <r> = {r_u:.4}");
        failed += 1;
    }

    // Check 12: CR(ξ=2) > CR(ξ=50)
    println!("\n--- Check 12: CR ordering ---");
    let cr_short = microbiome::colonization_resistance(2.0);
    let cr_long = microbiome::colonization_resistance(50.0);
    if cr_short > cr_long {
        println!("  [PASS] CR(ξ=2)={cr_short:.4} > CR(ξ=50)={cr_long:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 13: Pielou → disorder mapping
    println!("\n--- Check 13: Pielou → W ---");
    let w_h = microbiome::evenness_to_disorder(0.863, 10.0);
    let w_d = microbiome::evenness_to_disorder(0.303, 10.0);
    if w_h > w_d && (w_h - 8.63).abs() < 0.01 {
        println!("  [PASS] W(healthy)={w_h:.2} > W(dysbiotic)={w_d:.2}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 14: W=0 lattice is clean (all zeros on diagonal)
    println!("\n--- Check 14: W=0 clean lattice ---");
    let disorder_zero = vec![0.0; L];
    let h_clean = microbiome::anderson_hamiltonian_1d(&disorder_zero, T_HOP);
    let diag_zero = (0..L).all(|i| h_clean[i * L + i].abs() < 1e-14);
    if diag_zero {
        println!("  [PASS] all diagonal = 0");
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
