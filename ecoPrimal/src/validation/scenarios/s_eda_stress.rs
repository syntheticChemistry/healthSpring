// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::eda;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "eda-stress",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp081",
            description: "Electrodermal activity: SCL/SCR decomposition and stress detection.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — EDA Processing");

    let fs = 500.0;
    let duration = 10.0;
    let scr_times = [2.0, 5.0, 8.0];
    let signal = eda::generate_synthetic_eda(fs, duration, 4.0, &scr_times, 0.5, 42);

    v.check_bool(
        "eda_signal_non_empty",
        !signal.is_empty(),
        &format!("n_samples={}", signal.len()),
    );

    let window = (fs * 0.5) as usize;
    let tonic = eda::eda_scl(&signal, window);
    v.check_bool(
        "tonic_same_length",
        tonic.len() == signal.len(),
        &format!("tonic_len={}, signal_len={}", tonic.len(), signal.len()),
    );

    let phasic = eda::eda_phasic(&signal, window);
    v.check_bool(
        "phasic_all_nonneg",
        phasic.iter().all(|&x| x >= 0.0),
        "phasic >= 0",
    );

    let peaks = eda::eda_detect_scr(&phasic, 0.05, (fs * 0.5) as usize);
    v.check_bool(
        "scr_events_detected",
        !peaks.is_empty(),
        &format!("n_events={}", peaks.len()),
    );
}
