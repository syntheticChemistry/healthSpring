// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::math_dispatch;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "hill-dose-response",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp001",
            description: "Hill equation structural identities (IC50 half-effect + monotonicity).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let mid = math_dispatch::hill(10.0, 10.0, 1.0);
    v.check_abs_or_rel(
        "hill_at_ic50_is_half",
        mid,
        0.5,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let lower = math_dispatch::hill(10.0, 10.0, 1.0);
    let higher = math_dispatch::hill(20.0, 10.0, 1.0);
    v.check_bool(
        "hill_monotonic_in_concentration",
        higher > lower,
        &format!("hill(20)={higher} > hill(10)={lower}"),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "hill_live_optional",
        "no live hill IPC in standard composition map — Tier-1 scenario",
    );
}
