// SPDX-License-Identifier: AGPL-3.0-or-later
//! AUC trapezoidal, `find_cmax_tmax`, multiple dose superposition.

/// AUC by trapezoidal rule over `(time, concentration)` pairs.
///
/// # Panics
///
/// Panics if `times` and `concentrations` have different lengths.
///
/// ```
/// use healthspring_barracuda::pkpd::auc_trapezoidal;
///
/// let times = vec![0.0, 1.0, 2.0];
/// let concs = vec![0.0, 1.0, 0.0];
/// let auc = auc_trapezoidal(&times, &concs);
/// assert!((auc - 1.0).abs() < 1e-10); // triangle: base=2, height=1
/// ```
#[must_use]
pub fn auc_trapezoidal(times: &[f64], concentrations: &[f64]) -> f64 {
    assert_eq!(times.len(), concentrations.len());
    if times.len() < 2 {
        return 0.0;
    }
    let mut auc = 0.0;
    for i in 1..times.len() {
        let dt = times[i] - times[i - 1];
        auc += 0.5 * (concentrations[i - 1] + concentrations[i]) * dt;
    }
    auc
}

/// Find Cmax and Tmax from discrete concentration-time data.
///
/// Returns `(cmax, tmax)`. If the slice is empty, returns `(0.0, 0.0)`.
///
/// # Panics
///
/// Panics if `times` and `concentrations` have different lengths.
#[must_use]
pub fn find_cmax_tmax(times: &[f64], concentrations: &[f64]) -> (f64, f64) {
    assert_eq!(times.len(), concentrations.len());
    if concentrations.is_empty() {
        return (0.0, 0.0);
    }
    let (idx, &cmax) = concentrations
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or((0, &0.0));
    (cmax, times[idx])
}

/// Multiple dosing via superposition of a single-dose model.
///
/// Evaluates the single-dose function at each `t - n*interval` for `n_doses`
/// and sums the contributions.
pub fn pk_multiple_dose<F>(
    single_dose: F,
    interval_hr: f64,
    n_doses: usize,
    times: &[f64],
) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    times
        .iter()
        .map(|&t| {
            (0..n_doses)
                .map(|i| {
                    #[expect(clippy::cast_precision_loss, reason = "n_doses always small")]
                    let t_shifted = t - (i as f64) * interval_hr;
                    if t_shifted >= 0.0 {
                        single_dose(t_shifted)
                    } else {
                        0.0
                    }
                })
                .sum()
        })
        .collect()
}
