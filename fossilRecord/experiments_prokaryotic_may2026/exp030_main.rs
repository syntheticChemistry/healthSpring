// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp030 — Testosterone IM Injection PK (Rust validation)
//!
//! Cross-validates against Python control `exp030_testosterone_im_pk.py`.

use healthspring_barracuda::endocrine::{self, ImRegimen, testosterone_cypionate as tc};
use healthspring_barracuda::pkpd;
use healthspring_barracuda::tolerances::{
    ACCUMULATION_FACTOR, AUC_TRAPEZOIDAL, MACHINE_EPSILON, MACHINE_EPSILON_TIGHT, WASHOUT_HALF_LIFE,
};
use healthspring_barracuda::validation::ValidationHarness;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "n ≤ 2000 fits exactly in f64 mantissa"
    )]
    let denom = (n - 1) as f64;
    (0..n)
        .map(|i| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "i ≤ 2000 fits exactly in f64 mantissa"
            )]
            let frac = (i as f64) / denom;
            start + frac * (end - start)
        })
        .collect()
}

fn main() {
    let mut h = ValidationHarness::new("exp030 Testosterone IM PK");
    let times = linspace(0.0, 56.0, 2000);

    // --- Check 1: C(0) = 0 ---
    let c0 = endocrine::pk_im_depot(
        tc::DOSE_WEEKLY_MG,
        tc::F_IM,
        tc::VD_L,
        tc::K_A_IM,
        tc::K_E,
        0.0,
    );
    h.check_abs("C(0) = 0", c0, 0.0, MACHINE_EPSILON);

    // --- Check 2: Cmax > 0, Tmax > 0 ---
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| {
            endocrine::pk_im_depot(
                tc::DOSE_WEEKLY_MG,
                tc::F_IM,
                tc::VD_L,
                tc::K_A_IM,
                tc::K_E,
                t,
            )
        })
        .collect();
    let (cmax, tmax) = pkpd::find_cmax_tmax(&times, &concs);
    h.check_bool("Cmax > 0, Tmax > 0", cmax > 0.0 && tmax > 0.0);

    // --- Check 3: Tmax in range ---
    h.check_lower("Tmax >= 0.5 days", tmax, 0.5);
    h.check_upper("Tmax <= 5 days", tmax, 5.0);

    // --- Check 4: Decay below 15% Cmax by 4 half-lives ---
    println!("\n--- Check 4: Decay below 15% Cmax by 4 half-lives ---");
    let t_4hl = 4.0 * tc::T_HALF_DAYS;
    let c_4hl = endocrine::pk_im_depot(
        tc::DOSE_WEEKLY_MG,
        tc::F_IM,
        tc::VD_L,
        tc::K_A_IM,
        tc::K_E,
        t_4hl,
    );
    h.check_upper("C(4t½) < 15% Cmax", c_4hl, WASHOUT_HALF_LIFE * cmax);

    // --- Check 5: All non-negative ---
    h.check_bool(
        "All concentrations >= 0",
        concs.iter().all(|&c| c >= -MACHINE_EPSILON_TIGHT),
    );

    // --- Check 6: Weekly dosing accumulates ---
    println!("\n--- Check 6: Weekly dosing accumulates ---");
    let c_weekly = pkpd::pk_multiple_dose(
        |t| {
            endocrine::pk_im_depot(
                tc::DOSE_WEEKLY_MG,
                tc::F_IM,
                tc::VD_L,
                tc::K_A_IM,
                tc::K_E,
                t,
            )
        },
        tc::INTERVAL_WEEKLY,
        8,
        &times,
    );
    let threshold_t = 5.0 * tc::INTERVAL_WEEKLY;
    let cmax_ss = times
        .iter()
        .zip(c_weekly.iter())
        .filter(|(t, _)| **t >= threshold_t)
        .map(|(_, c)| *c)
        .fold(f64::NEG_INFINITY, f64::max);
    h.check_bool("SS Cmax > single Cmax", cmax_ss > cmax);

    // --- Check 7: Biweekly has larger fluctuation ---
    println!("\n--- Check 7: Biweekly larger fluctuation ---");
    let c_biweekly = pkpd::pk_multiple_dose(
        |t| {
            endocrine::pk_im_depot(
                tc::DOSE_BIWEEKLY_MG,
                tc::F_IM,
                tc::VD_L,
                tc::K_A_IM,
                tc::K_E,
                t,
            )
        },
        tc::INTERVAL_BIWEEKLY,
        4,
        &times,
    );
    let bw_regimen = ImRegimen {
        dose_mg: tc::DOSE_BIWEEKLY_MG,
        f: tc::F_IM,
        vd: tc::VD_L,
        ka: tc::K_A_IM,
        ke: tc::K_E,
        interval: tc::INTERVAL_BIWEEKLY,
        n_doses: 4,
    };
    let wk_regimen = ImRegimen {
        dose_mg: tc::DOSE_WEEKLY_MG,
        f: tc::F_IM,
        vd: tc::VD_L,
        ka: tc::K_A_IM,
        ke: tc::K_E,
        interval: tc::INTERVAL_WEEKLY,
        n_doses: 8,
    };
    let (cmax_bw, trough_bw) = endocrine::im_steady_state_metrics(&bw_regimen, &times);
    let (cmax_wk, trough_wk) = endocrine::im_steady_state_metrics(&wk_regimen, &times);
    let fluct_bw = if trough_bw > 0.0 {
        (cmax_bw - trough_bw) / trough_bw
    } else {
        f64::INFINITY
    };
    let fluct_wk = if trough_wk > 0.0 {
        (cmax_wk - trough_wk) / trough_wk
    } else {
        f64::INFINITY
    };
    h.check_bool("BW fluctuation > WK fluctuation", fluct_bw > fluct_wk);

    // --- Check 8: Same analytical AUC per 14 days ---
    println!("\n--- Check 8: Equal analytical AUC per 14 days ---");
    let auc_weekly = 2.0 * (tc::F_IM * tc::DOSE_WEEKLY_MG) / (tc::VD_L * tc::K_E);
    let auc_biweekly = (tc::F_IM * tc::DOSE_BIWEEKLY_MG) / (tc::VD_L * tc::K_E);
    let rel = (auc_weekly - auc_biweekly).abs() / auc_weekly.max(auc_biweekly);
    h.check_upper("AUC/14d relative diff", rel, AUC_TRAPEZOIDAL);

    // --- Check 9: Accumulation factor ---
    println!("\n--- Check 9: Accumulation factor ---");
    let r_ana = 1.0 / (1.0 - (-tc::K_E * tc::INTERVAL_WEEKLY).exp());
    let r_obs = if cmax > 0.0 { cmax_ss / cmax } else { 0.0 };
    h.check_rel("Accumulation factor", r_obs, r_ana, ACCUMULATION_FACTOR);

    // --- Check 10: Weekly trough > biweekly trough ---
    println!("\n--- Check 10: Weekly trough > biweekly trough ---");
    h.check_bool("Weekly trough > biweekly trough", trough_wk > trough_bw);

    // --- Check 11: Multi-dose non-negative ---
    println!("\n--- Check 11: Multi-dose non-negative ---");
    let weekly_non_neg = c_weekly.iter().all(|&c| c >= -MACHINE_EPSILON_TIGHT);
    let biweekly_non_neg = c_biweekly.iter().all(|&c| c >= -MACHINE_EPSILON_TIGHT);
    h.check_bool(
        "Multi-dose non-negative",
        weekly_non_neg && biweekly_non_neg,
    );

    h.exit();
}
