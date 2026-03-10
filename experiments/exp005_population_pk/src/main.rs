// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp005 validation: Population PK Monte Carlo
//!
//! Cross-validates `healthspring_barracuda::pkpd::population_pk_cpu`
//! against the Python control (`exp005_population_pk.py`).

use healthspring_barracuda::pkpd::{self, pop_baricitinib};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp005 — Rust CPU Validation: Population PK");
    println!("{}", "=".repeat(72));

    // Validate lognormal parameter computation
    let cl_p = pop_baricitinib::CL;
    let (mu, sigma) = cl_p.to_normal_params();

    // Check 1: mu recovers typical as median (median of lognormal = exp(mu))
    println!("\n--- Check 1: Lognormal μ recovers typical ---");
    let recovered = mu.exp();
    let err = (recovered - 10.0).abs() / 10.0;
    if err < 0.05 {
        println!(
            "  [PASS] exp(μ) = {recovered:.4}, typical = 10.0, err = {:.4}%",
            err * 100.0
        );
        passed += 1;
    } else {
        println!("  [FAIL] err = {:.4}%", err * 100.0);
        failed += 1;
    }

    // Check 2: sigma > 0
    println!("\n--- Check 2: σ > 0 ---");
    if sigma > 0.0 {
        println!("  [PASS] σ = {sigma:.6}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 3: Vd lognormal params
    println!("\n--- Check 3: Vd lognormal params ---");
    let vd_p = pop_baricitinib::VD;
    let (mu_vd, sigma_vd) = vd_p.to_normal_params();
    let rec_vd = mu_vd.exp();
    if (rec_vd - 80.0).abs() / 80.0 < 0.05 && sigma_vd > 0.0 {
        println!("  [PASS] Vd: exp(μ) = {rec_vd:.2}, σ = {sigma_vd:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 4: Ka lognormal params
    println!("\n--- Check 4: Ka lognormal params ---");
    let ka_p = pop_baricitinib::KA;
    let (mu_ka, sigma_ka) = ka_p.to_normal_params();
    let rec_ka = mu_ka.exp();
    if (rec_ka - 1.5).abs() / 1.5 < 0.1 && sigma_ka > 0.0 {
        println!("  [PASS] Ka: exp(μ) = {rec_ka:.4}, σ = {sigma_ka:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Generate deterministic cohort for CPU validation
    let n: usize = 20;
    let times: Vec<f64> = (0..500).map(|i| 24.0 * f64::from(i) / 499.0).collect();

    // Deterministic PK params (not random — that's GPU's job)
    let cl_vals: Vec<f64> = (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 20")]
            let fi = i as f64;
            0.2f64.mul_add(fi, 8.0)
        })
        .collect();
    let vd_vals: Vec<f64> = (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 20")]
            let fi = i as f64;
            1.0f64.mul_add(fi, 70.0)
        })
        .collect();
    let ka_vals: Vec<f64> = (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 20")]
            let fi = i as f64;
            0.05f64.mul_add(fi, 1.0)
        })
        .collect();

    let results = pkpd::population_pk_cpu(
        n,
        &cl_vals,
        &vd_vals,
        &ka_vals,
        pop_baricitinib::DOSE_MG,
        pop_baricitinib::F_BIOAVAIL,
        &times,
    );

    // Check 5: Correct cohort size
    println!("\n--- Check 5: Correct cohort size ---");
    if results.len() == n {
        println!("  [PASS] {} patients", results.len());
        passed += 1;
    } else {
        println!("  [FAIL] {} patients", results.len());
        failed += 1;
    }

    // Check 6: All AUC > 0
    println!("\n--- Check 6: All AUC > 0 ---");
    if results.iter().all(|r| r.auc > 0.0) {
        let min_auc = results.iter().map(|r| r.auc).fold(f64::INFINITY, f64::min);
        println!("  [PASS] min AUC = {min_auc:.6}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 7: All Cmax > 0
    println!("\n--- Check 7: All Cmax > 0 ---");
    if results.iter().all(|r| r.cmax > 0.0) {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 8: All Tmax > 0 (oral)
    println!("\n--- Check 8: All Tmax > 0 ---");
    if results.iter().all(|r| r.tmax > 0.0) {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 9: Higher CL → lower AUC
    println!("\n--- Check 9: Higher CL → lower AUC ---");
    let first = &results[0];
    let last = &results[n - 1];
    if first.auc > last.auc {
        println!(
            "  [PASS] AUC(CL=8.0)={:.6} > AUC(CL=11.8)={:.6}",
            first.auc, last.auc
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 10: AUC ≈ F*Dose/CL for first patient (truncated at 24hr, ~10% tolerance)
    println!("\n--- Check 10: AUC ≈ F*D/CL ---");
    let theoretical = pop_baricitinib::F_BIOAVAIL * pop_baricitinib::DOSE_MG / cl_vals[0];
    let err = (first.auc - theoretical).abs() / theoretical;
    if err < 0.10 {
        println!(
            "  [PASS] AUC={:.6}, F*D/CL={theoretical:.6}, err={:.3}%",
            first.auc,
            err * 100.0
        );
        passed += 1;
    } else {
        println!("  [FAIL] err={:.3}%", err * 100.0);
        failed += 1;
    }

    // Check 11: C(0) = 0 for oral dosing
    println!("\n--- Check 11: C(0) = 0 ---");
    let c0 = pkpd::pk_oral_one_compartment(
        pop_baricitinib::DOSE_MG,
        pop_baricitinib::F_BIOAVAIL,
        vd_vals[0],
        ka_vals[0],
        cl_vals[0] / vd_vals[0],
        0.0,
    );
    if c0.abs() < 1e-12 {
        println!("  [PASS] C(0) = {c0:.2e}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 12: Tmax in reasonable range
    println!("\n--- Check 12: Tmax range ---");
    let tmax_range = results.iter().all(|r| r.tmax > 0.1 && r.tmax < 10.0);
    if tmax_range {
        let min_t = results.iter().map(|r| r.tmax).fold(f64::INFINITY, f64::min);
        let max_t = results.iter().map(|r| r.tmax).fold(0.0_f64, f64::max);
        println!("  [PASS] Tmax range: [{min_t:.2}, {max_t:.2}] hr");
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
