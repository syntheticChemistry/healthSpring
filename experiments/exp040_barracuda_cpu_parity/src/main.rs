// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — 15 analytical parity checks"
)]

//! healthSpring Exp040 — barraCuda CPU parity validation
//!
//! Validates that healthSpring's pure-Rust math produces analytically-correct
//! results that any correct implementation (CPU or GPU) must match. This is
//! the mathematical contract for barraCuda GPU shader promotion.

use healthspring_barracuda::biosignal::{moving_window_integration, ppg_r_value, squaring};
use healthspring_barracuda::microbiome::{
    bray_curtis, chao1, pielou_evenness, shannon_index, simpson_index,
};
use healthspring_barracuda::pkpd::{
    auc_trapezoidal, hill_dose_response, pk_iv_bolus, pk_two_compartment_iv,
};

const TOL_ANALYTICAL: f64 = 1e-12;
const TOL_DERIVED: f64 = 1e-10;

fn main() {
    let mut passed = 0_u32;
    let mut failed = 0_u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp040 [Rust]: barraCuda CPU Parity — Analytical Contract");
    println!("{}", "=".repeat(72));

    // ═══════════════════════════════════════════════════════════════════════
    // PK/PD Parity
    // ═══════════════════════════════════════════════════════════════════════

    // Check 1: Hill E(EC50) = Emax/2
    {
        print!("\n--- Check 1: Hill E(EC50) = Emax/2 --- ");
        let ec50 = 10.0_f64;
        let emax = 1.0_f64;
        let e = hill_dose_response(ec50, ec50, 1.0, emax);
        if (e - emax / 2.0).abs() < TOL_ANALYTICAL {
            println!("[PASS] E(EC50) = {e:.12}");
            passed += 1;
        } else {
            println!("[FAIL] E(EC50) = {e:.12}, expected {:.12}", emax / 2.0);
            failed += 1;
        }
    }

    // Check 2: Hill E(0) = 0
    {
        print!("\n--- Check 2: Hill E(0) = 0 --- ");
        let e = hill_dose_response(0.0, 10.0, 1.0, 1.0);
        if e.abs() < TOL_ANALYTICAL {
            println!("[PASS] E(0) = {e:.12}");
            passed += 1;
        } else {
            println!("[FAIL] E(0) = {e:.12}");
            failed += 1;
        }
    }

    // Check 3: Hill E(∞) → Emax (saturation)
    {
        print!("\n--- Check 3: Hill E(∞) → Emax --- ");
        let emax = 1.0_f64;
        let e = hill_dose_response(1e12, 10.0, 1.0, emax);
        if (e - emax).abs() < TOL_DERIVED {
            println!("[PASS] E(∞) ≈ {e:.12}");
            passed += 1;
        } else {
            println!("[FAIL] E(∞) = {e:.12}");
            failed += 1;
        }
    }

    // Check 4: One-compartment C(0) = dose/Vd
    {
        print!("\n--- Check 4: One-compartment C(0) = dose/Vd --- ");
        let dose = 100.0_f64;
        let vd = 25.0_f64;
        let c0 = pk_iv_bolus(dose, vd, 6.0, 0.0);
        let expected = dose / vd;
        if (c0 - expected).abs() < TOL_ANALYTICAL {
            println!("[PASS] C(0) = {c0:.12}");
            passed += 1;
        } else {
            println!("[FAIL] C(0) = {c0:.12}, expected {expected:.12}");
            failed += 1;
        }
    }

    // Check 5: One-compartment C(t_half) = C(0)/2
    {
        print!("\n--- Check 5: One-compartment C(t½) = C(0)/2 --- ");
        let dose = 100.0_f64;
        let vd = 25.0_f64;
        let half_life = 6.0_f64;
        let c0 = dose / vd;
        let c_half = pk_iv_bolus(dose, vd, half_life, half_life);
        if (c_half - c0 / 2.0).abs() < TOL_DERIVED {
            println!("[PASS] C(t½) = {c_half:.12}");
            passed += 1;
        } else {
            println!("[FAIL] C(t½) = {c_half:.12}, expected {:.12}", c0 / 2.0);
            failed += 1;
        }
    }

    // Check 6: Two-compartment AUC = dose/CL (analytical)
    {
        print!("\n--- Check 6: Two-compartment AUC = dose/CL --- ");
        let dose = 240.0_f64;
        let v1 = 15.0_f64;
        let k10 = 0.35_f64;
        let k12 = 0.6_f64;
        let k21 = 0.15_f64;
        let cl = v1 * k10;
        let auc_analytical = dose / cl;
        let times: Vec<f64> = (0..2000).map(|i| f64::from(i) * 168.0 / 1999.0).collect();
        let concs: Vec<f64> = times
            .iter()
            .map(|&t| pk_two_compartment_iv(dose, v1, k10, k12, k21, t))
            .collect();
        let auc_numerical = auc_trapezoidal(&times, &concs);
        let rel_err = (auc_numerical - auc_analytical).abs() / auc_analytical;
        if rel_err < 0.01 {
            println!("[PASS] AUC_num={auc_numerical:.4}, AUC_ana={auc_analytical:.4}");
            passed += 1;
        } else {
            println!("[FAIL] rel_err={rel_err:.6}");
            failed += 1;
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Microbiome Parity
    // ═══════════════════════════════════════════════════════════════════════

    // Check 7: Shannon(uniform) = ln(S)
    {
        print!("\n--- Check 7: Shannon(uniform) = ln(S) --- ");
        let s = 10_usize;
        #[expect(clippy::cast_precision_loss, reason = "species count S ≪ 2^52")]
        let s_f64 = s as f64;
        let uniform: Vec<f64> = (0..s).map(|_| 1.0 / s_f64).collect();
        let h = shannon_index(&uniform);
        let expected = s_f64.ln();
        if (h - expected).abs() < TOL_ANALYTICAL {
            println!("[PASS] H' = {h:.12}");
            passed += 1;
        } else {
            println!("[FAIL] H' = {h:.12}, expected {expected:.12}");
            failed += 1;
        }
    }

    // Check 8: Simpson(uniform) = 1 - 1/S
    {
        print!("\n--- Check 8: Simpson(uniform) = 1 - 1/S --- ");
        let s = 10_usize;
        #[expect(clippy::cast_precision_loss, reason = "species count S ≪ 2^52")]
        let s_f64 = s as f64;
        let uniform: Vec<f64> = (0..s).map(|_| 1.0 / s_f64).collect();
        let d = simpson_index(&uniform);
        let expected = 1.0 - 1.0 / s_f64;
        if (d - expected).abs() < TOL_ANALYTICAL {
            println!("[PASS] D = {d:.12}");
            passed += 1;
        } else {
            println!("[FAIL] D = {d:.12}, expected {expected:.12}");
            failed += 1;
        }
    }

    // Check 9: Pielou(uniform) = 1.0
    {
        print!("\n--- Check 9: Pielou(uniform) = 1.0 --- ");
        let s = 10_usize;
        #[expect(clippy::cast_precision_loss, reason = "species count S ≪ 2^52")]
        let s_f64 = s as f64;
        let uniform: Vec<f64> = (0..s).map(|_| 1.0 / s_f64).collect();
        let j = pielou_evenness(&uniform);
        if (j - 1.0).abs() < TOL_ANALYTICAL {
            println!("[PASS] J = {j:.12}");
            passed += 1;
        } else {
            println!("[FAIL] J = {j:.12}");
            failed += 1;
        }
    }

    // Check 10: Chao1(no singletons) = S_obs
    {
        print!("\n--- Check 10: Chao1(no singletons) = S_obs --- ");
        let counts: Vec<u64> = vec![2, 2, 2, 5, 10];
        #[expect(clippy::cast_precision_loss, reason = "count values small")]
        let s_obs = counts.len() as f64;
        let c = chao1(&counts);
        if (c - s_obs).abs() < TOL_ANALYTICAL {
            println!("[PASS] Chao1 = {c:.12}");
            passed += 1;
        } else {
            println!("[FAIL] Chao1 = {c:.12}, expected {s_obs:.12}");
            failed += 1;
        }
    }

    // Check 11: Bray-Curtis(identical) = 0.0
    {
        print!("\n--- Check 11: Bray-Curtis(identical) = 0.0 --- ");
        let community = vec![0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01];
        let bc = bray_curtis(&community, &community);
        if bc.abs() < TOL_ANALYTICAL {
            println!("[PASS] BC = {bc:.12}");
            passed += 1;
        } else {
            println!("[FAIL] BC = {bc:.12}");
            failed += 1;
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Biosignal Parity
    // ═══════════════════════════════════════════════════════════════════════

    // Check 12: Squaring is non-negative
    {
        print!("\n--- Check 12: Squaring non-negative --- ");
        let signal = vec![-1.0, 0.0, 2.0, -3.5, 1e6];
        let sq = squaring(&signal);
        let all_nonneg = sq.iter().all(|&x| x >= 0.0);
        if all_nonneg {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }
    }

    // Check 13: Moving window integration preserves length
    {
        print!("\n--- Check 13: MWI preserves length --- ");
        let signal: Vec<f64> = (0..200).map(f64::from).collect();
        let mwi = moving_window_integration(&signal, 54);
        if mwi.len() == signal.len() {
            println!("[PASS] len={}", mwi.len());
            passed += 1;
        } else {
            println!("[FAIL] input={}, output={}", signal.len(), mwi.len());
            failed += 1;
        }
    }

    // Check 14: PPG R-value with known AC/DC → exact result
    {
        print!("\n--- Check 14: PPG R-value exact --- ");
        // R = (AC_red/DC_red) / (AC_ir/DC_ir) = (0.02/1.0) / (0.04/1.0) = 0.5
        let r = ppg_r_value(0.02, 1.0, 0.04, 1.0);
        let expected = 0.5_f64;
        if (r - expected).abs() < TOL_ANALYTICAL {
            println!("[PASS] R = {r:.12}");
            passed += 1;
        } else {
            println!("[FAIL] R = {r:.12}, expected {expected:.12}");
            failed += 1;
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Cross-cutting: Determinism
    // ═══════════════════════════════════════════════════════════════════════

    // Check 15: Determinism — bit-identical on repeat
    {
        print!("\n--- Check 15: Determinism (bit-identical) --- ");
        let community = vec![0.25, 0.20, 0.15, 0.12, 0.10];
        let h1 = shannon_index(&community);
        let h2 = shannon_index(&community);
        let d1 = simpson_index(&community);
        let d2 = simpson_index(&community);
        let e1 = hill_dose_response(10.0, 10.0, 1.0, 1.0);
        let e2 = hill_dose_response(10.0, 10.0, 1.0, 1.0);
        let c1 = pk_iv_bolus(100.0, 25.0, 6.0, 3.0);
        let c2 = pk_iv_bolus(100.0, 25.0, 6.0, 3.0);
        let r1 = ppg_r_value(0.02, 1.0, 0.04, 1.0);
        let r2 = ppg_r_value(0.02, 1.0, 0.04, 1.0);
        let det = h1.to_bits() == h2.to_bits()
            && d1.to_bits() == d2.to_bits()
            && e1.to_bits() == e2.to_bits()
            && c1.to_bits() == c2.to_bits()
            && r1.to_bits() == r2.to_bits();
        if det {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    std::process::exit(i32::from(failed > 0));
}
