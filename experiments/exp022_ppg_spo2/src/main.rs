#![forbid(unsafe_code)]
// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp022 validation: PPG `SpO2` R-value calibration
//!
//! Cross-validates `healthspring_barracuda::biosignal` PPG pipeline:
//! R-value, `SpO2` calibration, synthetic PPG generation, AC/DC extraction.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;
use std::env;
use std::fs;
use std::path::Path;

const FS: f64 = 256.0;

fn main() {
    let mut h = ValidationHarness::new("Exp022 PPG SpO2");
    let write_baseline = env::args().any(|a| a == "--write-baseline");

    // Check 1: R=0.4 → SpO2=100% (clamped)
    let spo2_04 = biosignal::spo2_from_r(0.4);
    h.check_abs("SpO2(R=0.4)", spo2_04, 100.0, tolerances::MACHINE_EPSILON);

    let spo2_06 = biosignal::spo2_from_r(0.6);
    h.check_abs("SpO2(R=0.6)", spo2_06, 95.0, tolerances::MACHINE_EPSILON);

    let spo2_08 = biosignal::spo2_from_r(0.8);
    h.check_abs("SpO2(R=0.8)", spo2_08, 90.0, tolerances::MACHINE_EPSILON);

    let spo2_10 = biosignal::spo2_from_r(1.0);
    h.check_abs("SpO2(R=1.0)", spo2_10, 85.0, tolerances::MACHINE_EPSILON);

    let ppg_97 = biosignal::generate_synthetic_ppg(FS, 5.0, 72.0, 97.0, 42);
    let (ac_red_97, dc_red_97) = biosignal::ppg_extract_ac_dc(&ppg_97.red);
    let (ac_ir_97, dc_ir_97) = biosignal::ppg_extract_ac_dc(&ppg_97.ir);
    let r_97 = biosignal::ppg_r_value(ac_red_97, dc_red_97, ac_ir_97, dc_ir_97);
    let spo2_97 = biosignal::spo2_from_r(r_97);
    h.check_abs(
        "SpO2 97% roundtrip",
        spo2_97,
        97.0,
        tolerances::SPO2_CLINICAL_TOLERANCE,
    );

    let ppg_90 = biosignal::generate_synthetic_ppg(FS, 5.0, 72.0, 90.0, 42);
    let (ac_red_90, dc_red_90) = biosignal::ppg_extract_ac_dc(&ppg_90.red);
    let (ac_ir_90, dc_ir_90) = biosignal::ppg_extract_ac_dc(&ppg_90.ir);
    let r_90 = biosignal::ppg_r_value(ac_red_90, dc_red_90, ac_ir_90, dc_ir_90);
    let spo2_90 = biosignal::spo2_from_r(r_90);
    h.check_abs(
        "SpO2 90% roundtrip",
        spo2_90,
        90.0,
        tolerances::SPO2_CLINICAL_TOLERANCE,
    );

    let (ac_red, dc_red) = biosignal::ppg_extract_ac_dc(&ppg_97.red);
    h.check_bool("AC/DC extraction", ac_red > 0.0 && dc_red > 0.0);

    let r_nan = biosignal::ppg_r_value(0.02, 1.0, 0.04, 0.0);
    h.check_bool("R-value div-by-zero → NaN", r_nan.is_nan());

    let spo2_high = biosignal::spo2_from_r(-1.0);
    let spo2_low = biosignal::spo2_from_r(10.0);
    h.check_abs(
        "Clamp R=-1 → 100%",
        spo2_high,
        100.0,
        tolerances::MACHINE_EPSILON,
    );
    h.check_abs(
        "Clamp R=10 → 0%",
        spo2_low,
        0.0,
        tolerances::MACHINE_EPSILON,
    );

    let ppg1 = biosignal::generate_synthetic_ppg(FS, 2.0, 72.0, 97.0, 42);
    let ppg2 = biosignal::generate_synthetic_ppg(FS, 2.0, 72.0, 97.0, 42);
    let deterministic = ppg1.red.len() == ppg2.red.len()
        && ppg1
            .red
            .iter()
            .zip(ppg2.red.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits());
    h.check_bool("Determinism", deterministic);

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "fs * 5 is small positive — safe truncation"
    )]
    let expected_len = (FS * 5.0) as usize;
    h.check_exact(
        "Signal length",
        ppg_97.red.len() as u64,
        expected_len as u64,
    );

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

    h.exit();
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
    let s = serde_json::to_string_pretty(&baseline).unwrap_or_default();
    if fs::write(&path, s).is_err() {
        eprintln!("FAIL: write baseline");
        std::process::exit(1);
    }
    println!("\nBaseline written to {}", path.display());
}
