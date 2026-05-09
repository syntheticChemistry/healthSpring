// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::pkpd::{MichaelisMentenParams, PHENYTOIN_PARAMS};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "michaelis-menten",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp077",
            description: "Michaelis-Menten elimination rate at C=Km equals Vmax/2 (local PKPD nonlinear model).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let params: &MichaelisMentenParams = &PHENYTOIN_PARAMS;
    let c = params.km;
    let elim_rate = params.vmax * c / (params.km + c);
    let half_vmax = params.vmax * 0.5;

    v.check_abs_or_rel(
        "mm_elim_rate_half_vmax_at_km",
        elim_rate,
        half_vmax,
        tolerances::MACHINE_EPSILON,
        tolerances::MACHINE_EPSILON,
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "michaelis_menten_live_optional",
        "nonlinear PKPD stays local — no generic MM IPC method",
    );
}
