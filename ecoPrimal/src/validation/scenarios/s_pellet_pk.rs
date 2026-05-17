// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::endocrine;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "pellet-pk",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp031",
            description: "Testosterone pellet implant zero-order release PK structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Pellet PK");

    let release_rate = 1.0;
    let ke = 0.1;
    let vd = 50.0;
    let duration = 90.0;

    let c_early = endocrine::pellet_concentration(10.0, release_rate, ke, vd, duration);
    v.check_bool(
        "pellet_early_concentration_positive",
        c_early > 0.0,
        &format!("c(10)={c_early}"),
    );

    let c_mid = endocrine::pellet_concentration(45.0, release_rate, ke, vd, duration);
    v.check_bool(
        "pellet_mid_gt_early",
        c_mid > c_early - tolerances::MACHINE_EPSILON,
        &format!("c(45)={c_mid}, c(10)={c_early}"),
    );

    let c_post = endocrine::pellet_concentration(120.0, release_rate, ke, vd, duration);
    v.check_bool(
        "pellet_post_depletion_decays",
        c_post < c_mid,
        &format!("c(120)={c_post}, c(45)={c_mid}"),
    );

    let c_zero = endocrine::pellet_concentration(0.0, release_rate, ke, vd, duration);
    v.check_abs_or_rel(
        "pellet_concentration_zero_at_t0",
        c_zero,
        0.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}
