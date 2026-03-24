// SPDX-License-Identifier: AGPL-3.0-or-later
//! Population dynamics: logistic growth and steady-state carrying capacity.
//!
//! Organism fitness feeds into population-level dynamics.
//! Fitness modulates carrying capacity and growth rate.

use crate::tolerances;

/// Logistic population growth with fitness-dependent carrying capacity.
///
/// `dN/dt = r × N × (1 - N / K(fitness))`
///
/// Higher fitness → higher K → larger sustainable population.
/// The fitness is the organism-level output of the causal chain.
///
/// `K(fitness) = K_base × (fitness / baseline)`
///
/// Returns population at time `t` via Euler integration.
#[must_use]
pub fn population_dynamics(
    initial_pop: f64,
    growth_rate: f64,
    k_base: f64,
    fitness: f64,
    baseline_fitness: f64,
    t_end: f64,
    dt: f64,
) -> Vec<f64> {
    let k_effective = if baseline_fitness > tolerances::DIVISION_GUARD {
        k_base * (fitness / baseline_fitness)
    } else {
        k_base
    };

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t_end/dt small"
    )]
    let n_steps = (t_end / dt) as usize;
    let mut trajectory = Vec::with_capacity(n_steps + 1);
    let mut n = initial_pop;
    trajectory.push(n);

    for _ in 0..n_steps {
        let dn = growth_rate * n * (1.0 - n / k_effective.max(tolerances::DIVISION_GUARD));
        n = (n + dn * dt).max(0.0);
        trajectory.push(n);
    }
    trajectory
}

/// Steady-state population for a given fitness level.
///
/// `N_ss = K_base × (fitness / baseline_fitness)`
///
/// When fitness > baseline: population grows beyond `K_base`.
/// When fitness < baseline: population shrinks below `K_base`.
#[must_use]
pub fn population_steady_state(k_base: f64, fitness: f64, baseline_fitness: f64) -> f64 {
    if baseline_fitness < tolerances::DIVISION_GUARD {
        return k_base;
    }
    k_base * (fitness / baseline_fitness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_steady_state_hormetic() {
        let ss = population_steady_state(10_000.0, 120.0, 100.0);
        assert!(
            (ss - 12_000.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "20% fitness gain → 20% more population: {ss}"
        );
    }

    #[test]
    fn population_steady_state_toxic() {
        let ss = population_steady_state(10_000.0, 50.0, 100.0);
        assert!(
            (ss - 5_000.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "50% fitness loss → 50% less population: {ss}"
        );
    }

    #[test]
    fn population_dynamics_converges() {
        let traj = population_dynamics(100.0, 0.5, 10_000.0, 120.0, 100.0, 100.0, 0.1);
        let final_pop = traj[traj.len() - 1];
        let expected_ss = 12_000.0;
        assert!(
            (final_pop - expected_ss).abs() / expected_ss < 0.05,
            "should converge to K_eff: {final_pop} vs {expected_ss}"
        );
    }
}
