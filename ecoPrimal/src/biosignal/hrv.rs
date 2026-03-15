// SPDX-License-Identifier: AGPL-3.0-or-later
//! Heart Rate Variability (HRV) metrics.
//!
//! Time-domain HRV from R-peak intervals: heart rate, SDNN, RMSSD, pNN50.

/// Compute heart rate from detected peak indices.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
pub fn heart_rate_from_peaks(peaks: &[usize], fs: f64) -> f64 {
    if peaks.len() < 2 {
        return 0.0;
    }
    let rr_intervals: Vec<f64> = peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs)
        .collect();
    let mean_rr = rr_intervals.iter().sum::<f64>() / rr_intervals.len() as f64;
    if mean_rr > 0.0 { 60.0 / mean_rr } else { 0.0 }
}

/// SDNN (standard deviation of NN intervals) in milliseconds.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
pub fn sdnn_ms(peaks: &[usize], fs: f64) -> f64 {
    if peaks.len() < 3 {
        return 0.0;
    }
    let rr_ms: Vec<f64> = peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs * 1000.0)
        .collect();
    let mean = rr_ms.iter().sum::<f64>() / rr_ms.len() as f64;
    let var = rr_ms.iter().map(|&r| (r - mean) * (r - mean)).sum::<f64>() / rr_ms.len() as f64;
    var.sqrt()
}

/// RMSSD (root mean square of successive differences) in milliseconds.
///
/// Standard short-term HRV metric reflecting parasympathetic activity.
/// `RMSSD = sqrt(mean(ΔRR²))` where `ΔRR = RR_{n+1} - RR_n`.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
pub fn rmssd_ms(peaks: &[usize], fs: f64) -> f64 {
    if peaks.len() < 3 {
        return 0.0;
    }
    let rr_ms: Vec<f64> = peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs * 1000.0)
        .collect();
    let successive_diffs_sq: f64 = rr_ms.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
    let n_diffs = rr_ms.len() - 1;
    if n_diffs == 0 {
        return 0.0;
    }
    (successive_diffs_sq / n_diffs as f64).sqrt()
}

/// pNN50: percentage of successive RR intervals differing by > 50 ms.
///
/// High pNN50 indicates strong vagal tone (parasympathetic).
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
pub fn pnn50(peaks: &[usize], fs: f64) -> f64 {
    if peaks.len() < 3 {
        return 0.0;
    }
    let rr_ms: Vec<f64> = peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs * 1000.0)
        .collect();
    let nn50_count = rr_ms
        .windows(2)
        .filter(|w| (w[1] - w[0]).abs() > 50.0)
        .count();
    let n_diffs = rr_ms.len() - 1;
    if n_diffs == 0 {
        return 0.0;
    }
    nn50_count as f64 / n_diffs as f64 * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biosignal::ecg::{generate_synthetic_ecg, pan_tompkins};

    const FS: f64 = 360.0;

    #[test]
    fn heart_rate_estimation() {
        let (ecg, _) = generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
        let result = pan_tompkins(&ecg, FS);
        let hr = heart_rate_from_peaks(&result.peaks, FS);
        assert!((hr - 72.0).abs() < 10.0, "HR={hr} should be ~72");
    }

    #[test]
    fn sdnn_synthetic() {
        let (ecg, _) = generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
        let result = pan_tompkins(&ecg, FS);
        let s = sdnn_ms(&result.peaks, FS);
        assert!(s < 200.0, "SDNN should be reasonable for synthetic: {s} ms");
    }

    #[test]
    fn rmssd_synthetic() {
        let (ecg, _) = generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
        let result = pan_tompkins(&ecg, FS);
        let r = rmssd_ms(&result.peaks, FS);
        assert!(r > 0.0 && r < 200.0, "RMSSD={r} should be reasonable");
    }

    #[test]
    fn pnn50_synthetic() {
        let (ecg, _) = generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
        let result = pan_tompkins(&ecg, FS);
        let p = pnn50(&result.peaks, FS);
        assert!((0.0..=100.0).contains(&p), "pNN50={p} should be 0-100%");
    }

    #[test]
    fn rmssd_constant_rr_is_zero() {
        let peaks: Vec<usize> = (0..10).map(|i| i * 360).collect();
        let r = rmssd_ms(&peaks, 360.0);
        assert!(r.abs() < 1e-10, "constant RR → RMSSD=0");
    }
}
