// SPDX-License-Identifier: AGPL-3.0-or-later
//! FOCE (First-Order Conditional Estimation) — NONMEM's workhorse algorithm.
//!
//! Estimates population parameters (theta, omega, sigma) by iterating between:
//! 1. **Inner loop**: optimise each individual's eta given current population parameters.
//! 2. **Outer loop**: update population parameters given all individual eta estimates.

use super::solver::{
    EstimationCtx, optimize_individual_eta, theta_gradient_step, total_objective_and_mse,
};
use super::{NlmeConfig, NlmeResult, StructuralModel, Subject};

/// FOCE: First-Order Conditional Estimation.
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

        let (obj, sse, n_obs) =
            total_objective_and_mse(model, &theta, &etas, subjects, &omega, sigma);

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
        let lr = 0.0001 / 0.01f64.mul_add(iter as f64, 1.0);
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
