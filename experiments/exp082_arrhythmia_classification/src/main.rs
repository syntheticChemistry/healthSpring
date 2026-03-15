// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp082: Arrhythmia Beat Classification
//!
//! Template-matching beat morphology classification: Normal, PVC, PAC.
//! Validates against synthetic ECG with known beat types embedded.
//!
//! Reference: MIT-BIH Arrhythmia Database (Moody & Mark 2001),
//!            AAMI EC57 performance standards.

use healthspring_barracuda::biosignal;

macro_rules! check {
    ($p:expr, $f:expr, $name:expr, $cond:expr) => {
        if $cond {
            $p += 1;
            println!("  [PASS] {}", $name);
        } else {
            $f += 1;
            println!("  [FAIL] {}", $name);
        }
    };
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp082 — Arrhythmia Beat Classification");
    println!("{}", "=".repeat(72));

    let half_width = 20;
    let n_samples = 2 * half_width + 1;

    // Build templates
    let normal_tmpl = biosignal::generate_normal_template(n_samples);
    let ventricular_tmpl = biosignal::generate_pvc_template(n_samples);
    let atrial_tmpl = biosignal::generate_pac_template(n_samples);

    let templates = vec![
        biosignal::BeatTemplate {
            class: biosignal::BeatClass::Normal,
            waveform: normal_tmpl.clone(),
        },
        biosignal::BeatTemplate {
            class: biosignal::BeatClass::Pvc,
            waveform: ventricular_tmpl.clone(),
        },
        biosignal::BeatTemplate {
            class: biosignal::BeatClass::Pac,
            waveform: atrial_tmpl.clone(),
        },
    ];

    // Check 1: Self-correlation = 1.0
    println!("\n--- Check 1: Self-correlation ---");
    let corr_self = biosignal::normalized_correlation(&normal_tmpl, &normal_tmpl);
    check!(
        passed,
        failed,
        format!("normal self-correlation = {corr_self:.6}"),
        (corr_self - 1.0).abs() < 1e-10
    );

    // Check 2: Normal vs PVC low correlation
    println!("\n--- Check 2: Normal vs PVC discrimination ---");
    let corr_np = biosignal::normalized_correlation(&normal_tmpl, &ventricular_tmpl);
    check!(
        passed,
        failed,
        format!("normal-PVC correlation = {corr_np:.4} < 0.5"),
        corr_np < 0.5
    );

    // Check 3: Classify normal beat correctly
    println!("\n--- Check 3: Classify normal beat ---");
    let (class, corr) = biosignal::classify_beat(&normal_tmpl, &templates, 0.7);
    check!(
        passed,
        failed,
        format!("class={class}, corr={corr:.4}"),
        class == biosignal::BeatClass::Normal && corr > 0.99
    );

    // Check 4: Classify PVC beat correctly
    println!("\n--- Check 4: Classify PVC beat ---");
    let (class, corr) = biosignal::classify_beat(&ventricular_tmpl, &templates, 0.7);
    check!(
        passed,
        failed,
        format!("class={class}, corr={corr:.4}"),
        class == biosignal::BeatClass::Pvc && corr > 0.99
    );

    // Check 5: Classify PAC beat correctly
    println!("\n--- Check 5: Classify PAC beat ---");
    let (class, corr) = biosignal::classify_beat(&atrial_tmpl, &templates, 0.7);
    check!(
        passed,
        failed,
        format!("class={class}, corr={corr:.4}"),
        class == biosignal::BeatClass::Pac && corr > 0.99
    );

    // Build synthetic signal with embedded beats
    let mut signal = vec![0.0; 1000];
    let beat_positions: [usize; 5] = [100, 300, 500, 700, 900];
    let beat_classes = [
        biosignal::BeatClass::Normal,
        biosignal::BeatClass::Pvc,
        biosignal::BeatClass::Normal,
        biosignal::BeatClass::Pac,
        biosignal::BeatClass::Normal,
    ];
    let beat_templates: [&[f64]; 5] = [
        &normal_tmpl,
        &ventricular_tmpl,
        &normal_tmpl,
        &atrial_tmpl,
        &normal_tmpl,
    ];

    for (&pos, tmpl) in beat_positions.iter().zip(beat_templates.iter()) {
        let start = pos.saturating_sub(half_width);
        let end = (pos + half_width + 1).min(signal.len());
        let tmpl_start = half_width.saturating_sub(pos);
        for (i, j) in (start..end).zip(tmpl_start..) {
            if j < tmpl.len() {
                signal[i] = tmpl[j];
            }
        }
    }

    // Check 6: Classify all embedded beats
    println!("\n--- Check 6: Batch classification ---");
    let results =
        biosignal::classify_all_beats(&signal, &beat_positions, &templates, half_width, 0.7);
    check!(
        passed,
        failed,
        format!("classified {} beats", results.len()),
        results.len() == 5
    );

    // Check 7: Confusion matrix for Normal
    println!("\n--- Check 7: Normal confusion matrix ---");
    let predicted: Vec<biosignal::BeatClass> = results.iter().map(|r| r.class).collect();
    let cm_n =
        biosignal::confusion_for_class(&predicted, &beat_classes, biosignal::BeatClass::Normal);
    check!(
        passed,
        failed,
        format!(
            "Normal: TP={}, FP={}, FN={}, sens={:.2}",
            cm_n.true_positive,
            cm_n.false_positive,
            cm_n.false_negative,
            cm_n.sensitivity()
        ),
        cm_n.sensitivity() >= 0.9
    );

    // Check 8: Confusion matrix for PVC
    println!("\n--- Check 8: PVC confusion matrix ---");
    let cm_v = biosignal::confusion_for_class(&predicted, &beat_classes, biosignal::BeatClass::Pvc);
    check!(
        passed,
        failed,
        format!(
            "PVC: TP={}, FP={}, FN={}, sens={:.2}",
            cm_v.true_positive,
            cm_v.false_positive,
            cm_v.false_negative,
            cm_v.sensitivity()
        ),
        cm_v.sensitivity() >= 0.9
    );

    // Check 9: Overall accuracy > 80%
    println!("\n--- Check 9: Overall accuracy ---");
    let correct = predicted
        .iter()
        .zip(beat_classes.iter())
        .filter(|(p, t)| *p == *t)
        .count();
    #[expect(clippy::cast_precision_loss, reason = "beat count fits f64")]
    let accuracy = correct as f64 / beat_classes.len() as f64;
    check!(
        passed,
        failed,
        format!(
            "accuracy = {accuracy:.2} ({correct}/{})",
            beat_classes.len()
        ),
        accuracy >= 0.80
    );

    // Check 10: PPV for Normal class
    println!("\n--- Check 10: Normal PPV ---");
    check!(
        passed,
        failed,
        format!("Normal PPV = {:.2}", cm_n.ppv()),
        cm_n.ppv() >= 0.8
    );

    // Check 11: No unknown classifications for clean signal
    println!("\n--- Check 11: No unknowns ---");
    let n_unknown = predicted
        .iter()
        .filter(|&&c| c == biosignal::BeatClass::Unknown)
        .count();
    check!(
        passed,
        failed,
        format!("{n_unknown} unknown classifications"),
        n_unknown == 0
    );

    // Check 12: Extract beat window correct length
    println!("\n--- Check 12: Beat window extraction ---");
    let window = biosignal::extract_beat_window(&signal, 300, half_width);
    check!(
        passed,
        failed,
        format!("window length = {} (expected {})", window.len(), n_samples),
        window.len() == n_samples
    );

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("Exp082 Arrhythmia Classification: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
