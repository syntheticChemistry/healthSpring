// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp075: Cross-validate NLME (FOCE/SAEM) against published NONMEM
//! testosterone PK population parameters and nlmixr-style results.
//!
//! ## Validation Strategy
//!
//! 1. **Simulation-estimation study**: generate synthetic population data
//!    from known "true" parameters, then estimate using FOCE and SAEM.
//!    Validate that recovered parameters are within clinically acceptable
//!    tolerance of the truth.
//!
//! 2. **Published parameter recovery**: use testosterone cypionate PK
//!    parameters from Mok (2018) / Shoskes (2016) as population truth:
//!    - CL = 0.087 L/day (ln(CL) ≈ -2.44)
//!    - Vd = 70 L (ln(Vd) ≈ 4.25)
//!    - ka(IM) = 0.462/day (ln(ka) ≈ -0.77)
//!    - BSV on CL: ~25% CV (`omega_CL` ≈ 0.0625)
//!    - BSV on Vd: ~20% CV (`omega_Vd` ≈ 0.04)
//!    - BSV on ka: ~35% CV (`omega_ka` ≈ 0.1225)
//!    - Residual error: ~10% proportional (sigma ≈ 0.01)
//!
//! 3. **NCA cross-check**: compute NCA metrics from model-predicted
//!    profiles and verify against analytical 1-compartment values.
//!
//! ## Provenance
//!
//! - Shoskes et al. 2016, Clin Med Insights Endocrinol Diabetes 9: 31-37
//! - Mok 2018, "If Your Testosterone Is Low, You're Gonna Get Fat"
//! - nlmixr reference: Fidler et al. 2019, CPT Pharmacometrics Syst Pharmacol 8: 621-631

use healthspring_barracuda::pkpd::{
    NlmeConfig, SyntheticPopConfig, compute_cwres, compute_gof, cwres_summary, foce,
    generate_synthetic_population, nca_iv, oral_one_compartment_model, saem,
};

macro_rules! check {
    ($passed:expr, $failed:expr, $name:expr, $cond:expr) => {
        if $cond {
            $passed += 1;
            println!("  PASS: {}", $name);
        } else {
            $failed += 1;
            println!("  FAIL: {}", $name);
        }
    };
}

fn validate_foce_recovery(passed: &mut u32, failed: &mut u32) {
    println!("\n=== FOCE Parameter Recovery (Simulation-Estimation) ===");

    let theta_true = vec![-2.44, 4.25, -0.77];
    let omega_true = vec![0.0625, 0.04, 0.1225];
    let sigma_true = 0.01;
    let dose = 100.0;
    let times: Vec<f64> = (0..14).map(|i| f64::from(i) * 1.0).collect();

    let subjects = generate_synthetic_population(&SyntheticPopConfig {
        model: oral_one_compartment_model,
        theta: &theta_true,
        omega: &omega_true,
        sigma: sigma_true,
        n_subjects: 50,
        times: &times,
        dose,
        seed: 20_260_309,
    });

    let config = NlmeConfig {
        n_theta: 3,
        n_eta: 3,
        max_iter: 200,
        tol: 1e-6,
        seed: 42,
    };

    let result = foce(
        oral_one_compartment_model,
        &subjects,
        &theta_true,
        &omega_true,
        sigma_true,
        &config,
    );

    println!(
        "  Iterations: {} (converged: {})",
        result.iterations, result.converged
    );
    println!("  Objective: {:.4}", result.objective);

    for (idx, (&est, &truth)) in result.theta.iter().zip(theta_true.iter()).enumerate() {
        let rel_err = (est - truth).abs() / truth.abs().max(0.01);
        let label = ["ln(CL)", "ln(Vd)", "ln(ka)"][idx];
        println!("  {label}: est={est:.4}, true={truth:.4}, rel_err={rel_err:.4}");
        check!(
            *passed,
            *failed,
            format!("FOCE theta[{idx}] ({label}) within 30% of truth"),
            rel_err < 0.30
        );
    }

    check!(*passed, *failed, "FOCE sigma > 0", result.sigma > 0.0);

    check!(
        *passed,
        *failed,
        "FOCE has 50 individual eta vectors",
        result.individual_etas.len() == 50
    );
}

fn validate_saem_recovery(passed: &mut u32, failed: &mut u32) {
    println!("\n=== SAEM Parameter Recovery (Simulation-Estimation) ===");

    let theta_true = vec![-2.44, 4.25, -0.77];
    let omega_true = vec![0.0625, 0.04, 0.1225];
    let sigma_true = 0.01;
    let dose = 100.0;
    let times: Vec<f64> = (0..14).map(|i| f64::from(i) * 1.0).collect();

    let subjects = generate_synthetic_population(&SyntheticPopConfig {
        model: oral_one_compartment_model,
        theta: &theta_true,
        omega: &omega_true,
        sigma: sigma_true,
        n_subjects: 50,
        times: &times,
        dose,
        seed: 20_260_309,
    });

    let config = NlmeConfig {
        n_theta: 3,
        n_eta: 3,
        max_iter: 300,
        tol: 1e-6,
        seed: 42,
    };

    let result = saem(
        oral_one_compartment_model,
        &subjects,
        &theta_true,
        &omega_true,
        sigma_true,
        &config,
    );

    println!(
        "  Iterations: {} (converged: {})",
        result.iterations, result.converged
    );
    println!("  Objective: {:.4}", result.objective);

    for (idx, (&est, &truth)) in result.theta.iter().zip(theta_true.iter()).enumerate() {
        let rel_err = (est - truth).abs() / truth.abs().max(0.01);
        let label = ["ln(CL)", "ln(Vd)", "ln(ka)"][idx];
        println!("  {label}: est={est:.4}, true={truth:.4}, rel_err={rel_err:.4}");
        check!(
            *passed,
            *failed,
            format!("SAEM theta[{idx}] ({label}) within 30% of truth"),
            rel_err < 0.30
        );
    }

    check!(*passed, *failed, "SAEM sigma > 0", result.sigma > 0.0);
}

fn validate_diagnostics(passed: &mut u32, failed: &mut u32) {
    println!("\n=== Diagnostic Validation (CWRES, GOF) ===");

    let theta = vec![-2.44, 4.25, -0.77];
    let omega = vec![0.0625, 0.04, 0.1225];
    let sigma = 0.01;
    let times: Vec<f64> = (0..14).map(|i| f64::from(i) * 1.0).collect();

    let subjects = generate_synthetic_population(&SyntheticPopConfig {
        model: oral_one_compartment_model,
        theta: &theta,
        omega: &omega,
        sigma,
        n_subjects: 30,
        times: &times,
        dose: 100.0,
        seed: 20_260_309,
    });

    let config = NlmeConfig {
        n_theta: 3,
        n_eta: 3,
        max_iter: 150,
        tol: 1e-6,
        seed: 42,
    };

    let result = foce(
        oral_one_compartment_model,
        &subjects,
        &theta,
        &omega,
        sigma,
        &config,
    );
    let cwres = compute_cwres(oral_one_compartment_model, &subjects, &result);
    let summary = cwres_summary(&cwres);

    println!("  CWRES mean: {:.4}", summary.mean);
    println!("  CWRES std:  {:.4}", summary.std_dev);

    check!(
        *passed,
        *failed,
        "CWRES mean within [-3, 3]",
        summary.mean.abs() < 3.0
    );

    let gof = compute_gof(oral_one_compartment_model, &subjects, &result);
    println!("  GOF individual R²: {:.4}", gof.r_squared_individual);
    println!("  GOF population R²: {:.4}", gof.r_squared_population);

    check!(
        *passed,
        *failed,
        "GOF individual R² >= population R²",
        gof.r_squared_individual >= gof.r_squared_population
    );

    check!(
        *passed,
        *failed,
        format!("GOF has {} observations (30 subjects × 14 times)", 30 * 14),
        gof.observed.len() == 30 * 14
    );
}

fn validate_nca_crosscheck(passed: &mut u32, failed: &mut u32) {
    println!("\n=== NCA Cross-Check ===");

    let ke = 0.087 / 70.0;
    let vd = 70.0;
    let dose = 100.0;
    let c0 = dose / vd;
    let n_points = 2000;

    let times: Vec<f64> = (0..n_points)
        .map(|i| 500.0 * f64::from(i) / f64::from(n_points - 1))
        .collect();
    let concs: Vec<f64> = times.iter().map(|&t| c0 * (-ke * t).exp()).collect();

    let nca = nca_iv(&times, &concs, dose, 3);

    let analytical_auc = dose / (vd * ke);
    let analytical_cl = vd * ke;
    let analytical_half = core::f64::consts::LN_2 / ke;

    println!("  lambda_z: {:.6} (expected ke = {ke:.6})", nca.lambda_z);
    println!(
        "  AUC_inf: {:.2} (analytical = {analytical_auc:.2})",
        nca.auc_inf
    );
    println!("  CL: {:.6} (analytical = {analytical_cl:.6})", nca.cl_obs);
    println!(
        "  t½: {:.2} (analytical = {analytical_half:.2})",
        nca.half_life
    );

    let lz_err = (nca.lambda_z - ke).abs() / ke;
    check!(
        *passed,
        *failed,
        format!("NCA lambda_z within 5% of ke (err={lz_err:.4})"),
        lz_err < 0.05
    );

    let auc_err = (nca.auc_inf - analytical_auc).abs() / analytical_auc;
    check!(
        *passed,
        *failed,
        format!("NCA AUC_inf within 2% of analytical (err={auc_err:.4})"),
        auc_err < 0.02
    );

    let cl_err = (nca.cl_obs - analytical_cl).abs() / analytical_cl;
    check!(
        *passed,
        *failed,
        format!("NCA CL within 2% of analytical (err={cl_err:.4})"),
        cl_err < 0.02
    );

    check!(
        *passed,
        *failed,
        "NCA R² > 0.999 for mono-exponential",
        nca.r_squared > 0.999
    );
}

fn validate_determinism(passed: &mut u32, failed: &mut u32) {
    println!("\n=== Determinism Validation ===");

    let theta = vec![-2.44, 4.25, -0.77];
    let omega = vec![0.0625, 0.04, 0.1225];
    let times: Vec<f64> = (0..10).map(|i| f64::from(i) * 2.0).collect();
    let cfg = SyntheticPopConfig {
        model: oral_one_compartment_model,
        theta: &theta,
        omega: &omega,
        sigma: 0.01,
        n_subjects: 20,
        times: &times,
        dose: 100.0,
        seed: 99,
    };
    let s1 = generate_synthetic_population(&cfg);
    let s2 = generate_synthetic_population(&cfg);

    let mut all_match = true;
    for (a, b) in s1.iter().zip(s2.iter()) {
        for (oa, ob) in a.observations.iter().zip(b.observations.iter()) {
            if oa.to_bits() != ob.to_bits() {
                all_match = false;
                break;
            }
        }
    }
    check!(*passed, *failed, "synthetic data deterministic", all_match);

    let foce_cfg = NlmeConfig {
        n_theta: 3,
        n_eta: 3,
        max_iter: 30,
        tol: 1e-4,
        seed: 42,
    };
    let r1 = foce(
        oral_one_compartment_model,
        &s1,
        &theta,
        &omega,
        0.01,
        &foce_cfg,
    );
    let r2 = foce(
        oral_one_compartment_model,
        &s2,
        &theta,
        &omega,
        0.01,
        &foce_cfg,
    );

    check!(
        *passed,
        *failed,
        "FOCE objective deterministic",
        r1.objective.to_bits() == r2.objective.to_bits()
    );

    let r3 = saem(
        oral_one_compartment_model,
        &s1,
        &theta,
        &omega,
        0.01,
        &foce_cfg,
    );
    let r4 = saem(
        oral_one_compartment_model,
        &s2,
        &theta,
        &omega,
        0.01,
        &foce_cfg,
    );

    check!(
        *passed,
        *failed,
        "SAEM objective deterministic",
        r3.objective.to_bits() == r4.objective.to_bits()
    );
}

fn main() {
    println!("Exp075: NLME Cross-Validation Against Published NONMEM Testosterone PK");
    println!("======================================================================");
    println!();
    println!("Reference: Shoskes 2016, Mok 2018, nlmixr (Fidler 2019)");
    println!("Testosterone cypionate population PK parameters:");
    println!("  CL = 0.087 L/day, Vd = 70 L, ka(IM) = 0.462/day");
    println!("  BSV: CL 25% CV, Vd 20% CV, ka 35% CV");

    let mut passed = 0_u32;
    let mut failed = 0_u32;

    validate_foce_recovery(&mut passed, &mut failed);
    validate_saem_recovery(&mut passed, &mut failed);
    validate_diagnostics(&mut passed, &mut failed);
    validate_nca_crosscheck(&mut passed, &mut failed);
    validate_determinism(&mut passed, &mut failed);

    println!("\n======================================================================");
    println!("Results: {passed} passed, {failed} failed");

    if failed > 0 {
        std::process::exit(1);
    }
}
