// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::discovery::matrix_score::{TissueContext, matrix_combined_score, score_compound};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "matrix-scoring",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp090",
            description: "MATRIX–Anderson compound scoring produces bounded combined score.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let ctx_mx = TissueContext {
        localization_length: 50.0,
        tissue_thickness: 10.0,
        w_baseline: 5.0,
        w_treated: 7.5,
    };
    let entry = score_compound(
        "test-compound",
        "test-indication",
        10.0,
        &[1000.0, 5000.0, 8000.0],
        &ctx_mx,
    );

    let recombined = matrix_combined_score(
        entry.pathway_score,
        entry.tissue_geometry,
        entry.disorder_factor,
    );
    v.check_abs_or_rel(
        "matrix_entry_combined_matches_product",
        entry.combined_score,
        recombined,
        tolerances::MACHINE_EPSILON,
        tolerances::MACHINE_EPSILON,
    );

    v.check_bool(
        "matrix_combined_non_negative",
        entry.combined_score >= 0.0,
        &format!("combined={}", entry.combined_score),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip("matrix_scoring_live_optional", "discovery scoring local");
}
