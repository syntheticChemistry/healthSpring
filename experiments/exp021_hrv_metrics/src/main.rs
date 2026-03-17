#![forbid(unsafe_code)]
// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp021 validation: HRV metrics (RMSSD, pNN50)
//!
//! Cross-validates `healthspring_barracuda::biosignal` HRV pipeline:
//! synthetic ECG → Pan-Tompkins → SDNN, RMSSD, pNN50, HR, mean RR.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;
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
    let mut h = ValidationHarness::new("Exp021 HRV Metrics");
    let write_baseline = env::args().any(|a| a == "--write-baseline");

    // Generate synthetic ECG (same params as exp020)
    let (ecg, _true_peaks) = biosignal::generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
    let result = biosignal::pan_tompkins(&ecg, FS);

    let sdnn = biosignal::sdnn_ms(&result.peaks, FS);
    let rmssd = biosignal::rmssd_ms(&result.peaks, FS);
    let pnn50_val = biosignal::pnn50(&result.peaks, FS);
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, FS);
    let mean_rr = mean_rr_ms(&result.peaks, FS);

    // Check 1: SDNN > 0 and < 200 ms
    h.check_lower("SDNN > 0", sdnn, 0.0);
    h.check_upper("SDNN < 200 ms", sdnn, 200.0);

    // Check 2: RMSSD > 0 and < 200 ms
    h.check_lower("RMSSD > 0", rmssd, 0.0);
    h.check_upper("RMSSD < 200 ms", rmssd, 200.0);

    // Check 3: pNN50 in [0, 100]%
    h.check_lower("pNN50 ≥ 0", pnn50_val, 0.0);
    h.check_upper("pNN50 ≤ 100", pnn50_val, 100.0);

    // Check 4: HR in [60, 90] bpm
    h.check_lower("HR ≥ 60", hr, 60.0);
    h.check_upper("HR ≤ 90", hr, 90.0);

    // Check 5: RMSSD vs SDNN (RMSSD ≤ 2×SDNN for typical HRV; √2×SDNN holds
    // for non-negative autocorrelation only; short segments can exceed)
    let rmssd_bound = 2.0 * sdnn;
    h.check_upper(
        "RMSSD ≤ 2×SDNN",
        rmssd,
        rmssd_bound + tolerances::HALF_LIFE_POINT,
    );

    // Check 6: Mean RR ≈ 60000/HR (within 5%)
    let expected_rr = 60_000.0 / hr;
    let rr_ratio = mean_rr / expected_rr;
    h.check_abs("Mean RR vs HR", rr_ratio, 1.0, 0.05);

    // Check 7: All RR intervals positive
    #[expect(clippy::cast_precision_loss, reason = "sample diffs < 2^52")]
    let rr_ms: Vec<f64> = result
        .peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / FS * 1000.0)
        .collect();
    let all_positive = rr_ms.iter().all(|&r| r > 0.0);
    h.check_bool("All RR intervals positive", all_positive);

    // Check 8: Number of detected beats ≈ 12
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "beat count < 1000 fits in i32"
    )]
    let diff = (result.peaks.len() as i32 - 12).abs();
    h.check_bool("Beat count ~12", diff <= 1);

    // Check 9: pNN50 consistency (for low jitter, pNN50 should be small)
    h.check_bool("pNN50 consistency (low jitter)", pnn50_val < 50.0);

    // Check 10: Determinism (run twice, bit-identical)
    let (ecg2, _) = biosignal::generate_synthetic_ecg(FS, 10.0, 72.0, 0.05, 42);
    let result2 = biosignal::pan_tompkins(&ecg2, FS);
    let deterministic = ecg.len() == ecg2.len()
        && ecg
            .iter()
            .zip(ecg2.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits())
        && result.peaks == result2.peaks;
    h.check_bool("Determinism", deterministic);

    if write_baseline {
        write_baseline_json(sdnn, rmssd, pnn50_val, hr, mean_rr, result.peaks.len());
    }

    h.exit();
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
    let s = serde_json::to_string_pretty(&baseline).unwrap_or_default();
    if fs::write(&path, s).is_err() {
        eprintln!("FAIL: write baseline");
        std::process::exit(1);
    }
    println!("\nBaseline written to {}", path.display());
}
