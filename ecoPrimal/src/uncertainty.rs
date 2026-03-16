// SPDX-License-Identifier: AGPL-3.0-or-later
//! Uncertainty quantification — bootstrap, jackknife, and bias–variance decomposition.
//!
//! Absorbed from groundSpring's measurement-science patterns and adapted for
//! healthSpring's PK/PD, microbiome, and biosignal domains.
//!
//! ## Components
//!
//! - **Bootstrap**: percentile confidence intervals for mean, median, std
//! - **Jackknife**: delete-one variance estimation and bias correction
//! - **Decomposition**: RMSE → bias² + variance partitioning
//!
//! All functions are pure, deterministic (given a seed), and `#[must_use]`.

use crate::rng::normal_sample;

/// Bootstrap confidence interval result.
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    /// Point estimate (mean/median/std of original data).
    pub estimate: f64,
    /// Lower bound of the confidence interval.
    pub ci_lower: f64,
    /// Upper bound of the confidence interval.
    pub ci_upper: f64,
    /// Bootstrap standard error.
    pub std_error: f64,
}

/// Jackknife variance result.
#[derive(Debug, Clone)]
pub struct JackknifeResult {
    /// Full-sample estimate.
    pub estimate: f64,
    /// Jackknife variance of the estimate.
    pub variance: f64,
    /// Jackknife standard error.
    pub std_error: f64,
    /// Bias estimate: `(n-1) * (mean_of_leave_one_out - full_estimate)`.
    pub bias: f64,
}

/// Bias–variance decomposition of RMSE.
#[derive(Debug, Clone)]
pub struct Decomposition {
    /// Mean bias error.
    pub bias: f64,
    /// Absolute bias.
    pub bias_abs: f64,
    /// Random error standard deviation.
    pub random_std: f64,
    /// Total RMSE.
    pub total_rmse: f64,
    /// Bias² component.
    pub bias_sq: f64,
    /// Variance (random²) component.
    pub variance: f64,
    /// Fraction of MSE attributable to bias.
    pub bias_fraction: f64,
    /// Fraction of MSE attributable to noise.
    pub noise_fraction: f64,
}

const fn lcg_step(state: u64) -> u64 {
    state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1)
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "resampling index arithmetic on bounded RNG output"
)]
fn resample_index(rng: &mut u64, n: usize) -> usize {
    *rng = lcg_step(*rng);
    ((*rng >> 33) as f64 / (1u64 << 31) as f64 * n as f64) as usize % n
}

fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / crate::validation::len_f64(data.len())
}

/// Percentile bootstrap confidence interval for the mean.
///
/// # Panics
///
/// Panics if `data` is empty.
#[must_use]
pub fn bootstrap_mean(data: &[f64], n_replicates: usize, confidence: f64, seed: u64) -> BootstrapResult {
    assert!(!data.is_empty(), "bootstrap requires non-empty data");
    let estimate = mean(data);
    let n = data.len();
    let mut rng = seed;
    let mut estimates = Vec::with_capacity(n_replicates);

    for _ in 0..n_replicates {
        let mut sum = 0.0;
        for _ in 0..n {
            sum += data[resample_index(&mut rng, n)];
        }
        estimates.push(sum / crate::validation::len_f64(n));
    }

    estimates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let alpha = (1.0 - confidence) / 2.0;
    let ci = percentile_pair(&estimates, alpha);
    let se = std_dev(&estimates);

    BootstrapResult {
        estimate,
        ci_lower: ci.0,
        ci_upper: ci.1,
        std_error: se,
    }
}

/// Percentile bootstrap confidence interval for the median.
///
/// # Panics
///
/// Panics if `data` is empty.
#[must_use]
pub fn bootstrap_median(data: &[f64], n_replicates: usize, confidence: f64, seed: u64) -> BootstrapResult {
    assert!(!data.is_empty(), "bootstrap requires non-empty data");
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let estimate = sorted[sorted.len() / 2];
    let n = data.len();
    let mut rng = seed;
    let mut estimates = Vec::with_capacity(n_replicates);

    for _ in 0..n_replicates {
        let mut resample: Vec<f64> = (0..n).map(|_| data[resample_index(&mut rng, n)]).collect();
        resample.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        estimates.push(resample[resample.len() / 2]);
    }

    estimates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let alpha = (1.0 - confidence) / 2.0;
    let ci = percentile_pair(&estimates, alpha);
    let se = std_dev(&estimates);

    BootstrapResult {
        estimate,
        ci_lower: ci.0,
        ci_upper: ci.1,
        std_error: se,
    }
}

/// Delete-one jackknife variance estimation for the mean.
///
/// # Panics
///
/// Panics if `data` has fewer than 2 elements.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "jackknife n fits f64")]
pub fn jackknife_mean_variance(data: &[f64]) -> JackknifeResult {
    assert!(data.len() >= 2, "jackknife requires at least 2 observations");
    let n = data.len();
    let n_f = n as f64;
    let full_mean = mean(data);
    let total_sum: f64 = data.iter().sum();

    let leave_one_means: Vec<f64> = data
        .iter()
        .map(|&xi| (total_sum - xi) / (n_f - 1.0))
        .collect();

    let jk_mean = mean(&leave_one_means);
    let jk_var = leave_one_means
        .iter()
        .map(|&m| (m - jk_mean).powi(2))
        .sum::<f64>()
        * (n_f - 1.0) / n_f;
    let bias = (n_f - 1.0) * (jk_mean - full_mean);

    JackknifeResult {
        estimate: full_mean,
        variance: jk_var,
        std_error: jk_var.sqrt(),
        bias,
    }
}

/// Decompose RMSE into bias² and variance components.
///
/// Given `MBE` (mean bias error) and `RMSE`:
/// - `RMSE² = MBE² + σ²(random)`
/// - `bias_fraction = MBE² / RMSE²`
#[must_use]
pub fn decompose_error(mbe: f64, rmse: f64) -> Decomposition {
    let rmse_sq = rmse * rmse;
    let bias_sq = mbe * mbe;
    let variance = (rmse_sq - bias_sq).max(0.0);
    let bias_frac = if rmse_sq > 1e-30 { bias_sq / rmse_sq } else { 0.0 };

    Decomposition {
        bias: mbe,
        bias_abs: mbe.abs(),
        random_std: variance.sqrt(),
        total_rmse: rmse,
        bias_sq,
        variance,
        bias_fraction: bias_frac,
        noise_fraction: 1.0 - bias_frac,
    }
}

/// Mean Bias Error between observed and modeled values.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn mbe(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(observed.len(), modeled.len());
    if observed.is_empty() {
        return 0.0;
    }
    let sum: f64 = observed.iter().zip(modeled).map(|(o, m)| m - o).sum();
    sum / crate::validation::len_f64(observed.len())
}

/// Monte Carlo uncertainty propagation (generic).
///
/// Perturbs the `base_params` by adding Gaussian noise with the given
/// `sigma` vector, runs `model_fn` for each draw, and returns the mean
/// and standard deviation of the outputs.
///
/// # Panics
///
/// Panics if `base_params` and `sigmas` have different lengths.
#[must_use]
pub fn monte_carlo_propagate(
    base_params: &[f64],
    sigmas: &[f64],
    n_draws: usize,
    seed: u64,
    model_fn: impl Fn(&[f64]) -> f64,
) -> (f64, f64) {
    assert_eq!(base_params.len(), sigmas.len());
    let mut rng_state = seed;
    let mut results = Vec::with_capacity(n_draws);
    let mut perturbed = vec![0.0; base_params.len()];

    for _ in 0..n_draws {
        for (i, (&base, &sigma)) in base_params.iter().zip(sigmas).enumerate() {
            let (z, new_state) = normal_sample(rng_state);
            rng_state = new_state;
            perturbed[i] = sigma.mul_add(z, base);
        }
        results.push(model_fn(&perturbed));
    }

    let m = mean(&results);
    let sd = std_dev(&results);
    (m, sd)
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "percentile index arithmetic on bounded non-negative floats"
)]
fn percentile_pair(sorted: &[f64], alpha: f64) -> (f64, f64) {
    let n = sorted.len();
    let lo_idx = (alpha * n as f64).floor().max(0.0) as usize;
    let hi_idx = ((1.0 - alpha) * n as f64).ceil().min(n as f64 - 1.0).max(0.0) as usize;
    (sorted[lo_idx], sorted[hi_idx])
}

fn std_dev(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let var = data.iter().map(|&x| (x - m).powi(2)).sum::<f64>()
        / crate::validation::len_f64(data.len() - 1);
    var.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_mean_contains_true_value() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = bootstrap_mean(&data, 2000, 0.95, 42);
        assert!((result.estimate - 5.5).abs() < 1e-10);
        assert!(result.ci_lower <= 5.5, "CI lower {}", result.ci_lower);
        assert!(result.ci_upper >= 5.5, "CI upper {}", result.ci_upper);
        assert!(result.std_error > 0.0);
    }

    #[test]
    fn bootstrap_median_reasonable() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let result = bootstrap_median(&data, 1000, 0.95, 123);
        assert!((result.estimate - 3.0).abs() < 1e-10);
        assert!(result.ci_lower <= result.ci_upper);
    }

    #[test]
    fn jackknife_variance_positive() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let result = jackknife_mean_variance(&data);
        assert!((result.estimate - 3.5).abs() < 1e-10);
        assert!(result.variance > 0.0);
        assert!(result.std_error > 0.0);
    }

    #[test]
    fn jackknife_bias_near_zero_for_mean() {
        let data = [10.0, 20.0, 30.0, 40.0];
        let result = jackknife_mean_variance(&data);
        assert!(
            result.bias.abs() < 1e-10,
            "jackknife bias for mean should be ~0: {}",
            result.bias
        );
    }

    #[test]
    fn decompose_pure_bias() {
        let d = decompose_error(5.0, 5.0);
        assert!((d.bias_fraction - 1.0).abs() < 1e-10);
        assert!(d.random_std.abs() < 1e-10);
    }

    #[test]
    fn decompose_pure_noise() {
        let d = decompose_error(0.0, 3.0);
        assert!(d.bias_fraction.abs() < 1e-10);
        assert!((d.noise_fraction - 1.0).abs() < 1e-10);
        assert!((d.random_std - 3.0).abs() < 1e-10);
    }

    #[test]
    fn decompose_mixed() {
        let d = decompose_error(3.0, 5.0);
        assert!((d.bias_sq - 9.0).abs() < 1e-10);
        assert!((d.variance - 16.0).abs() < 1e-10);
        assert!((d.random_std - 4.0).abs() < 1e-10);
        assert!((d.bias_fraction - 0.36).abs() < 1e-10);
    }

    #[test]
    fn mbe_positive_bias() {
        let obs = [1.0, 2.0, 3.0];
        let model = [2.0, 3.0, 4.0];
        assert!((mbe(&obs, &model) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mbe_unbiased() {
        let obs = [1.0, 2.0, 3.0];
        let model = [1.0, 2.0, 3.0];
        assert!(mbe(&obs, &model).abs() < 1e-10);
    }

    #[test]
    fn monte_carlo_propagate_identity() {
        let base = [10.0, 20.0];
        let sigmas = [0.0, 0.0];
        let (m, sd) = monte_carlo_propagate(&base, &sigmas, 100, 42, |p| p[0] + p[1]);
        assert!((m - 30.0).abs() < 1e-10, "zero noise → exact result");
        assert!(sd.abs() < 1e-10, "zero noise → zero std");
    }

    #[test]
    fn monte_carlo_propagate_with_noise() {
        let base = [100.0];
        let sigmas = [10.0];
        let (m, sd) = monte_carlo_propagate(&base, &sigmas, 10_000, 42, |p| p[0]);
        assert!(
            (m - 100.0).abs() < 5.0,
            "mean should be near base: {m}"
        );
        assert!(
            sd > 5.0 && sd < 20.0,
            "std should be in reasonable range of sigma: {sd}"
        );
    }
}
