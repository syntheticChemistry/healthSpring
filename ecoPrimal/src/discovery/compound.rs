// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compound library batch IC50/EC50 profiling.
//!
//! Supports batch Hill dose-response fitting across compound panels (e.g.,
//! ADDRC 8,000-compound library) and selectivity profiling across kinase
//! target panels (e.g., JAK1/JAK2/JAK3/TYK2).
//!
//! References:
//! - Hill AV (1910) *J Physiol* 40:iv — dose-response equation
//! - Changelian PS et al. (2003) *Science* 302:875 — tofacitinib JAK IC50s
//! - Quintás-Cardama A et al. (2010) *Blood* 115:3109 — ruxolitinib IC50s
//! - Fridman JS et al. (2010) *J Immunol* 184:5298 — baricitinib IC50s
//! - Gonzales AJ et al. (2014) *JVPT* 37:317 — oclacitinib JAK1 selectivity

use serde::{Deserialize, Serialize};

/// IC50 profile for a single compound across one target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundProfile {
    pub name: String,
    pub ic50_nm: Vec<f64>,
    pub target_names: Vec<String>,
}

/// Target kinase profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProfile {
    pub name: String,
    pub disease_relevance: f64,
}

/// IC50 estimate from Hill fit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ic50Estimate {
    pub ic50: f64,
    pub hill_n: f64,
    pub emax: f64,
    pub r_squared: f64,
}

/// Scorecard for a single compound across all targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundScorecard {
    pub compound: String,
    pub ic50_by_target: Vec<(String, f64)>,
    pub primary_selectivity: f64,
    pub overall_rank_score: f64,
}

/// Selectivity index: ratio of off-target to on-target IC50.
///
/// SI = `IC50_off / IC50_on`
///
/// - SI > 100: highly selective
/// - SI > 10: selective
/// - SI ≈ 1: non-selective
#[must_use]
pub fn selectivity_index(ic50_on_target: f64, ic50_off_target: f64) -> f64 {
    if ic50_on_target <= 0.0 {
        return 0.0;
    }
    ic50_off_target / ic50_on_target
}

/// Estimate IC50 from an 8-point dose-response curve via Hill fit.
///
/// Uses bisection on the Hill equation: `R = Emax × C^n / (IC50^n + C^n)`.
/// Given `concentrations` and `responses` (fraction of max, 0–1), estimates
/// IC50 as the concentration giving 50% response.
///
/// The Hill coefficient `n` is estimated from the slope at IC50. `Emax` is
/// taken as the maximum observed response.
#[must_use]
pub fn estimate_ic50(concentrations: &[f64], responses: &[f64]) -> Ic50Estimate {
    if concentrations.len() < 2 || concentrations.len() != responses.len() {
        return Ic50Estimate {
            ic50: f64::NAN,
            hill_n: f64::NAN,
            emax: f64::NAN,
            r_squared: 0.0,
        };
    }

    let emax = responses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let half_max = emax * 0.5;

    let mut ic50 = f64::NAN;
    for w in concentrations.windows(2).zip(responses.windows(2)) {
        let (conc_pair, resp_pair) = w;
        let (c0, c1) = (conc_pair[0], conc_pair[1]);
        let (r0, r1) = (resp_pair[0], resp_pair[1]);
        if (r0 <= half_max && r1 >= half_max) || (r0 >= half_max && r1 <= half_max) {
            let frac = if (r1 - r0).abs() < 1e-15 {
                0.5
            } else {
                (half_max - r0) / (r1 - r0)
            };
            ic50 = c0 + frac * (c1 - c0);
            break;
        }
    }

    let hill_n = if ic50.is_finite() && ic50 > 0.0 {
        estimate_hill_coefficient(concentrations, responses, ic50, emax)
    } else {
        1.0
    };

    let r_squared = if ic50.is_finite() {
        compute_r_squared(concentrations, responses, ic50, hill_n, emax)
    } else {
        0.0
    };

    Ic50Estimate {
        ic50,
        hill_n,
        emax,
        r_squared,
    }
}

/// Estimate Hill coefficient from the slope at IC50.
fn estimate_hill_coefficient(
    concentrations: &[f64],
    responses: &[f64],
    ic50: f64,
    emax: f64,
) -> f64 {
    let mut best_n = 1.0;
    let mut best_sse = f64::MAX;

    for n_candidate_x10 in 5..=40 {
        let n = f64::from(n_candidate_x10) / 10.0;
        let sse: f64 = concentrations
            .iter()
            .zip(responses.iter())
            .map(|(&c, &r)| {
                let predicted = emax * c.powf(n) / (ic50.powf(n) + c.powf(n));
                (r - predicted).powi(2)
            })
            .sum();
        if sse < best_sse {
            best_sse = sse;
            best_n = n;
        }
    }
    best_n
}

/// Compute R² for Hill fit quality.
fn compute_r_squared(
    concentrations: &[f64],
    responses: &[f64],
    ic50: f64,
    hill_n: f64,
    emax: f64,
) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "response count ≪ 2^52")]
    let mean_r = responses.iter().sum::<f64>() / responses.len() as f64;
    let ss_tot: f64 = responses.iter().map(|&r| (r - mean_r).powi(2)).sum();
    let ss_res: f64 = concentrations
        .iter()
        .zip(responses.iter())
        .map(|(&c, &r)| {
            let predicted = emax * c.powf(hill_n) / (ic50.powf(hill_n) + c.powf(hill_n));
            (r - predicted).powi(2)
        })
        .sum();

    if ss_tot < 1e-15 {
        return 0.0;
    }
    1.0 - ss_res / ss_tot
}

/// Batch IC50 sweep across a compound panel for a single target.
///
/// Each compound's dose-response data yields an IC50 estimate. The panel is
/// then ranked by selectivity against the primary target.
#[must_use]
pub fn batch_ic50_sweep(
    compounds: &[CompoundProfile],
    primary_target_idx: usize,
) -> Vec<CompoundScorecard> {
    compounds
        .iter()
        .map(|cp| {
            let primary_ic50 = cp
                .ic50_nm
                .get(primary_target_idx)
                .copied()
                .unwrap_or(f64::INFINITY);

            let ic50_by_target: Vec<(String, f64)> = cp
                .target_names
                .iter()
                .zip(cp.ic50_nm.iter())
                .map(|(t, &ic)| (t.clone(), ic))
                .collect();

            let min_off_target = cp
                .ic50_nm
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != primary_target_idx)
                .map(|(_, &v)| v)
                .fold(f64::INFINITY, f64::min);

            let sel = selectivity_index(primary_ic50, min_off_target);

            CompoundScorecard {
                compound: cp.name.clone(),
                ic50_by_target,
                primary_selectivity: sel,
                overall_rank_score: sel * (1.0 / (1.0 + primary_ic50.ln())),
            }
        })
        .collect()
}

/// Rank compounds by selectivity (descending).
pub fn rank_by_selectivity(scorecards: &mut [CompoundScorecard]) {
    scorecards.sort_by(|a, b| {
        b.primary_selectivity
            .partial_cmp(&a.primary_selectivity)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn selectivity_index_selective() {
        let si = selectivity_index(5.0, 500.0);
        assert!((si - 100.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn selectivity_index_nonselective() {
        let si = selectivity_index(10.0, 10.0);
        assert!((si - 1.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn selectivity_index_zero_on_target() {
        assert_eq!(selectivity_index(0.0, 100.0), 0.0);
    }

    #[test]
    fn estimate_ic50_from_hill_curve() {
        let ic50_true: f64 = 10.0;
        let n: f64 = 1.0;
        let concs: Vec<f64> = (0..8)
            .map(|i| 0.1 * 10.0_f64.powf(0.5 * f64::from(i)))
            .collect();
        let responses: Vec<f64> = concs
            .iter()
            .map(|&c| c.powf(n) / (ic50_true.powf(n) + c.powf(n)))
            .collect();

        let est = estimate_ic50(&concs, &responses);
        assert!(est.ic50.is_finite());
        assert!((est.ic50 - ic50_true).abs() / ic50_true < 0.1);
        assert!(est.r_squared > 0.95);
    }

    #[test]
    fn estimate_ic50_insufficient_data() {
        let est = estimate_ic50(&[1.0], &[0.5]);
        assert!(est.ic50.is_nan());
    }

    #[test]
    fn batch_sweep_ordering() {
        let compounds = vec![
            CompoundProfile {
                name: "selective".into(),
                ic50_nm: vec![1.0, 1000.0],
                target_names: vec!["JAK1".into(), "JAK2".into()],
            },
            CompoundProfile {
                name: "nonselective".into(),
                ic50_nm: vec![500.0, 600.0],
                target_names: vec!["JAK1".into(), "JAK2".into()],
            },
        ];
        let mut scores = batch_ic50_sweep(&compounds, 0);
        rank_by_selectivity(&mut scores);
        assert_eq!(scores[0].compound, "selective");
    }
}
