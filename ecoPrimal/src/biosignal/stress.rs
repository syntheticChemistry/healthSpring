// SPDX-License-Identifier: AGPL-3.0-or-later
//! Autonomic stress detection from EDA and HRV.
//!
//! Combines skin conductance (SCR frequency, SCL level) with HRV
//! metrics to produce a composite autonomic stress index.
//!
//! Reference: Boucsein 2012, *Electrodermal Activity*. 2nd ed.
//! Braithwaite et al. 2013, EDA guidelines.

/// Composite autonomic stress assessment.
#[derive(Debug, Clone)]
pub struct StressAssessment {
    /// SCR events per minute
    pub scr_rate: f64,
    /// Mean tonic SCL (µS)
    pub mean_scl: f64,
    /// Mean SCR recovery half-time (seconds)
    pub mean_recovery_s: f64,
    /// Autonomic stress index (0–100)
    pub stress_index: f64,
}

/// Compute SCR rate (events per minute).
///
/// Delegates to `barracuda::health::biosignal::scr_rate` — identical signature.
#[must_use]
pub fn scr_rate(n_scr_events: usize, duration_s: f64) -> f64 {
    crate::math_dispatch::scr_rate(n_scr_events, duration_s)
}

/// Compute mean SCR recovery half-time from phasic EDA signal.
///
/// For each SCR peak, measures time to decay to 50% of peak amplitude.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "sample indices fit f64 mantissa"
)]
pub fn scr_recovery_time(phasic: &[f64], peaks: &[usize], fs: f64) -> f64 {
    if peaks.is_empty() || fs <= 0.0 {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut count = 0;
    for &pk in peaks {
        let half_amp = phasic[pk] * 0.5;
        let mut recovery_idx = pk;
        for (i, &val) in phasic.iter().enumerate().skip(pk + 1) {
            if val <= half_amp {
                recovery_idx = i;
                break;
            }
        }
        if recovery_idx > pk {
            sum += (recovery_idx - pk) as f64 / fs;
            count += 1;
        }
    }
    if count > 0 {
        sum / f64::from(count)
    } else {
        0.0
    }
}

/// Composite autonomic stress index from EDA features.
///
/// Combines SCR frequency, SCL level, and recovery time into a 0–100 score.
/// Higher = more stressed.
///
/// Norms (Boucsein 2012):
/// - Resting SCR rate: 1-3/min (low stress), >5/min (high stress)
/// - Resting SCL: 2-4 µS (normal), >6 µS (high arousal)
/// - Recovery half-time: <2s (normal), >4s (sustained stress)
#[must_use]
pub fn compute_stress_index(scr_rate_per_min: f64, mean_scl: f64, recovery_s: f64) -> f64 {
    let scr_part = sigmoid_scale(scr_rate_per_min, 3.0, 1.5) * 40.0;
    let tonic_part = sigmoid_scale(mean_scl, 4.0, 1.5) * 30.0;
    let recovery_part = sigmoid_scale(recovery_s, 3.0, 1.0) * 30.0;
    (scr_part + tonic_part + recovery_part).clamp(0.0, 100.0)
}

fn sigmoid_scale(value: f64, midpoint: f64, steepness: f64) -> f64 {
    1.0 / (1.0 + (-(value - midpoint) / steepness).exp())
}

/// Full stress assessment from raw EDA signal.
#[must_use]
pub fn assess_stress(
    phasic: &[f64],
    scl: &[f64],
    peaks: &[usize],
    duration_s: f64,
    fs: f64,
) -> StressAssessment {
    let rate = scr_rate(peaks.len(), duration_s);
    #[expect(clippy::cast_precision_loss, reason = "scl.len() fits f64")]
    let mean = if scl.is_empty() {
        0.0
    } else {
        scl.iter().sum::<f64>() / scl.len() as f64
    };
    let recovery = scr_recovery_time(phasic, peaks, fs);
    let index = compute_stress_index(rate, mean, recovery);

    StressAssessment {
        scr_rate: rate,
        mean_scl: mean,
        mean_recovery_s: recovery,
        stress_index: index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn scr_rate_basic() {
        let r = scr_rate(6, 120.0);
        assert!(
            (r - 3.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "6 events in 2 min = 3/min"
        );
    }

    #[test]
    fn stress_low_values_low_index() {
        let idx = compute_stress_index(1.0, 2.0, 1.0);
        assert!(idx < 40.0, "low arousal should give low index: {idx}");
    }

    #[test]
    fn stress_high_values_high_index() {
        let idx = compute_stress_index(8.0, 8.0, 5.0);
        assert!(idx > 60.0, "high arousal should give high index: {idx}");
    }

    #[test]
    fn stress_index_bounded() {
        let idx = compute_stress_index(100.0, 100.0, 100.0);
        assert!(idx <= 100.0);
        let idx2 = compute_stress_index(0.0, 0.0, 0.0);
        assert!(idx2 >= 0.0);
    }
}
