// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
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
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::{OrExit, ValidationHarness};

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
    let mut h = ValidationHarness::new("Exp002 One-Compartment PK");
    let times = linspace(0.0, 48.0, 1000);
    let k_e_iv = core::f64::consts::LN_2 / HL_IV;
    let k_e_oral = core::f64::consts::LN_2 / HL_ORAL;
    let c0_iv = DOSE_IV / VD_IV;

    // Check 1: IV C(0)
    let c_at_0 = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, 0.0);
    h.check_abs(
        "IV C(0) = Dose/Vd",
        c_at_0,
        c0_iv,
        tolerances::MACHINE_EPSILON,
    );

    // Check 2: IV at half-life
    let c_at_hl = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, HL_IV);
    h.check_abs(
        "IV at half-life → C0/2",
        c_at_hl,
        c0_iv / 2.0,
        tolerances::HALF_LIFE_POINT,
    );

    // Check 3: IV monotonically decreasing
    let c_iv: Vec<f64> = times
        .iter()
        .map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t))
        .collect();
    let mono_dec = c_iv
        .windows(2)
        .all(|w| w[0] >= w[1] - tolerances::MACHINE_EPSILON_STRICT);
    h.check_bool("IV monotonically decreasing", mono_dec);

    // Check 4: IV AUC
    let auc_iv_num = auc_trapezoidal(&times, &c_iv);
    let auc_iv_ana = DOSE_IV / (VD_IV * k_e_iv);
    h.check_rel(
        "IV AUC analytical",
        auc_iv_num,
        auc_iv_ana,
        tolerances::AUC_TRAPEZOIDAL,
    );

    // Check 5: Oral C(0) = 0
    let c_oral_0 = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e_oral, 0.0);
    h.check_abs(
        "Oral C(0) = 0",
        c_oral_0.abs(),
        0.0,
        tolerances::MACHINE_EPSILON,
    );

    // Check 6: Oral Cmax at Tmax > 0
    let c_oral: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e_oral, t))
        .collect();
    let (cmax, tmax) = find_cmax_tmax(&times, &c_oral);
    h.check_bool("Oral Cmax at Tmax > 0", cmax > 0.0 && tmax > 0.0);

    // Check 7: Tmax analytical
    let tmax_ana = oral_tmax(KA_ORAL, k_e_oral);
    h.check_abs(
        "Tmax analytical",
        tmax,
        tmax_ana,
        tolerances::TMAX_NUMERICAL,
    );

    // Check 8: Oral → 0 by 48hr
    let c_48 = c_oral
        .last()
        .copied()
        .or_exit("oral curve has at least one point");
    h.check_upper("Oral → 0 by 48hr", c_48, tolerances::EXPONENTIAL_RESIDUAL);

    // Check 9: Oral AUC > 0
    let auc_oral = auc_trapezoidal(&times, &c_oral);
    h.check_bool("Oral AUC > 0", auc_oral > 0.0);

    // Check 10: Oral AUC analytical
    let auc_oral_ana = (F_ORAL * DOSE_ORAL) / (VD_ORAL * k_e_oral);
    h.check_rel(
        "Oral AUC analytical",
        auc_oral,
        auc_oral_ana,
        tolerances::AUC_TRAPEZOIDAL,
    );

    // Check 11: Multiple doses accumulate
    let c_multi = pk_multiple_dose(|t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t), 8.0, 6, &times);
    let peak_after_first = times
        .iter()
        .zip(c_multi.iter())
        .filter_map(|(&t, &c)| if t >= 8.0 { Some(c) } else { None })
        .fold(f64::NEG_INFINITY, f64::max);
    h.check_bool("Multiple IV doses accumulate", peak_after_first > c0_iv);

    // Check 12: All non-negative
    let all_ok = c_iv.iter().all(|&c| c >= 0.0)
        && c_oral.iter().all(|&c| c >= 0.0)
        && c_multi
            .iter()
            .all(|&c| c >= -tolerances::MACHINE_EPSILON_TIGHT);
    h.check_bool("All concentrations ≥ 0", all_ok);

    h.exit();
}
