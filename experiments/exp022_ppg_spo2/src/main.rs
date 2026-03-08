#![forbid(unsafe_code)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp022 validation: PPG `SpO2` R-value calibration
//!
//! Cross-validates `healthspring_barracuda::biosignal` PPG pipeline:
//! R-value, `SpO2` calibration, synthetic PPG generation, AC/DC extraction.

use healthspring_barracuda::biosignal;
use std::env;
use std::fs;
use std::path::Path;

const FS: f64 = 256.0;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let write_baseline = env::args().any(|a| a == "--write-baseline");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp022 — PPG SpO2 R-value Calibration");
    println!("  R-value = (AC_red/DC_red)/(AC_ir/DC_ir), SpO2 = 110 - 25*R");
    println!("{}", "=".repeat(72));

    // Check 1: R=0.4 → SpO2=100% (clamped)
    println!("\n--- Check 1: R=0.4 → SpO2=100% ---");
    let spo2_04 = biosignal::spo2_from_r(0.4);
    if (spo2_04 - 100.0).abs() < 1e-10 {
        println!("  [PASS] SpO2(R=0.4) = {spo2_04:.2}%");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2(R=0.4) = {spo2_04:.2}% (expected 100%)");
        failed += 1;
    }

    // Check 2: R=0.6 → SpO2=95%
    println!("\n--- Check 2: R=0.6 → SpO2=95% ---");
    let spo2_06 = biosignal::spo2_from_r(0.6);
    if (spo2_06 - 95.0).abs() < 1e-10 {
        println!("  [PASS] SpO2(R=0.6) = {spo2_06:.2}%");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2(R=0.6) = {spo2_06:.2}% (expected 95%)");
        failed += 1;
    }

    // Check 3: R=0.8 → SpO2=90%
    println!("\n--- Check 3: R=0.8 → SpO2=90% ---");
    let spo2_08 = biosignal::spo2_from_r(0.8);
    if (spo2_08 - 90.0).abs() < 1e-10 {
        println!("  [PASS] SpO2(R=0.8) = {spo2_08:.2}%");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2(R=0.8) = {spo2_08:.2}% (expected 90%)");
        failed += 1;
    }

    // Check 4: R=1.0 → SpO2=85%
    println!("\n--- Check 4: R=1.0 → SpO2=85% ---");
    let spo2_10 = biosignal::spo2_from_r(1.0);
    if (spo2_10 - 85.0).abs() < 1e-10 {
        println!("  [PASS] SpO2(R=1.0) = {spo2_10:.2}%");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2(R=1.0) = {spo2_10:.2}% (expected 85%)");
        failed += 1;
    }

    // Check 5: Synthetic PPG SpO2(97% target) → recovered within 5%
    println!("\n--- Check 5: Synthetic PPG 97% target roundtrip ---");
    let ppg_97 = biosignal::generate_synthetic_ppg(FS, 5.0, 72.0, 97.0, 42);
    let (ac_red_97, dc_red_97) = biosignal::ppg_extract_ac_dc(&ppg_97.red);
    let (ac_ir_97, dc_ir_97) = biosignal::ppg_extract_ac_dc(&ppg_97.ir);
    let r_97 = biosignal::ppg_r_value(ac_red_97, dc_red_97, ac_ir_97, dc_ir_97);
    let spo2_97 = biosignal::spo2_from_r(r_97);
    if (spo2_97 - 97.0).abs() < 5.0 {
        println!("  [PASS] SpO2 recovered = {spo2_97:.2}% (target 97%)");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2 recovered = {spo2_97:.2}% (target 97%)");
        failed += 1;
    }

    // Check 6: Synthetic PPG SpO2(90% target) → recovered within 5%
    println!("\n--- Check 6: Synthetic PPG 90% target roundtrip ---");
    let ppg_90 = biosignal::generate_synthetic_ppg(FS, 5.0, 72.0, 90.0, 42);
    let (ac_red_90, dc_red_90) = biosignal::ppg_extract_ac_dc(&ppg_90.red);
    let (ac_ir_90, dc_ir_90) = biosignal::ppg_extract_ac_dc(&ppg_90.ir);
    let r_90 = biosignal::ppg_r_value(ac_red_90, dc_red_90, ac_ir_90, dc_ir_90);
    let spo2_90 = biosignal::spo2_from_r(r_90);
    if (spo2_90 - 90.0).abs() < 5.0 {
        println!("  [PASS] SpO2 recovered = {spo2_90:.2}% (target 90%)");
        passed += 1;
    } else {
        println!("  [FAIL] SpO2 recovered = {spo2_90:.2}% (target 90%)");
        failed += 1;
    }

    // Check 7: AC/DC extraction — AC > 0 for pulsatile signal
    println!("\n--- Check 7: AC/DC extraction (AC > 0) ---");
    let (ac_red, dc_red) = biosignal::ppg_extract_ac_dc(&ppg_97.red);
    if ac_red > 0.0 && dc_red > 0.0 {
        println!("  [PASS] AC_red={ac_red:.6}, DC_red={dc_red:.6}");
        passed += 1;
    } else {
        println!("  [FAIL] AC_red={ac_red}, DC_red={dc_red}");
        failed += 1;
    }

    // Check 8: R-value division by zero → NaN
    println!("\n--- Check 8: R-value division by zero → NaN ---");
    let r_nan = biosignal::ppg_r_value(0.02, 1.0, 0.04, 0.0);
    if r_nan.is_nan() {
        println!("  [PASS] ppg_r_value(..., dc_ir=0) → NaN");
        passed += 1;
    } else {
        println!("  [FAIL] ppg_r_value(..., dc_ir=0) = {r_nan} (expected NaN)");
        failed += 1;
    }

    // Check 9: Clamping — extreme R → [0, 100]
    println!("\n--- Check 9: Clamping extreme R ---");
    let spo2_high = biosignal::spo2_from_r(-1.0);
    let spo2_low = biosignal::spo2_from_r(10.0);
    if (spo2_high - 100.0).abs() < 1e-10 && spo2_low.abs() < 1e-10 {
        println!("  [PASS] R=-1 → 100%, R=10 → 0%");
        passed += 1;
    } else {
        println!("  [FAIL] R=-1 → {spo2_high}%, R=10 → {spo2_low}%");
        failed += 1;
    }

    // Check 10: Determinism — same seed → bit-identical PPG
    println!("\n--- Check 10: Determinism ---");
    let ppg1 = biosignal::generate_synthetic_ppg(FS, 2.0, 72.0, 97.0, 42);
    let ppg2 = biosignal::generate_synthetic_ppg(FS, 2.0, 72.0, 97.0, 42);
    let deterministic = ppg1.red.len() == ppg2.red.len()
        && ppg1
            .red
            .iter()
            .zip(ppg2.red.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits());
    if deterministic {
        println!("  [PASS] same seed → bit-identical PPG");
        passed += 1;
    } else {
        println!("  [FAIL] non-deterministic PPG output");
        failed += 1;
    }

    // Check 11: Signal length matches fs * duration
    println!("\n--- Check 11: Signal length ---");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "fs * 5 is small positive — safe truncation"
    )]
    let expected_len = (FS * 5.0) as usize;
    if ppg_97.red.len() == expected_len {
        println!(
            "  [PASS] n_samples = {} (fs*5 = {expected_len})",
            ppg_97.red.len()
        );
        passed += 1;
    } else {
        println!(
            "  [FAIL] n_samples = {} (expected {expected_len})",
            ppg_97.red.len()
        );
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if write_baseline {
        write_baseline_json(
            spo2_04,
            spo2_06,
            spo2_08,
            spo2_10,
            spo2_97,
            spo2_90,
            ac_red,
            dc_red,
            r_97,
            ppg_97.red.len(),
        );
    }

    if failed > 0 {
        std::process::exit(1);
    }
}

#[expect(clippy::too_many_arguments, reason = "baseline JSON fields")]
fn write_baseline_json(
    spo2_r04: f64,
    spo2_r06: f64,
    spo2_r08: f64,
    spo2_r10: f64,
    spo2_97_recovered: f64,
    spo2_90_recovered: f64,
    ac_red: f64,
    dc_red: f64,
    r_97: f64,
    n_samples: usize,
) {
    let baseline = serde_json::json!({
        "_source": "healthSpring Exp022: PPG SpO2 R-value Calibration",
        "_method": "R-value, SpO2 calibration, synthetic PPG roundtrip",
        "spo2_r_values": {
            "r_0.4": spo2_r04,
            "r_0.6": spo2_r06,
            "r_0.8": spo2_r08,
            "r_1.0": spo2_r10
        },
        "synthetic_ppg": {
            "fs": FS,
            "duration_s": 5.0,
            "heart_rate_bpm": 72,
            "spo2_target_97": 97.0,
            "spo2_target_90": 90.0,
            "spo2_recovered_97": spo2_97_recovered,
            "spo2_recovered_90": spo2_90_recovered,
            "r_recovered_97": r_97,
            "ac_red": ac_red,
            "dc_red": dc_red,
            "n_samples": n_samples,
            "seed": 42
        },
        "_provenance": {
            "date": "2026-03-08",
            "source": "Rust exp022_ppg_spo2 --write-baseline",
            "script": "control/biosignal/exp022_ppg_spo2.py"
        }
    });

    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../control/biosignal/exp022_baseline.json");
    let s = serde_json::to_string_pretty(&baseline).expect("JSON serialize");
    fs::write(&path, s).expect("write baseline");
    println!("\nBaseline written to {}", path.display());
}
