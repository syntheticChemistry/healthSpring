// SPDX-License-Identifier: AGPL-3.0-or-later
//! Non-Linear Mixed-Effects (NLME) population PK — sovereign replacement for NONMEM/Monolix.
//!
//! Implements two estimation algorithms:
//! - **FOCE** (First-Order Conditional Estimation): NONMEM's workhorse, linearizes
//!   around conditional (individual) parameter estimates rather than population means.
//! - **SAEM** (Stochastic Approximation `EM`): Monolix's approach,
//!   alternates between stochastic E-step (MCMC) and deterministic M-step.
//!
//! Both algorithms estimate:
//! - Fixed effects (population typical values: theta)
//! - Random effects variance (between-subject variability: omega)
//! - Residual error variance (sigma)
//!
//! ## Structural model
//!
//! The user provides a structural model function `f(theta, eta, t) -> C` that
//! predicts concentration given fixed effects, individual random effects, and time.
//! The standard one-compartment oral model is provided as [`oral_one_compartment_model`].
//!
//! ## References
//!
//! - Beal & Sheiner, "NONMEM Users Guides" (1989-2024)
//! - Kuhn & Lavielle, "Maximum likelihood estimation in nonlinear mixed effects
//!   models," CSDA (2005)
//! - Gabrielsson & Weiner, "Pharmacokinetic and Pharmacodynamic Data Analysis," 5th ed.

mod foce_impl;
mod saem_impl;
mod solver;

pub use foce_impl::foce;
pub use saem_impl::saem;

use crate::rng::normal_sample;

/// Individual subject data for NLME estimation.
#[derive(Debug, Clone)]
pub struct Subject {
    /// Observation times (hours).
    pub times: Vec<f64>,
    /// Observed concentrations at each time.
    pub observations: Vec<f64>,
    /// Dose administered to this subject.
    pub dose: f64,
}

/// Configuration for NLME estimation.
#[derive(Debug, Clone)]
pub struct NlmeConfig {
    /// Number of fixed-effect parameters (theta).
    pub n_theta: usize,
    /// Number of random-effect parameters (eta) per subject.
    pub n_eta: usize,
    /// Maximum iterations for the algorithm.
    pub max_iter: usize,
    /// Convergence tolerance (relative change in objective function).
    pub tol: f64,
    /// PRNG seed for reproducibility.
    pub seed: u64,
}

impl Default for NlmeConfig {
    fn default() -> Self {
        Self {
            n_theta: 3,
            n_eta: 3,
            max_iter: 200,
            tol: 1e-6,
            seed: 42,
        }
    }
}

/// Result of NLME estimation.
#[derive(Debug, Clone)]
pub struct NlmeResult {
    /// Estimated population fixed effects.
    pub theta: Vec<f64>,
    /// Estimated between-subject variability (diagonal of omega matrix).
    pub omega_diag: Vec<f64>,
    /// Estimated residual error variance.
    pub sigma: f64,
    /// Final objective function value (−2 log-likelihood).
    pub objective: f64,
    /// Number of iterations to convergence.
    pub iterations: usize,
    /// Whether the algorithm converged.
    pub converged: bool,
    /// Per-subject eta estimates.
    pub individual_etas: Vec<Vec<f64>>,
}

/// Type alias for structural model function.
///
/// `model(theta, eta, dose, t) -> predicted_concentration`
pub type StructuralModel = fn(&[f64], &[f64], f64, f64) -> f64;

/// Standard one-compartment oral PK model for NLME.
///
/// Parameters (log-scale):
/// - `theta[0]` = ln(CL), `theta[1]` = ln(Vd), `theta[2]` = ln(ka)
/// - `eta[0..3]` = individual deviations on CL, Vd, ka
#[must_use]
pub fn oral_one_compartment_model(theta: &[f64], eta: &[f64], dose: f64, time: f64) -> f64 {
    let clearance = (theta[0] + eta[0]).exp();
    let volume = (theta[1] + eta[1]).exp();
    let absorption = (theta[2] + eta[2]).exp();
    let elimination = clearance / volume;

    if (absorption - elimination).abs() < 1e-12 || volume <= 0.0 {
        return 0.0;
    }

    let coeff = (dose * absorption) / (volume * (absorption - elimination));
    coeff * ((-elimination * time).exp() - (-absorption * time).exp())
}

/// Standard one-compartment IV bolus PK model for NLME.
///
/// Parameters (log-scale):
/// - `theta[0]` = ln(CL), `theta[1]` = ln(Vd)
/// - `eta[0..2]` = individual deviations on CL, Vd
#[must_use]
pub fn iv_one_compartment_model(theta: &[f64], eta: &[f64], dose: f64, time: f64) -> f64 {
    let clearance = (theta[0] + eta.first().copied().unwrap_or(0.0)).exp();
    let volume = (theta[1] + eta.get(1).copied().unwrap_or(0.0)).exp();
    let elimination = clearance / volume;
    (dose / volume) * (-elimination * time).exp()
}

/// Configuration for synthetic population generation.
pub struct SyntheticPopConfig<'a> {
    pub model: StructuralModel,
    pub theta: &'a [f64],
    pub omega: &'a [f64],
    pub sigma: f64,
    pub n_subjects: usize,
    pub times: &'a [f64],
    pub dose: f64,
    pub seed: u64,
}

/// Generate synthetic population PK data for testing/validation.
///
/// Creates `n_subjects` worth of concentration-time data from a known model
/// with specified population parameters and BSV.
#[must_use]
pub fn generate_synthetic_population(cfg: &SyntheticPopConfig<'_>) -> Vec<Subject> {
    let n_eta = cfg.omega.len();
    let mut rng = cfg.seed;
    let mut subjects = Vec::with_capacity(cfg.n_subjects);

    for _ in 0..cfg.n_subjects {
        let mut eta = vec![0.0; n_eta];
        for (ek, &ok) in eta.iter_mut().zip(cfg.omega.iter()) {
            let (z_val, new_st) = normal_sample(rng);
            rng = new_st;
            *ek = ok.sqrt() * z_val;
        }

        let mut observations = Vec::with_capacity(cfg.times.len());
        for &time in cfg.times {
            let pred = (cfg.model)(cfg.theta, &eta, cfg.dose, time);
            let (eps, new_st) = normal_sample(rng);
            rng = new_st;
            observations.push(cfg.sigma.sqrt().mul_add(eps, pred).max(0.0));
        }

        subjects.push(Subject {
            times: cfg.times.to_vec(),
            observations,
            dose: cfg.dose,
        });
    }

    subjects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    fn default_test_config() -> NlmeConfig {
        NlmeConfig {
            n_theta: 3,
            n_eta: 3,
            max_iter: 100,
            tol: 1e-4,
            seed: 12_345,
        }
    }

    fn generate_test_data() -> (Vec<Subject>, Vec<f64>, Vec<f64>, f64) {
        let theta_true = vec![2.3, 4.4, 0.4];
        let omega_true = vec![0.04, 0.04, 0.09];
        let sigma_true = 0.01;
        let times: Vec<f64> = (0..12).map(|i| f64::from(i) * 2.0).collect();
        let subjects = generate_synthetic_population(&SyntheticPopConfig {
            model: oral_one_compartment_model,
            theta: &theta_true,
            omega: &omega_true,
            sigma: sigma_true,
            n_subjects: 20,
            times: &times,
            dose: 4.0,
            seed: 42,
        });
        (subjects, theta_true, omega_true, sigma_true)
    }

    #[test]
    fn synthetic_data_reasonable() {
        let (subjects, _, _, _) = generate_test_data();
        assert_eq!(subjects.len(), 20);
        for subj in &subjects {
            assert_eq!(subj.times.len(), 12);
            assert_eq!(subj.observations.len(), 12);
            assert!(subj.observations.iter().all(|&c| c >= 0.0));
        }
    }

    #[test]
    fn foce_runs_and_returns() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = default_test_config();
        let result = foce(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );

        assert_eq!(result.theta.len(), 3);
        assert_eq!(result.omega_diag.len(), 3);
        assert!(result.sigma > 0.0);
        assert!(result.iterations > 0);
        assert_eq!(result.individual_etas.len(), 20);
    }

    #[test]
    fn foce_recovers_parameters() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = NlmeConfig {
            max_iter: 200,
            tol: 1e-6,
            ..default_test_config()
        };

        let result = foce(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );

        for (dim, (&est, &truth)) in result.theta.iter().zip(theta_true.iter()).enumerate() {
            let rel_err = (est - truth).abs() / truth.abs().max(0.01);
            assert!(
                rel_err < 0.5,
                "theta[{dim}] within 50%: est={est:.4}, true={truth:.4}"
            );
        }
    }

    #[test]
    fn foce_objective_decreases() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();

        let r10 = foce(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &NlmeConfig {
                max_iter: 10,
                ..default_test_config()
            },
        );

        let r50 = foce(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &NlmeConfig {
                max_iter: 50,
                ..default_test_config()
            },
        );

        assert!(
            r50.objective <= r10.objective + 1.0,
            "more iterations should not dramatically increase objective"
        );
    }

    #[test]
    fn foce_deterministic() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = default_test_config();

        let r1 = foce(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );
        let r2 = foce(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );

        assert_eq!(r1.objective.to_bits(), r2.objective.to_bits());
        for dim in 0..3 {
            assert_eq!(r1.theta[dim].to_bits(), r2.theta[dim].to_bits());
        }
    }

    #[test]
    fn saem_runs_and_returns() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let result = saem(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &default_test_config(),
        );

        assert_eq!(result.theta.len(), 3);
        assert!(result.sigma > 0.0);
        assert!(result.iterations > 0);
    }

    #[test]
    fn saem_recovers_parameters() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = NlmeConfig {
            max_iter: 300,
            tol: 1e-6,
            ..default_test_config()
        };

        let result = saem(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );

        for (dim, (&est, &truth)) in result.theta.iter().zip(theta_true.iter()).enumerate() {
            let rel_err = (est - truth).abs() / truth.abs().max(0.01);
            assert!(
                rel_err < 0.5,
                "theta[{dim}] within 50%: est={est:.4}, true={truth:.4}"
            );
        }
    }

    #[test]
    fn saem_deterministic() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = default_test_config();
        let r1 = saem(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );
        let r2 = saem(
            oral_one_compartment_model,
            &subjects,
            &theta_true,
            &omega_true,
            sigma_true,
            &config,
        );
        assert_eq!(r1.objective.to_bits(), r2.objective.to_bits());
    }

    #[test]
    fn iv_model_basic() {
        let theta = vec![1.0, 3.0];
        let eta = vec![0.0, 0.0];
        let c0 = iv_one_compartment_model(&theta, &eta, 100.0, 0.0);
        let vd = theta[1].exp();
        assert!(
            (c0 - 100.0 / vd).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "C(0) = Dose/Vd"
        );
        assert!(iv_one_compartment_model(&theta, &eta, 100.0, 100.0) < c0);
    }

    #[test]
    fn generate_synthetic_population_deterministic() {
        let theta = vec![2.3, 4.4, 0.4];
        let omega = vec![0.04, 0.04, 0.09];
        let times: Vec<f64> = (0..6).map(|i| f64::from(i) * 4.0).collect();
        let cfg = SyntheticPopConfig {
            model: oral_one_compartment_model,
            theta: &theta,
            omega: &omega,
            sigma: 0.01,
            n_subjects: 5,
            times: &times,
            dose: 4.0,
            seed: 99,
        };
        let s1 = generate_synthetic_population(&cfg);
        let s2 = generate_synthetic_population(&cfg);

        for (a, b) in s1.iter().zip(s2.iter()) {
            for (oa, ob) in a.observations.iter().zip(b.observations.iter()) {
                assert_eq!(oa.to_bits(), ob.to_bits());
            }
        }
    }
}
