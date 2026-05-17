// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::classification::{self, BeatClass, BeatTemplate};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "beat-classification",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp082",
            description: "Arrhythmia beat classification (normal/PVC/PAC template correlation).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Beat Classification");

    let waveform_normal = classification::generate_normal_template(50);
    let waveform_pvc = classification::generate_pvc_template(50);
    let waveform_premature_atrial = classification::generate_pac_template(50);

    v.check_bool(
        "templates_correct_length",
        waveform_normal.len() == 50
            && waveform_pvc.len() == 50
            && waveform_premature_atrial.len() == 50,
        "all templates 50 samples",
    );

    let self_corr = classification::normalized_correlation(&waveform_normal, &waveform_normal);
    v.check_bool(
        "self_correlation_is_one",
        (self_corr - 1.0).abs() < 1e-10,
        &format!("corr(normal,normal)={self_corr}"),
    );

    let cross_corr = classification::normalized_correlation(&waveform_normal, &waveform_pvc);
    v.check_bool(
        "normal_pvc_correlation_lt_one",
        cross_corr < 1.0,
        &format!("corr(normal,pvc)={cross_corr}"),
    );

    let templates = vec![
        BeatTemplate { class: BeatClass::Normal, waveform: waveform_normal.clone() },
        BeatTemplate { class: BeatClass::Pvc, waveform: waveform_pvc },
        BeatTemplate { class: BeatClass::Pac, waveform: waveform_premature_atrial },
    ];

    let (label, _) = classification::classify_beat(&waveform_normal, &templates, 0.9);
    v.check_bool(
        "normal_beat_classified_correctly",
        label == BeatClass::Normal,
        &format!("label={label:?}"),
    );
}
