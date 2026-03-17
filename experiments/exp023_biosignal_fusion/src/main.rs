#![forbid(unsafe_code)]
// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp023 validation: Multi-channel biosignal fusion (ECG + PPG + EDA)
//!
//! Cross-validates `healthspring_barracuda::biosignal` fusion pipeline:
//! synthetic ECG → Pan-Tompkins; PPG → `SpO2`; EDA → SCR; `fuse_channels`.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::validation::ValidationHarness;
use std::env;
use std::fs;
use std::path::Path;

const ECG_FS: f64 = 360.0;
const PPG_FS: f64 = 256.0;
const EDA_FS: f64 = 32.0;
const SEED: u64 = 42;

fn main() {
    let mut h = ValidationHarness::new("Exp023 Biosignal Fusion");
    let write_baseline = env::args().any(|a| a == "--write-baseline");

    // Generate synthetic ECG (fs=360, 10s, 72bpm, noise=0.05, seed=42)
    let (ecg, _) = biosignal::generate_synthetic_ecg(ECG_FS, 10.0, 72.0, 0.05, SEED);
    let ecg_result = biosignal::pan_tompkins(&ecg, ECG_FS);
    let ecg_peaks = &ecg_result.peaks;

    // Generate synthetic PPG (fs=256, 5s, 72bpm, SpO2=97, seed=42)
    let ppg = biosignal::generate_synthetic_ppg(PPG_FS, 5.0, 72.0, 97.0, SEED);
    let (ac_red, dc_red) = biosignal::ppg_extract_ac_dc(&ppg.red);
    let (ac_ir, dc_ir) = biosignal::ppg_extract_ac_dc(&ppg.ir);
    let r_val = biosignal::ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
    let spo2 = biosignal::spo2_from_r(r_val);

    // Generate synthetic EDA (fs=32, 30s, SCL=2.0, SCR at [5,12,20,25], amp=0.5, seed=42)
    let scr_times = [5.0, 12.0, 20.0, 25.0];
    let eda_duration_s = 30.0;
    let eda = biosignal::generate_synthetic_eda(EDA_FS, eda_duration_s, 2.0, &scr_times, 0.5, SEED);
    let scl_window = 32;
    let scl = biosignal::eda_scl(&eda, scl_window);
    let phasic = biosignal::eda_phasic(&eda, scl_window);
    // threshold 0.08 µS, min_interval 48 samples (1.5s) to reduce noise-induced false SCR
    let scr_peaks = biosignal::eda_detect_scr(&phasic, 0.08, 48);
    let scr_count = scr_peaks.len();

    // Fuse all channels
    let fused = biosignal::fuse_channels(ecg_peaks, ECG_FS, spo2, scr_count, eda_duration_s);

    #[expect(clippy::cast_precision_loss, reason = "scl.len() < 2^52")]
    let mean_scl: f64 = scl.iter().sum::<f64>() / scl.len() as f64;
    h.check_abs("EDA SCL near 2.0 µS", mean_scl, 2.0, 0.5);

    h.check_bool("SCR detection ~4 peaks", (3..=7).contains(&scr_count));

    h.check_bool("Phasic EDA non-negative", phasic.iter().all(|&x| x >= 0.0));

    h.check_lower("HR ≥ 60", fused.heart_rate_bpm, 60.0);
    h.check_upper("HR ≤ 90", fused.heart_rate_bpm, 90.0);

    h.check_lower("SpO2 ≥ 92", spo2, 92.0);
    h.check_upper("SpO2 ≤ 102", spo2, 102.0);

    h.check_upper("SCR rate < 20/min", fused.scr_rate_per_min, 20.0);

    h.check_lower("Stress index ≥ 0", fused.stress_index, 0.0);
    h.check_upper("Stress index ≤ 1", fused.stress_index, 1.0);

    h.check_lower("Overall score ≥ 0", fused.overall_score, 0.0);
    h.check_upper("Overall score ≤ 100", fused.overall_score, 100.0);

    h.check_lower("Low-stress score ≥ 50", fused.overall_score, 50.0);

    let fused2 = biosignal::fuse_channels(ecg_peaks, ECG_FS, spo2, scr_count, eda_duration_s);
    let deterministic = fused.heart_rate_bpm.to_bits() == fused2.heart_rate_bpm.to_bits()
        && fused.overall_score.to_bits() == fused2.overall_score.to_bits()
        && fused.stress_index.to_bits() == fused2.stress_index.to_bits();
    h.check_bool("Fusion determinism", deterministic);

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "EDA_FS * 30 is small positive"
    )]
    let expected_eda_len = (EDA_FS * eda_duration_s) as usize;
    h.check_exact(
        "EDA signal length",
        eda.len() as u64,
        expected_eda_len as u64,
    );

    if write_baseline {
        write_baseline_json(
            &fused,
            mean_scl,
            scr_count,
            spo2,
            eda.len(),
            ecg_peaks.len(),
        );
    }

    h.exit();
}

fn write_baseline_json(
    fused: &biosignal::FusedHealthAssessment,
    mean_scl: f64,
    scr_count: usize,
    spo2: f64,
    eda_len: usize,
    ecg_peaks_count: usize,
) {
    let baseline = serde_json::json!({
        "_source": "healthSpring Exp023: Multi-Channel Biosignal Fusion",
        "_method": "ECG+PPG+EDA fusion, stress index, overall score",
        "ecg": {
            "fs": ECG_FS,
            "duration_s": 10.0,
            "heart_rate_bpm": 72,
            "noise": 0.05,
            "n_peaks": ecg_peaks_count
        },
        "ppg": {
            "fs": PPG_FS,
            "duration_s": 5.0,
            "spo2": spo2
        },
        "eda": {
            "fs": EDA_FS,
            "duration_s": 30.0,
            "scl_baseline": 2.0,
            "scr_times": [5.0, 12.0, 20.0, 25.0],
            "scr_count": scr_count,
            "mean_scl": mean_scl,
            "n_samples": eda_len
        },
        "fusion": {
            "heart_rate_bpm": fused.heart_rate_bpm,
            "hrv_sdnn_ms": fused.hrv_sdnn_ms,
            "hrv_rmssd_ms": fused.hrv_rmssd_ms,
            "spo2_percent": fused.spo2_percent,
            "scr_rate_per_min": fused.scr_rate_per_min,
            "stress_index": fused.stress_index,
            "overall_score": fused.overall_score
        },
        "seed": SEED,
        "_provenance": {
            "date": "2026-03-08",
            "source": "Rust exp023_biosignal_fusion --write-baseline",
            "script": "control/biosignal/exp023_fusion.py"
        }
    });

    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../control/biosignal/exp023_baseline.json");
    let s = serde_json::to_string_pretty(&baseline).unwrap_or_default();
    if fs::write(&path, s).is_err() {
        eprintln!("FAIL: write baseline");
        std::process::exit(1);
    }
    println!("\nBaseline written to {}", path.display());
}
