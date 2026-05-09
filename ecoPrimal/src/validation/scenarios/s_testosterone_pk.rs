// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::endocrine::pk_im_depot;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "testosterone-pk",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp030",
            description: "Testosterone IM depot PK structural anchor at t=0.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let c0 = pk_im_depot(100.0, 1.0, 70.0, 0.46, 0.087, 0.0);
    v.check_abs_or_rel(
        "im_depot_zero_at_t0",
        c0,
        0.0,
        tolerances::MACHINE_EPSILON_TIGHT,
        tolerances::MACHINE_EPSILON_TIGHT,
    );

    let c_pos = pk_im_depot(100.0, 1.0, 70.0, 0.46, 0.087, 24.0);
    v.check_bool(
        "im_depot_positive_after_absorption",
        c_pos > tolerances::MACHINE_EPSILON_STRICT,
        &format!("C(24h)={c_pos}"),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "testosterone_pk_live_optional",
        "endocrine PK local — no IPC in standard map",
    );
}
