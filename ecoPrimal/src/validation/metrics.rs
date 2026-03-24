// SPDX-License-Identifier: AGPL-3.0-or-later
//! Length helpers and forecast–observation error metrics.

use core::fmt;

use crate::tolerances;

/// Convert a `usize` length to `f64` with an explicit precision-loss acknowledgement.
///
/// Useful at IPC boundaries where collection lengths appear in arithmetic.
#[expect(clippy::cast_precision_loss, reason = "collection size ≪ 2^52")]
#[must_use]
#[inline]
pub const fn len_f64(n: usize) -> f64 {
    n as f64
}

/// Length mismatch error for metric functions.
#[derive(Debug, Clone, Copy)]
pub struct LengthMismatch {
    /// Length of the observed slice.
    pub observed: usize,
    /// Length of the predicted slice.
    pub predicted: usize,
}

impl fmt::Display for LengthMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "length mismatch: observed={}, predicted={}",
            self.observed, self.predicted
        )
    }
}

impl std::error::Error for LengthMismatch {}

/// Verify equal lengths, returning `Err` on mismatch.
const fn check_lengths(observed: &[f64], predicted: &[f64]) -> Result<usize, LengthMismatch> {
    if observed.len() != predicted.len() {
        return Err(LengthMismatch {
            observed: observed.len(),
            predicted: predicted.len(),
        });
    }
    Ok(observed.len())
}

/// Root Mean Square Error between observed and predicted.
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn rmse(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let sum_sq: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    Ok((sum_sq / n as f64).sqrt())
}

/// Mean Absolute Error.
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn mae(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let sum_abs: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).abs())
        .sum();
    Ok(sum_abs / n as f64)
}

/// Nash-Sutcliffe Efficiency (1.0 = perfect, 0.0 = mean model, negative = worse).
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn nse(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
    let ss_res: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    let ss_tot: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();
    if ss_tot < tolerances::MACHINE_EPSILON_STRICT {
        return Ok(1.0);
    }
    Ok(1.0 - ss_res / ss_tot)
}

/// Coefficient of determination (R-squared).
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
pub fn r_squared(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    nse(observed, predicted)
}

/// Willmott Index of Agreement (0.0 to 1.0).
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn index_of_agreement(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
    let ss_res: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    let ss_pot: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| ((p - mean_obs).abs() + (o - mean_obs).abs()).powi(2))
        .sum();
    if ss_pot < tolerances::MACHINE_EPSILON_STRICT {
        return Ok(1.0);
    }
    Ok(1.0 - ss_res / ss_pot)
}

#[cfg(test)]
mod tests {
    use crate::tolerances;

    use super::{index_of_agreement, mae, nse, rmse};

    #[test]
    fn rmse_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(rmse(&data, &data).expect("same length") < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn mae_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(mae(&data, &data).expect("same length") < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn nse_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!((nse(&data, &data).expect("same length") - 1.0).abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn index_of_agreement_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(
            (index_of_agreement(&data, &data).expect("same length") - 1.0).abs()
                < tolerances::DIVISION_GUARD
        );
    }

    #[test]
    fn rmse_length_mismatch() {
        assert!(rmse(&[1.0, 2.0], &[1.0]).is_err());
    }

    #[test]
    fn mae_length_mismatch() {
        assert!(mae(&[1.0, 2.0], &[1.0]).is_err());
    }
}
