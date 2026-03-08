#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! healthSpring Exp030 — Testosterone IM Injection PK (Rust validation)
//!
//! Cross-validates against Python control `exp030_testosterone_im_pk.py`.

use healthspring_barracuda::endocrine::{self, ImRegimen, testosterone_cypionate as tc};
use healthspring_barracuda::pkpd;

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
    let mut passed = 0u32;
    let mut failed = 0u32;
    let times = linspace(0.0, 56.0, 2000);

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp030: Testosterone IM Injection PK (Rust)");
    println!("{}", "=".repeat(72));

    // --- Check 1: C(0) = 0 ---
    println!("\n--- Check 1: Single dose C(0) = 0 ---");
    let c0 = endocrine::pk_im_depot(
        tc::DOSE_WEEKLY_MG,
        tc::F_IM,
        tc::VD_L,
        tc::K_A_IM,
        tc::K_E,
        0.0,
    );
    if c0.abs() < 1e-10 {
        println!("  [PASS] C(0) = {c0:.10}");
        passed += 1;
    } else {
        println!("  [FAIL] C(0) = {c0:.10}");
        failed += 1;
    }

    // --- Check 2: Cmax > 0, Tmax > 0 ---
    println!("\n--- Check 2: Single dose Cmax > 0, Tmax > 0 ---");
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
    if cmax > 0.0 && tmax > 0.0 {
        println!("  [PASS] Cmax={cmax:.4}, Tmax={tmax:.2} days");
        passed += 1;
    } else {
        println!("  [FAIL] Cmax={cmax}, Tmax={tmax}");
        failed += 1;
    }

    // --- Check 3: Tmax in range ---
    println!("\n--- Check 3: Tmax in [0.5, 5] days ---");
    if (0.5..=5.0).contains(&tmax) {
        println!("  [PASS] Tmax = {tmax:.2} days");
        passed += 1;
    } else {
        println!("  [FAIL] Tmax = {tmax:.2}");
        failed += 1;
    }

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
    if c_4hl < 0.15 * cmax {
        println!("  [PASS] C(4t½) = {c_4hl:.4} < {:.4}", 0.15 * cmax);
        passed += 1;
    } else {
        println!("  [FAIL] C(4t½) = {c_4hl:.4}");
        failed += 1;
    }

    // --- Check 5: All non-negative ---
    println!("\n--- Check 5: All concentrations >= 0 ---");
    if concs.iter().all(|&c| c >= -1e-12) {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

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
    if cmax_ss > cmax {
        println!("  [PASS] SS Cmax={cmax_ss:.4} > single {cmax:.4}");
        passed += 1;
    } else {
        println!("  [FAIL] SS Cmax={cmax_ss:.4}");
        failed += 1;
    }

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
    if fluct_bw > fluct_wk {
        println!("  [PASS] BW fluct={fluct_bw:.2} > WK={fluct_wk:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] BW={fluct_bw:.2}, WK={fluct_wk:.2}");
        failed += 1;
    }

    // --- Check 8: Same analytical AUC per 14 days ---
    println!("\n--- Check 8: Equal analytical AUC per 14 days ---");
    let auc_weekly = 2.0 * (tc::F_IM * tc::DOSE_WEEKLY_MG) / (tc::VD_L * tc::K_E);
    let auc_biweekly = (tc::F_IM * tc::DOSE_BIWEEKLY_MG) / (tc::VD_L * tc::K_E);
    let rel = (auc_weekly - auc_biweekly).abs() / auc_weekly.max(auc_biweekly);
    if rel < 0.01 {
        println!("  [PASS] AUC/14d: WK={auc_weekly:.2}, BW={auc_biweekly:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] rel={rel:.4}");
        failed += 1;
    }

    // --- Check 9: Accumulation factor ---
    println!("\n--- Check 9: Accumulation factor ---");
    let r_ana = 1.0 / (1.0 - (-tc::K_E * tc::INTERVAL_WEEKLY).exp());
    let r_obs = if cmax > 0.0 { cmax_ss / cmax } else { 0.0 };
    if (r_obs - r_ana).abs() / r_ana < 0.25 {
        println!("  [PASS] R_obs={r_obs:.3}, R_ana={r_ana:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] R_obs={r_obs:.3}, R_ana={r_ana:.3}");
        failed += 1;
    }

    // --- Check 10: Weekly trough > biweekly trough ---
    println!("\n--- Check 10: Weekly trough > biweekly trough ---");
    if trough_wk > trough_bw {
        println!("  [PASS] WK trough={trough_wk:.4} > BW={trough_bw:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 11: Multi-dose non-negative ---
    println!("\n--- Check 11: Multi-dose non-negative ---");
    let weekly_non_neg = c_weekly.iter().all(|&c| c >= -1e-12);
    let biweekly_non_neg = c_biweekly.iter().all(|&c| c >= -1e-12);
    if weekly_non_neg && biweekly_non_neg {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
