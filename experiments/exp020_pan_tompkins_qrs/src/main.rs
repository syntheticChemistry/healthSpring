// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp020 validation: Pan-Tompkins QRS Detection
//!
//! Cross-validates `healthspring_barracuda::biosignal` Pan-Tompkins
//! pipeline against the Python control (`exp020_pan_tompkins_qrs.py`).

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("Exp020 Pan-Tompkins QRS");

    let fs = 360.0;

    // Generate synthetic ECG
    let (ecg, true_peaks) = biosignal::generate_synthetic_ecg(fs, 10.0, 72.0, 0.05, 42);

    // Check 1: ECG sample count
    h.check_exact("ECG sample count", ecg.len() as u64, 3600);

    // Check 2: Beat count
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "beat count < 1000 fits in i32"
    )]
    let diff = (true_peaks.len() as i32 - 12).abs();
    h.check_bool("Beat count ~12", diff <= 1);

    // Run full pipeline
    let result = biosignal::pan_tompkins(&ecg, fs);

    // Check 3: Bandpass preserves length
    h.check_bool(
        "Bandpass length preserved",
        result.bandpass.len() == ecg.len(),
    );

    // Check 4: Bandpass reduces amplitude
    let max_ecg = ecg.iter().copied().fold(0.0_f64, |a, b| a.max(b.abs()));
    let max_bp = result
        .bandpass
        .iter()
        .copied()
        .fold(0.0_f64, |a, b| a.max(b.abs()));
    h.check_bool("Bandpass reduces amplitude", max_bp < max_ecg);

    // Check 5: Squared non-negative
    h.check_bool("Squared ≥ 0", result.squared.iter().all(|&x| x >= 0.0));

    // Check 6: MWI non-negative
    h.check_bool(
        "MWI ≥ 0",
        result
            .mwi
            .iter()
            .all(|&x| x >= -tolerances::MACHINE_EPSILON_TIGHT),
    );

    // Check 7: Detections > 0
    h.check_bool("Peaks detected", !result.peaks.is_empty());

    // Evaluate against truth — peak match tolerance from TOLERANCE_REGISTRY.
    // 75 ms ≈ 27 samples at 360 Hz (ANSI/AAMI EC57:2012 recommends 150 ms;
    // we use half for synthetic signals with known truth).
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "tol_samples < 1000 fits in usize"
    )]
    let tol_samples = (tolerances::QRS_PEAK_MATCH_MS * fs / 1000.0) as usize;
    let metrics = biosignal::evaluate_detection(&result.peaks, &true_peaks, tol_samples);

    // Check 8: Sensitivity > 80% (ANSI/AAMI EC57:2012 requires Se ≥ 99.5%
    // on MIT-BIH; we relax to 80% for synthetic signals with noise).
    h.check_lower(
        "Sensitivity",
        metrics.sensitivity,
        tolerances::QRS_SENSITIVITY,
    );

    // Check 9: PPV > 80% (same relaxation as Se for synthetic signals)
    h.check_lower("PPV", metrics.ppv, tolerances::QRS_SENSITIVITY);

    // Check 10: Heart rate — expected 72 bpm (synthetic signal parameter)
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    h.check_abs("Heart rate", hr, 72.0, tolerances::HR_DETECTION_BPM);

    // Check 11: SDNN — healthy resting SDNN is 50–200 ms (Task Force of ESC
    // and NASPE, Circulation 1996). Synthetic signal should be below upper bound.
    let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
    h.check_upper("SDNN", sdnn, tolerances::SDNN_UPPER_MS);

    // Check 12: All pipeline stages same length
    h.check_bool(
        "Pipeline length consistency",
        result.derivative.len() == ecg.len()
            && result.squared.len() == ecg.len()
            && result.mwi.len() == ecg.len(),
    );

    h.exit();
}
