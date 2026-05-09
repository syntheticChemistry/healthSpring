// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::{generate_synthetic_ecg, pan_tompkins};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "pan-tompkins-qrs",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp020",
            description: "Pan-Tompkins QRS detection on synthetic ECG yields peaks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let fs = 360.0_f64;
    let (signal, true_peaks) = generate_synthetic_ecg(fs, 12.0, 72.0, 0.02, 42);
    let result = pan_tompkins(&signal, fs);

    v.check_minimum("pan_tompkins_peaks_detected", result.peaks.len(), 3);
    v.check_minimum("synthetic_ecg_ground_truth_peaks", true_peaks.len(), 3);

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "pan_tompkins_live_optional",
        "biosignal Tier-1 — GPU/NPU dispatch not in composition map",
    );
}
