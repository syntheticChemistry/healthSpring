// SPDX-License-Identifier: AGPL-3.0-or-later
//! Competitive ecosystem dynamics (Lotka-Volterra) with stress-modulated capacities.
//!
//! Multiple species, each with their own stress response.
//! A stressor reshapes the competitive landscape.

use crate::tolerances;

use super::population::population_steady_state;
use super::stress::{StressPathway, mechanistic_cell_fitness};

/// Species in a competitive ecosystem.
#[derive(Debug, Clone)]
pub struct Species {
    /// Name.
    pub name: &'static str,
    /// Population.
    pub population: f64,
    /// Intrinsic growth rate.
    pub growth_rate: f64,
    /// Carrying capacity at baseline (no stressor).
    pub k_base: f64,
    /// Damage IC50 — how resistant this species is.
    pub damage_ic50: f64,
    /// Stress response pathways.
    pub pathways: Vec<StressPathway>,
}

/// Lotka-Volterra competition step with stress-modulated capacities.
///
/// Each species' carrying capacity is modulated by its fitness response
/// to the stressor. Competition coefficients are symmetric (α = 1 for
/// intraspecific, configurable for interspecific).
///
/// Returns updated populations after one time step.
pub fn ecosystem_step(
    species: &mut [Species],
    dose: f64,
    baseline_fitness: f64,
    damage_hill_n: f64,
    competition_alpha: f64,
    dt: f64,
) {
    let n_species = species.len();

    let fitnesses: Vec<f64> = species
        .iter()
        .map(|s| {
            mechanistic_cell_fitness(
                dose,
                baseline_fitness,
                &s.pathways,
                s.damage_ic50,
                damage_hill_n,
            )
        })
        .collect();

    let effective_k: Vec<f64> = species
        .iter()
        .zip(&fitnesses)
        .map(|(s, &f)| population_steady_state(s.k_base, f, baseline_fitness))
        .collect();

    let pops: Vec<f64> = species.iter().map(|s| s.population).collect();

    for i in 0..n_species {
        let ki = effective_k[i].max(tolerances::DIVISION_GUARD);
        let competition: f64 = (0..n_species)
            .map(|j| {
                let alpha = if i == j { 1.0 } else { competition_alpha };
                alpha * pops[j] / ki
            })
            .sum();
        let dn = species[i].growth_rate * pops[i] * (1.0 - competition);
        species[i].population = (species[i].population + dn * dt).max(0.0);
    }
}

/// Run ecosystem simulation for multiple time steps.
///
/// Returns a matrix of population trajectories: `species × time`.
pub fn ecosystem_simulate(
    species: &mut [Species],
    dose: f64,
    baseline_fitness: f64,
    damage_hill_n: f64,
    competition_alpha: f64,
    t_end: f64,
    dt: f64,
) -> Vec<Vec<f64>> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t_end/dt small"
    )]
    let n_steps = (t_end / dt) as usize;
    let mut trajectories: Vec<Vec<f64>> = species.iter().map(|s| vec![s.population]).collect();

    for _ in 0..n_steps {
        ecosystem_step(
            species,
            dose,
            baseline_fitness,
            damage_hill_n,
            competition_alpha,
            dt,
        );
        for (i, s) in species.iter().enumerate() {
            trajectories[i].push(s.population);
        }
    }
    trajectories
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::population::population_dynamics;
    use super::super::stress::standard_eukaryotic_pathways;

    #[test]
    fn ecosystem_competition_reduces_populations() {
        let mut species = vec![
            Species {
                name: "A",
                population: 100.0,
                growth_rate: 0.3,
                k_base: 5000.0,
                damage_ic50: 50.0,
                pathways: standard_eukaryotic_pathways(),
            },
            Species {
                name: "B",
                population: 100.0,
                growth_rate: 0.3,
                k_base: 5000.0,
                damage_ic50: 50.0,
                pathways: standard_eukaryotic_pathways(),
            },
        ];
        let solo_traj = population_dynamics(100.0, 0.3, 5000.0, 100.0, 100.0, 50.0, 0.1);
        let solo_final = solo_traj[solo_traj.len() - 1];

        let trajs = ecosystem_simulate(&mut species, 0.0, 100.0, 2.0, 0.5, 50.0, 0.1);
        let comp_final = trajs[0][trajs[0].len() - 1];

        assert!(
            comp_final < solo_final,
            "competition reduces population: {comp_final} < {solo_final}"
        );
    }

    #[test]
    fn ecosystem_stress_reshapes_competition() {
        let resistant_pathways = vec![
            StressPathway {
                name: "HSP",
                max_benefit: 0.20,
                k_half: 0.5,
                hill_n: 1.5,
            },
            StressPathway {
                name: "autophagy",
                max_benefit: 0.15,
                k_half: 1.0,
                hill_n: 1.0,
            },
        ];
        let sensitive_pathways = vec![StressPathway {
            name: "HSP",
            max_benefit: 0.05,
            k_half: 2.0,
            hill_n: 1.5,
        }];

        let mut species = vec![
            Species {
                name: "resistant",
                population: 500.0,
                growth_rate: 0.2,
                k_base: 5000.0,
                damage_ic50: 80.0,
                pathways: resistant_pathways,
            },
            Species {
                name: "sensitive",
                population: 500.0,
                growth_rate: 0.3,
                k_base: 5000.0,
                damage_ic50: 15.0,
                pathways: sensitive_pathways,
            },
        ];

        let trajs = ecosystem_simulate(&mut species, 10.0, 100.0, 2.0, 0.8, 80.0, 0.1);
        let resistant_final = trajs[0][trajs[0].len() - 1];
        let sensitive_final = trajs[1][trajs[1].len() - 1];

        assert!(
            resistant_final > sensitive_final,
            "stress favors resistant species: {resistant_final} > {sensitive_final}"
        );
    }
}
