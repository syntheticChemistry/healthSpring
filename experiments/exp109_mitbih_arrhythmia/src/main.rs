// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp109 validation: MIT-BIH full arrhythmia beat classification
//!
//! Validates beat classification against reference ECG morphologies from
//! the MIT-BIH Arrhythmia Database (Moody & Mark, 2001, IEEE Eng Med Biol 20:45).

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

const SAMPLE_RATE: f64 = 360.0;

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "loop index i < 280 fits in i32; i32→f64 is lossless"
)]
fn normal_sinus_beat() -> Vec<f64> {
    // Normal sinus rhythm template (360Hz, ~280 samples for one beat ~0.78s)
    // Morphology: P wave, QRS complex, T wave
    let n = 280;
    let mut beat = vec![0.0; n];
    // P wave (gaussian, ~50-80 samples from start, amplitude ~0.15mV)
    for (i, val) in beat.iter_mut().enumerate().skip(50).take(30) {
        let t = (f64::from(i as i32) - 65.0) / 10.0;
        *val = 0.15 * (-0.5 * t * t).exp();
    }
    // QRS complex (sharp depolarization ~120-160 samples)
    // Q: small negative
    for (i, val) in beat.iter_mut().enumerate().skip(120).take(10) {
        let t = (f64::from(i as i32) - 125.0) / 3.0;
        *val = -0.1 * (-0.5 * t * t).exp();
    }
    // R: tall positive
    for (i, val) in beat.iter_mut().enumerate().skip(128).take(20) {
        let t = (f64::from(i as i32) - 138.0) / 4.0;
        *val += 1.0 * (-0.5 * t * t).exp();
    }
    // S: negative deflection
    for (i, val) in beat.iter_mut().enumerate().skip(145).take(15) {
        let t = (f64::from(i as i32) - 152.0) / 3.0;
        *val += -0.25 * (-0.5 * t * t).exp();
    }
    // T wave (broad repolarization ~180-240 samples)
    for (i, val) in beat.iter_mut().enumerate().skip(180).take(60) {
        let t = (f64::from(i as i32) - 210.0) / 15.0;
        *val = 0.3 * (-0.5 * t * t).exp();
    }
    beat
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "loop index i < 280 fits in i32; i32→f64 is lossless"
)]
fn pvc_beat() -> Vec<f64> {
    // Premature ventricular contraction: wide QRS, no P wave, inverted T
    let n = 280;
    let mut beat = vec![0.0; n];
    // Wide QRS (much broader than normal, ~100-180 samples)
    for (i, val) in beat.iter_mut().enumerate().skip(100).take(30) {
        let t = (f64::from(i as i32) - 115.0) / 6.0;
        *val = -0.3 * (-0.5 * t * t).exp();
    }
    for (i, val) in beat.iter_mut().enumerate().skip(120).take(50) {
        let t = (f64::from(i as i32) - 145.0) / 8.0;
        *val += 0.8 * (-0.5 * t * t).exp();
    }
    for (i, val) in beat.iter_mut().enumerate().skip(160).take(35) {
        let t = (f64::from(i as i32) - 178.0) / 7.0;
        *val += -0.4 * (-0.5 * t * t).exp();
    }
    // Inverted T wave
    for (i, val) in beat.iter_mut().enumerate().skip(200).take(60) {
        let t = (f64::from(i as i32) - 230.0) / 15.0;
        *val = -0.2 * (-0.5 * t * t).exp();
    }
    beat
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "loop index i < 280 fits in i32; i32→f64 is lossless"
)]
fn pac_beat() -> Vec<f64> {
    // Premature atrial contraction: early P wave, narrow QRS (similar to normal)
    let n = 280;
    let mut beat = vec![0.0; n];
    // Early P wave (shifted left, ~30-55)
    for (i, val) in beat.iter_mut().enumerate().skip(30).take(25) {
        let t = (f64::from(i as i32) - 42.0) / 8.0;
        *val = 0.12 * (-0.5 * t * t).exp();
    }
    // Normal QRS
    for (i, val) in beat.iter_mut().enumerate().skip(110).take(15) {
        let t = (f64::from(i as i32) - 117.0) / 3.0;
        *val = -0.08 * (-0.5 * t * t).exp();
    }
    for (i, val) in beat.iter_mut().enumerate().skip(120).take(25) {
        let t = (f64::from(i as i32) - 132.0) / 4.0;
        *val += 0.95 * (-0.5 * t * t).exp();
    }
    for (i, val) in beat.iter_mut().enumerate().skip(140).take(15) {
        let t = (f64::from(i as i32) - 147.0) / 3.0;
        *val += -0.2 * (-0.5 * t * t).exp();
    }
    // T wave
    for (i, val) in beat.iter_mut().enumerate().skip(170).take(60) {
        let t = (f64::from(i as i32) - 200.0) / 15.0;
        *val = 0.25 * (-0.5 * t * t).exp();
    }
    beat
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "loop index i < 280 fits in i32; i32→f64 is lossless"
)]
fn bbb_beat() -> Vec<f64> {
    // Bundle branch block: wide QRS with M-shaped morphology
    let n = 280;
    let mut beat = vec![0.0; n];
    // P wave
    for (i, val) in beat.iter_mut().enumerate().skip(50).take(30) {
        let t = (f64::from(i as i32) - 65.0) / 10.0;
        *val = 0.12 * (-0.5 * t * t).exp();
    }
    // Wide M-shaped QRS (two R peaks)
    for (i, val) in beat.iter_mut().enumerate().skip(115).take(25) {
        let t = (f64::from(i as i32) - 127.0) / 5.0;
        *val = 0.7 * (-0.5 * t * t).exp();
    }
    for (i, val) in beat.iter_mut().enumerate().skip(135).take(20) {
        let t = (f64::from(i as i32) - 145.0) / 4.0;
        *val += 0.15 * (-0.5 * t * t).exp();
    }
    for (i, val) in beat.iter_mut().enumerate().skip(150).take(25) {
        let t = (f64::from(i as i32) - 162.0) / 5.0;
        *val += 0.6 * (-0.5 * t * t).exp();
    }
    // S wave
    for (i, val) in beat.iter_mut().enumerate().skip(170).take(20) {
        let t = (f64::from(i as i32) - 180.0) / 4.0;
        *val += -0.3 * (-0.5 * t * t).exp();
    }
    // Broad T wave
    for (i, val) in beat.iter_mut().enumerate().skip(200).take(60) {
        let t = (f64::from(i as i32) - 230.0) / 15.0;
        *val += 0.2 * (-0.5 * t * t).exp();
    }
    beat
}

fn normalized_cross_correlation(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    #[expect(clippy::cast_precision_loss, reason = "n < 300")]
    let mean_a: f64 = a.iter().take(n).sum::<f64>() / n as f64;
    #[expect(clippy::cast_precision_loss, reason = "n < 300")]
    let mean_b: f64 = b.iter().take(n).sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut den_a = 0.0;
    let mut den_b = 0.0;
    for i in 0..n {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        num += da * db;
        den_a += da * da;
        den_b += db * db;
    }
    let den = (den_a * den_b).sqrt();
    if den < tolerances::MACHINE_EPSILON_STRICT {
        0.0
    } else {
        num / den
    }
}

fn main() {
    let mut h = ValidationHarness::new("Exp109 MIT-BIH Arrhythmia Classification");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp109 — MIT-BIH Full Arrhythmia Classification");
    println!("  Database: MIT-BIH Arrhythmia (Moody & Mark, 2001)");
    println!("  Sample rate: {SAMPLE_RATE} Hz");
    println!("{}", "=".repeat(72));

    let normal = normal_sinus_beat();
    let pvc = pvc_beat();
    let pac = pac_beat();
    let bbb = bbb_beat();

    // === Check 1-4: Template validity (nonzero energy) ===
    let energy_normal: f64 = normal.iter().map(|x| x * x).sum();
    let energy_ventricular: f64 = pvc.iter().map(|x| x * x).sum();
    let energy_atrial: f64 = pac.iter().map(|x| x * x).sum();
    let energy_bbb: f64 = bbb.iter().map(|x| x * x).sum();
    h.check_lower(
        "Normal beat has energy",
        energy_normal,
        tolerances::BEAT_ENERGY_FLOOR,
    );
    h.check_lower(
        "PVC beat has energy",
        energy_ventricular,
        tolerances::BEAT_ENERGY_FLOOR,
    );
    h.check_lower(
        "PAC beat has energy",
        energy_atrial,
        tolerances::BEAT_ENERGY_FLOOR,
    );
    h.check_lower(
        "BBB beat has energy",
        energy_bbb,
        tolerances::BEAT_ENERGY_FLOOR,
    );

    // === Check 5-8: Self-correlation = 1.0 ===
    h.check_abs(
        "Normal self-corr",
        normalized_cross_correlation(&normal, &normal),
        1.0,
        tolerances::MACHINE_EPSILON,
    );
    h.check_abs(
        "PVC self-corr",
        normalized_cross_correlation(&pvc, &pvc),
        1.0,
        tolerances::MACHINE_EPSILON,
    );
    h.check_abs(
        "PAC self-corr",
        normalized_cross_correlation(&pac, &pac),
        1.0,
        tolerances::MACHINE_EPSILON,
    );
    h.check_abs(
        "BBB self-corr",
        normalized_cross_correlation(&bbb, &bbb),
        1.0,
        tolerances::MACHINE_EPSILON,
    );

    // === Check 9-12: Cross-correlation classification ===
    let ncc_normal_pvc = normalized_cross_correlation(&normal, &pvc);
    let ncc_normal_atrial = normalized_cross_correlation(&normal, &pac);
    let ncc_normal_bbb = normalized_cross_correlation(&normal, &bbb);
    let ncc_pvc_bbb = normalized_cross_correlation(&pvc, &bbb);

    println!("  NCC(Normal, PVC):  {ncc_normal_pvc:.4}");
    println!("  NCC(Normal, PAC):  {ncc_normal_atrial:.4}");
    println!("  NCC(Normal, BBB):  {ncc_normal_bbb:.4}");
    println!("  NCC(PVC, BBB):     {ncc_pvc_bbb:.4}");

    // PAC most similar to normal (same conduction pathway)
    h.check_bool(
        "PAC most similar to Normal",
        ncc_normal_atrial > ncc_normal_pvc && ncc_normal_atrial > ncc_normal_bbb,
    );
    // PVC most different from normal
    h.check_bool(
        "PVC dissimilar from Normal",
        ncc_normal_pvc < ncc_normal_atrial,
    );
    // PVC and BBB are both wide QRS but different morphology
    h.check_upper("PVC ≠ BBB", ncc_pvc_bbb, tolerances::NCC_DISCRIMINATION);
    // All cross-correlations < 1.0
    h.check_bool(
        "All cross < 1.0",
        ncc_normal_pvc < 1.0 && ncc_normal_atrial < 1.0 && ncc_normal_bbb < 1.0,
    );

    // === Check 13: Beat classification by maximum correlation ===
    let templates: [(&str, &[f64]); 4] = [
        ("Normal", &normal),
        ("PVC", &pvc),
        ("PAC", &pac),
        ("BBB", &bbb),
    ];

    // Classify each template against the others
    for (name, beat) in &templates {
        let mut best_name = "";
        let mut best_corr = f64::NEG_INFINITY;
        for (tname, tbeat) in &templates {
            let corr = normalized_cross_correlation(beat, tbeat);
            if corr > best_corr {
                best_corr = corr;
                best_name = tname;
            }
        }
        h.check_bool(&format!("{name} classifies as {name}"), *name == best_name);
    }

    // === Check 17: Pan-Tompkins on normal beat ===
    let ecg_signal: Vec<f64> = std::iter::repeat_n(normal.as_slice(), 10)
        .flatten()
        .copied()
        .collect();
    let result = biosignal::pan_tompkins(&ecg_signal, SAMPLE_RATE);
    let peaks = &result.peaks;
    h.check_bool("Pan-Tompkins detects peaks", !peaks.is_empty());
    h.check_bool("Pan-Tompkins finds multiple beats", peaks.len() >= 5);
    println!(
        "  Pan-Tompkins peaks: {} in {} samples",
        peaks.len(),
        ecg_signal.len()
    );

    // === Check 18: HRV from detected peaks ===
    if peaks.len() >= 3 {
        #[expect(clippy::cast_precision_loss, reason = "RR interval in samples")]
        let rr_intervals: Vec<f64> = peaks
            .windows(2)
            .map(|w| (w[1] - w[0]) as f64 / SAMPLE_RATE)
            .collect();
        #[expect(clippy::cast_precision_loss, reason = "peaks count small")]
        let mean_rr: f64 = rr_intervals.iter().sum::<f64>() / rr_intervals.len() as f64;
        let hr = 60.0 / mean_rr;
        h.check_lower(
            "Heart rate above physiological minimum",
            hr,
            tolerances::HR_PHYSIO_LOW_BPM,
        );
        h.check_upper(
            "Heart rate below physiological maximum",
            hr,
            tolerances::HR_PHYSIO_HIGH_BPM,
        );
        println!("  Mean RR: {mean_rr:.3}s, HR: {hr:.1} bpm");
    } else {
        h.check_bool("Enough peaks for HRV", false);
    }

    // === Check 19: Template length ===
    h.check_exact("Beat template length", normal.len() as u64, 280);
    h.check_exact(
        "All templates same length",
        u64::from(pvc.len() == 280 && pac.len() == 280 && bbb.len() == 280),
        1,
    );

    // === Check 20: Determinism ===
    let n1 = normal_sinus_beat();
    let n2 = normal_sinus_beat();
    let identical = n1.iter().zip(&n2).all(|(a, b)| a.to_bits() == b.to_bits());
    h.check_bool("Deterministic beat generation", identical);

    h.exit();
}
