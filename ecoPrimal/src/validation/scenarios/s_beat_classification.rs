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

    let normal_waveform = classification::generate_normal_template(50);
    let pvc_waveform = classification::generate_pvc_template(50);
    let pac_waveform = classification::generate_pac_template(50);

    v.check_bool(
        "templates_correct_length",
        normal_waveform.len() == 50 && pvc_waveform.len() == 50 && pac_waveform.len() == 50,
        "all templates 50 samples",
    );

    let corr_nn = classification::normalized_correlation(&normal_waveform, &normal_waveform);
    v.check_bool(
        "self_correlation_is_one",
        (corr_nn - 1.0).abs() < 1e-10,
        &format!("corr(normal,normal)={corr_nn}"),
    );

    let corr_np = classification::normalized_correlation(&normal_waveform, &pvc_waveform);
    v.check_bool(
        "normal_pvc_correlation_lt_one",
        corr_np < 1.0,
        &format!("corr(normal,pvc)={corr_np}"),
    );

    let templates = vec![
        BeatTemplate { class: BeatClass::Normal, waveform: normal_waveform.clone() },
        BeatTemplate { class: BeatClass::Pvc, waveform: pvc_waveform },
        BeatTemplate { class: BeatClass::Pac, waveform: pac_waveform },
    ];

    let (label, _) = classification::classify_beat(&normal_waveform, &templates, 0.9);
    v.check_bool(
        "normal_beat_classified_correctly",
        label == BeatClass::Normal,
        &format!("label={label:?}"),
    );
}
