// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::{generate_synthetic_ecg, pan_tompkins, rmssd_ms, sdnn_ms};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "hrv-metrics",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp021",
            description: "Time-domain HRV metrics from detected R-peaks on synthetic ECG.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let fs = 360.0_f64;
    let (signal, _) = generate_synthetic_ecg(fs, 15.0, 68.0, 0.015, 99);
    let peaks = pan_tompkins(&signal, fs).peaks;

    let sdnn = sdnn_ms(&peaks, fs);
    let rmssd = rmssd_ms(&peaks, fs);

    v.check_bool(
        "sdnn_positive_with_rr_series",
        sdnn > tolerances::MACHINE_EPSILON_STRICT,
        &format!("SDNN={sdnn} ms"),
    );
    v.check_bool(
        "rmssd_finite_non_negative",
        rmssd.is_finite() && rmssd >= 0.0,
        &format!("RMSSD={rmssd} ms"),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "hrv_live_optional",
        "wearable pipeline not wired through CompositionContext",
    );
}
