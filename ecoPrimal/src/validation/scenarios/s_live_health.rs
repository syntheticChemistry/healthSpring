// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::composition::validate_liveness;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "live-health",
            track: Track::Composition,
            tier: Tier::Live,
            source_experiment: "exp121",
            description: "Normalized health probes across core NUCLEUS capabilities.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    v.check_minimum(
        "healthspring_declares_multiple_caps",
        crate::composition::ALL_CAPS.len(),
        4,
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");

    let _alive = validate_liveness(
        ctx,
        v,
        &["tensor", "security", "discovery", "compute", "storage"],
    );
}
