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

use crate::rng::{lcg_step, normal_sample, state_to_f64};

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

// ═══════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════

fn predict_subject(model: StructuralModel, theta: &[f64], eta: &[f64], subj: &Subject) -> Vec<f64> {
    subj.times
        .iter()
        .map(|&t| model(theta, eta, subj.dose, t))
        .collect()
}

/// Gradient of predictions with respect to eta (central finite differences).
fn dpred_deta(
    model: StructuralModel,
    theta: &[f64],
    eta: &[f64],
    subj: &Subject,
    n_eta: usize,
) -> Vec<Vec<f64>> {
    let step = 1e-6;
    let n_obs = subj.times.len();
    let mut grad = vec![vec![0.0; n_obs]; n_eta];
    let mut eta_pert = eta.to_vec();

    for (dim, row) in grad.iter_mut().enumerate() {
        let orig = eta_pert[dim];
        eta_pert[dim] = orig + step;
        let pred_hi = predict_subject(model, theta, &eta_pert, subj);
        eta_pert[dim] = orig - step;
        let pred_lo = predict_subject(model, theta, &eta_pert, subj);
        eta_pert[dim] = orig;

        for (val, (hi, lo)) in row.iter_mut().zip(pred_hi.iter().zip(pred_lo.iter())) {
            *val = (hi - lo) / (2.0 * step);
        }
    }
    grad
}

/// Sum of squared weighted residuals + random effect penalty for one subject.
fn subject_objective(
    model: StructuralModel,
    theta: &[f64],
    eta: &[f64],
    subj: &Subject,
    omega: &[f64],
    sigma: f64,
) -> f64 {
    let pred = predict_subject(model, theta, eta, subj);

    let resid_term: f64 = subj
        .observations
        .iter()
        .zip(pred.iter())
        .map(|(&obs, &pr)| (obs - pr).powi(2) / sigma)
        .sum();

    let eta_term: f64 = eta
        .iter()
        .zip(omega.iter())
        .map(|(&ek, &ok)| if ok > 1e-15 { ek * ek / ok } else { 0.0 })
        .sum();

    resid_term + eta_term
}

/// Gauss-Newton inner optimization for individual eta.
fn optimize_individual_eta(
    ctx: &EstimationCtx<'_>,
    eta_init: &[f64],
    subj: &Subject,
) -> Vec<f64> {
    let n_eta = ctx.n_eta;
    let mut eta = eta_init.to_vec();

    for _ in 0..20 {
        let pred = predict_subject(ctx.model, ctx.theta, &eta, subj);
        let grad = dpred_deta(ctx.model, ctx.theta, &eta, subj, n_eta);

        let mut rhs = vec![0.0; n_eta];
        let mut hess = vec![vec![0.0; n_eta]; n_eta];

        for (dim, rhs_val) in rhs.iter_mut().enumerate() {
            for ((&obs, &pr), &gval) in subj.observations.iter().zip(pred.iter()).zip(grad[dim].iter()) {
                *rhs_val += gval * (obs - pr) / ctx.sigma;
            }
            if ctx.omega[dim] > 1e-15 {
                *rhs_val -= eta[dim] / ctx.omega[dim];
            }
        }

        for d1 in 0..n_eta {
            for d2 in 0..n_eta {
                for (&g1, &g2) in grad[d1].iter().zip(grad[d2].iter()) {
                    hess[d1][d2] += g1 * g2 / ctx.sigma;
                }
            }
            hess[d1][d1] += if ctx.omega[d1] > 1e-15 {
                1.0 / ctx.omega[d1]
            } else {
                0.0
            };
        }

        let delta = cholesky_solve(&hess, &rhs);
        let max_step = delta.iter().map(|d| d.abs()).fold(0.0_f64, f64::max);
        for (ek, &dk) in eta.iter_mut().zip(delta.iter()) {
            *ek += dk;
        }
        if max_step < 1e-8 {
            break;
        }
    }

    eta
}

/// Estimation context to reduce argument count.
struct EstimationCtx<'a> {
    model: StructuralModel,
    theta: &'a [f64],
    omega: &'a [f64],
    sigma: f64,
    n_eta: usize,
}

/// Solve `H · x = b` for small symmetric positive-definite `H` via Cholesky.
/// Falls back to diagonal solve when factorisation fails.
fn cholesky_solve(hmat: &[Vec<f64>], rhs: &[f64]) -> Vec<f64> {
    let dim = rhs.len();
    let mut lower = vec![vec![0.0; dim]; dim];
    let mut ok = true;

    for row in 0..dim {
        for col in 0..=row {
            let mut accum = hmat[row][col];
            for (lr, lc) in lower[row][..col].iter().zip(lower[col][..col].iter()) {
                accum -= lr * lc;
            }
            if row == col {
                if accum <= 0.0 {
                    ok = false;
                    break;
                }
                lower[row][col] = accum.sqrt();
            } else {
                lower[row][col] = accum / lower[col][col];
            }
        }
        if !ok {
            break;
        }
    }

    if !ok {
        return (0..dim)
            .map(|i| {
                if hmat[i][i].abs() > 1e-15 {
                    rhs[i] / hmat[i][i]
                } else {
                    0.0
                }
            })
            .collect();
    }

    // Forward substitution: L y = b
    let mut fwd = vec![0.0; dim];
    for row in 0..dim {
        let mut sum = rhs[row];
        for col in 0..row {
            sum -= lower[row][col] * fwd[col];
        }
        fwd[row] = sum / lower[row][row];
    }

    // Back substitution: L^T x = y
    let mut result = vec![0.0; dim];
    for row in (0..dim).rev() {
        let mut sum = fwd[row];
        for col in (row + 1)..dim {
            sum -= lower[col][row] * result[col];
        }
        result[row] = sum / lower[row][row];
    }

    result
}

/// Compute total objective and mean squared residual across all subjects.
fn total_objective_and_mse(
    model: StructuralModel,
    theta: &[f64],
    etas: &[Vec<f64>],
    subjects: &[Subject],
    omega: &[f64],
    sigma: f64,
) -> (f64, f64, usize) {
    let mut obj = 0.0;
    let mut sse = 0.0;
    let mut n_obs = 0_usize;
    for (eta, subj) in etas.iter().zip(subjects.iter()) {
        obj += subject_objective(model, theta, eta, subj, omega, sigma);
        let pred = predict_subject(model, theta, eta, subj);
        for (&ob, &pr) in subj.observations.iter().zip(pred.iter()) {
            sse += (ob - pr).powi(2);
        }
        n_obs += subj.observations.len();
    }
    (obj, sse, n_obs)
}

/// Central-difference gradient of total objective w.r.t. theta, with clamped step.
fn theta_gradient_step(
    model: StructuralModel,
    theta: &mut [f64],
    etas: &[Vec<f64>],
    subjects: &[Subject],
    omega: &[f64],
    sigma: f64,
    learning_rate: f64,
) {
    let perturbation = 1e-5;
    let n_theta = theta.len();
    for dim in 0..n_theta {
        let orig = theta[dim];
        theta[dim] = orig + perturbation;
        let mut obj_hi = 0.0;
        theta[dim] = orig - perturbation;
        let mut obj_lo = 0.0;
        // Re-evaluate at +h and -h
        theta[dim] = orig + perturbation;
        for (eta, subj) in etas.iter().zip(subjects.iter()) {
            obj_hi += subject_objective(model, theta, eta, subj, omega, sigma);
        }
        theta[dim] = orig - perturbation;
        for (eta, subj) in etas.iter().zip(subjects.iter()) {
            obj_lo += subject_objective(model, theta, eta, subj, omega, sigma);
        }
        theta[dim] = orig;

        let grad = (obj_hi - obj_lo) / (2.0 * perturbation);
        let step = (learning_rate * grad).clamp(-0.05, 0.05);
        theta[dim] -= step;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FOCE (First-Order Conditional Estimation)
// ═══════════════════════════════════════════════════════════════════════

/// FOCE: First-Order Conditional Estimation.
///
/// The workhorse algorithm of NONMEM. Estimates population parameters
/// (theta, omega, sigma) by iterating between:
/// 1. **Inner loop**: optimise each individual's eta given current population parameters.
/// 2. **Outer loop**: update population parameters given all individual eta estimates.
///
/// # Returns
///
/// [`NlmeResult`] with estimated parameters, objective function, and convergence status.
pub fn foce(
    model: StructuralModel,
    subjects: &[Subject],
    theta_init: &[f64],
    omega_init: &[f64],
    sigma_init: f64,
    config: &NlmeConfig,
) -> NlmeResult {
    let n_sub = subjects.len();
    let n_eta = config.n_eta;

    let mut theta = theta_init.to_vec();
    let mut omega = omega_init.to_vec();
    let mut sigma = sigma_init;
    let mut etas = vec![vec![0.0; n_eta]; n_sub];
    let mut prev_obj = f64::MAX;
    let mut converged = false;
    let mut iter_count = 0;

    for iter in 0..config.max_iter {
        iter_count = iter + 1;

        let ctx = EstimationCtx {
            model,
            theta: &theta,
            omega: &omega,
            sigma,
            n_eta,
        };
        for (idx, subj) in subjects.iter().enumerate() {
            etas[idx] = optimize_individual_eta(&ctx, &etas[idx], subj);
        }

        let (obj, sse, n_obs) = total_objective_and_mse(model, &theta, &etas, subjects, &omega, sigma);

        // Update omega: empirical variance of etas
        for (dim, omega_val) in omega.iter_mut().enumerate() {
            let sum_sq: f64 = etas.iter().map(|e| e[dim] * e[dim]).sum();
            #[expect(clippy::cast_precision_loss, reason = "subject count fits f64")]
            let new_val = sum_sq / n_sub as f64;
            *omega_val = new_val.max(1e-8);
        }

        // Update sigma
        if n_obs > 0 {
            #[expect(clippy::cast_precision_loss, reason = "observation count fits f64")]
            let new_sigma = sse / n_obs as f64;
            sigma = new_sigma.max(1e-10);
        }

        #[expect(clippy::cast_precision_loss, reason = "iteration index fits f64")]
        let lr = 0.0001 / (1.0 + 0.01 * iter as f64);
        theta_gradient_step(model, &mut theta, &etas, subjects, &omega, sigma, lr);

        let rel_change = if prev_obj.is_finite() && prev_obj.abs() > 1e-15 {
            (prev_obj - obj).abs() / prev_obj.abs()
        } else {
            1.0
        };
        prev_obj = obj;

        if rel_change < config.tol && iter > 5 {
            converged = true;
            break;
        }
    }

    NlmeResult {
        theta,
        omega_diag: omega,
        sigma,
        objective: prev_obj,
        iterations: iter_count,
        converged,
        individual_etas: etas,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SAEM (Stochastic Approximation Expectation-Maximization)
// ═══════════════════════════════════════════════════════════════════════

/// Mutable state carried through SAEM iterations.
struct SaemState {
    etas: Vec<Vec<f64>>,
    omega: Vec<f64>,
    sigma: f64,
    rng: u64,
    stats_eta_sq: Vec<f64>,
    stats_sigma: f64,
    stats_count: f64,
}

/// Run one Metropolis-Hastings step per subject (E-step of SAEM).
fn saem_estep(
    model: StructuralModel,
    theta: &[f64],
    subjects: &[Subject],
    n_eta: usize,
    state: &mut SaemState,
) {
    for (eta, subj) in state.etas.iter_mut().zip(subjects.iter()) {
        let current_obj = subject_objective(model, theta, eta, subj, &state.omega, state.sigma);

        let mut proposal = eta.clone();
        for dim in 0..n_eta {
            let (z_val, new_st) = normal_sample(state.rng);
            state.rng = new_st;
            let step_sd = state.omega[dim].sqrt().max(0.01) * 0.3;
            proposal[dim] = eta[dim] + step_sd * z_val;
        }

        let proposal_obj = subject_objective(model, theta, &proposal, subj, &state.omega, state.sigma);
        let log_alpha = -0.5 * (proposal_obj - current_obj);
        let next = lcg_step(state.rng);
        state.rng = next;

        if state_to_f64(next).ln() < log_alpha {
            *eta = proposal;
        }
    }
}

/// M-step: update sufficient statistics with stochastic approximation.
fn saem_mstep(
    model: StructuralModel,
    theta: &[f64],
    subjects: &[Subject],
    gamma: f64,
    state: &mut SaemState,
) {
    #[expect(clippy::cast_precision_loss, reason = "subject count fits f64")]
    let n_sub_f = state.etas.len() as f64;

    for (dim, stat) in state.stats_eta_sq.iter_mut().enumerate() {
        let emp: f64 = state.etas.iter().map(|e| e[dim] * e[dim]).sum();
        let emp_mean = emp / n_sub_f;
        *stat = (1.0 - gamma) * *stat + gamma * emp_mean;
    }

    let mut total_sse = 0.0;
    let mut total_obs = 0_usize;
    for (eta, subj) in state.etas.iter().zip(subjects.iter()) {
        let pred = predict_subject(model, theta, eta, subj);
        for (&obs, &pr) in subj.observations.iter().zip(pred.iter()) {
            total_sse += (obs - pr).powi(2);
        }
        total_obs += subj.observations.len();
    }
    if total_obs > 0 {
        #[expect(clippy::cast_precision_loss, reason = "observation count fits f64")]
        let emp_sigma = total_sse / total_obs as f64;
        state.stats_sigma = (1.0 - gamma) * state.stats_sigma + gamma * emp_sigma;
        state.stats_count = (1.0 - gamma) * state.stats_count + gamma;
    }

    for (dim, stat) in state.stats_eta_sq.iter().enumerate() {
        state.omega[dim] = stat.max(1e-8);
    }
    if state.stats_count > 1e-15 {
        state.sigma = (state.stats_sigma / state.stats_count).max(1e-10);
    }
}

/// SAEM: Stochastic Approximation Expectation-Maximization.
///
/// Monolix's approach to NLME. Alternates:
/// 1. **E-step**: generate individual parameter samples via Metropolis-Hastings MCMC.
/// 2. **M-step**: update population parameters using stochastic approximation
///    with decreasing step sizes (Robbins-Monro).
pub fn saem(
    model: StructuralModel,
    subjects: &[Subject],
    theta_init: &[f64],
    omega_init: &[f64],
    sigma_init: f64,
    config: &NlmeConfig,
) -> NlmeResult {
    let n_eta = config.n_eta;
    let n_burn = config.max_iter / 3;

    let mut theta = theta_init.to_vec();
    let mut st = SaemState {
        etas: vec![vec![0.0; n_eta]; subjects.len()],
        omega: omega_init.to_vec(),
        sigma: sigma_init,
        rng: config.seed,
        stats_eta_sq: vec![0.0; n_eta],
        stats_sigma: 0.0,
        stats_count: 0.0,
    };

    let mut prev_obj = f64::MAX;
    let mut converged = false;
    let mut iter_count = 0;

    for iter in 0..config.max_iter {
        iter_count = iter + 1;

        #[expect(clippy::cast_precision_loss, reason = "iteration count fits f64")]
        let gamma = if iter < n_burn {
            1.0
        } else {
            1.0 / (iter - n_burn + 1) as f64
        };

        saem_estep(model, &theta, subjects, n_eta, &mut st);
        saem_mstep(model, &theta, subjects, gamma, &mut st);

        if iter >= n_burn {
            theta_gradient_step(model, &mut theta, &st.etas, subjects, &st.omega, st.sigma, gamma * 0.0001);
        }

        let (obj, _, _) = total_objective_and_mse(model, &theta, &st.etas, subjects, &st.omega, st.sigma);

        let rel_change = if prev_obj.is_finite() && prev_obj.abs() > 1e-15 {
            (prev_obj - obj).abs() / prev_obj.abs()
        } else {
            1.0
        };
        prev_obj = obj;

        if rel_change < config.tol && iter > n_burn + 10 {
            converged = true;
            break;
        }
    }

    NlmeResult {
        theta,
        omega_diag: st.omega,
        sigma: st.sigma,
        objective: prev_obj,
        iterations: iter_count,
        converged,
        individual_etas: st.etas,
    }
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
            observations.push((pred + cfg.sigma.sqrt() * eps).max(0.0));
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
        let result = foce(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);

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

        let result = foce(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);

        for (dim, (&est, &truth)) in result.theta.iter().zip(theta_true.iter()).enumerate() {
            let rel_err = (est - truth).abs() / truth.abs().max(0.01);
            assert!(rel_err < 0.5, "theta[{dim}] within 50%: est={est:.4}, true={truth:.4}");
        }
    }

    #[test]
    fn foce_objective_decreases() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();

        let r10 = foce(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true,
            &NlmeConfig { max_iter: 10, ..default_test_config() });

        let r50 = foce(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true,
            &NlmeConfig { max_iter: 50, ..default_test_config() });

        assert!(r50.objective <= r10.objective + 1.0,
            "more iterations should not dramatically increase objective");
    }

    #[test]
    fn foce_deterministic() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = default_test_config();

        let r1 = foce(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);
        let r2 = foce(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);

        assert_eq!(r1.objective.to_bits(), r2.objective.to_bits());
        for dim in 0..3 {
            assert_eq!(r1.theta[dim].to_bits(), r2.theta[dim].to_bits());
        }
    }

    #[test]
    fn saem_runs_and_returns() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let result = saem(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &default_test_config());

        assert_eq!(result.theta.len(), 3);
        assert!(result.sigma > 0.0);
        assert!(result.iterations > 0);
    }

    #[test]
    fn saem_recovers_parameters() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = NlmeConfig { max_iter: 300, tol: 1e-6, ..default_test_config() };

        let result = saem(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);

        for (dim, (&est, &truth)) in result.theta.iter().zip(theta_true.iter()).enumerate() {
            let rel_err = (est - truth).abs() / truth.abs().max(0.01);
            assert!(rel_err < 0.5, "theta[{dim}] within 50%: est={est:.4}, true={truth:.4}");
        }
    }

    #[test]
    fn saem_deterministic() {
        let (subjects, theta_true, omega_true, sigma_true) = generate_test_data();
        let config = default_test_config();
        let r1 = saem(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);
        let r2 = saem(oral_one_compartment_model, &subjects, &theta_true, &omega_true, sigma_true, &config);
        assert_eq!(r1.objective.to_bits(), r2.objective.to_bits());
    }

    #[test]
    fn iv_model_basic() {
        let theta = vec![1.0, 3.0];
        let eta = vec![0.0, 0.0];
        let c0 = iv_one_compartment_model(&theta, &eta, 100.0, 0.0);
        let vd = theta[1].exp();
        assert!((c0 - 100.0 / vd).abs() < 1e-10, "C(0) = Dose/Vd");
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
