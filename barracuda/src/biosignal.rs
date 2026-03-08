// SPDX-License-Identifier: AGPL-3.0-or-later
//! Physiological biosignal processing pipelines.
//!
//! Leverages `BarraCUDA` attention mechanisms + NPU inference for
//! real-time health monitoring on sovereign hardware.
//! - ECG anomaly detection (R-peak, arrhythmia)
//! - PPG-based `SpO2` estimation
//! - Continuous glucose monitoring analytics
//! - Heart rate variability (HRV) analysis
//! - Wearable sensor fusion (IMU + PPG + temperature)
//!
//! ## Tier 1 (CPU) — Exp020
//!
//! Pan-Tompkins QRS detection algorithm:
//! 1. Bandpass filter (5–15 Hz via frequency domain)
//! 2. Five-point derivative
//! 3. Squaring (nonlinear amplification)
//! 4. Moving-window integration
//! 5. Adaptive peak detection with refractory period

use std::f64::consts::PI;

// ═══════════════════════════════════════════════════════════════════════
// Pan-Tompkins QRS Detection (Exp020)
// ═══════════════════════════════════════════════════════════════════════

/// Simple frequency-domain bandpass filter.
///
/// Zeros out frequency components outside `[low_hz, high_hz]`.
#[must_use]
pub fn bandpass_filter(signal: &[f64], fs: f64, low_hz: f64, high_hz: f64) -> Vec<f64> {
    let n = signal.len();
    if n == 0 {
        return vec![];
    }

    let (re, im) = rfft(signal);
    let n_freq = re.len();

    let mut out_re = vec![0.0; n_freq];
    let mut out_im = vec![0.0; n_freq];

    for k in 0..n_freq {
        let freq = idx_to_f64(k) * fs / idx_to_f64(n);
        if freq >= low_hz && freq <= high_hz {
            out_re[k] = re[k];
            out_im[k] = im[k];
        }
    }

    irfft(&out_re, &out_im, n)
}

/// Five-point derivative filter (Pan-Tompkins).
///
/// `d[i] = (-x[i-2] - 2*x[i-1] + 2*x[i+1] + x[i+2]) / 8`
#[must_use]
pub fn derivative_filter(signal: &[f64]) -> Vec<f64> {
    let n = signal.len();
    let mut d = vec![0.0; n];
    for i in 2..n.saturating_sub(2) {
        d[i] = (-signal[i - 2] - 2.0 * signal[i - 1]
            + 2.0 * signal[i + 1]
            + signal[i + 2])
            / 8.0;
    }
    d
}

/// Nonlinear squaring: `y[i] = x[i]²`.
#[must_use]
pub fn squaring(signal: &[f64]) -> Vec<f64> {
    signal.iter().map(|&x| x * x).collect()
}

/// Moving-window integration with `window_size` samples.
#[must_use]
pub fn moving_window_integration(signal: &[f64], window_size: usize) -> Vec<f64> {
    let n = signal.len();
    if n == 0 || window_size == 0 {
        return vec![0.0; n];
    }
    let half = window_size / 2;
    let ws = idx_to_f64(window_size);
    let mut out = vec![0.0; n];
    for (i, slot) in out.iter_mut().enumerate() {
        let start = i.saturating_sub(half);
        let end = (i + window_size - half).min(n);
        let sum: f64 = signal[start..end].iter().sum();
        *slot = sum / ws;
    }
    out
}

/// Peak detection with refractory period (simplified adaptive threshold).
///
/// Returns indices of detected peaks.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "refractory_ms * fs / 1000 is small positive — safe truncation"
)]
pub fn detect_peaks(mwi: &[f64], fs: f64, refractory_ms: f64) -> Vec<usize> {
    let n = mwi.len();
    if n < 3 {
        return vec![];
    }
    let threshold = 0.4 * mwi.iter().copied().fold(0.0_f64, f64::max);
    let refractory_samples = (refractory_ms * fs / 1000.0) as usize;
    let mut peaks = Vec::new();
    let mut last_peak: usize = 0;

    for i in 1..n - 1 {
        if mwi[i] > mwi[i - 1]
            && mwi[i] > mwi[i + 1]
            && mwi[i] > threshold
            && (peaks.is_empty() || i - last_peak > refractory_samples)
        {
            peaks.push(i);
            last_peak = i;
        }
    }
    peaks
}

/// Full Pan-Tompkins pipeline: bandpass → derivative → squaring → MWI → peaks.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "0.15 * fs is small positive — safe truncation"
)]
pub fn pan_tompkins(signal: &[f64], fs: f64) -> PanTompkinsResult {
    let bp = bandpass_filter(signal, fs, 5.0, 15.0);
    let deriv = derivative_filter(&bp);
    let sq = squaring(&deriv);
    let window_size = (0.15 * fs) as usize;
    let mwi = moving_window_integration(&sq, window_size);
    let peaks = detect_peaks(&mwi, fs, 200.0);

    PanTompkinsResult {
        bandpass: bp,
        derivative: deriv,
        squared: sq,
        mwi,
        peaks,
    }
}

/// Result of the full Pan-Tompkins pipeline.
#[derive(Debug, Clone)]
pub struct PanTompkinsResult {
    pub bandpass: Vec<f64>,
    pub derivative: Vec<f64>,
    pub squared: Vec<f64>,
    pub mwi: Vec<f64>,
    pub peaks: Vec<usize>,
}

/// QRS detection metrics.
#[derive(Debug, Clone, Copy)]
pub struct DetectionMetrics {
    pub tp: usize,
    pub fp: usize,
    pub fn_count: usize,
    pub sensitivity: f64,
    pub ppv: f64,
}

/// Evaluate QRS detection against known R-peak locations.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "small counts → f64 for ratios")]
pub fn evaluate_detection(
    detected: &[usize],
    true_peaks: &[usize],
    tolerance_samples: usize,
) -> DetectionMetrics {
    let mut tp = 0usize;
    let mut matched = vec![false; true_peaks.len()];

    for &d in detected {
        for (j, &t) in true_peaks.iter().enumerate() {
            if !matched[j] && d.abs_diff(t) <= tolerance_samples {
                tp += 1;
                matched[j] = true;
                break;
            }
        }
    }

    let fn_count = true_peaks.len() - tp;
    let fp = detected.len() - tp;
    let sensitivity = if tp + fn_count > 0 {
        tp as f64 / (tp + fn_count) as f64
    } else {
        0.0
    };
    let ppv = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };

    DetectionMetrics { tp, fp, fn_count, sensitivity, ppv }
}

/// Compute heart rate from detected peak indices.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
pub fn heart_rate_from_peaks(peaks: &[usize], fs: f64) -> f64 {
    if peaks.len() < 2 {
        return 0.0;
    }
    let rr_intervals: Vec<f64> = peaks.windows(2).map(|w| (w[1] - w[0]) as f64 / fs).collect();
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
    let rr_ms: Vec<f64> = peaks.windows(2).map(|w| (w[1] - w[0]) as f64 / fs * 1000.0).collect();
    let mean = rr_ms.iter().sum::<f64>() / rr_ms.len() as f64;
    let var = rr_ms.iter().map(|&r| (r - mean) * (r - mean)).sum::<f64>() / rr_ms.len() as f64;
    var.sqrt()
}

// ═══════════════════════════════════════════════════════════════════════
// Minimal real FFT (DFT-based, no external dependency)
// ═══════════════════════════════════════════════════════════════════════

/// Convert index to f64 (avoids repeated `clippy::cast_precision_loss`).
#[expect(clippy::cast_precision_loss, reason = "indices ≪ 2^52")]
fn idx_to_f64(v: usize) -> f64 {
    v as f64
}

#[expect(clippy::cast_precision_loss, reason = "indices ≪ 2^52")]
fn u64_to_f64(v: u64) -> f64 {
    v as f64
}

fn rfft(signal: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = signal.len();
    let n_freq = n / 2 + 1;
    let mut re = vec![0.0; n_freq];
    let mut im = vec![0.0; n_freq];
    let nf = idx_to_f64(n);
    for k in 0..n_freq {
        let kf = idx_to_f64(k);
        for (j, &s) in signal.iter().enumerate() {
            let angle = 2.0 * PI * kf * idx_to_f64(j) / nf;
            re[k] += s * angle.cos();
            im[k] -= s * angle.sin();
        }
    }
    (re, im)
}

fn irfft(re: &[f64], im: &[f64], n: usize) -> Vec<f64> {
    let n_freq = re.len();
    let mut out = vec![0.0; n];
    let nf = idx_to_f64(n);
    for (j, slot) in out.iter_mut().enumerate() {
        let jf = idx_to_f64(j);
        for k in 0..n_freq {
            let angle = 2.0 * PI * idx_to_f64(k) * jf / nf;
            let mut contribution = re[k] * angle.cos() - im[k] * angle.sin();
            if k > 0 && k < n_freq - 1 {
                contribution *= 2.0;
            }
            *slot += contribution;
        }
        *slot /= nf;
    }
    out
}

/// Generate synthetic ECG for testing (Gaussian P-QRS-T model).
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "fs * duration and beat_time * fs are small positive — safe"
)]
pub fn generate_synthetic_ecg(
    fs: f64,
    duration_s: f64,
    heart_rate_bpm: f64,
    noise_std: f64,
    seed: u64,
) -> (Vec<f64>, Vec<usize>) {
    let n_samples = (fs * duration_s) as usize;
    let mut ecg = vec![0.0; n_samples];
    let rr = 60.0 / heart_rate_bpm;
    let mut r_peaks = Vec::new();

    let mut beat_time = 0.1;
    let mut rng_state = seed;

    while beat_time < duration_s - 0.5 {
        let r_idx = (beat_time * fs) as usize;
        if r_idx < n_samples {
            r_peaks.push(r_idx);
        }

        for (i, sample) in ecg.iter_mut().enumerate() {
            let t = idx_to_f64(i) / fs;
            *sample += 0.15 * (-((t - (beat_time - 0.16)).powi(2)) / (2.0 * 0.01_f64.powi(2))).exp();
            *sample -= 0.10 * (-((t - (beat_time - 0.04)).powi(2)) / (2.0 * 0.005_f64.powi(2))).exp();
            *sample += 1.0 * (-((t - beat_time).powi(2)) / (2.0 * 0.008_f64.powi(2))).exp();
            *sample -= 0.25 * (-((t - (beat_time + 0.04)).powi(2)) / (2.0 * 0.008_f64.powi(2))).exp();
            *sample += 0.30 * (-((t - (beat_time + 0.25)).powi(2)) / (2.0 * 0.04_f64.powi(2))).exp();
        }

        rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let jitter = (u64_to_f64(rng_state >> 33) / f64::from(u32::MAX) - 0.5) * 0.04;
        beat_time += rr + jitter;
    }

    if noise_std > 0.0 {
        for sample in &mut ecg {
            rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let u1 = u64_to_f64(rng_state >> 33) / f64::from(u32::MAX);
            rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let u2 = u64_to_f64(rng_state >> 33) / f64::from(u32::MAX);
            let z = (-2.0 * u1.max(1e-30).ln()).sqrt() * (2.0 * PI * u2).cos();
            *sample += noise_std * z;
        }
    }

    (ecg, r_peaks)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FS: f64 = 360.0;

    #[test]
    fn derivative_filter_length_preserved() {
        let sig = vec![0.0; 100];
        let d = derivative_filter(&sig);
        assert_eq!(d.len(), 100);
    }

    #[test]
    fn squaring_non_negative() {
        let sig = vec![-1.0, 0.0, 2.0, -3.5];
        let sq = squaring(&sig);
        assert!(sq.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn mwi_length_preserved() {
        let sig = vec![1.0; 200];
        let m = moving_window_integration(&sig, 54);
        assert_eq!(m.len(), 200);
    }

    #[test]
    fn detect_peaks_refractory() {
        let mut mwi = vec![0.0; 1000];
        mwi[100] = 10.0;
        mwi[120] = 10.0;
        mwi[400] = 10.0;
        let peaks = detect_peaks(&mwi, 360.0, 200.0);
        assert!(peaks.contains(&100));
        assert!(peaks.contains(&400));
        assert!(!peaks.contains(&120), "refractory should suppress close peak");
    }

    #[test]
    fn synthetic_ecg_beat_count() {
        let (ecg, r_peaks) = generate_synthetic_ecg(360.0, 10.0, 72.0, 0.0, 42);
        assert_eq!(ecg.len(), 3600);
        assert!((r_peaks.len() as i32 - 12).abs() <= 1, "~12 beats at 72bpm in 10s");
    }

    #[test]
    fn full_pipeline_detects_beats() {
        let (ecg, true_peaks) = generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
        let result = pan_tompkins(&ecg, FS);
        assert_eq!(result.bandpass.len(), ecg.len());
        assert_eq!(result.derivative.len(), ecg.len());
        assert_eq!(result.squared.len(), ecg.len());
        assert_eq!(result.mwi.len(), ecg.len());
        assert!(!result.peaks.is_empty(), "should detect at least one peak");

        let tol = (75.0 * FS / 1000.0) as usize;
        let metrics = evaluate_detection(&result.peaks, &true_peaks, tol);
        assert!(metrics.sensitivity > 0.8, "Se={}", metrics.sensitivity);
        assert!(metrics.ppv > 0.8, "PPV={}", metrics.ppv);
    }

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
    fn bandpass_reduces_amplitude() {
        let (ecg, _) = generate_synthetic_ecg(FS, 5.0, 72.0, 0.0, 42);
        let bp = bandpass_filter(&ecg, FS, 5.0, 15.0);
        let max_ecg: f64 = ecg.iter().copied().fold(0.0_f64, |a, b| a.max(b.abs()));
        let max_bp: f64 = bp.iter().copied().fold(0.0_f64, |a, b| a.max(b.abs()));
        assert!(max_bp < max_ecg, "bandpass should reduce amplitude");
    }

    #[test]
    fn evaluate_detection_perfect() {
        let true_peaks = vec![100, 200, 300];
        let detected = vec![101, 199, 302];
        let m = evaluate_detection(&detected, &true_peaks, 5);
        assert_eq!(m.tp, 3);
        assert_eq!(m.fp, 0);
        assert_eq!(m.fn_count, 0);
        assert!((m.sensitivity - 1.0).abs() < 1e-10);
    }
}
