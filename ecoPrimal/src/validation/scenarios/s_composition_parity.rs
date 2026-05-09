// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::tolerances::CPU_GPU_PARITY_TOL;
use primalspring::validation::ValidationResult;

use crate::composition::{validate_liveness, validate_parity};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "composition-parity",
            track: Track::Composition,
            tier: Tier::Live,
            source_experiment: "exp119",
            description: "Scalar composition parity vs local baseline (tensor stats.mean).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    v.check_bool(
        "routing_maps_tensor_to_barracuda",
        !crate::composition::capability_to_primal("tensor").is_empty(),
        "tensor capability resolves to a primal id string",
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");

    validate_liveness(ctx, v, &["tensor"]);

    validate_parity(
        ctx,
        v,
        "composition_tensor_mean_parity",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "result",
        3.0,
        CPU_GPU_PARITY_TOL,
    );
}
