// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
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
use healthspring_barracuda::tolerances::{AUC_TRAPEZOIDAL, CPU_PARITY, MACHINE_EPSILON_TIGHT};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp040 barraCuda CPU Parity");

    // ═══════════════════════════════════════════════════════════════════════
    // PK/PD Parity
    // ═══════════════════════════════════════════════════════════════════════

    // Check 1: Hill E(EC50) = Emax/2
    {
        let ec50 = 10.0_f64;
        let emax = 1.0_f64;
        let e = hill_dose_response(ec50, ec50, 1.0, emax);
        h.check_abs(
            "Hill E(EC50) = Emax/2",
            e,
            emax / 2.0,
            MACHINE_EPSILON_TIGHT,
        );
    }

    // Check 2: Hill E(0) = 0
    {
        let e = hill_dose_response(0.0, 10.0, 1.0, 1.0);
        h.check_abs("Hill E(0) = 0", e, 0.0, MACHINE_EPSILON_TIGHT);
    }

    // Check 3: Hill E(∞) → Emax (saturation)
    {
        let emax = 1.0_f64;
        let e = hill_dose_response(1e12, 10.0, 1.0, emax);
        h.check_abs("Hill E(∞) → Emax", e, emax, CPU_PARITY);
    }

    // Check 4: One-compartment C(0) = dose/Vd
    {
        let dose = 100.0_f64;
        let vd = 25.0_f64;
        let c0 = pk_iv_bolus(dose, vd, 6.0, 0.0);
        let expected = dose / vd;
        h.check_abs(
            "One-compartment C(0) = dose/Vd",
            c0,
            expected,
            MACHINE_EPSILON_TIGHT,
        );
    }

    // Check 5: One-compartment C(t_half) = C(0)/2
    {
        let dose = 100.0_f64;
        let vd = 25.0_f64;
        let half_life = 6.0_f64;
        let c0 = dose / vd;
        let c_half = pk_iv_bolus(dose, vd, half_life, half_life);
        h.check_abs(
            "One-compartment C(t½) = C(0)/2",
            c_half,
            c0 / 2.0,
            CPU_PARITY,
        );
    }

    // Check 6: Two-compartment AUC = dose/CL (analytical)
    {
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
        h.check_upper("Two-compartment AUC rel_err", rel_err, AUC_TRAPEZOIDAL);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Microbiome Parity
    // ═══════════════════════════════════════════════════════════════════════

    // Check 7: Shannon(uniform) = ln(S)
    {
        let s = 10_usize;
        #[expect(clippy::cast_precision_loss, reason = "species count S ≪ 2^52")]
        let s_f64 = s as f64;
        let uniform: Vec<f64> = (0..s).map(|_| 1.0 / s_f64).collect();
        let h_shannon = shannon_index(&uniform);
        let expected = s_f64.ln();
        h.check_abs(
            "Shannon(uniform) = ln(S)",
            h_shannon,
            expected,
            MACHINE_EPSILON_TIGHT,
        );
    }

    // Check 8: Simpson(uniform) = 1 - 1/S
    {
        let s = 10_usize;
        #[expect(clippy::cast_precision_loss, reason = "species count S ≪ 2^52")]
        let s_f64 = s as f64;
        let uniform: Vec<f64> = (0..s).map(|_| 1.0 / s_f64).collect();
        let d = simpson_index(&uniform);
        let expected = 1.0 - 1.0 / s_f64;
        h.check_abs(
            "Simpson(uniform) = 1 - 1/S",
            d,
            expected,
            MACHINE_EPSILON_TIGHT,
        );
    }

    // Check 9: Pielou(uniform) = 1.0
    {
        let s = 10_usize;
        #[expect(clippy::cast_precision_loss, reason = "species count S ≪ 2^52")]
        let s_f64 = s as f64;
        let uniform: Vec<f64> = (0..s).map(|_| 1.0 / s_f64).collect();
        let j = pielou_evenness(&uniform);
        h.check_abs("Pielou(uniform) = 1.0", j, 1.0, MACHINE_EPSILON_TIGHT);
    }

    // Check 10: Chao1(no singletons) = S_obs
    {
        let counts: Vec<u64> = vec![2, 2, 2, 5, 10];
        #[expect(clippy::cast_precision_loss, reason = "count values small")]
        let s_obs = counts.len() as f64;
        let c = chao1(&counts);
        h.check_abs(
            "Chao1(no singletons) = S_obs",
            c,
            s_obs,
            MACHINE_EPSILON_TIGHT,
        );
    }

    // Check 11: Bray-Curtis(identical) = 0.0
    {
        let community = vec![0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01];
        let bc = bray_curtis(&community, &community);
        h.check_abs(
            "Bray-Curtis(identical) = 0.0",
            bc,
            0.0,
            MACHINE_EPSILON_TIGHT,
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Biosignal Parity
    // ═══════════════════════════════════════════════════════════════════════

    // Check 12: Squaring is non-negative
    {
        let signal = vec![-1.0, 0.0, 2.0, -3.5, 1e6];
        let sq = squaring(&signal);
        let all_nonneg = sq.iter().all(|&x| x >= 0.0);
        h.check_bool("Squaring non-negative", all_nonneg);
    }

    // Check 13: Moving window integration preserves length
    {
        let signal: Vec<f64> = (0..200).map(f64::from).collect();
        let mwi = moving_window_integration(&signal, 54);
        h.check_exact(
            "MWI preserves length",
            mwi.len() as u64,
            signal.len() as u64,
        );
    }

    // Check 14: PPG R-value with known AC/DC → exact result
    {
        let r = ppg_r_value(0.02, 1.0, 0.04, 1.0);
        let expected = 0.5_f64;
        h.check_abs("PPG R-value exact", r, expected, MACHINE_EPSILON_TIGHT);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Cross-cutting: Determinism
    // ═══════════════════════════════════════════════════════════════════════

    // Check 15: Determinism — bit-identical on repeat
    {
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
        h.check_bool("Determinism (bit-identical)", det);
    }

    h.exit();
}
