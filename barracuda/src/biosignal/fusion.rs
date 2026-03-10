// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-channel biosignal fusion (ECG + PPG + EDA).
//!
//! Combines HRV, `SpO2`, and electrodermal metrics into a unified health
//! assessment with stress index and overall score.

use super::hrv::{heart_rate_from_peaks, rmssd_ms, sdnn_ms};

/// Fused health assessment from ECG + PPG + EDA channels.
#[derive(Debug, Clone)]
pub struct FusedHealthAssessment {
    pub heart_rate_bpm: f64,
    pub hrv_sdnn_ms: f64,
    pub hrv_rmssd_ms: f64,
    pub spo2_percent: f64,
    pub scr_rate_per_min: f64,
    pub stress_index: f64,
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

    let sdnn_stress = (1.0 - (sdnn / 100.0).min(1.0)).max(0.0);
    let scr_stress = (scr_rate / 20.0).min(1.0);
    let spo2_stress = (1.0 - (ppg_spo2 - 90.0) / 10.0).clamp(0.0, 1.0);
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
