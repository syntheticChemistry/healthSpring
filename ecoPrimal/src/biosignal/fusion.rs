// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-channel biosignal fusion (ECG + PPG + EDA).
//!
//! Combines HRV, `SpO2`, and electrodermal metrics into a unified health
//! assessment with stress index and overall score.

use super::hrv::{heart_rate_from_peaks, rmssd_ms, sdnn_ms};

/// Stress index normalization parameters.
///
/// Each biosignal component is normalized to [0, 1] before averaging.
/// Thresholds are derived from clinical ranges (Task Force of ESC/NASPE 1996).
pub mod stress_params {
    /// SDNN value (ms) at which HRV stress component saturates at 0 (healthy).
    pub const SDNN_HEALTHY_MS: f64 = 100.0;
    /// SCR rate (events/min) at which EDA stress component saturates at 1.
    pub const SCR_STRESS_RATE: f64 = 20.0;
    /// `SpO2` (%) below which hypoxia stress component saturates at 1.
    pub const SPO2_FLOOR: f64 = 90.0;
    /// `SpO2` range (%) over which hypoxia stress interpolates to 0.
    pub const SPO2_RANGE: f64 = 10.0;
}

/// Fused health assessment from ECG + PPG + EDA channels.
#[derive(Debug, Clone)]
pub struct FusedHealthAssessment {
    /// Heart rate from R-R intervals (bpm).
    pub heart_rate_bpm: f64,
    /// SDNN HRV in ms.
    pub hrv_sdnn_ms: f64,
    /// RMSSD HRV in ms.
    pub hrv_rmssd_ms: f64,
    /// Peripheral oxygen saturation from PPG (%).
    pub spo2_percent: f64,
    /// Skin conductance response rate (events/min).
    pub scr_rate_per_min: f64,
    /// Combined normalized stress index in [0, 1].
    pub stress_index: f64,
    /// Overall wellness score in [0, 100] (higher is healthier).
    pub overall_score: f64,
}

/// Fuse ECG, PPG, and EDA channels into a unified health assessment.
///
/// Stress index combines HRV (low SDNN = stress), `SpO2`, and SCR rate.
/// Overall score: weighted combination (higher = healthier).
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "scr_count small — safe f64")]
pub fn fuse_channels(
    ecg_peaks: &[usize],
    ecg_fs: f64,
    ppg_spo2: f64,
    scr_count: usize,
    eda_duration_s: f64,
) -> FusedHealthAssessment {
    let hr = heart_rate_from_peaks(ecg_peaks, ecg_fs);
    let sdnn = sdnn_ms(ecg_peaks, ecg_fs);
    let rmssd = rmssd_ms(ecg_peaks, ecg_fs);

    let scr_rate = if eda_duration_s > 0.0 {
        scr_count as f64 / eda_duration_s * 60.0
    } else {
        0.0
    };

    let sdnn_stress = (1.0 - (sdnn / stress_params::SDNN_HEALTHY_MS).min(1.0)).max(0.0);
    let scr_stress = (scr_rate / stress_params::SCR_STRESS_RATE).min(1.0);
    let spo2_stress =
        (1.0 - (ppg_spo2 - stress_params::SPO2_FLOOR) / stress_params::SPO2_RANGE).clamp(0.0, 1.0);
    let stress_index = (sdnn_stress + scr_stress + spo2_stress) / 3.0;

    let overall_score = (100.0 * (1.0 - stress_index)).clamp(0.0, 100.0);

    FusedHealthAssessment {
        heart_rate_bpm: hr,
        hrv_sdnn_ms: sdnn,
        hrv_rmssd_ms: rmssd,
        spo2_percent: ppg_spo2,
        scr_rate_per_min: scr_rate,
        stress_index,
        overall_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuse_channels_stress_index_range() {
        let peaks: Vec<usize> = (0..12).map(|i| i * 300).collect();
        let fused = fuse_channels(&peaks, 360.0, 97.0, 4, 30.0);
        assert!(
            (0.0..=1.0).contains(&fused.stress_index),
            "stress_index={} must be in [0,1]",
            fused.stress_index
        );
    }

    #[test]
    fn fuse_channels_overall_score_range() {
        let peaks: Vec<usize> = (0..12).map(|i| i * 300).collect();
        let fused = fuse_channels(&peaks, 360.0, 97.0, 2, 30.0);
        assert!(
            (0.0..=100.0).contains(&fused.overall_score),
            "overall_score={} must be in [0,100]",
            fused.overall_score
        );
    }

    #[test]
    fn fuse_channels_deterministic() {
        let peaks: Vec<usize> = (0..12).map(|i| i * 300).collect();
        let f1 = fuse_channels(&peaks, 360.0, 97.0, 4, 30.0);
        let f2 = fuse_channels(&peaks, 360.0, 97.0, 4, 30.0);
        assert_eq!(f1.heart_rate_bpm.to_bits(), f2.heart_rate_bpm.to_bits());
        assert_eq!(f1.overall_score.to_bits(), f2.overall_score.to_bits());
    }
}
