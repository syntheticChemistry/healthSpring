// SPDX-License-Identifier: AGPL-3.0-or-later
//! healthSpring Exp031 — Testosterone Pellet Depot PK (Rust validation)

use healthspring_barracuda::endocrine::{self, testosterone_cypionate as tc, pellet_params as pp};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp031: Testosterone Pellet Depot PK (Rust)");
    println!("{}", "=".repeat(72));

    let c_ss = pp::RELEASE_RATE / (tc::VD_L * tc::K_E);

    // --- Check 1: C(0) = 0 ---
    println!("\n--- Check 1: C(0) = 0 ---");
    let c0 = endocrine::pellet_concentration(0.0, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS);
    if c0.abs() < 1e-10 { println!("  [PASS] C(0) = {c0:.10}"); passed += 1; }
    else { println!("  [FAIL] C(0) = {c0}"); failed += 1; }

    // --- Check 2: Approaches steady-state by 5 half-lives ---
    println!("\n--- Check 2: Approaches steady-state ---");
    let c_5hl = endocrine::pellet_concentration(5.0 * tc::T_HALF_DAYS, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS);
    let ratio = c_5hl / c_ss;
    if ratio > 0.95 { println!("  [PASS] C(5t½)/C_ss = {ratio:.4}"); passed += 1; }
    else { println!("  [FAIL] ratio = {ratio:.4}"); failed += 1; }

    // --- Check 3: Stable plateau ---
    println!("\n--- Check 3: Stable plateau CV < 5% ---");
    let plateau: Vec<f64> = (0..800).filter_map(|i: i32| {
        let t = 60.0 + 80.0 * f64::from(i) / 799.0;
        if t <= 140.0 { Some(endocrine::pellet_concentration(t, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS)) }
        else { None }
    }).collect();
    let mean_p: f64 = plateau.iter().sum::<f64>() / plateau.len() as f64;
    let var_p: f64 = plateau.iter().map(|c| (c - mean_p).powi(2)).sum::<f64>() / plateau.len() as f64;
    let cv = var_p.sqrt() / mean_p;
    if cv < 0.05 { println!("  [PASS] Plateau CV = {cv:.4}"); passed += 1; }
    else { println!("  [FAIL] CV = {cv:.4}"); failed += 1; }

    // --- Check 4: Plateau > 0 ---
    println!("\n--- Check 4: Positive plateau ---");
    if mean_p > 0.0 { println!("  [PASS] Mean plateau = {mean_p:.4}"); passed += 1; }
    else { println!("  [FAIL]"); failed += 1; }

    // --- Check 5: Washout begins after duration ---
    println!("\n--- Check 5: Washout active ---");
    let c_end = endocrine::pellet_concentration(pp::DURATION_DAYS, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS);
    let c_post = endocrine::pellet_concentration(180.0, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS);
    if c_post < c_end * 0.50 { println!("  [PASS] C(end)={c_end:.4}, C(6mo)={c_post:.4}"); passed += 1; }
    else { println!("  [FAIL]"); failed += 1; }

    // --- Check 6: Washout t½ ≈ 8 days ---
    println!("\n--- Check 6: Washout t½ ---");
    let half_c = c_end / 2.0;
    let mut t_half_obs = None;
    for i in 0..1000_i32 {
        let t = pp::DURATION_DAYS + 30.0 * f64::from(i) / 999.0;
        let c = endocrine::pellet_concentration(t, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS);
        if c <= half_c { t_half_obs = Some(t - pp::DURATION_DAYS); break; }
    }
    if let Some(th) = t_half_obs {
        if (th - tc::T_HALF_DAYS).abs() / tc::T_HALF_DAYS < 0.15 { println!("  [PASS] t½ = {th:.2} days"); passed += 1; }
        else { println!("  [FAIL] t½ = {th:.2}"); failed += 1; }
    } else { println!("  [FAIL] no crossing"); failed += 1; }

    // --- Check 7: Pellet fluctuation < 10% ---
    println!("\n--- Check 7: Pellet fluctuation < 10% ---");
    let p_max = plateau.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let p_min = plateau.iter().copied().fold(f64::INFINITY, f64::min);
    let fluct = if p_min > 0.0 { (p_max - p_min) / p_min } else { f64::INFINITY };
    if fluct < 0.10 { println!("  [PASS] fluctuation = {fluct:.4}"); passed += 1; }
    else { println!("  [FAIL] fluctuation = {fluct:.4}"); failed += 1; }

    // --- Check 8: AUC proportional to dose ---
    println!("\n--- Check 8: AUC ≈ D/(Vd*ke) ---");
    let n = 3000usize;
    let dt = 180.0 / (n - 1) as f64;
    let mut auc = 0.0;
    let mut c_prev = 0.0_f64;
    for i in 0..n {
        let t = 180.0 * (i as f64) / ((n - 1) as f64);
        let c = endocrine::pellet_concentration(t, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS);
        if i > 0 { auc += 0.5 * (c_prev + c) * dt; }
        c_prev = c;
    }
    let auc_ana = pp::DOSE_MG / (tc::VD_L * tc::K_E);
    let rel = (auc - auc_ana).abs() / auc_ana;
    if rel < 0.10 { println!("  [PASS] AUC num={auc:.2}, ana={auc_ana:.2} (err={rel:.3})"); passed += 1; }
    else { println!("  [FAIL] err={rel:.4}"); failed += 1; }

    // --- Check 9: Dose-weight scaling ---
    println!("\n--- Check 9: Linear dose scaling ---");
    let dose_150 = 10.0 * 150.0;
    let rr_150 = dose_150 / pp::DURATION_DAYS;
    let c_150: Vec<f64> = (0..800).filter_map(|i: i32| {
        let t = 60.0 + 80.0 * f64::from(i) / 799.0;
        if t <= 140.0 { Some(endocrine::pellet_concentration(t, rr_150, tc::K_E, tc::VD_L, pp::DURATION_DAYS)) }
        else { None }
    }).collect();
    let mean_150 = c_150.iter().sum::<f64>() / c_150.len() as f64;
    let ratio_c = if mean_150 > 0.0 { mean_p / mean_150 } else { 0.0 };
    let ratio_d = pp::DOSE_MG / dose_150;
    if (ratio_c - ratio_d).abs() / ratio_d < 0.01 { println!("  [PASS] ratio={ratio_c:.4}, expected={ratio_d:.4}"); passed += 1; }
    else { println!("  [FAIL] ratio={ratio_c:.4}"); failed += 1; }

    // --- Check 10: All non-negative ---
    println!("\n--- Check 10: All non-negative ---");
    let all_nn = (0..3000_i32).all(|i| {
        let t = 180.0 * f64::from(i) / 2999.0;
        endocrine::pellet_concentration(t, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS) >= -1e-12
    });
    if all_nn { println!("  [PASS]"); passed += 1; }
    else { println!("  [FAIL]"); failed += 1; }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
