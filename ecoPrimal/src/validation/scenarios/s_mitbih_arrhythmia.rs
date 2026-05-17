// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::classification;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "mitbih-arrhythmia",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp109",
            description: "MIT-BIH arrhythmia beat classification with template matching.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Template generation");

    let n = 280;
    let normal = classification::generate_normal_template(n);
    let pvc = classification::generate_pvc_template(n);
    let pac = classification::generate_pac_template(n);

    v.check_bool(
        "normal_template_length",
        normal.len() == n,
        &format!("len={}", normal.len()),
    );
    v.check_bool(
        "pvc_template_length",
        pvc.len() == n,
        &format!("len={}", pvc.len()),
    );
    v.check_bool(
        "pac_template_length",
        pac.len() == n,
        &format!("len={}", pac.len()),
    );

    v.section("Phase 2: Structural — Classification of known morphologies");

    let min_corr = 0.5;
    let templates = vec![
        classification::BeatTemplate {
            class: classification::BeatClass::Normal,
            waveform: normal.clone(),
        },
        classification::BeatTemplate {
            class: classification::BeatClass::Pvc,
            waveform: pvc.clone(),
        },
        classification::BeatTemplate {
            class: classification::BeatClass::Pac,
            waveform: pac.clone(),
        },
    ];

    let (normal_class, normal_corr) = classification::classify_beat(&normal, &templates, min_corr);
    v.check_bool(
        "normal_classified_as_normal",
        normal_class == classification::BeatClass::Normal,
        &format!("class={normal_class:?}, corr={normal_corr:.4}"),
    );
    v.check_bool(
        "normal_high_correlation",
        normal_corr > 0.95,
        &format!("corr={normal_corr:.4}"),
    );

    let (ventricular_class, _) = classification::classify_beat(&pvc, &templates, min_corr);
    v.check_bool(
        "pvc_classified_as_pvc",
        ventricular_class == classification::BeatClass::Pvc,
        &format!("class={ventricular_class:?}"),
    );

    let (atrial_class, _) = classification::classify_beat(&pac, &templates, min_corr);
    v.check_bool(
        "pac_classified_as_pac",
        atrial_class == classification::BeatClass::Pac,
        &format!("class={atrial_class:?}"),
    );

    v.section("Phase 3: Structural — Normalized correlation properties");

    let self_corr = classification::normalized_correlation(&normal, &normal);
    v.check_abs_or_rel(
        "self_correlation_is_one",
        self_corr,
        1.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let cross_corr = classification::normalized_correlation(&normal, &pvc);
    v.check_bool(
        "cross_correlation_below_self",
        cross_corr < self_corr,
        &format!("cross={cross_corr:.4}, self={self_corr:.4}"),
    );
}
