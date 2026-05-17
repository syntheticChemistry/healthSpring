// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::discovery::compound;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "compound-library",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp092",
            description: "Compound library: IC50 estimation, selectivity index structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — IC50 Estimation");

    let concentrations = [0.01, 0.1, 1.0, 10.0, 100.0];
    let responses = [0.95, 0.85, 0.5, 0.15, 0.05];
    let ic50_est = compound::estimate_ic50(&concentrations, &responses);
    v.check_bool(
        "ic50_positive",
        ic50_est.ic50 > 0.0,
        &format!("IC50={}", ic50_est.ic50),
    );

    v.section("Phase 1b: Selectivity");

    let si = compound::selectivity_index(1.0, 100.0);
    v.check_abs_or_rel(
        "selectivity_index_ratio",
        si,
        100.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let si_poor = compound::selectivity_index(100.0, 100.0);
    v.check_abs_or_rel(
        "no_selectivity_when_equal",
        si_poor,
        1.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}
