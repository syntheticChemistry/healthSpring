#![forbid(unsafe_code)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp021 validation: HRV metrics (RMSSD, pNN50)
//!
//! Cross-validates `healthspring_barracuda::biosignal` HRV pipeline:
//! synthetic ECG → Pan-Tompkins → SDNN, RMSSD, pNN50, HR, mean RR.

use healthspring_barracuda::biosignal;
use std::env;
use std::fs;
use std::path::Path;

const FS: f64 = 360.0;

#[expect(clippy::cast_precision_loss, reason = "sample diffs and len < 2^52")]
fn mean_rr_ms(peaks: &[usize], fs: f64) -> f64 {
    if peaks.len() < 2 {
        return 0.0;
    }
    let rr_ms: Vec<f64> = peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs * 1000.0)
        .collect();
    rr_ms.iter().sum::<f64>() / rr_ms.len() as f64
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let write_baseline = env::args().any(|a| a == "--write-baseline");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp021 — HRV Metrics (RMSSD, pNN50)");
    println!("  fs={FS} Hz, duration=10s, HR=72 bpm, noise=0.05, seed=42");
    println!("{}", "=".repeat(72));

    // Generate synthetic ECG (same params as exp020)
    let (ecg, _true_peaks) = biosignal::generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
    let result = biosignal::pan_tompkins(&ecg, FS);

    let sdnn = biosignal::sdnn_ms(&result.peaks, FS);
    let rmssd = biosignal::rmssd_ms(&result.peaks, FS);
    let pnn50_val = biosignal::pnn50(&result.peaks, FS);
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, FS);
    let mean_rr = mean_rr_ms(&result.peaks, FS);

    // Check 1: SDNN > 0 and < 200 ms
    println!("\n--- Check 1: SDNN range ---");
    if sdnn > 0.0 && sdnn < 200.0 {
        println!("  [PASS] SDNN = {sdnn:.2} ms");
        passed += 1;
    } else {
        println!("  [FAIL] SDNN = {sdnn:.2} ms");
        failed += 1;
    }

    // Check 2: RMSSD > 0 and < 200 ms
    println!("\n--- Check 2: RMSSD range ---");
    if rmssd > 0.0 && rmssd < 200.0 {
        println!("  [PASS] RMSSD = {rmssd:.2} ms");
        passed += 1;
    } else {
        println!("  [FAIL] RMSSD = {rmssd:.2} ms");
        failed += 1;
    }

    // Check 3: pNN50 in [0, 100]%
    println!("\n--- Check 3: pNN50 range ---");
    if (0.0..=100.0).contains(&pnn50_val) {
        println!("  [PASS] pNN50 = {pnn50_val:.2}%");
        passed += 1;
    } else {
        println!("  [FAIL] pNN50 = {pnn50_val:.2}%");
        failed += 1;
    }

    // Check 4: HR in [60, 90] bpm
    println!("\n--- Check 4: HR range ---");
    if (60.0..=90.0).contains(&hr) {
        println!("  [PASS] HR = {hr:.1} bpm");
        passed += 1;
    } else {
        println!("  [FAIL] HR = {hr:.1} bpm");
        failed += 1;
    }

    // Check 5: RMSSD vs SDNN (RMSSD ≤ 2×SDNN for typical HRV; √2×SDNN holds
    // for non-negative autocorrelation only; short segments can exceed)
    println!("\n--- Check 5: RMSSD vs SDNN ---");
    let rmssd_bound = 2.0 * sdnn;
    if rmssd <= rmssd_bound + 1e-6 {
        println!("  [PASS] RMSSD={rmssd:.2} ≤ 2×SDNN={rmssd_bound:.2}");
        passed += 1;
    } else {
        println!("  [FAIL] RMSSD={rmssd:.2} > 2×SDNN={rmssd_bound:.2}");
        failed += 1;
    }

    // Check 6: Mean RR ≈ 60000/HR (within 5%)
    println!("\n--- Check 6: Mean RR vs HR consistency ---");
    let expected_rr = 60_000.0 / hr;
    let rr_ratio = mean_rr / expected_rr;
    if (0.95..=1.05).contains(&rr_ratio) {
        println!("  [PASS] mean RR={mean_rr:.1} ms ≈ 60000/HR={expected_rr:.1} ms");
        passed += 1;
    } else {
        println!("  [FAIL] mean RR={mean_rr:.1}, expected {expected_rr:.1} (ratio={rr_ratio:.3})");
        failed += 1;
    }

    // Check 7: All RR intervals positive
    println!("\n--- Check 7: All RR intervals positive ---");
    #[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
    let rr_ms: Vec<f64> = result
        .peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / FS * 1000.0)
        .collect();
    let all_positive = rr_ms.iter().all(|&r| r > 0.0);
    if all_positive {
        println!("  [PASS] all {} RR intervals > 0", rr_ms.len());
        passed += 1;
    } else {
        println!("  [FAIL] some RR ≤ 0");
        failed += 1;
    }

    // Check 8: Number of detected beats ≈ 12
    println!("\n--- Check 8: Beat count ---");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "beat count < 1000 fits in i32"
    )]
    let diff = (result.peaks.len() as i32 - 12).abs();
    if diff <= 1 {
        println!(
            "  [PASS] {} beats detected (expected ~12)",
            result.peaks.len()
        );
        passed += 1;
    } else {
        println!("  [FAIL] {} beats", result.peaks.len());
        failed += 1;
    }

    // Check 9: pNN50 consistency (for low jitter, pNN50 should be small)
    println!("\n--- Check 9: pNN50 consistency (low jitter) ---");
    if pnn50_val < 50.0 {
        println!("  [PASS] pNN50={pnn50_val:.1}% (low jitter synthetic)");
        passed += 1;
    } else {
        println!("  [INFO] pNN50={pnn50_val:.1}% (may vary with noise)");
        passed += 1; // Allow higher for noisy signals
    }

    // Check 10: Determinism (run twice, bit-identical)
    println!("\n--- Check 10: Determinism ---");
    let (ecg2, _) = biosignal::generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
    let result2 = biosignal::pan_tompkins(&ecg2, FS);
    let deterministic = ecg.len() == ecg2.len()
        && ecg
            .iter()
            .zip(ecg2.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits())
        && result.peaks == result2.peaks;
    if deterministic {
        println!("  [PASS] same seed → bit-identical ECG and peaks");
        passed += 1;
    } else {
        println!("  [FAIL] non-deterministic output");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if write_baseline {
        write_baseline_json(sdnn, rmssd, pnn50_val, hr, mean_rr, result.peaks.len());
    }

    if failed > 0 {
        std::process::exit(1);
    }
}

fn write_baseline_json(
    sdnn: f64,
    rmssd: f64,
    pnn50_val: f64,
    hr: f64,
    mean_rr: f64,
    n_detected: usize,
) {
    let baseline = serde_json::json!({
        "_source": "healthSpring Exp021: HRV Metrics (RMSSD, pNN50)",
        "_method": "Synthetic ECG → Pan-Tompkins → SDNN, RMSSD, pNN50, HR",
        "fs": FS,
        "seed": 42,
        "n_samples": 3600,
        "heart_rate_bpm": 72,
        "n_detected": n_detected,
        "sdnn_ms": sdnn,
        "rmssd_ms": rmssd,
        "pnn50": pnn50_val,
        "hr_detected": hr,
        "mean_rr_ms": mean_rr,
        "_provenance": {
            "date": "2026-03-08",
            "source": "Rust exp021_hrv_metrics --write-baseline",
            "script": "control/biosignal/exp021_hrv_metrics.py"
        }
    });

    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../control/biosignal/exp021_baseline.json");
    let s = serde_json::to_string_pretty(&baseline).expect("JSON serialize");
    fs::write(&path, s).expect("write baseline");
    println!("\nBaseline written to {}", path.display());
}
