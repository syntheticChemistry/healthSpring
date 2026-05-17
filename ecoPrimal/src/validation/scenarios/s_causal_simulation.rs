// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::simulation;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "causal-simulation",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp111",
            description: "Causal chain simulation: stress pathways, fitness, ecosystem dynamics.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Stress Pathways");

    let pathways = simulation::standard_eukaryotic_pathways();
    v.check_bool(
        "standard_pathways_non_empty",
        !pathways.is_empty(),
        &format!("n_pathways={}", pathways.len()),
    );

    let fitness = simulation::mechanistic_cell_fitness(0.0, 1.0, &pathways, 50.0, 2.0);
    v.check_abs_or_rel(
        "zero_dose_full_fitness",
        fitness,
        1.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let fitness_stressed = simulation::mechanistic_cell_fitness(100.0, 1.0, &pathways, 50.0, 2.0);
    v.check_bool(
        "stress_reduces_fitness",
        fitness_stressed < fitness,
        &format!("fitness(0)={fitness}, fitness(100)={fitness_stressed}"),
    );

    v.section("Phase 1b: Causal Chain");

    let params = simulation::CausalChainParams {
        pathways: &pathways,
        damage_ic50: 50.0,
        damage_hill_n: 2.0,
        baseline_fitness: 100.0,
        tissue_sensitivity: 1.0,
        tissue_repair: 0.05,
        pop_k_base: 10_000.0,
    };

    let output = simulation::causal_chain(50.0, &params);
    v.check_bool(
        "causal_chain_cell_fitness_positive",
        output.cell_fitness > 0.0,
        &format!("cell_fitness={}", output.cell_fitness),
    );

    let output_low = simulation::causal_chain(1.0, &params);
    v.check_bool(
        "low_dose_hormetic",
        output_low.is_hormetic,
        &format!("cell_fitness={}, baseline=100", output_low.cell_fitness),
    );
}
