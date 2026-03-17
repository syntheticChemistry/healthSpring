// SPDX-License-Identifier: AGPL-3.0-or-later

//! CWRES (Conditional Weighted Residuals) computation.
//!
//! Residuals standardised by the FOCE conditional variance approximation.
//! `CWRES_ij = (y_ij - f(theta, eta_i, t_ij)) / sqrt(sigma)`.

use super::super::nlme::{NlmeResult, StructuralModel, Subject};

/// Conditional Weighted Residuals for one subject.
#[derive(Debug, Clone)]
pub struct SubjectCwres {
    pub subject_idx: usize,
    pub times: Vec<f64>,
    pub residuals: Vec<f64>,
}

/// Compute CWRES for all subjects given an NLME result.
///
/// `CWRES_ij = (y_ij - f(theta, eta_i, t_ij)) / sqrt(sigma)`
///
/// This is the FOCE-I approximation where residuals are scaled by the
/// square root of the estimated residual variance.
#[must_use]
pub fn compute_cwres(
    model: StructuralModel,
    subjects: &[Subject],
    result: &NlmeResult,
) -> Vec<SubjectCwres> {
    let sigma_sqrt = result.sigma.sqrt().max(1e-15);

    subjects
        .iter()
        .enumerate()
        .map(|(idx, subj)| {
            let eta = &result.individual_etas[idx];
            let residuals: Vec<f64> = subj
                .times
                .iter()
                .zip(subj.observations.iter())
                .map(|(&time, &obs)| {
                    let pred = model(&result.theta, eta, subj.dose, time);
                    (obs - pred) / sigma_sqrt
                })
                .collect();
            SubjectCwres {
                subject_idx: idx,
                times: subj.times.clone(),
                residuals,
            }
        })
        .collect()
}

/// Aggregate CWRES statistics across all subjects.
#[derive(Debug, Clone)]
pub struct CwresSummary {
    /// All CWRES values (concatenated across subjects).
    pub all_residuals: Vec<f64>,
    /// Corresponding time points.
    pub all_times: Vec<f64>,
    /// Mean of CWRES (should be ~0 for well-specified model).
    pub mean: f64,
    /// Standard deviation of CWRES (should be ~1).
    pub std_dev: f64,
}

/// Compute aggregate CWRES summary for model evaluation.
#[must_use]
pub fn cwres_summary(subject_cwres: &[SubjectCwres]) -> CwresSummary {
    let mut all_residuals = Vec::new();
    let mut all_times = Vec::new();

    for sc in subject_cwres {
        all_residuals.extend_from_slice(&sc.residuals);
        all_times.extend_from_slice(&sc.times);
    }

    let count = all_residuals.len();
    let (mean, std_dev) = if count > 0 {
        #[expect(clippy::cast_precision_loss, reason = "residual count fits f64")]
        let n_f = count as f64;
        let m: f64 = all_residuals.iter().sum::<f64>() / n_f;
        let variance: f64 = all_residuals.iter().map(|&r| (r - m).powi(2)).sum::<f64>() / n_f;
        (m, variance.sqrt())
    } else {
        (0.0, 0.0)
    };

    CwresSummary {
        all_residuals,
        all_times,
        mean,
        std_dev,
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
    fn cwres_has_correct_size() {
        let (subjects, result) = fitted_result();
        let cwres = compute_cwres(oral_one_compartment_model, &subjects, &result);
        assert_eq!(cwres.len(), 20);
        for sc in &cwres {
            assert_eq!(sc.residuals.len(), 12);
        }
    }

    #[test]
    fn cwres_summary_mean_near_zero() {
        let (subjects, result) = fitted_result();
        let cwres = compute_cwres(oral_one_compartment_model, &subjects, &result);
        let summary = cwres_summary(&cwres);
        assert!(
            summary.mean.abs() < 2.0,
            "CWRES mean should be near 0: got {}",
            summary.mean
        );
    }
}
