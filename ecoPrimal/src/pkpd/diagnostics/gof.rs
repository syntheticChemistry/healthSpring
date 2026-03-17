// SPDX-License-Identifier: AGPL-3.0-or-later

//! GOF (Goodness-of-Fit) diagnostics.
//!
//! Observed vs predicted, residual vs time, and R-squared metrics.
//! Individual predictions use estimated etas; population predictions use eta=0.

use super::super::nlme::{NlmeResult, StructuralModel, Subject};

/// Goodness-of-fit diagnostic data.
#[derive(Debug, Clone)]
pub struct GofResult {
    /// Observed concentrations.
    pub observed: Vec<f64>,
    /// Individual predicted concentrations (using individual etas).
    pub individual_predicted: Vec<f64>,
    /// Population predicted concentrations (eta = 0).
    pub population_predicted: Vec<f64>,
    /// Observation times.
    pub times: Vec<f64>,
    /// Individual weighted residuals.
    pub individual_residuals: Vec<f64>,
    /// R-squared (observed vs individual predicted).
    pub r_squared_individual: f64,
    /// R-squared (observed vs population predicted).
    pub r_squared_population: f64,
}

/// Compute goodness-of-fit diagnostics.
#[must_use]
pub fn compute_gof(model: StructuralModel, subjects: &[Subject], result: &NlmeResult) -> GofResult {
    let n_eta = result.omega_diag.len();
    let eta_zero = vec![0.0; n_eta];
    let sigma_sqrt = result.sigma.sqrt().max(1e-15);

    let mut observed = Vec::new();
    let mut individual_predicted = Vec::new();
    let mut population_predicted = Vec::new();
    let mut times = Vec::new();
    let mut individual_residuals = Vec::new();

    for (idx, subj) in subjects.iter().enumerate() {
        let eta = &result.individual_etas[idx];
        for (&time, &obs) in subj.times.iter().zip(subj.observations.iter()) {
            let ipred = model(&result.theta, eta, subj.dose, time);
            let ppred = model(&result.theta, &eta_zero, subj.dose, time);

            observed.push(obs);
            individual_predicted.push(ipred);
            population_predicted.push(ppred);
            times.push(time);
            individual_residuals.push((obs - ipred) / sigma_sqrt);
        }
    }

    let r_sq_ind = r_squared(&observed, &individual_predicted);
    let r_sq_pop = r_squared(&observed, &population_predicted);

    GofResult {
        observed,
        individual_predicted,
        population_predicted,
        times,
        individual_residuals,
        r_squared_individual: r_sq_ind,
        r_squared_population: r_sq_pop,
    }
}

/// R-squared between observed and predicted values.
fn r_squared(observed: &[f64], predicted: &[f64]) -> f64 {
    if observed.is_empty() {
        return 0.0;
    }
    #[expect(clippy::cast_precision_loss, reason = "observation count fits f64")]
    let mean_obs = observed.iter().sum::<f64>() / observed.len() as f64;
    let ss_tot: f64 = observed.iter().map(|&o| (o - mean_obs).powi(2)).sum();
    let ss_res: f64 = observed
        .iter()
        .zip(predicted.iter())
        .map(|(&o, &p)| (o - p).powi(2))
        .sum();
    if ss_tot > 1e-15 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkpd::nlme::{
        NlmeConfig, SyntheticPopConfig, foce, generate_synthetic_population,
        oral_one_compartment_model,
    };

    fn fitted_result() -> (Vec<Subject>, NlmeResult) {
        let theta = vec![2.3, 4.4, 0.4];
        let omega = vec![0.04, 0.04, 0.09];
        let sigma = 0.01;
        let times: Vec<f64> = (0..12).map(|i| f64::from(i) * 2.0).collect();
        let subjects = generate_synthetic_population(&SyntheticPopConfig {
            model: oral_one_compartment_model,
            theta: &theta,
            omega: &omega,
            sigma,
            n_subjects: 20,
            times: &times,
            dose: 4.0,
            seed: 42,
        });
        let config = NlmeConfig {
            n_theta: 3,
            n_eta: 3,
            max_iter: 150,
            tol: 1e-6,
            seed: 12_345,
        };
        let result = foce(
            oral_one_compartment_model,
            &subjects,
            &theta,
            &omega,
            sigma,
            &config,
        );
        (subjects, result)
    }

    #[test]
    fn gof_individual_better_than_population() {
        let (subjects, result) = fitted_result();
        let gof = compute_gof(oral_one_compartment_model, &subjects, &result);
        assert!(
            gof.r_squared_individual >= gof.r_squared_population,
            "individual R² ({}) >= population R² ({})",
            gof.r_squared_individual,
            gof.r_squared_population
        );
    }

    #[test]
    fn gof_sizes_correct() {
        let (subjects, result) = fitted_result();
        let gof = compute_gof(oral_one_compartment_model, &subjects, &result);
        let expected_n = 20 * 12;
        assert_eq!(gof.observed.len(), expected_n);
        assert_eq!(gof.individual_predicted.len(), expected_n);
        assert_eq!(gof.population_predicted.len(), expected_n);
        assert_eq!(gof.individual_residuals.len(), expected_n);
    }

    #[test]
    fn gof_deterministic() {
        let (subjects, result) = fitted_result();
        let g1 = compute_gof(oral_one_compartment_model, &subjects, &result);
        let g2 = compute_gof(oral_one_compartment_model, &subjects, &result);
        assert_eq!(
            g1.r_squared_individual.to_bits(),
            g2.r_squared_individual.to_bits()
        );
    }
}
