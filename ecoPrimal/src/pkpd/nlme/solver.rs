// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared numerical infrastructure for NLME estimation.
//!
//! Provides the inner-loop helpers used by both FOCE and SAEM:
//! - Individual eta optimization (Gauss-Newton)
//! - Cholesky solve for small symmetric positive-definite systems
//! - Objective function evaluation
//! - Finite-difference gradient descent on theta

use super::{StructuralModel, Subject};

/// Estimation context passed through inner loops to reduce argument count.
pub(super) struct EstimationCtx<'a> {
    pub model: StructuralModel,
    pub theta: &'a [f64],
    pub omega: &'a [f64],
    pub sigma: f64,
    pub n_eta: usize,
}

pub(super) fn predict_subject(
    model: StructuralModel,
    theta: &[f64],
    eta: &[f64],
    subj: &Subject,
) -> Vec<f64> {
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
pub(super) fn subject_objective(
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
pub(super) fn optimize_individual_eta(
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
            for ((&obs, &pr), &gval) in subj
                .observations
                .iter()
                .zip(pred.iter())
                .zip(grad[dim].iter())
            {
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

/// Solve `H · x = b` for small symmetric positive-definite `H` via Cholesky.
/// Falls back to diagonal solve when factorisation fails.
///
/// Intentionally local rather than delegating to `barracuda::linalg::cholesky`:
/// the NLME inner loop operates on 2×2 or 3×3 `Vec<Vec<f64>>` matrices with
/// integrated fallback. barraCuda's Cholesky targets larger flat-layout matrices
/// with GPU promotion; wrapping it here would add conversion overhead with no
/// precision or performance benefit at this scale.
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
pub(super) fn total_objective_and_mse(
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
pub(super) fn theta_gradient_step(
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
