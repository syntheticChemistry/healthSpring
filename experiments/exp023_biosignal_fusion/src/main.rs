#![forbid(unsafe_code)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp023 validation: Multi-channel biosignal fusion (ECG + PPG + EDA)
//!
//! Cross-validates `healthspring_barracuda::biosignal` fusion pipeline:
//! synthetic ECG → Pan-Tompkins; PPG → `SpO2`; EDA → SCR; `fuse_channels`.

use healthspring_barracuda::biosignal;
use std::env;
use std::fs;
use std::path::Path;

const ECG_FS: f64 = 360.0;
const PPG_FS: f64 = 256.0;
const EDA_FS: f64 = 32.0;
const SEED: u64 = 42;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let write_baseline = env::args().any(|a| a == "--write-baseline");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp023 — Multi-Channel Biosignal Fusion (ECG + PPG + EDA)");
    println!("  ECG: fs=360, 10s, 72bpm, noise=0.05");
    println!("  PPG: fs=256, 5s, 72bpm, SpO2=97");
    println!("  EDA: fs=32, 30s, SCL=2.0, SCR at [5,12,20,25]s");
    println!("{}", "=".repeat(72));

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

    // Check 1: EDA SCL extraction — SCL values near baseline (2.0 µS)
    println!("\n--- Check 1: EDA SCL near baseline 2.0 µS ---");
    #[expect(clippy::cast_precision_loss, reason = "scl.len() < 2^52")]
    let mean_scl: f64 = scl.iter().sum::<f64>() / scl.len() as f64;
    if (mean_scl - 2.0).abs() < 0.5 {
        println!("  [PASS] mean SCL = {mean_scl:.3} µS");
        passed += 1;
    } else {
        println!("  [FAIL] mean SCL = {mean_scl:.3} µS (expected ~2.0)");
        failed += 1;
    }

    // Check 2: SCR detection — finds ~4 peaks
    println!("\n--- Check 2: SCR detection (~4 peaks) ---");
    if (3..=7).contains(&scr_count) {
        println!("  [PASS] {scr_count} SCR peaks detected");
        passed += 1;
    } else {
        println!("  [FAIL] {scr_count} SCR peaks (expected 3–7)");
        failed += 1;
    }

    // Check 3: Phasic EDA non-negative
    println!("\n--- Check 3: Phasic EDA non-negative ---");
    if phasic.iter().all(|&x| x >= 0.0) {
        println!("  [PASS] all phasic values ≥ 0");
        passed += 1;
    } else {
        println!("  [FAIL] some phasic values < 0");
        failed += 1;
    }

    // Check 4: HR in [60, 90] bpm
    println!("\n--- Check 4: HR in [60, 90] bpm ---");
    if (60.0..=90.0).contains(&fused.heart_rate_bpm) {
        println!("  [PASS] HR = {:.1} bpm", fused.heart_rate_bpm);
        passed += 1;
    } else {
        println!("  [FAIL] HR = {:.1} bpm", fused.heart_rate_bpm);
        failed += 1;
    }

    // Check 5: SpO2 in [92, 102]%
    println!("\n--- Check 5: SpO2 in [92, 102]% ---");
    if (92.0..=102.0).contains(&spo2) {
        println!("  [PASS] SpO2 = {spo2:.1}%");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2 = {spo2:.1}%");
        failed += 1;
    }

    // Check 6: SCR rate reasonable (< 20/min)
    println!("\n--- Check 6: SCR rate < 20/min ---");
    if fused.scr_rate_per_min < 20.0 {
        println!("  [PASS] SCR rate = {:.1}/min", fused.scr_rate_per_min);
        passed += 1;
    } else {
        println!("  [FAIL] SCR rate = {:.1}/min", fused.scr_rate_per_min);
        failed += 1;
    }

    // Check 7: Stress index in [0, 1]
    println!("\n--- Check 7: Stress index in [0, 1] ---");
    if (0.0..=1.0).contains(&fused.stress_index) {
        println!("  [PASS] stress_index = {:.3}", fused.stress_index);
        passed += 1;
    } else {
        println!("  [FAIL] stress_index = {:.3}", fused.stress_index);
        failed += 1;
    }

    // Check 8: Overall score in [0, 100]
    println!("\n--- Check 8: Overall score in [0, 100] ---");
    if (0.0..=100.0).contains(&fused.overall_score) {
        println!("  [PASS] overall_score = {:.1}", fused.overall_score);
        passed += 1;
    } else {
        println!("  [FAIL] overall_score = {:.1}", fused.overall_score);
        failed += 1;
    }

    // Check 9: Low-stress scenario — healthy HR + good SpO2 + low SCR → score ≥ 50
    // (Synthetic ECG has limited HRV jitter, so >60 is rarely reached)
    println!("\n--- Check 9: Low-stress → overall score ≥ 50 ---");
    if fused.overall_score >= 50.0 {
        println!("  [PASS] overall_score = {:.1} (≥50)", fused.overall_score);
        passed += 1;
    } else {
        println!(
            "  [FAIL] overall_score = {:.1} (expected ≥50)",
            fused.overall_score
        );
        failed += 1;
    }

    // Check 10: Fusion determinism (bit-identical on repeat)
    println!("\n--- Check 10: Fusion determinism ---");
    let fused2 = biosignal::fuse_channels(ecg_peaks, ECG_FS, spo2, scr_count, eda_duration_s);
    let deterministic = fused.heart_rate_bpm.to_bits() == fused2.heart_rate_bpm.to_bits()
        && fused.overall_score.to_bits() == fused2.overall_score.to_bits()
        && fused.stress_index.to_bits() == fused2.stress_index.to_bits();
    if deterministic {
        println!("  [PASS] bit-identical on repeat");
        passed += 1;
    } else {
        println!("  [FAIL] non-deterministic fusion");
        failed += 1;
    }

    // Check 11: EDA signal length = fs * duration
    println!("\n--- Check 11: EDA signal length ---");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "EDA_FS * 30 is small positive"
    )]
    let expected_eda_len = (EDA_FS * eda_duration_s) as usize;
    if eda.len() == expected_eda_len {
        println!("  [PASS] EDA len = {} (fs×duration)", eda.len());
        passed += 1;
    } else {
        println!(
            "  [FAIL] EDA len = {} (expected {expected_eda_len})",
            eda.len()
        );
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

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

    if failed > 0 {
        std::process::exit(1);
    }
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
    let s = serde_json::to_string_pretty(&baseline).expect("JSON serialize");
    fs::write(&path, s).expect("write baseline");
    println!("\nBaseline written to {}", path.display());
}
