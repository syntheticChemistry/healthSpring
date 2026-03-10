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
//! ## Constants
//!
//! ECG bandpass range and MWI window follow Pan & Tompkins (1985).
//! `SpO2` calibration coefficients are from Beer-Lambert linearization.
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
        d[i] = (-signal[i - 2] - 2.0 * signal[i - 1] + 2.0 * signal[i + 1] + signal[i + 2]) / 8.0;
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

    DetectionMetrics {
        tp,
        fp,
        fn_count,
        sensitivity,
        ppv,
    }
}

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
            *sample +=
                0.15 * (-((t - (beat_time - 0.16)).powi(2)) / (2.0 * 0.01_f64.powi(2))).exp();
            *sample -=
                0.10 * (-((t - (beat_time - 0.04)).powi(2)) / (2.0 * 0.005_f64.powi(2))).exp();
            *sample += 1.0 * (-((t - beat_time).powi(2)) / (2.0 * 0.008_f64.powi(2))).exp();
            *sample -=
                0.25 * (-((t - (beat_time + 0.04)).powi(2)) / (2.0 * 0.008_f64.powi(2))).exp();
            *sample +=
                0.30 * (-((t - (beat_time + 0.25)).powi(2)) / (2.0 * 0.04_f64.powi(2))).exp();
        }

        rng_state = rng_state
            .wrapping_mul(crate::rng::LCG_MULTIPLIER)
            .wrapping_add(1);
        let jitter = (u64_to_f64(rng_state >> 33) / f64::from(u32::MAX) - 0.5) * 0.04;
        beat_time += rr + jitter;
    }

    if noise_std > 0.0 {
        for sample in &mut ecg {
            rng_state = crate::rng::lcg_step(rng_state);
            let u1 = crate::rng::state_to_f64(rng_state);
            rng_state = crate::rng::lcg_step(rng_state);
            let u2 = crate::rng::state_to_f64(rng_state);
            *sample += noise_std * crate::rng::box_muller(u1, u2);
        }
    }

    (ecg, r_peaks)
}

// ═══════════════════════════════════════════════════════════════════════
// PPG SpO2 Estimation (Exp022)
// ═══════════════════════════════════════════════════════════════════════

/// PPG R-value: ratio of pulsatile-to-static components.
///
/// `R = (AC_red / DC_red) / (AC_ir / DC_ir)`
/// R ≈ 0.4–0.6 for normal oxygenation, R > 0.8 for hypoxia.
#[must_use]
pub fn ppg_r_value(ac_red: f64, dc_red: f64, ac_ir: f64, dc_ir: f64) -> f64 {
    if dc_red.abs() < 1e-15 || dc_ir.abs() < 1e-15 || ac_ir.abs() < 1e-15 {
        return f64::NAN;
    }
    (ac_red / dc_red) / (ac_ir / dc_ir)
}

/// Standard empirical `SpO2` calibration from R-value.
///
/// `SpO2 = 110 - 25 * R` (Beer–Lambert empirical approximation).
/// Returns percentage [0, 100], clamped.
#[must_use]
pub fn spo2_from_r(r_value: f64) -> f64 {
    (110.0 - 25.0 * r_value).clamp(0.0, 100.0)
}

/// Generate synthetic PPG signal pair (red + IR) for testing.
///
/// Models pulsatile AC component + DC baseline for both wavelengths.
/// `spo2_target` controls the ratio between AC components.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "fs * duration is small positive — safe truncation"
)]
pub fn generate_synthetic_ppg(
    fs: f64,
    duration_s: f64,
    heart_rate_bpm: f64,
    spo2_target: f64,
    seed: u64,
) -> SyntheticPpg {
    let n_samples = (fs * duration_s) as usize;
    let mut red = vec![0.0; n_samples];
    let mut ir = vec![0.0; n_samples];

    let rr = 60.0 / heart_rate_bpm;
    let r_target = (110.0 - spo2_target) / 25.0;

    let dc_red = 1.0;
    let dc_ir = 1.0;
    let ac_ir = 0.02;
    let ac_red = r_target * ac_ir * (dc_red / dc_ir);

    let mut rng_state = seed;

    for i in 0..n_samples {
        let t = idx_to_f64(i) / fs;
        let phase = 2.0 * std::f64::consts::PI * t / rr;
        let pulse = phase.sin().max(0.0).powi(2);

        rng_state = rng_state
            .wrapping_mul(crate::rng::LCG_MULTIPLIER)
            .wrapping_add(1);
        let noise = (u64_to_f64(rng_state >> 33) / f64::from(u32::MAX) - 0.5) * 0.001;

        red[i] = dc_red + ac_red * pulse + noise;

        rng_state = rng_state
            .wrapping_mul(crate::rng::LCG_MULTIPLIER)
            .wrapping_add(1);
        let noise_ir = (u64_to_f64(rng_state >> 33) / f64::from(u32::MAX) - 0.5) * 0.001;

        ir[i] = dc_ir + ac_ir * pulse + noise_ir;
    }

    SyntheticPpg {
        red,
        ir,
        fs,
        dc_red,
        dc_ir,
        ac_red,
        ac_ir,
        r_target,
    }
}

/// Result of synthetic PPG generation.
#[derive(Debug, Clone)]
pub struct SyntheticPpg {
    pub red: Vec<f64>,
    pub ir: Vec<f64>,
    pub fs: f64,
    pub dc_red: f64,
    pub dc_ir: f64,
    pub ac_red: f64,
    pub ac_ir: f64,
    pub r_target: f64,
}

/// Extract AC and DC components from a PPG signal.
///
/// DC = mean(signal), AC = max(signal) - min(signal) as simple envelope.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "signal.len() < 2^52")]
pub fn ppg_extract_ac_dc(signal: &[f64]) -> (f64, f64) {
    if signal.is_empty() {
        return (0.0, 0.0);
    }
    let dc = signal.iter().sum::<f64>() / signal.len() as f64;
    let max = signal.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min = signal.iter().copied().fold(f64::INFINITY, f64::min);
    let ac = max - min;
    (ac, dc)
}

// ═══════════════════════════════════════════════════════════════════════
// EDA — Electrodermal Activity (Exp023)
// ═══════════════════════════════════════════════════════════════════════

/// Tonic skin conductance level (SCL) — moving average of EDA signal.
#[must_use]
pub fn eda_scl(signal: &[f64], window_samples: usize) -> Vec<f64> {
    moving_window_integration(signal, window_samples)
}

/// Phasic EDA: subtract tonic SCL to get SCR events.
#[must_use]
pub fn eda_phasic(signal: &[f64], window_samples: usize) -> Vec<f64> {
    let tonic = eda_scl(signal, window_samples);
    signal
        .iter()
        .zip(tonic.iter())
        .map(|(&s, &t)| (s - t).max(0.0))
        .collect()
}

/// Detect SCR peaks in phasic EDA signal.
/// Returns indices of peaks above `threshold_us` (microsiemens).
#[must_use]
pub fn eda_detect_scr(
    phasic: &[f64],
    threshold_us: f64,
    min_interval_samples: usize,
) -> Vec<usize> {
    let n = phasic.len();
    if n < 3 {
        return vec![];
    }
    let mut peaks = Vec::new();
    let mut last_peak: usize = 0;
    for i in 1..n - 1 {
        if phasic[i] > phasic[i - 1]
            && phasic[i] > phasic[i + 1]
            && phasic[i] > threshold_us
            && (peaks.is_empty() || i - last_peak > min_interval_samples)
        {
            peaks.push(i);
            last_peak = i;
        }
    }
    peaks
}

/// Generate synthetic EDA signal for testing.
///
/// Produces tonic baseline + phasic SCR events at known times.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "fs * duration is small positive"
)]
pub fn generate_synthetic_eda(
    fs: f64,
    duration_s: f64,
    scl_baseline: f64,
    scr_times: &[f64],
    scr_amplitude: f64,
    seed: u64,
) -> Vec<f64> {
    let n_samples = (fs * duration_s) as usize;
    let mut eda = vec![scl_baseline; n_samples];

    // Add SCR events (Gaussian-shaped responses)
    for &t_event in scr_times {
        let center = (t_event * fs) as usize;
        let rise_width = (0.5 * fs) as usize; // 0.5s rise
        let decay_width = (2.0 * fs) as usize; // 2s decay

        let start = center.saturating_sub(rise_width);
        let end = n_samples.min(center + decay_width);
        for (i, sample) in eda.iter_mut().enumerate().skip(start).take(end - start) {
            let t_rel = idx_to_f64(i) / fs - t_event;
            if t_rel < 0.0 {
                *sample += scr_amplitude * (-(t_rel * t_rel) / (2.0 * 0.3 * 0.3)).exp();
            } else {
                *sample += scr_amplitude * (-t_rel / 1.5).exp();
            }
        }
    }

    // Add noise
    let mut rng_state = seed;
    for sample in &mut eda {
        rng_state = rng_state
            .wrapping_mul(crate::rng::LCG_MULTIPLIER)
            .wrapping_add(1);
        let noise = (u64_to_f64(rng_state >> 33) / f64::from(u32::MAX) - 0.5) * 0.01;
        *sample += noise;
    }

    eda
}

// ═══════════════════════════════════════════════════════════════════════
// Multi-Channel Biosignal Fusion (Exp023)
// ═══════════════════════════════════════════════════════════════════════

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

    // Stress index: normalized [0, 1], higher = more stressed
    // Low SDNN (<20ms) = stress, high SCR rate (>10/min) = stress
    let sdnn_stress = (1.0 - (sdnn / 100.0).min(1.0)).max(0.0);
    let scr_stress = (scr_rate / 20.0).min(1.0);
    let spo2_stress = (1.0 - (ppg_spo2 - 90.0) / 10.0).clamp(0.0, 1.0);
    let stress_index = (sdnn_stress + scr_stress + spo2_stress) / 3.0;

    // Overall health score [0, 100]
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
        assert!(
            !peaks.contains(&120),
            "refractory should suppress close peak"
        );
    }

    #[test]
    fn synthetic_ecg_beat_count() {
        let (ecg, r_peaks) = generate_synthetic_ecg(360.0, 10.0, 72.0, 0.0, 42);
        assert_eq!(ecg.len(), 3600);
        assert!(
            (i32::try_from(r_peaks.len()).unwrap_or(0) - 12).abs() <= 1,
            "~12 beats at 72bpm in 10s"
        );
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

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "constant arithmetic produces known-small positive result"
        )]
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
        // Perfectly regular peaks → RMSSD = 0
        let peaks: Vec<usize> = (0..10).map(|i| i * 360).collect();
        let r = rmssd_ms(&peaks, 360.0);
        assert!(r.abs() < 1e-10, "constant RR → RMSSD=0");
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

    #[test]
    fn synthetic_ecg_deterministic() {
        // Same seed must produce identical ECG
        let (ecg1, _) = generate_synthetic_ecg(360.0, 5.0, 72.0, 0.05, 42);
        let (ecg2, _) = generate_synthetic_ecg(360.0, 5.0, 72.0, 0.05, 42);
        assert_eq!(ecg1.len(), ecg2.len());
        for (a, b) in ecg1.iter().zip(ecg2.iter()) {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "ECG must be bit-identical with same seed"
            );
        }
    }

    #[test]
    fn ppg_r_value_normal() {
        let r = ppg_r_value(0.02, 1.0, 0.04, 1.0);
        assert!((r - 0.5).abs() < 1e-10, "R = 0.5 for normal SpO2");
    }

    #[test]
    fn spo2_from_r_normal() {
        let spo2 = spo2_from_r(0.4);
        assert!((spo2 - 100.0).abs() < 1e-10, "SpO2 = 100% at R=0.4");
    }

    #[test]
    fn spo2_from_r_clamped() {
        let spo2 = spo2_from_r(-1.0);
        assert!((spo2 - 100.0).abs() < 1e-10, "clamped to 100%");
        let spo2_low = spo2_from_r(10.0);
        assert!(spo2_low.abs() < 1e-10, "clamped to 0%");
    }

    #[test]
    fn synthetic_ppg_roundtrip() {
        let ppg = generate_synthetic_ppg(256.0, 5.0, 72.0, 97.0, 42);
        assert_eq!(ppg.red.len(), 1280);
        let (ac_red, dc_red) = ppg_extract_ac_dc(&ppg.red);
        let (ac_ir, dc_ir) = ppg_extract_ac_dc(&ppg.ir);
        let r = ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
        let spo2 = spo2_from_r(r);
        assert!((spo2 - 97.0).abs() < 5.0, "SpO2={spo2} should be ~97%");
    }

    #[test]
    fn ppg_deterministic() {
        let ppg1 = generate_synthetic_ppg(256.0, 2.0, 72.0, 97.0, 42);
        let ppg2 = generate_synthetic_ppg(256.0, 2.0, 72.0, 97.0, 42);
        for (a, b) in ppg1.red.iter().zip(ppg2.red.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "PPG must be bit-identical");
        }
    }

    // ─── EDA (Exp023) ───────────────────────────────────────────────────

    #[test]
    fn eda_scl_near_baseline() {
        let eda = generate_synthetic_eda(32.0, 30.0, 2.0, &[5.0, 12.0, 20.0, 25.0], 0.5, 42);
        let scl = eda_scl(&eda, 32);
        #[expect(
            clippy::cast_precision_loss,
            reason = "EDA sample count fits f64 mantissa"
        )]
        let mean_scl: f64 = scl.iter().sum::<f64>() / scl.len() as f64;
        assert!(
            (mean_scl - 2.0).abs() < 0.5,
            "SCL mean={mean_scl} should be near baseline 2.0 µS"
        );
    }

    #[test]
    fn eda_phasic_non_negative() {
        let eda = generate_synthetic_eda(32.0, 30.0, 2.0, &[5.0, 12.0], 0.5, 42);
        let phasic = eda_phasic(&eda, 32);
        assert!(
            phasic.iter().all(|&x| x >= 0.0),
            "phasic EDA must be non-negative"
        );
    }

    #[test]
    fn eda_detect_scr_finds_peaks() {
        let eda = generate_synthetic_eda(32.0, 30.0, 2.0, &[5.0, 12.0, 20.0, 25.0], 0.5, 42);
        let phasic = eda_phasic(&eda, 32);
        let peaks = eda_detect_scr(&phasic, 0.05, 32);
        assert!(
            peaks.len() >= 3 && peaks.len() <= 7,
            "should find ~4 SCR peaks (noise may add extras), got {}",
            peaks.len()
        );
    }

    #[test]
    fn eda_signal_length() {
        let eda = generate_synthetic_eda(32.0, 30.0, 2.0, &[], 0.0, 42);
        assert_eq!(eda.len(), 960, "fs=32 * 30s = 960 samples");
    }

    #[test]
    fn eda_deterministic() {
        let eda1 = generate_synthetic_eda(32.0, 10.0, 2.0, &[3.0, 7.0], 0.5, 42);
        let eda2 = generate_synthetic_eda(32.0, 10.0, 2.0, &[3.0, 7.0], 0.5, 42);
        for (a, b) in eda1.iter().zip(eda2.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "EDA must be bit-identical");
        }
    }

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
