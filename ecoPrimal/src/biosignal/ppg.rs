// SPDX-License-Identifier: AGPL-3.0-or-later
//! PPG (photoplethysmography) `SpO2` estimation.
//!
//! Beer-Lambert empirical calibration: `SpO2 = 110 - 25 * R`.

use super::fft::{idx_to_f64, u64_to_f64};

/// Beer-Lambert empirical calibration coefficients.
///
/// `SpO2 = INTERCEPT - SLOPE * R` is the standard empirical approximation
/// from pulse oximetry (Tremper 1989; Jubran 1999).
pub mod spo2_calibration {
    /// Y-intercept of the SpO2-vs-R linear calibration.
    pub const INTERCEPT: f64 = 110.0;
    /// Slope of the SpO2-vs-R linear calibration.
    pub const SLOPE: f64 = 25.0;
}

/// Guard against division by near-zero DC or AC components.
const DIVISION_GUARD: f64 = 1e-15;

/// PPG R-value: ratio of pulsatile-to-static components.
///
/// `R = (AC_red / DC_red) / (AC_ir / DC_ir)`
/// R ≈ 0.4–0.6 for normal oxygenation, R > 0.8 for hypoxia.
#[must_use]
pub fn ppg_r_value(ac_red: f64, dc_red: f64, ac_ir: f64, dc_ir: f64) -> f64 {
    if dc_red.abs() < DIVISION_GUARD || dc_ir.abs() < DIVISION_GUARD || ac_ir.abs() < DIVISION_GUARD
    {
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
    spo2_calibration::SLOPE
        .mul_add(-r_value, spo2_calibration::INTERCEPT)
        .clamp(0.0, 100.0)
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
