// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp005 validation: Population PK Monte Carlo
//!
//! Cross-validates `healthspring_barracuda::pkpd::population_pk_cpu`
//! against the Python control (`exp005_population_pk.py`).

use healthspring_barracuda::pkpd::{self, pop_baricitinib};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("Exp005 Population PK");

    // Validate lognormal parameter computation
    let cl_p = pop_baricitinib::CL;
    let (mu, sigma) = cl_p.to_normal_params();

    // Check 1: mu recovers typical as median (median of lognormal = exp(mu))
    let recovered = mu.exp();
    h.check_rel(
        "Lognormal μ recovers typical",
        recovered,
        10.0,
        tolerances::LOGNORMAL_RECOVERY,
    );

    // Check 2: sigma > 0
    h.check_bool("σ > 0", sigma > 0.0);

    // Check 3: Vd lognormal params
    let vd_p = pop_baricitinib::VD;
    let (mu_vd, sigma_vd) = vd_p.to_normal_params();
    let rec_vd = mu_vd.exp();
    h.check_bool(
        "Vd lognormal params",
        (rec_vd - 80.0).abs() / 80.0 < tolerances::POP_VD_MEDIAN && sigma_vd > 0.0,
    );

    // Check 4: Ka lognormal params
    let ka_p = pop_baricitinib::KA;
    let (mu_ka, sigma_ka) = ka_p.to_normal_params();
    let rec_ka = mu_ka.exp();
    h.check_bool(
        "Ka lognormal params",
        (rec_ka - 1.5).abs() / 1.5 < tolerances::POP_KA_MEDIAN && sigma_ka > 0.0,
    );

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
    h.check_exact("Correct cohort size", results.len() as u64, n as u64);

    // Check 6: All AUC > 0
    h.check_bool("All AUC > 0", results.iter().all(|r| r.auc > 0.0));

    // Check 7: All Cmax > 0
    h.check_bool("All Cmax > 0", results.iter().all(|r| r.cmax > 0.0));

    // Check 8: All Tmax > 0 (oral)
    h.check_bool("All Tmax > 0", results.iter().all(|r| r.tmax > 0.0));

    // Check 9: Higher CL → lower AUC
    let first = &results[0];
    let last = &results[n - 1];
    h.check_bool("Higher CL → lower AUC", first.auc > last.auc);

    // Check 10: AUC ≈ F*Dose/CL for first patient (truncated at 24hr, ~10% tolerance)
    let theoretical = pop_baricitinib::F_BIOAVAIL * pop_baricitinib::DOSE_MG / cl_vals[0];
    h.check_rel(
        "AUC ≈ F*D/CL",
        first.auc,
        theoretical,
        tolerances::AUC_TRUNCATED,
    );

    // Check 11: C(0) = 0 for oral dosing
    let c0 = pkpd::pk_oral_one_compartment(
        pop_baricitinib::DOSE_MG,
        pop_baricitinib::F_BIOAVAIL,
        vd_vals[0],
        ka_vals[0],
        cl_vals[0] / vd_vals[0],
        0.0,
    );
    h.check_bool("C(0) = 0", c0.abs() < tolerances::MACHINE_EPSILON_TIGHT);

    // Check 12: Tmax in reasonable range
    let tmax_range = results.iter().all(|r| r.tmax > 0.1 && r.tmax < 10.0);
    h.check_bool("Tmax range", tmax_range);

    h.exit();
}
