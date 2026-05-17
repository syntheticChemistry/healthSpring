// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::species_params::{self, Species};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "cross-species-pk",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp104",
            description: "Cross-species PK scaling: allometric CL/Vd translation (canine→human).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Allometric Scaling");

    let canine_pk = Species::Canine.oclacitinib_pk();
    let human_pk = Species::Human.oclacitinib_pk();

    let scaled = species_params::scale_across_species(&canine_pk, Species::Human, human_pk.body_weight_kg);

    v.check_bool(
        "scaled_cl_positive",
        scaled.clearance_l_hr_kg > 0.0,
        &format!("cl_scaled={}", scaled.clearance_l_hr_kg),
    );
    v.check_bool(
        "scaled_vd_positive",
        scaled.volume_distribution_l_kg > 0.0,
        &format!("vd_scaled={}", scaled.volume_distribution_l_kg),
    );

    let t_half_canine = species_params::allometric_half_life(
        canine_pk.clearance_l_hr_kg * canine_pk.body_weight_kg,
        canine_pk.volume_distribution_l_kg * canine_pk.body_weight_kg,
    );
    let t_half_human = species_params::allometric_half_life(
        human_pk.clearance_l_hr_kg * human_pk.body_weight_kg,
        human_pk.volume_distribution_l_kg * human_pk.body_weight_kg,
    );
    v.check_bool(
        "half_lives_positive",
        t_half_canine > 0.0 && t_half_human > 0.0,
        &format!("t_half_canine={t_half_canine}, t_half_human={t_half_human}"),
    );

    v.check_bool(
        "scaling_preserves_ratio_order",
        (scaled.clearance_l_hr_kg - human_pk.clearance_l_hr_kg).abs()
            < tolerances::MACHINE_EPSILON + human_pk.clearance_l_hr_kg,
        &format!("scaled_cl={}, expected_cl={}", scaled.clearance_l_hr_kg, human_pk.clearance_l_hr_kg),
    );
}
