// SPDX-License-Identifier: AGPL-3.0-only
//! Electrodermal Activity (EDA) processing.
//!
//! Tonic SCL extraction, phasic SCR detection, and synthetic signal generation.

use super::ecg::moving_window_integration;
use super::fft::{idx_to_f64, u64_to_f64};

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

    for &t_event in scr_times {
        let center = (t_event * fs) as usize;
        let rise_width = (0.5 * fs) as usize;
        let decay_width = (2.0 * fs) as usize;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
