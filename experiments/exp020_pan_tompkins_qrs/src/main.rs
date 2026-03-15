// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp020 validation: Pan-Tompkins QRS Detection
//!
//! Cross-validates `healthspring_barracuda::biosignal` Pan-Tompkins
//! pipeline against the Python control (`exp020_pan_tompkins_qrs.py`).

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    let fs = 360.0;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp020 — Rust CPU Validation: Pan-Tompkins QRS");
    println!("  fs={fs} Hz");
    println!("{}", "=".repeat(72));

    // Generate synthetic ECG
    let (ecg, true_peaks) = biosignal::generate_synthetic_ecg(fs, 10.0, 72.0, 0.05, 42);

    // Check 1: ECG sample count
    println!("\n--- Check 1: ECG sample count ---");
    if ecg.len() == 3600 {
        println!("  [PASS] {} samples", ecg.len());
        passed += 1;
    } else {
        println!("  [FAIL] {} samples", ecg.len());
        failed += 1;
    }

    // Check 2: Beat count
    println!("\n--- Check 2: Beat count ---");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "beat count < 1000 fits in i32"
    )]
    let diff = (true_peaks.len() as i32 - 12).abs();
    if diff <= 1 {
        println!("  [PASS] {} beats (expected ~12)", true_peaks.len());
        passed += 1;
    } else {
        println!("  [FAIL] {} beats", true_peaks.len());
        failed += 1;
    }

    // Run full pipeline
    let result = biosignal::pan_tompkins(&ecg, fs);

    // Check 3: Bandpass preserves length
    println!("\n--- Check 3: Bandpass length preserved ---");
    if result.bandpass.len() == ecg.len() {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 4: Bandpass reduces amplitude
    println!("\n--- Check 4: Bandpass reduces amplitude ---");
    let max_ecg = ecg.iter().copied().fold(0.0_f64, |a, b| a.max(b.abs()));
    let max_bp = result
        .bandpass
        .iter()
        .copied()
        .fold(0.0_f64, |a, b| a.max(b.abs()));
    if max_bp < max_ecg {
        println!("  [PASS] max|BP|={max_bp:.4} < max|ECG|={max_ecg:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 5: Squared non-negative
    println!("\n--- Check 5: Squared ≥ 0 ---");
    if result.squared.iter().all(|&x| x >= 0.0) {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 6: MWI non-negative
    println!("\n--- Check 6: MWI ≥ 0 ---");
    if result
        .mwi
        .iter()
        .all(|&x| x >= -tolerances::MACHINE_EPSILON_TIGHT)
    {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 7: Detections > 0
    println!("\n--- Check 7: Peaks detected ---");
    if result.peaks.is_empty() {
        println!("  [FAIL] no peaks");
        failed += 1;
    } else {
        println!("  [PASS] {} peaks detected", result.peaks.len());
        passed += 1;
    }

    // Evaluate against truth
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "tol_samples < 1000 fits in usize"
    )]
    let tol_samples = (75.0 * fs / 1000.0) as usize;
    let metrics = biosignal::evaluate_detection(&result.peaks, &true_peaks, tol_samples);

    // Check 8: Sensitivity > 80%
    println!("\n--- Check 8: Sensitivity > 80% ---");
    if metrics.sensitivity > 0.8 {
        println!(
            "  [PASS] Se = {:.3} ({}/{})",
            metrics.sensitivity,
            metrics.tp,
            metrics.tp + metrics.fn_count
        );
        passed += 1;
    } else {
        println!("  [FAIL] Se = {:.3}", metrics.sensitivity);
        failed += 1;
    }

    // Check 9: PPV > 80%
    println!("\n--- Check 9: PPV > 80% ---");
    if metrics.ppv > 0.8 {
        println!(
            "  [PASS] PPV = {:.3} ({}/{})",
            metrics.ppv,
            metrics.tp,
            metrics.tp + metrics.fp
        );
        passed += 1;
    } else {
        println!("  [FAIL] PPV = {:.3}", metrics.ppv);
        failed += 1;
    }

    // Check 10: Heart rate
    println!("\n--- Check 10: Heart rate ---");
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    if (hr - 72.0).abs() < 10.0 {
        println!("  [PASS] HR = {hr:.1} bpm");
        passed += 1;
    } else {
        println!("  [FAIL] HR = {hr:.1}");
        failed += 1;
    }

    // Check 11: SDNN
    println!("\n--- Check 11: SDNN ---");
    let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
    if sdnn < 200.0 {
        println!("  [PASS] SDNN = {sdnn:.1} ms");
        passed += 1;
    } else {
        println!("  [FAIL] SDNN = {sdnn:.1}");
        failed += 1;
    }

    // Check 12: All pipeline stages same length
    println!("\n--- Check 12: Pipeline length consistency ---");
    if result.derivative.len() == ecg.len()
        && result.squared.len() == ecg.len()
        && result.mwi.len() == ecg.len()
    {
        println!("  [PASS] all {} samples", ecg.len());
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
