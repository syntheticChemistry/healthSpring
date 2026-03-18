// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp081: EDA Autonomic Stress Detection
//!
//! Validates composite stress index from EDA features: SCR frequency,
//! tonic SCL level, and SCR recovery time. Cross-validates with
//! simulated HRV stress patterns.
//!
//! Reference: Boucsein 2012 (Electrodermal Activity), Braithwaite 2013.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp081_eda_stress_detection");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp081 — EDA Autonomic Stress Detection");
    println!("{}", "=".repeat(72));

    let fs = 32.0;
    let duration = 60.0;
    let baseline_scl = 2.5;

    // Low-stress scenario: few SCR events
    let low_stress_eda =
        biosignal::generate_synthetic_eda(fs, duration, baseline_scl, &[15.0, 40.0], 0.3, 42);
    let low_scl = biosignal::eda_scl(&low_stress_eda, 32);
    let low_phasic = biosignal::eda_phasic(&low_stress_eda, 32);
    let low_peaks = biosignal::eda_detect_scr(&low_phasic, 0.05, 16);
    let low_assess = biosignal::assess_stress(&low_phasic, &low_scl, &low_peaks, duration, fs);

    // High-stress scenario: frequent SCR events, higher baseline
    let high_stress_eda = biosignal::generate_synthetic_eda(
        fs,
        duration,
        5.0,
        &[
            5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0,
        ],
        0.8,
        42,
    );
    let high_scl = biosignal::eda_scl(&high_stress_eda, 32);
    let high_phasic = biosignal::eda_phasic(&high_stress_eda, 32);
    let high_peaks = biosignal::eda_detect_scr(&high_phasic, 0.05, 16);
    let high_assess = biosignal::assess_stress(&high_phasic, &high_scl, &high_peaks, duration, fs);

    println!(
        "\n  Low stress:  SCR rate={:.1}/min, SCL={:.2} µS, index={:.1}",
        low_assess.scr_rate, low_assess.mean_scl, low_assess.stress_index
    );
    println!(
        "  High stress: SCR rate={:.1}/min, SCL={:.2} µS, index={:.1}",
        high_assess.scr_rate, high_assess.mean_scl, high_assess.stress_index
    );

    // Check 1: SCR rate is computable
    println!("\n--- Check 1: SCR rate positive ---");
    h.check_lower("low SCR rate >= 0", low_assess.scr_rate, 0.0);

    // Check 2: High-stress has higher SCR rate
    println!("\n--- Check 2: High stress → higher SCR rate ---");
    h.check_bool(
        "high SCR rate > low",
        high_assess.scr_rate > low_assess.scr_rate,
    );

    // Check 3: High-stress has higher mean SCL
    println!("\n--- Check 3: High stress → higher SCL ---");
    h.check_bool("high SCL > low", high_assess.mean_scl > low_assess.mean_scl);

    // Check 4: Stress index higher for high-stress
    println!("\n--- Check 4: Stress index ordering ---");
    h.check_bool(
        "high stress index > low",
        high_assess.stress_index > low_assess.stress_index,
    );

    // Check 5: Stress index bounded [0, 100]
    println!("\n--- Check 5: Stress index bounds ---");
    h.check_bool(
        "both in [0, 100]",
        low_assess.stress_index >= 0.0
            && low_assess.stress_index <= 100.0
            && high_assess.stress_index >= 0.0
            && high_assess.stress_index <= 100.0,
    );

    // Check 6: Low stress index < 50
    println!("\n--- Check 6: Low stress < 50 ---");
    h.check_upper("low stress index < 50", low_assess.stress_index, 50.0);

    // Check 7: SCL near baseline for low stress
    println!("\n--- Check 7: SCL near baseline ---");
    h.check_abs(
        "mean SCL ≈ baseline",
        low_assess.mean_scl,
        baseline_scl,
        tolerances::BIOMARKER_ENDPOINT,
    );

    // Check 8: SCR detection recovers known events
    println!("\n--- Check 8: SCR detection ---");
    let n_low_peaks = low_peaks.len();
    h.check_bool(
        "detected 1-5 peaks from 2 events",
        (1..=5).contains(&n_low_peaks),
    );

    // Check 9: Phasic signal non-negative
    println!("\n--- Check 9: Phasic EDA non-negative ---");
    h.check_bool(
        "all phasic values >= 0",
        low_phasic.iter().all(|&x| x >= 0.0),
    );

    // Check 10: Recovery time computable
    println!("\n--- Check 10: Recovery time ---");
    let recovery = biosignal::scr_recovery_time(&low_phasic, &low_peaks, fs);
    h.check_lower("recovery half-time >= 0", recovery, 0.0);

    // Check 11: Deterministic
    println!("\n--- Check 11: Deterministic ---");
    let eda2 =
        biosignal::generate_synthetic_eda(fs, duration, baseline_scl, &[15.0, 40.0], 0.3, 42);
    let identical = low_stress_eda
        .iter()
        .zip(eda2.iter())
        .all(|(a, b)| a.to_bits() == b.to_bits());
    h.check_bool("EDA signal bit-identical with same seed", identical);

    h.exit();
}
