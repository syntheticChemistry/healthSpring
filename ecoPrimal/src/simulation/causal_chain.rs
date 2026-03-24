// SPDX-License-Identifier: AGPL-3.0-or-later
//! Full causal chain: dose through pathways, tissue, to population steady state.

use crate::tolerances;

use super::population::population_steady_state;
use super::stress::{StressPathway, mechanistic_cell_fitness_detailed};

/// Parameters for the full causal chain.
#[derive(Debug, Clone)]
pub struct CausalChainParams<'a> {
    /// Stress-response pathways.
    pub pathways: &'a [StressPathway],
    /// IC50 for damage accumulation.
    pub damage_ic50: f64,
    /// Hill coefficient for damage.
    pub damage_hill_n: f64,
    /// Baseline cellular fitness.
    pub baseline_fitness: f64,
    /// Tissue sensitivity weight.
    pub tissue_sensitivity: f64,
    /// Tissue repair capacity.
    pub tissue_repair: f64,
    /// Baseline carrying capacity.
    pub pop_k_base: f64,
}

/// Output of the full causal chain at one dose level.
#[derive(Debug, Clone)]
pub struct CausalChainOutput {
    /// External dose.
    pub dose: f64,
    /// Per-pathway activation levels (Level 2).
    pub pathway_activations: Vec<f64>,
    /// Total damage fraction (Level 2).
    pub damage_fraction: f64,
    /// Cellular fitness (Level 2 output).
    pub cell_fitness: f64,
    /// Organism fitness after tissue integration (Level 3-4).
    pub organism_fitness: f64,
    /// Predicted steady-state population (Level 5).
    pub population_ss: f64,
    /// Whether the organism is in the hormetic zone.
    pub is_hormetic: bool,
}

/// Run the full causal chain: dose → binding → pathways → cell → tissue → population.
///
/// This traces causality from an external stressor through every
/// mechanistic level, producing a complete causal narrative at each dose.
#[must_use]
pub fn causal_chain(dose: f64, params: &CausalChainParams<'_>) -> CausalChainOutput {
    let (cell_fitness, activations, damage) = mechanistic_cell_fitness_detailed(
        dose,
        params.baseline_fitness,
        params.pathways,
        params.damage_ic50,
        params.damage_hill_n,
    );

    let tissue_burden =
        (cell_fitness / params.baseline_fitness - 1.0).abs() * params.tissue_sensitivity;
    let organism_fitness = if cell_fitness >= params.baseline_fitness {
        cell_fitness
    } else {
        let excess = tissue_burden.mul_add(1.0, -params.tissue_repair).max(0.0);
        cell_fitness * (1.0 - excess.min(tolerances::TISSUE_EXCESS_CAP))
    };

    let pop_ss =
        population_steady_state(params.pop_k_base, organism_fitness, params.baseline_fitness);
    let is_hormetic = cell_fitness > params.baseline_fitness;

    CausalChainOutput {
        dose,
        pathway_activations: activations,
        damage_fraction: damage,
        cell_fitness,
        organism_fitness,
        population_ss: pop_ss,
        is_hormetic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::stress::standard_eukaryotic_pathways;

    fn test_chain_params() -> CausalChainParams<'static> {
        // Leak the pathways to get 'static — only used in tests.
        let pathways: &'static [StressPathway] =
            Box::leak(standard_eukaryotic_pathways().into_boxed_slice());
        CausalChainParams {
            pathways,
            damage_ic50: 50.0,
            damage_hill_n: 2.0,
            baseline_fitness: 100.0,
            tissue_sensitivity: 1.0,
            tissue_repair: 0.05,
            pop_k_base: 10_000.0,
        }
    }

    #[test]
    fn causal_chain_hormetic_zone() {
        let out = causal_chain(1.0, &test_chain_params());
        assert!(out.is_hormetic, "low dose → hormetic");
        assert!(
            out.cell_fitness > 100.0,
            "cell fitness above baseline: {}",
            out.cell_fitness
        );
        assert!(
            out.population_ss > 10_000.0,
            "population above K_base: {}",
            out.population_ss
        );
    }

    #[test]
    fn causal_chain_toxic_zone() {
        let out = causal_chain(80.0, &test_chain_params());
        assert!(!out.is_hormetic, "high dose → toxic");
        assert!(
            out.cell_fitness < 100.0,
            "cell fitness below baseline: {}",
            out.cell_fitness
        );
        assert!(
            out.population_ss < 10_000.0,
            "population below K_base: {}",
            out.population_ss
        );
    }

    #[test]
    fn causal_chain_damage_increases_with_dose() {
        let p = test_chain_params();
        let low = causal_chain(1.0, &p);
        let high = causal_chain(80.0, &p);
        assert!(
            high.damage_fraction > low.damage_fraction,
            "damage increases with dose"
        );
    }
}
