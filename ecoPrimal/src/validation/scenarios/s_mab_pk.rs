// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::pkpd;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "mab-pk-allometry",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp004",
            description: "Monoclonal antibody SC PK + allometric scaling structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — mAb PK");

    let dose = 150.0;
    let vd = 5.0;
    let half_life = 21.0;
    let c0 = pkpd::mab_pk_sc(dose, vd, half_life, 0.0);
    v.check_bool(
        "mab_c0_positive",
        c0 > 0.0,
        &format!("c0={c0}"),
    );

    let c_half = pkpd::mab_pk_sc(dose, vd, half_life, half_life);
    v.check_abs_or_rel(
        "mab_half_life_halves_concentration",
        c_half / c0,
        0.5,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 1b: Allometric Scaling");

    let cl_mouse = 0.01;
    let bw_mouse = 0.025;
    let bw_human = 70.0;
    let scaled = pkpd::allometric_scale(cl_mouse, bw_mouse, bw_human, 0.75);
    v.check_bool(
        "allometric_scale_increases_for_larger_species",
        scaled > cl_mouse,
        &format!("mouse_cl={cl_mouse}, human_scaled={scaled}"),
    );

    let scaled_unity = pkpd::allometric_scale(cl_mouse, bw_mouse, bw_mouse, 0.75);
    v.check_abs_or_rel(
        "allometric_scale_identity_for_same_weight",
        scaled_unity,
        cl_mouse,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );
}
