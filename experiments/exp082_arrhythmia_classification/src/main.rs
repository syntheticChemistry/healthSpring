// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp082: Arrhythmia Beat Classification
//!
//! Template-matching beat morphology classification: Normal, PVC, PAC.
//! Validates against synthetic ECG with known beat types embedded.
//!
//! Reference: MIT-BIH Arrhythmia Database (Moody & Mark 2001),
//!            AAMI EC57 performance standards.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::tolerances::{MACHINE_EPSILON, QRS_SENSITIVITY};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp082 Arrhythmia Classification");

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
    let corr_self = biosignal::normalized_correlation(&normal_tmpl, &normal_tmpl);
    h.check_abs(
        "Normal self-correlation = 1.0",
        corr_self,
        1.0,
        MACHINE_EPSILON,
    );

    // Check 2: Normal vs PVC low correlation
    let corr_np = biosignal::normalized_correlation(&normal_tmpl, &ventricular_tmpl);
    h.check_bool("Normal-PVC correlation < 0.5", corr_np < 0.5);

    // Check 3: Classify normal beat correctly
    let (class, corr) = biosignal::classify_beat(&normal_tmpl, &templates, 0.7);
    h.check_bool(
        "Classify normal beat",
        class == biosignal::BeatClass::Normal && corr > 0.99,
    );

    // Check 4: Classify PVC beat correctly
    let (class, corr) = biosignal::classify_beat(&ventricular_tmpl, &templates, 0.7);
    h.check_bool(
        "Classify PVC beat",
        class == biosignal::BeatClass::Pvc && corr > 0.99,
    );

    // Check 5: Classify PAC beat correctly
    let (class, corr) = biosignal::classify_beat(&atrial_tmpl, &templates, 0.7);
    h.check_bool(
        "Classify PAC beat",
        class == biosignal::BeatClass::Pac && corr > 0.99,
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
    let results =
        biosignal::classify_all_beats(&signal, &beat_positions, &templates, half_width, 0.7);
    h.check_exact("Batch classification count", results.len() as u64, 5);

    // Check 7: Confusion matrix for Normal
    let predicted: Vec<biosignal::BeatClass> = results.iter().map(|r| r.class).collect();
    let cm_n =
        biosignal::confusion_for_class(&predicted, &beat_classes, biosignal::BeatClass::Normal);
    h.check_lower("Normal sensitivity >= 0.9", cm_n.sensitivity(), 0.9);

    // Check 8: Confusion matrix for PVC
    let cm_v = biosignal::confusion_for_class(&predicted, &beat_classes, biosignal::BeatClass::Pvc);
    h.check_lower("PVC sensitivity >= 0.9", cm_v.sensitivity(), 0.9);

    // Check 9: Overall accuracy > 80%
    let correct = predicted
        .iter()
        .zip(beat_classes.iter())
        .filter(|(p, t)| *p == *t)
        .count();
    #[expect(clippy::cast_precision_loss, reason = "beat count fits f64")]
    let accuracy = correct as f64 / beat_classes.len() as f64;
    h.check_lower(
        "Overall accuracy >= QRS_SENSITIVITY",
        accuracy,
        QRS_SENSITIVITY,
    );

    // Check 10: PPV for Normal class
    h.check_lower("Normal PPV >= QRS_SENSITIVITY", cm_n.ppv(), QRS_SENSITIVITY);

    // Check 11: No unknown classifications for clean signal
    let n_unknown = predicted
        .iter()
        .filter(|&&c| c == biosignal::BeatClass::Unknown)
        .count();
    h.check_exact("No unknown classifications", n_unknown as u64, 0);

    // Check 12: Extract beat window correct length
    let window = biosignal::extract_beat_window(&signal, 300, half_width);
    h.check_exact("Beat window length", window.len() as u64, n_samples as u64);

    h.exit();
}
