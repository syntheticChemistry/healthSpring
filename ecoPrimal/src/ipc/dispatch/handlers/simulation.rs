// SPDX-License-Identifier: AGPL-3.0-or-later
//! Simulation and causal chain capability handlers.

use serde_json::Value;

use super::{f, missing};
use crate::simulation;
use crate::tolerances;

pub fn dispatch_mechanistic_fitness(params: &Value) -> Value {
    let (Some(dose), Some(baseline), Some(damage_ic50), Some(damage_hill_n)) = (
        f(params, "dose"),
        f(params, "baseline"),
        f(params, "damage_ic50"),
        f(params, "damage_hill_n"),
    ) else {
        return missing("dose, baseline, damage_ic50, damage_hill_n");
    };
    let pathways = simulation::standard_eukaryotic_pathways();
    let (fitness, activations, damage) = simulation::mechanistic_cell_fitness_detailed(
        dose,
        baseline,
        &pathways,
        damage_ic50,
        damage_hill_n,
    );
    serde_json::json!({
        "fitness": fitness,
        "pathway_activations": activations,
        "damage_fraction": damage,
        "is_hormetic": fitness > baseline,
    })
}

pub fn dispatch_ecosystem_simulate(params: &Value) -> Value {
    // Simplified: two species, user-defined parameters
    let (Some(dose), Some(baseline), Some(damage_hill_n), Some(t_end)) = (
        f(params, "dose"),
        f(params, "baseline"),
        f(params, "damage_hill_n"),
        f(params, "t_end"),
    ) else {
        return missing("dose, baseline, damage_hill_n, t_end");
    };
    let competition = f(params, "competition").unwrap_or(tolerances::DEFAULT_COMPETITION_COEFF);
    let dt = f(params, "dt").unwrap_or(tolerances::DEFAULT_ECOSYSTEM_DT);

    let mut species = vec![
        simulation::Species {
            name: "resistant",
            population: 500.0,
            growth_rate: 0.2,
            k_base: 5000.0,
            damage_ic50: 80.0,
            pathways: vec![
                simulation::StressPathway {
                    name: "HSP",
                    max_benefit: 0.20,
                    k_half: 0.5,
                    hill_n: 1.5,
                },
                simulation::StressPathway {
                    name: "autophagy",
                    max_benefit: 0.15,
                    k_half: 1.0,
                    hill_n: 1.0,
                },
            ],
        },
        simulation::Species {
            name: "sensitive",
            population: 500.0,
            growth_rate: 0.4,
            k_base: 5000.0,
            damage_ic50: 15.0,
            pathways: vec![simulation::StressPathway {
                name: "HSP",
                max_benefit: 0.05,
                k_half: 3.0,
                hill_n: 1.5,
            }],
        },
    ];

    let trajectories = simulation::ecosystem_simulate(
        &mut species,
        dose,
        baseline,
        damage_hill_n,
        competition,
        t_end,
        dt,
    );

    serde_json::json!({
        "species": ["resistant", "sensitive"],
        "final_populations": [
            trajectories[0].last().copied().unwrap_or(0.0),
            trajectories[1].last().copied().unwrap_or(0.0),
        ],
    })
}
