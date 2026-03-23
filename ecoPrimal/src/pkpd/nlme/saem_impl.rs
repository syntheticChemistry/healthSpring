// SPDX-License-Identifier: AGPL-3.0-or-later
//! SAEM (Stochastic Approximation Expectation-Maximization) — Monolix's algorithm.
//!
//! Alternates:
//! 1. **E-step**: generate individual parameter samples via Metropolis-Hastings MCMC.
//! 2. **M-step**: update population parameters using stochastic approximation
//!    with decreasing step sizes (Robbins-Monro).
//!
//! Reference: Kuhn & Lavielle, "Maximum likelihood estimation in nonlinear mixed
//! effects models," CSDA (2005)

use crate::rng::{lcg_step, normal_sample, state_to_f64};

use super::solver::{
    predict_subject, subject_objective, theta_gradient_step, total_objective_and_mse,
};
use super::{NlmeConfig, NlmeResult, StructuralModel, Subject};
use crate::tolerances;

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
fn estep(
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
            let step_sd = state.omega[dim].sqrt().max(tolerances::SAEM_MH_MIN_SD)
                * tolerances::SAEM_MH_PROPOSAL_SCALE;
            proposal[dim] = step_sd.mul_add(z_val, eta[dim]);
        }

        let proposal_obj =
            subject_objective(model, theta, &proposal, subj, &state.omega, state.sigma);
        let log_alpha = -0.5 * (proposal_obj - current_obj);
        let next = lcg_step(state.rng);
        state.rng = next;

        if state_to_f64(next).ln() < log_alpha {
            *eta = proposal;
        }
    }
}

/// M-step: update sufficient statistics with stochastic approximation.
fn mstep(
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
        *stat = (1.0 - gamma).mul_add(*stat, gamma * emp_mean);
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
        state.stats_sigma = (1.0 - gamma).mul_add(state.stats_sigma, gamma * emp_sigma);
        state.stats_count = (1.0 - gamma).mul_add(state.stats_count, gamma);
    }

    for (dim, stat) in state.stats_eta_sq.iter().enumerate() {
        state.omega[dim] = stat.max(tolerances::NLME_OMEGA_FLOOR);
    }
    if state.stats_count > tolerances::DIVISION_GUARD {
        state.sigma = (state.stats_sigma / state.stats_count).max(tolerances::NLME_SIGMA_FLOOR);
    }
}

/// SAEM: Stochastic Approximation Expectation-Maximization.
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

        estep(model, &theta, subjects, n_eta, &mut st);
        mstep(model, &theta, subjects, gamma, &mut st);

        if iter >= n_burn {
            theta_gradient_step(
                model,
                &mut theta,
                &st.etas,
                subjects,
                &st.omega,
                st.sigma,
                gamma * tolerances::FOCE_LR_BASE,
            );
        }

        let (obj, _, _) =
            total_objective_and_mse(model, &theta, &st.etas, subjects, &st.omega, st.sigma);

        let rel_change = if prev_obj.is_finite() && prev_obj.abs() > tolerances::DIVISION_GUARD {
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
