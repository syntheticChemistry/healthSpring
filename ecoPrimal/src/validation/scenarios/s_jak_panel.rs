// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::discovery::matrix_score;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "jak-panel-scoring",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp093",
            description: "JAK selectivity panel: pathway/geometry/disorder scoring structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Panel Scoring");

    let pathway = matrix_score::pathway_selectivity_score(1.0, &[100.0, 200.0, 50.0]);
    v.check_bool(
        "pathway_selectivity_positive",
        pathway > 0.0,
        &format!("pathway={pathway}"),
    );

    let geo = matrix_score::tissue_geometry_factor(5.0, 10.0);
    v.check_bool(
        "geometry_factor_bounded",
        geo >= 0.0 && geo <= 1.0,
        &format!("geo={geo}"),
    );

    let disorder = matrix_score::disorder_impact_factor(3.0, 1.0);
    v.check_bool(
        "disorder_impact_positive",
        disorder > 0.0,
        &format!("disorder={disorder}"),
    );

    let combined = matrix_score::matrix_combined_score(pathway, geo, disorder);
    v.check_bool(
        "combined_score_positive",
        combined > 0.0,
        &format!("combined={combined}"),
    );

    let no_offtarget = matrix_score::pathway_selectivity_score(1.0, &[]);
    v.check_bool(
        "no_off_target_max_selectivity",
        no_offtarget >= pathway,
        &format!("no_offtarget={no_offtarget}"),
    );
}
