// SPDX-License-Identifier: AGPL-3.0-or-later
//! Non-Compartmental Analysis (NCA) — sovereign replacement for Phoenix `WinNonlin`.
//!
//! Computes standard NCA metrics from concentration-time data:
//! - `lambda_z`: terminal elimination rate constant via log-linear regression
//! - AUC(0-t), AUC(0-inf), AUC extrapolation percentage
//! - `t_half`: terminal half-life
//! - MRT (mean residence time)
//! - `Vss`, `CLss` (steady-state volume and clearance)
//!
//! All algorithms follow FDA Guidance for Industry: Bioanalytical Method
//! Validation and the standard NCA methodology described in Gabrielsson
//! and Weiner, "Pharmacokinetic and Pharmacodynamic Data Analysis."

use super::util::{auc_trapezoidal, find_cmax_tmax};
use crate::tolerances;

/// Complete NCA result for a single concentration-time profile.
#[derive(Debug, Clone)]
pub struct NcaResult {
    /// Maximum observed concentration.
    pub cmax: f64,
    /// Time of Cmax.
    pub tmax: f64,
    /// AUC from first time to last observed time (trapezoidal).
    pub auc_last: f64,
    /// AUC extrapolated to infinity (includes terminal tail when λz > 0).
    pub auc_inf: f64,
    /// Percentage of AUC(0–∞) from extrapolation beyond last sample.
    pub auc_extrap_pct: f64,
    /// Terminal elimination rate constant from log-linear regression.
    pub lambda_z: f64,
    /// Terminal half-life (ln 2 / λz).
    pub half_life: f64,
    /// AUMC to last observed time.
    pub aumc_last: f64,
    /// AUMC extrapolated to infinity.
    pub aumc_inf: f64,
    /// Mean residence time (AUMC / AUC).
    pub mrt: f64,
    /// Observed clearance (dose / AUC(0–∞)).
    pub cl_obs: f64,
    /// Observed steady-state volume (CL × MRT).
    pub vss_obs: f64,
    /// Number of concentration points used in λz regression.
    pub n_terminal_points: usize,
    /// Goodness of fit (R²) of the terminal log-linear phase.
    pub r_squared: f64,
}

/// Lambda-z regression result from log-linear terminal phase.
#[derive(Debug, Clone, Copy)]
struct TerminalFit {
    lambda_z: f64,
    r_squared: f64,
    n_points: usize,
}

/// Estimate terminal elimination rate constant (`lambda_z`) by log-linear
/// regression on the terminal decline phase.
///
/// Selects the best-fit subset of at least `min_points` terminal points
/// that maximizes R-squared.
fn estimate_lambda_z(times: &[f64], concentrations: &[f64], min_points: usize) -> TerminalFit {
    let len = times.len();
    let empty = TerminalFit {
        lambda_z: 0.0,
        r_squared: 0.0,
        n_points: 0,
    };
    if len < min_points {
        return empty;
    }

    let cmax_idx = concentrations
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(core::cmp::Ordering::Equal))
        .map_or(0, |(i, _)| i);

    let terminal_start = cmax_idx + 1;
    let terminal_n = len - terminal_start;
    if terminal_n < min_points {
        return empty;
    }

    let mut best = TerminalFit {
        lambda_z: 0.0,
        r_squared: -1.0,
        n_points: 0,
    };

    for start in terminal_start..=(len - min_points) {
        let pts = len - start;
        if pts < min_points {
            break;
        }
        let t_slice = &times[start..];
        let c_slice = &concentrations[start..];

        if c_slice.iter().any(|&c| c <= 0.0) {
            continue;
        }

        let result = log_linear_regression(t_slice, c_slice);
        if result.r_squared > best.r_squared && result.lambda_z > 0.0 {
            best = result;
        }
    }

    best
}

/// Slope and R-squared from linear regression of ln(C) vs time.
///
/// Returns `lambda_z` = −slope (positive for elimination).
fn log_linear_regression(times: &[f64], concentrations: &[f64]) -> TerminalFit {
    let len = times.len();
    if len < 2 {
        return TerminalFit {
            lambda_z: 0.0,
            r_squared: 0.0,
            n_points: len,
        };
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "regression point count fits f64"
    )]
    let nf = len as f64;
    let ln_c: Vec<f64> = concentrations.iter().map(|&c| c.ln()).collect();

    let sum_t: f64 = times.iter().sum();
    let sum_lnc: f64 = ln_c.iter().sum();
    let sum_t2: f64 = times.iter().map(|&t| t * t).sum();
    let sum_t_lnc: f64 = times.iter().zip(ln_c.iter()).map(|(&t, &lc)| t * lc).sum();

    let denom = nf.mul_add(sum_t2, -(sum_t * sum_t));
    if denom.abs() < tolerances::DIVISION_GUARD {
        return TerminalFit {
            lambda_z: 0.0,
            r_squared: 0.0,
            n_points: len,
        };
    }

    let slope = nf.mul_add(sum_t_lnc, -(sum_t * sum_lnc)) / denom;
    let intercept = slope.mul_add(-sum_t, sum_lnc) / nf;
    let mean_lnc = sum_lnc / nf;

    let ss_tot: f64 = ln_c.iter().map(|&lc| (lc - mean_lnc).powi(2)).sum();
    let ss_res: f64 = times
        .iter()
        .zip(ln_c.iter())
        .map(|(&t, &lc)| slope.mul_add(-t, lc - intercept).powi(2))
        .sum();

    let r_squared = if ss_tot > tolerances::DIVISION_GUARD {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    TerminalFit {
        lambda_z: -slope,
        r_squared,
        n_points: len,
    }
}

/// AUMC (area under the first moment curve) by trapezoidal rule.
///
/// AUMC = integral of t × C(t) dt.
///
/// # Panics
///
/// Panics if `times` and `concentrations` have different lengths.
#[must_use]
pub fn aumc_trapezoidal(times: &[f64], concentrations: &[f64]) -> f64 {
    assert_eq!(times.len(), concentrations.len());
    if times.len() < 2 {
        return 0.0;
    }
    let mut aumc = 0.0;
    for i in 1..times.len() {
        let dt = times[i] - times[i - 1];
        let tc_prev = times[i - 1] * concentrations[i - 1];
        let tc_curr = times[i] * concentrations[i];
        aumc = (0.5 * dt).mul_add(tc_prev + tc_curr, aumc);
    }
    aumc
}

/// Perform full non-compartmental analysis on a concentration-time profile.
///
/// `dose` is the administered dose (in consistent units with concentrations).
/// `min_terminal_points` is the minimum number of points for `lambda_z`
/// regression (typically 3).
///
/// # Panics
///
/// Panics if `times` and `concentrations` have different lengths.
#[must_use]
pub fn nca_iv(
    times: &[f64],
    concentrations: &[f64],
    dose: f64,
    min_terminal_points: usize,
) -> NcaResult {
    assert_eq!(times.len(), concentrations.len());

    let (cmax, tmax) = find_cmax_tmax(times, concentrations);
    let area_last = auc_trapezoidal(times, concentrations);
    let moment_last = aumc_trapezoidal(times, concentrations);

    let fit = estimate_lambda_z(times, concentrations, min_terminal_points);

    let (area_inf, extrap_pct, moment_inf) = if fit.lambda_z > 0.0 {
        let c_last = *concentrations.last().unwrap_or(&0.0);
        let t_last = *times.last().unwrap_or(&0.0);
        let tail = c_last / fit.lambda_z;
        let total = area_last + tail;
        let pct = if total > 0.0 {
            100.0 * tail / total
        } else {
            0.0
        };
        let moment_tail = c_last * t_last / fit.lambda_z + c_last / (fit.lambda_z * fit.lambda_z);
        (total, pct, moment_last + moment_tail)
    } else {
        (area_last, 0.0, moment_last)
    };

    let half_life = if fit.lambda_z > 0.0 {
        core::f64::consts::LN_2 / fit.lambda_z
    } else {
        f64::INFINITY
    };

    let mrt = if area_inf > 0.0 {
        moment_inf / area_inf
    } else {
        0.0
    };

    let cl_obs = if area_inf > 0.0 { dose / area_inf } else { 0.0 };

    let vss_obs = cl_obs * mrt;

    NcaResult {
        cmax,
        tmax,
        auc_last: area_last,
        auc_inf: area_inf,
        auc_extrap_pct: extrap_pct,
        lambda_z: fit.lambda_z,
        half_life,
        aumc_last: moment_last,
        aumc_inf: moment_inf,
        mrt,
        cl_obs,
        vss_obs,
        n_terminal_points: fit.n_points,
        r_squared: fit.r_squared,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[expect(clippy::cast_precision_loss, reason = "test profile indices fit f64")]
    fn iv_profile(
        dose: f64,
        vd: f64,
        ke: f64,
        n_points: usize,
        t_max: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let last = (n_points - 1) as f64;
        let times: Vec<f64> = (0..n_points).map(|i| t_max * (i as f64) / last).collect();
        let c0 = dose / vd;
        let concs: Vec<f64> = times.iter().map(|&t| c0 * (-ke * t).exp()).collect();
        (times, concs)
    }

    #[test]
    fn nca_iv_bolus_basic() {
        let dose = 100.0;
        let vd = 10.0;
        let ke = 0.1;
        let (times, concs) = iv_profile(dose, vd, ke, 1000, 48.0);

        let result = nca_iv(&times, &concs, dose, 3);

        assert!(
            (result.cmax - dose / vd).abs() < tolerances::NCA_TOLERANCE,
            "Cmax = Dose/Vd"
        );
        assert!(
            result.tmax.abs() < tolerances::NCA_TOLERANCE,
            "Tmax = 0 for IV bolus"
        );
        assert!(result.lambda_z > 0.0, "lambda_z estimated");
        assert!(
            (result.lambda_z - ke).abs() < tolerances::TEST_ASSERTION_LOOSE,
            "lambda_z ~ ke: got {}",
            result.lambda_z
        );
        let expected_half = core::f64::consts::LN_2 / ke;
        assert!(
            (result.half_life - expected_half).abs() < tolerances::TMAX_NUMERICAL,
            "t½ ~ ln2/ke"
        );
    }

    #[test]
    fn nca_iv_bolus_auc_analytical() {
        let dose = 100.0;
        let vd = 10.0;
        let ke = 0.1;
        let analytical_auc = dose / (vd * ke);

        let (times, concs) = iv_profile(dose, vd, ke, 2000, 100.0);
        let result = nca_iv(&times, &concs, dose, 3);

        let rel_err = (result.auc_inf - analytical_auc).abs() / analytical_auc;
        assert!(
            rel_err < tolerances::TEST_ASSERTION_LOOSE,
            "AUC(0-inf) within 1%: got {}, expected {}",
            result.auc_inf,
            analytical_auc
        );
    }

    #[test]
    fn nca_iv_bolus_clearance() {
        let dose = 100.0;
        let vd = 10.0;
        let ke = 0.1;
        let cl_expected = vd * ke;

        let (times, concs) = iv_profile(dose, vd, ke, 2000, 100.0);
        let result = nca_iv(&times, &concs, dose, 3);

        let rel_err = (result.cl_obs - cl_expected).abs() / cl_expected;
        assert!(
            rel_err < tolerances::TEST_ASSERTION_2_PERCENT,
            "CL within 2% of analytical"
        );
    }

    #[test]
    fn nca_iv_bolus_vss() {
        let dose = 100.0;
        let vd = 10.0;
        let ke = 0.1;

        let (times, concs) = iv_profile(dose, vd, ke, 2000, 100.0);
        let result = nca_iv(&times, &concs, dose, 3);

        let rel_err = (result.vss_obs - vd).abs() / vd;
        assert!(
            rel_err < tolerances::POP_VD_MEDIAN,
            "Vss within 5% of Vd for 1-comp model"
        );
    }

    #[test]
    fn nca_r_squared_good() {
        let (times, concs) = iv_profile(100.0, 10.0, 0.1, 1000, 48.0);
        let result = nca_iv(&times, &concs, 100.0, 3);
        assert!(
            result.r_squared > 0.99,
            "R² > 0.99: got {}",
            result.r_squared
        );
    }

    #[test]
    fn nca_extrapolation_reasonable() {
        let (times, concs) = iv_profile(100.0, 10.0, 0.1, 500, 24.0);
        let result = nca_iv(&times, &concs, 100.0, 3);
        assert!(
            result.auc_extrap_pct > 0.0 && result.auc_extrap_pct < 30.0,
            "AUC extrapolation 0-30%: got {:.1}%",
            result.auc_extrap_pct
        );
    }

    #[test]
    fn nca_mrt_one_compartment() {
        let ke = 0.1;
        let mrt_expected = 1.0 / ke;
        let (times, concs) = iv_profile(100.0, 10.0, ke, 2000, 100.0);
        let result = nca_iv(&times, &concs, 100.0, 3);

        let rel_err = (result.mrt - mrt_expected).abs() / mrt_expected;
        assert!(
            rel_err < tolerances::POP_VD_MEDIAN,
            "MRT within 5%: got {}, expected {}",
            result.mrt,
            mrt_expected
        );
    }

    #[test]
    fn nca_deterministic() {
        let (times, concs) = iv_profile(100.0, 10.0, 0.1, 500, 48.0);
        let r1 = nca_iv(&times, &concs, 100.0, 3);
        let r2 = nca_iv(&times, &concs, 100.0, 3);
        assert_eq!(r1.lambda_z.to_bits(), r2.lambda_z.to_bits());
        assert_eq!(r1.auc_inf.to_bits(), r2.auc_inf.to_bits());
        assert_eq!(r1.cl_obs.to_bits(), r2.cl_obs.to_bits());
    }

    #[test]
    fn aumc_trapezoidal_basic() {
        let times = [0.0, 1.0, 2.0];
        let concs = [1.0, 1.0, 1.0];
        let aumc = super::aumc_trapezoidal(&times, &concs);
        assert!(
            (aumc - 2.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "AUMC of constant C=1 over [0,2]: integral of t dt = 2"
        );
    }
}
