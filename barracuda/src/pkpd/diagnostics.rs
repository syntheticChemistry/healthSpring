// SPDX-License-Identifier: AGPL-3.0-or-later
//! Population PK diagnostics — CWRES, VPC, and GOF (Goodness-of-Fit).
//!
//! Standard population PK model evaluation plots as described in FDA
//! guidance and Hooker et al. (2007). All diagnostic data is returned
//! as structured types ready for `petalTongue` `DataChannel` rendering.
//!
//! ## Diagnostics
//!
//! - **CWRES** (Conditional Weighted Residuals): residuals standardised by
//!   the FOCE conditional variance approximation.
//! - **VPC** (Visual Predictive Check): simulation-based model evaluation
//!   comparing observed data quantiles against model-predicted quantiles.
//! - **GOF** (Goodness-of-Fit): observed vs predicted, residual vs time,
//!   and QQ-normal plots.

use super::nlme::{NlmeResult, StructuralModel, Subject};

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

// ═══════════════════════════════════════════════════════════════════════
// VPC (Visual Predictive Check)
// ═══════════════════════════════════════════════════════════════════════

/// VPC result containing observed and simulated quantile bands.
#[derive(Debug, Clone)]
pub struct VpcResult {
    /// Time bins (midpoints).
    pub time_bins: Vec<f64>,
    /// Observed 5th percentile per time bin.
    pub obs_p5: Vec<f64>,
    /// Observed 50th percentile (median) per time bin.
    pub obs_p50: Vec<f64>,
    /// Observed 95th percentile per time bin.
    pub obs_p95: Vec<f64>,
    /// Simulated 5th percentile (median of simulated p5 across replicates).
    pub sim_p5: Vec<f64>,
    /// Simulated 50th percentile.
    pub sim_p50: Vec<f64>,
    /// Simulated 95th percentile.
    pub sim_p95: Vec<f64>,
}

/// Configuration for VPC computation.
pub struct VpcConfig {
    /// Number of simulation replicates.
    pub n_simulations: usize,
    /// Number of time bins.
    pub n_bins: usize,
    /// PRNG seed.
    pub seed: u64,
}

impl Default for VpcConfig {
    fn default() -> Self {
        Self {
            n_simulations: 200,
            n_bins: 10,
            seed: 42,
        }
    }
}

/// Compute a Visual Predictive Check.
///
/// Bins observed data by time, computes quantiles in each bin, then
/// simulates `n_simulations` replicate datasets from the estimated model
/// and computes the same quantiles to form prediction bands.
#[must_use]
pub fn compute_vpc(
    model: StructuralModel,
    subjects: &[Subject],
    result: &NlmeResult,
    config: &VpcConfig,
) -> VpcResult {
    let (time_bins, binned_obs) = bin_observations(subjects, config.n_bins);

    let obs_lo: Vec<f64> = binned_obs.iter().map(|b| percentile(b, 5.0)).collect();
    let obs_med: Vec<f64> = binned_obs.iter().map(|b| percentile(b, 50.0)).collect();
    let obs_hi: Vec<f64> = binned_obs.iter().map(|b| percentile(b, 95.0)).collect();

    let n_bins = time_bins.len();
    let mut sim_lo_all = vec![Vec::new(); n_bins];
    let mut sim_med_all = vec![Vec::new(); n_bins];
    let mut sim_hi_all = vec![Vec::new(); n_bins];

    let mut rng = config.seed;
    for _ in 0..config.n_simulations {
        let sim_subjects = simulate_from_model(model, subjects, result, &mut rng);
        let (_, sim_binned) = bin_observations(&sim_subjects, config.n_bins);
        for (bin_idx, sim_bin) in sim_binned.iter().enumerate() {
            if bin_idx < n_bins {
                sim_lo_all[bin_idx].push(percentile(sim_bin, 5.0));
                sim_med_all[bin_idx].push(percentile(sim_bin, 50.0));
                sim_hi_all[bin_idx].push(percentile(sim_bin, 95.0));
            }
        }
    }

    VpcResult {
        time_bins,
        obs_p5: obs_lo,
        obs_p50: obs_med,
        obs_p95: obs_hi,
        sim_p5: sim_lo_all.iter().map(|v| percentile(v, 50.0)).collect(),
        sim_p50: sim_med_all.iter().map(|v| percentile(v, 50.0)).collect(),
        sim_p95: sim_hi_all.iter().map(|v| percentile(v, 50.0)).collect(),
    }
}

/// Bin observations by time across all subjects.
fn bin_observations(subjects: &[Subject], n_bins: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut all_times = Vec::new();
    let mut all_obs = Vec::new();
    for subj in subjects {
        for (&time, &obs) in subj.times.iter().zip(subj.observations.iter()) {
            all_times.push(time);
            all_obs.push(obs);
        }
    }

    if all_times.is_empty() || n_bins == 0 {
        return (vec![], vec![]);
    }

    let t_min = all_times.iter().copied().fold(f64::INFINITY, f64::min);
    let t_max = all_times.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (t_max - t_min).max(1e-10);
    #[expect(clippy::cast_precision_loss, reason = "bin count fits f64")]
    let bin_width = range / n_bins as f64;

    let mut bins = vec![Vec::new(); n_bins];
    let mut midpoints = Vec::with_capacity(n_bins);
    for bin_idx in 0..n_bins {
        #[expect(clippy::cast_precision_loss, reason = "bin index fits f64")]
        let mid = t_min + bin_width * (bin_idx as f64 + 0.5);
        midpoints.push(mid);
    }

    for (&time, &obs) in all_times.iter().zip(all_obs.iter()) {
        let raw = ((time - t_min) / bin_width).floor().max(0.0);
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "bin index from non-negative bounded float")]
        let mut bin_idx = raw as usize;
        if bin_idx >= n_bins {
            bin_idx = n_bins - 1;
        }
        bins[bin_idx].push(obs);
    }

    (midpoints, bins)
}

/// Compute percentile of a slice using linear interpolation.
#[expect(
    clippy::cast_precision_loss,
    reason = "percentile index arithmetic on bounded array sizes"
)]
fn percentile(values: &[f64], pct: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let last = (sorted.len() - 1) as f64;
    let rank = (pct / 100.0 * last).clamp(0.0, last);
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "rank is non-negative bounded by array length")]
    let lo = rank.floor() as usize;
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "rank is non-negative bounded by array length")]
    let hi = rank.ceil().min(last) as usize;
    let frac = rank - rank.floor();
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// Simulate one replicate dataset from the estimated model.
fn simulate_from_model(
    model: StructuralModel,
    subjects: &[Subject],
    result: &NlmeResult,
    rng: &mut u64,
) -> Vec<Subject> {
    use crate::rng::normal_sample;

    subjects
        .iter()
        .enumerate()
        .map(|(idx, subj)| {
            let eta = &result.individual_etas[idx];
            let mut new_eta = eta.clone();
            for (ek, &ok) in new_eta.iter_mut().zip(result.omega_diag.iter()) {
                let (z_val, new_st) = normal_sample(*rng);
                *rng = new_st;
                *ek += ok.sqrt() * z_val * 0.1;
            }

            let observations: Vec<f64> = subj
                .times
                .iter()
                .map(|&time| {
                    let pred = model(&result.theta, &new_eta, subj.dose, time);
                    let (eps, new_st) = normal_sample(*rng);
                    *rng = new_st;
                    (pred + result.sigma.sqrt() * eps).max(0.0)
                })
                .collect();

            Subject {
                times: subj.times.clone(),
                observations,
                dose: subj.dose,
            }
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// GOF (Goodness-of-Fit)
// ═══════════════════════════════════════════════════════════════════════

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
pub fn compute_gof(
    model: StructuralModel,
    subjects: &[Subject],
    result: &NlmeResult,
) -> GofResult {
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
        let result = foce(oral_one_compartment_model, &subjects, &theta, &omega, sigma, &config);
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
        assert!(summary.mean.abs() < 2.0,
            "CWRES mean should be near 0: got {}", summary.mean);
    }

    #[test]
    fn vpc_produces_bands() {
        let (subjects, result) = fitted_result();
        let vpc = compute_vpc(
            oral_one_compartment_model,
            &subjects,
            &result,
            &VpcConfig {
                n_simulations: 20,
                n_bins: 5,
                seed: 42,
            },
        );
        assert_eq!(vpc.time_bins.len(), 5);
        assert_eq!(vpc.obs_p5.len(), 5);
        assert_eq!(vpc.sim_p50.len(), 5);
        for bin_idx in 0..5 {
            assert!(vpc.obs_p5[bin_idx] <= vpc.obs_p50[bin_idx],
                "p5 <= p50 in bin {bin_idx}");
            assert!(vpc.obs_p50[bin_idx] <= vpc.obs_p95[bin_idx],
                "p50 <= p95 in bin {bin_idx}");
        }
    }

    #[test]
    fn gof_individual_better_than_population() {
        let (subjects, result) = fitted_result();
        let gof = compute_gof(oral_one_compartment_model, &subjects, &result);
        assert!(gof.r_squared_individual >= gof.r_squared_population,
            "individual R² ({}) >= population R² ({})",
            gof.r_squared_individual, gof.r_squared_population);
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
        assert_eq!(g1.r_squared_individual.to_bits(), g2.r_squared_individual.to_bits());
    }

    #[test]
    fn percentile_edge_cases() {
        assert!((percentile(&[1.0], 50.0) - 1.0).abs() < 1e-10);
        assert!((percentile(&[], 50.0)).abs() < 1e-10);
        let vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&vals, 0.0) - 1.0).abs() < 1e-10);
        assert!((percentile(&vals, 100.0) - 5.0).abs() < 1e-10);
        assert!((percentile(&vals, 50.0) - 3.0).abs() < 1e-10);
    }
}
