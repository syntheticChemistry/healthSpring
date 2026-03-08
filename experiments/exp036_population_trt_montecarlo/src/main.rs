#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
#![expect(
    clippy::similar_names,
    reason = "PK parameter families (ka/ke/vd) are domain-standard naming"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! healthSpring Exp036 — Population TRT Monte Carlo (Rust validation)
//!
//! Cross-validates against Python control. Uses a simple deterministic
//! population (evenly spaced percentiles) rather than RNG, to validate
//! the pure math without seed dependency.

use healthspring_barracuda::endocrine::{self, pop_trt, testosterone_cypionate as tc};
use healthspring_barracuda::pkpd;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "n ≤ 500 fits exactly in f64 mantissa"
    )]
    let denom = (n - 1) as f64;
    (0..n)
        .map(|i| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "i ≤ 500 fits exactly in f64 mantissa"
            )]
            let frac = (i as f64) / denom;
            start + frac * (end - start)
        })
        .collect()
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp036: Population TRT Monte Carlo (Rust)");
    println!("{}", "=".repeat(72));

    let n_patients: usize = 100;
    let times = linspace(0.0, 56.0, 500);

    // Deterministic "population": spread Vd, ka, ke across ±2σ
    let (mu_vd, sig_vd) = endocrine::lognormal_params(pop_trt::VD_TYPICAL, pop_trt::VD_CV);
    let (mu_ka, sig_ka) = endocrine::lognormal_params(pop_trt::KA_TYPICAL, pop_trt::KA_CV);
    let (mu_ke, sig_ke) = endocrine::lognormal_params(pop_trt::KE_TYPICAL, pop_trt::KE_CV);

    #[expect(
        clippy::cast_precision_loss,
        reason = "n_patients = 100 fits exactly in f64 mantissa"
    )]
    let patient_denom = (n_patients - 1) as f64;

    // Decouple parameter ordering to preserve population variability.
    // Vd: sorted ascending, ka: interleaved, ke: sorted descending.
    let vd_arr: Vec<f64> = (0..n_patients)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 100")]
            let z = -3.0 + 6.0 * (i as f64) / patient_denom;
            (mu_vd + sig_vd * z).exp()
        })
        .collect();

    let ka_arr: Vec<f64> = (0..n_patients)
        .map(|i| {
            let phase = if i % 2 == 0 {
                i / 2
            } else {
                n_patients - 1 - i / 2
            };
            #[expect(clippy::cast_precision_loss, reason = "phase < 100")]
            let z = -2.0 + 4.0 * (phase as f64) / patient_denom;
            (mu_ka + sig_ka * z).exp()
        })
        .collect();

    let ke_arr: Vec<f64> = (0..n_patients)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 100")]
            let z = 3.0 - 6.0 * (i as f64) / patient_denom;
            (mu_ke + sig_ke * z).exp()
        })
        .collect();

    // Age spread [40, 75]
    let ages: Vec<f64> = linspace(40.0, 75.0, n_patients);
    let t0_adj: Vec<f64> = ages
        .iter()
        .map(|&age| endocrine::age_adjusted_t0(pop_trt::T0_TYPICAL, age, pop_trt::DECLINE_RATE))
        .collect();

    let mut cmax_arr = vec![0.0_f64; n_patients];
    let mut auc_arr = vec![0.0_f64; n_patients];
    let mut trough_arr = vec![0.0_f64; n_patients];

    for i in 0..n_patients {
        let concs = pkpd::pk_multiple_dose(
            |t| {
                endocrine::pk_im_depot(
                    tc::DOSE_WEEKLY_MG,
                    tc::F_IM,
                    vd_arr[i],
                    ka_arr[i],
                    ke_arr[i],
                    t,
                )
            },
            tc::INTERVAL_WEEKLY,
            8,
            &times,
        );
        let (cmax, _) = pkpd::find_cmax_tmax(&times, &concs);
        cmax_arr[i] = cmax;
        auc_arr[i] = pkpd::auc_trapezoidal(&times, &concs);

        let last_start = 7.0 * tc::INTERVAL_WEEKLY;
        trough_arr[i] = times
            .iter()
            .zip(concs.iter())
            .filter(|(t, _)| **t >= last_start)
            .map(|(_, c)| *c)
            .fold(f64::INFINITY, f64::min);
    }

    // --- Check 1: All PK params positive ---
    println!("\n--- Check 1: All PK params positive ---");
    if vd_arr.iter().all(|&v| v > 0.0)
        && ka_arr.iter().all(|&v| v > 0.0)
        && ke_arr.iter().all(|&v| v > 0.0)
    {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 2: Vd median near typical ---
    println!("\n--- Check 2: Vd median near typical ---");
    let mut vd_sorted = vd_arr.clone();
    vd_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let vd_med = vd_sorted[n_patients / 2];
    let vd_relative_err = (vd_med - pop_trt::VD_TYPICAL).abs() / pop_trt::VD_TYPICAL;
    if vd_relative_err < 0.05 {
        println!("  [PASS] Vd median={vd_med:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] err={vd_relative_err:.3}");
        failed += 1;
    }

    // --- Check 3: All Cmax > 0 ---
    println!("\n--- Check 3: All Cmax > 0 ---");
    if cmax_arr.iter().all(|&c| c > 0.0) {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 4: All AUC > 0 ---
    println!("\n--- Check 4: All AUC > 0 ---");
    if auc_arr.iter().all(|&a| a > 0.0) {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 5: AUC has variability ---
    println!("\n--- Check 5: AUC variability ---");
    #[expect(clippy::cast_precision_loss, reason = "n_patients = 100")]
    let n_patients_f64 = n_patients as f64;
    let auc_mean: f64 = auc_arr.iter().sum::<f64>() / n_patients_f64;
    let auc_var: f64 =
        auc_arr.iter().map(|&a| (a - auc_mean).powi(2)).sum::<f64>() / n_patients_f64;
    let auc_cv = auc_var.sqrt() / auc_mean;
    if auc_cv > 0.15 {
        println!("  [PASS] CV={auc_cv:.3}");
        passed += 1;
    } else {
        println!("  [FAIL] CV={auc_cv:.3}");
        failed += 1;
    }

    // --- Check 6: Analytical AUC = F*D/(Vd*ke) relationship ---
    println!("\n--- Check 6: Analytical AUC inversely proportional to ke*Vd ---");
    let ke_vd_products: Vec<f64> = ke_arr
        .iter()
        .zip(vd_arr.iter())
        .map(|(&k, &v)| k * v)
        .collect();
    // Patients with higher ke*Vd should have lower AUC (analytical: AUC ∝ 1/(ke*Vd))
    let mid = n_patients / 2;
    let mut paired: Vec<(f64, f64)> = ke_vd_products
        .iter()
        .zip(auc_arr.iter())
        .map(|(&kv, &a)| (kv, a))
        .collect();
    paired.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    #[expect(clippy::cast_precision_loss, reason = "mid = 50")]
    let mid_f64 = mid as f64;
    let low_kv_auc: f64 = paired[..mid].iter().map(|(_, a)| a).sum::<f64>() / mid_f64;
    #[expect(clippy::cast_precision_loss, reason = "n_patients - mid = 50")]
    let high_denom = (n_patients - mid) as f64;
    let high_kv_auc: f64 = paired[mid..].iter().map(|(_, a)| a).sum::<f64>() / high_denom;
    if low_kv_auc > high_kv_auc {
        println!("  [PASS] low ke·Vd AUC={low_kv_auc:.2} > high ke·Vd={high_kv_auc:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] low={low_kv_auc:.2}, high={high_kv_auc:.2}");
        failed += 1;
    }

    // --- Check 7: Age-adjusted T0 declines ---
    println!("\n--- Check 7: Age-adjusted T0 declines ---");
    if t0_adj[0] > t0_adj[n_patients - 1] {
        println!(
            "  [PASS] T0(40)={:.1} > T0(75)={:.1}",
            t0_adj[0],
            t0_adj[n_patients - 1]
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 8: AUC percentiles ordered ---
    println!("\n--- Check 8: AUC percentiles ordered ---");
    let mut auc_sorted = auc_arr.clone();
    auc_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p5 = auc_sorted[n_patients / 20];
    let p50 = auc_sorted[n_patients / 2];
    let p95 = auc_sorted[19 * n_patients / 20];
    if p5 < p50 && p50 < p95 {
        println!("  [PASS] P5={p5:.2} < P50={p50:.2} < P95={p95:.2}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 9: Cohort size ---
    println!("\n--- Check 9: Cohort size ---");
    if cmax_arr.len() == n_patients {
        println!("  [PASS] {n_patients}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // --- Check 10: Trough levels positive ---
    println!("\n--- Check 10: Trough levels > 0 ---");
    if trough_arr.iter().all(|&t| t > 0.0) {
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
