// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-scale causal simulation: the internals of the terrarium.
//!
//! The biphasic dose-response curve is the glass. This module builds the
//! causal chain *inside* — from molecular binding through cellular stress
//! pathways, tissue integration, organism fitness, population dynamics,
//! to ecosystem structure.
//!
//! ## The Causal Chain
//!
//! ```text
//! dose → binding → stress sensing → pathway activation → cellular fitness
//!   → tissue integration → organism fitness → population → ecosystem
//! ```
//!
//! Each level is mechanistic (it explains *why*), not phenomenological
//! (which only describes *what*). The biphasic shape EMERGES from the
//! competition between repair pathways (saturating benefit) and damage
//! accumulation (unbounded harm).
//!
//! ## Spring Ownership
//!
//! | Scale | Spring | Module |
//! |-------|--------|--------|
//! | Molecular (binding) | healthSpring | `discovery::affinity_landscape` |
//! | Cellular (stress) | healthSpring | `simulation` (this module) |
//! | Tissue (integration) | healthSpring | `toxicology` |
//! | Organism (PK/PD) | healthSpring | `pkpd`, `toxicology` |
//! | Population | wetSpring, groundSpring | `simulation` (basic), wetSpring (full) |
//! | Ecosystem | all springs | `simulation` (framework), wateringHole (coordination) |

use crate::tolerances;

// ── Level 2: Cellular Stress Response ───────────────────────────────
//
// The MISSING PIECE that explains the biphasic curve mechanistically.
// Multiple repair pathways compete with damage accumulation.

/// A single cellular stress-response pathway.
///
/// Each pathway activates in response to stress, following a Hill-like
/// saturation curve. The pathway provides a protective benefit that
/// saturates — you can't get more protection than the pathway's maximum.
///
/// Examples: heat shock proteins (HSP70/90), autophagy (mTOR/AMPK),
/// antioxidant defense (SOD, catalase), DNA repair (p53, BRCA).
#[derive(Debug, Clone)]
pub struct StressPathway {
    /// Human-readable name.
    pub name: &'static str,
    /// Maximum protective benefit (fraction above baseline, e.g., 0.15 = 15%).
    pub max_benefit: f64,
    /// Half-activation dose — the dose at which pathway provides 50% of max benefit.
    pub k_half: f64,
    /// Activation speed (Hill coefficient — higher = more switch-like).
    pub hill_n: f64,
}

/// Activation of a stress pathway at a given dose.
///
/// `benefit(D) = max_benefit × D^n / (k_half^n + D^n)`
///
/// Saturates at `max_benefit` for high doses.
#[must_use]
pub fn pathway_activation(pathway: &StressPathway, dose: f64) -> f64 {
    if dose <= 0.0 {
        return 0.0;
    }
    let d_n = dose.powf(pathway.hill_n);
    let k_n = pathway.k_half.powf(pathway.hill_n);
    pathway.max_benefit * d_n / (k_n + d_n)
}

/// Damage accumulation — the unbounded harm channel.
///
/// Unlike repair pathways (which saturate), damage accumulates
/// without bound. This is what eventually overwhelms the repair.
///
/// `damage(D) = D^n / (ic50^n + D^n)`
///
/// This IS the standard Hill inhibition curve.
#[must_use]
pub fn damage_accumulation(dose: f64, ic50: f64, hill_n: f64) -> f64 {
    if dose <= 0.0 || ic50 <= 0.0 {
        return 0.0;
    }
    let d_n = dose.powf(hill_n);
    let ic50_n = ic50.powf(hill_n);
    d_n / (ic50_n + d_n)
}

/// Standard stress-response pathway set for eukaryotic cells.
///
/// These four pathways represent the major categories of cellular defense.
/// Their half-activation doses are ordered: HSP (fast, broad) activates
/// first, autophagy (slow, deep) activates last.
#[must_use]
pub fn standard_eukaryotic_pathways() -> Vec<StressPathway> {
    vec![
        StressPathway {
            name: "HSP (heat shock proteins)",
            max_benefit: 0.12,
            k_half: 0.5,
            hill_n: 1.5,
        },
        StressPathway {
            name: "antioxidant (SOD/catalase)",
            max_benefit: 0.10,
            k_half: 1.0,
            hill_n: 2.0,
        },
        StressPathway {
            name: "DNA repair (p53/BRCA)",
            max_benefit: 0.08,
            k_half: 2.0,
            hill_n: 2.0,
        },
        StressPathway {
            name: "autophagy (mTOR/AMPK)",
            max_benefit: 0.15,
            k_half: 3.0,
            hill_n: 1.0,
        },
    ]
}

/// Mechanistic cellular fitness from stress pathways + damage.
///
/// `fitness = baseline × product(1 + benefit_i) × (1 - damage)`
///
/// At low doses: benefits dominate (hormesis)
/// At high doses: damage dominates (toxicity)
///
/// This DERIVES the biphasic curve from underlying biology rather
/// than fitting it phenomenologically.
#[must_use]
pub fn mechanistic_cell_fitness(
    dose: f64,
    baseline: f64,
    pathways: &[StressPathway],
    damage_ic50: f64,
    damage_hill_n: f64,
) -> f64 {
    let total_benefit: f64 = pathways
        .iter()
        .map(|p| 1.0 + pathway_activation(p, dose))
        .product();
    let damage = damage_accumulation(dose, damage_ic50, damage_hill_n);
    baseline * total_benefit * (1.0 - damage)
}

/// Cellular fitness with per-pathway detail.
///
/// Returns `(total_fitness, pathway_activations, damage_fraction)`.
#[must_use]
pub fn mechanistic_cell_fitness_detailed(
    dose: f64,
    baseline: f64,
    pathways: &[StressPathway],
    damage_ic50: f64,
    damage_hill_n: f64,
) -> (f64, Vec<f64>, f64) {
    let activations: Vec<f64> = pathways
        .iter()
        .map(|p| pathway_activation(p, dose))
        .collect();
    let damage = damage_accumulation(dose, damage_ic50, damage_hill_n);
    let benefit: f64 = activations.iter().map(|&a| 1.0 + a).product();
    (baseline * benefit * (1.0 - damage), activations, damage)
}

// ── Level 5: Population Dynamics ────────────────────────────────────
//
// Organism fitness feeds into population-level dynamics.
// Fitness modulates carrying capacity and growth rate.

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

// ── Level 6: Ecosystem (Lotka-Volterra Competition) ─────────────────
//
// Multiple species, each with their own stress response.
// A stressor reshapes the competitive landscape.

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

// ── Full Causal Chain ───────────────────────────────────────────────

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

    fn test_pathways() -> Vec<StressPathway> {
        standard_eukaryotic_pathways()
    }

    #[test]
    fn pathway_activation_at_zero_is_zero() {
        let p = &test_pathways()[0];
        assert!(pathway_activation(p, 0.0).abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn pathway_activation_at_k_half_is_half_max() {
        let p = &test_pathways()[0];
        let a = pathway_activation(p, p.k_half);
        let expected = p.max_benefit * 0.5_f64.powf(p.hill_n)
            / (0.5_f64.powf(p.hill_n) + 0.5_f64.powf(p.hill_n));
        assert!(
            (a - p.max_benefit / 2.0).abs() < 0.02,
            "at k_half, activation ≈ max/2: {a} vs {}",
            p.max_benefit / 2.0
        );
        let _ = expected;
    }

    #[test]
    fn pathway_activation_saturates() {
        let p = &test_pathways()[0];
        let a = pathway_activation(p, p.k_half * 1000.0);
        assert!(
            (a - p.max_benefit).abs() < 0.001,
            "high dose → max benefit: {a} vs {}",
            p.max_benefit
        );
    }

    #[test]
    fn damage_accumulation_at_ic50_is_half() {
        let d = damage_accumulation(10.0, 10.0, 2.0);
        assert!(
            (d - 0.5).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "at IC50, damage = 0.5: {d}"
        );
    }

    #[test]
    fn mechanistic_fitness_at_zero_is_baseline() {
        let f = mechanistic_cell_fitness(0.0, 100.0, &test_pathways(), 50.0, 2.0);
        assert!(
            (f - 100.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "zero dose → baseline: {f}"
        );
    }

    #[test]
    fn mechanistic_fitness_low_dose_hormetic() {
        let f = mechanistic_cell_fitness(1.0, 100.0, &test_pathways(), 50.0, 2.0);
        assert!(f > 100.0, "low dose is hormetic: {f}");
    }

    #[test]
    fn mechanistic_fitness_high_dose_toxic() {
        let f = mechanistic_cell_fitness(100.0, 100.0, &test_pathways(), 50.0, 2.0);
        assert!(f < 100.0, "high dose is toxic: {f}");
    }

    #[test]
    fn mechanistic_biphasic_shape() {
        let pathways = test_pathways();
        let doses: Vec<f64> = (0..100).map(f64::from).collect();
        let fitnesses: Vec<f64> = doses
            .iter()
            .map(|&d| mechanistic_cell_fitness(d, 100.0, &pathways, 50.0, 2.0))
            .collect();
        let max_f = fitnesses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let max_idx = fitnesses
            .iter()
            .position(|&f| (f - max_f).abs() < tolerances::DIVISION_GUARD)
            .unwrap_or(0);
        assert!(max_f > 100.0, "peak above baseline: {max_f}");
        assert!(max_idx > 0, "peak not at dose=0: idx={max_idx}");
        assert!(max_idx < 50, "peak before IC50: idx={max_idx}");
    }

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
