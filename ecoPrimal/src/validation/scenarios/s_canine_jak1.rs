// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::canine;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "canine-jak1-selectivity",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp101",
            description: "Canine JAK1 oclacitinib selectivity vs human reference panels.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — JAK Selectivity");

    let panel = canine::canine_jak_ic50_panel();
    let sel = panel.jak1_selectivity();
    v.check_bool(
        "oclacitinib_jak1_selective",
        sel > 1.0,
        &format!("JAK1_selectivity={sel}"),
    );

    let human_refs = canine::human_jak_reference_panels();
    v.check_bool(
        "human_reference_panels_non_empty",
        !human_refs.is_empty(),
        &format!("n_panels={}", human_refs.len()),
    );

    for (i, ref_panel) in human_refs.iter().enumerate() {
        let ref_sel = ref_panel.jak1_selectivity();
        v.check_bool(
            &format!("human_panel_{i}_selectivity_positive"),
            ref_sel > 0.0,
            &format!("panel[{i}].jak1_selectivity={ref_sel}"),
        );
    }
}
