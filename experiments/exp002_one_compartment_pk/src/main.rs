// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
#![expect(
    clippy::similar_names,
    reason = "iv/oral parameter variants are intentionally parallel"
)]

//! healthSpring Exp002 — Rust validation binary
//!
//! Cross-validates one-compartment PK models against the Python control
//! baseline (`control/pkpd/exp002_baseline.json`).

use healthspring_barracuda::pkpd::{
    auc_trapezoidal, find_cmax_tmax, oral_tmax, pk_iv_bolus, pk_multiple_dose,
    pk_oral_one_compartment,
};

const DOSE_IV: f64 = 500.0;
const VD_IV: f64 = 50.0;
const HL_IV: f64 = 6.0;

const DOSE_ORAL: f64 = 250.0;
const F_ORAL: f64 = 0.8;
const VD_ORAL: f64 = 35.0;
const HL_ORAL: f64 = 4.0;
const KA_ORAL: f64 = 1.5;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "n always small")]
            let frac = i as f64 / (n - 1) as f64;
            start + frac * (end - start)
        })
        .collect()
}

fn main() {
    let mut passed = 0_u32;
    let mut failed = 0_u32;
    let times = linspace(0.0, 48.0, 1000);
    let k_e_iv = core::f64::consts::LN_2 / HL_IV;
    let k_e_oral = core::f64::consts::LN_2 / HL_ORAL;
    let c0_iv = DOSE_IV / VD_IV;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp002 [Rust]: One-Compartment PK Models");
    println!("{}", "=".repeat(72));

    // Check 1: IV C(0)
    print!("\n--- Check 1: IV C(0) = Dose/Vd --- ");
    let c_at_0 = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, 0.0);
    if (c_at_0 - c0_iv).abs() < 1e-10 {
        println!("[PASS] C(0) = {c_at_0:.4}");
        passed += 1;
    } else {
        println!("[FAIL] C(0) = {c_at_0:.4}");
        failed += 1;
    }

    // Check 2: IV at half-life
    print!("\n--- Check 2: IV at half-life → C0/2 --- ");
    let c_at_hl = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, HL_IV);
    if (c_at_hl - c0_iv / 2.0).abs() < 1e-6 {
        println!("[PASS] C(t½) = {c_at_hl:.6}");
        passed += 1;
    } else {
        println!("[FAIL] C(t½) = {c_at_hl:.6}");
        failed += 1;
    }

    // Check 3: IV monotonically decreasing
    print!("\n--- Check 3: IV monotonically decreasing --- ");
    let c_iv: Vec<f64> = times
        .iter()
        .map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t))
        .collect();
    let mono_dec = c_iv.windows(2).all(|w| w[0] >= w[1] - 1e-15);
    if mono_dec {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 4: IV AUC
    print!("\n--- Check 4: IV AUC analytical --- ");
    let auc_iv_num = auc_trapezoidal(&times, &c_iv);
    let auc_iv_ana = DOSE_IV / (VD_IV * k_e_iv);
    let rel_err_iv = (auc_iv_num - auc_iv_ana).abs() / auc_iv_ana;
    if rel_err_iv < 0.01 {
        println!("[PASS] num={auc_iv_num:.2}, ana={auc_iv_ana:.2} (err={rel_err_iv:.4})");
        passed += 1;
    } else {
        println!("[FAIL] err={rel_err_iv:.4}");
        failed += 1;
    }

    // Check 5: Oral C(0) = 0
    print!("\n--- Check 5: Oral C(0) = 0 --- ");
    let c_oral_0 = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e_oral, 0.0);
    if c_oral_0.abs() < 1e-10 {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL] C(0) = {c_oral_0}");
        failed += 1;
    }

    // Check 6: Oral Cmax at Tmax > 0
    print!("\n--- Check 6: Oral Cmax at Tmax > 0 --- ");
    let c_oral: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e_oral, t))
        .collect();
    let (cmax, tmax) = find_cmax_tmax(&times, &c_oral);
    if cmax > 0.0 && tmax > 0.0 {
        println!("[PASS] Cmax={cmax:.4} at Tmax={tmax:.2} hr");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 7: Tmax analytical
    print!("\n--- Check 7: Tmax analytical --- ");
    let tmax_ana = oral_tmax(KA_ORAL, k_e_oral);
    if (tmax - tmax_ana).abs() < 0.1 {
        println!("[PASS] num={tmax:.3}, ana={tmax_ana:.3}");
        passed += 1;
    } else {
        println!("[FAIL] num={tmax:.3}, ana={tmax_ana:.3}");
        failed += 1;
    }

    // Check 8: Oral → 0 by 48hr
    print!("\n--- Check 8: Oral → 0 by 48hr --- ");
    let c_48 = *c_oral.last().unwrap();
    if c_48 < 0.01 {
        println!("[PASS] C(48hr) = {c_48:.6}");
        passed += 1;
    } else {
        println!("[FAIL] C(48hr) = {c_48:.6}");
        failed += 1;
    }

    // Check 9: Oral AUC > 0
    print!("\n--- Check 9: Oral AUC > 0 --- ");
    let auc_oral = auc_trapezoidal(&times, &c_oral);
    if auc_oral > 0.0 {
        println!("[PASS] AUC = {auc_oral:.2}");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 10: Oral AUC analytical
    print!("\n--- Check 10: Oral AUC analytical --- ");
    let auc_oral_ana = (F_ORAL * DOSE_ORAL) / (VD_ORAL * k_e_oral);
    let rel_err_oral = (auc_oral - auc_oral_ana).abs() / auc_oral_ana;
    if rel_err_oral < 0.01 {
        println!("[PASS] num={auc_oral:.2}, ana={auc_oral_ana:.2} (err={rel_err_oral:.4})");
        passed += 1;
    } else {
        println!("[FAIL] err={rel_err_oral:.4}");
        failed += 1;
    }

    // Check 11: Multiple doses accumulate
    print!("\n--- Check 11: Multiple IV doses accumulate --- ");
    let c_multi = pk_multiple_dose(|t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t), 8.0, 6, &times);
    let peak_after_first = times
        .iter()
        .zip(c_multi.iter())
        .filter_map(|(&t, &c)| if t >= 8.0 { Some(c) } else { None })
        .fold(f64::NEG_INFINITY, f64::max);
    if peak_after_first > c0_iv {
        println!("[PASS] multi={peak_after_first:.4} > C0={c0_iv:.4}");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 12: All non-negative
    print!("\n--- Check 12: All concentrations ≥ 0 --- ");
    let all_ok = c_iv.iter().all(|&c| c >= 0.0)
        && c_oral.iter().all(|&c| c >= 0.0)
        && c_multi.iter().all(|&c| c >= -1e-12);
    if all_ok {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    std::process::exit(i32::from(failed > 0));
}
