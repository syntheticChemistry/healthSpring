// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp111: Causal terrarium — tracing causality from molecular binding
//! through stress pathways, cellular fitness, tissue integration,
//! organism fitness, population dynamics, to ecosystem restructuring.
//!
//! The biphasic curve is the glass. This traces what happens INSIDE.
//!
//! Four studies:
//! 1. Mechanistic derivation: stress pathways → biphasic curve
//! 2. Full causal chain: dose → cell → organism → population
//! 3. Ecosystem reshaping: pesticide changes competitive landscape
//! 4. Spring connectivity: every layer maps to a spring

use healthspring_barracuda::simulation;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp111_causal_terrarium");

    study_1_mechanistic_derivation(&mut h);
    study_2_full_causal_chain(&mut h);
    study_3_ecosystem_reshaping(&mut h);
    study_4_spring_connectivity(&mut h);

    h.exit();
}

/// Study 1: The biphasic curve EMERGES from stress pathway competition.
///
/// Four repair pathways (HSP, antioxidant, DNA repair, autophagy)
/// each activate at different doses. Together they create the
/// hormetic peak. Damage accumulation eventually overwhelms them all.
fn study_1_mechanistic_derivation(h: &mut ValidationHarness) {
    println!("\n─── Study 1: Mechanistic Derivation of the Biphasic Curve ───");

    let pathways = simulation::standard_eukaryotic_pathways();
    let baseline = 100.0;
    let damage_ic50 = 50.0;
    let damage_hill_n = 2.0;

    println!("  Stress pathways (the repair machinery inside the cell):");
    for p in &pathways {
        println!(
            "    {:30} max={:.0}%  k_half={:.1}  n={:.1}",
            p.name,
            p.max_benefit * 100.0,
            p.k_half,
            p.hill_n,
        );
    }

    println!("\n  Dose → pathway activations → damage → fitness:");
    let doses = [0.0, 0.5, 1.0, 3.0, 10.0, 30.0, 50.0, 80.0];
    for &d in &doses {
        let (fitness, activations, damage) = simulation::mechanistic_cell_fitness_detailed(
            d,
            baseline,
            &pathways,
            damage_ic50,
            damage_hill_n,
        );
        let total_benefit: f64 = activations.iter().sum::<f64>() * 100.0;
        let marker = if fitness > baseline {
            "↑"
        } else if fitness < baseline {
            "↓"
        } else {
            "="
        };
        println!(
            "    dose={d:5.1} | repair={total_benefit:5.1}% | damage={:.1}% | fitness={fitness:6.1} {marker}",
            damage * 100.0,
        );
    }

    let f_zero =
        simulation::mechanistic_cell_fitness(0.0, baseline, &pathways, damage_ic50, damage_hill_n);
    let f_low =
        simulation::mechanistic_cell_fitness(3.0, baseline, &pathways, damage_ic50, damage_hill_n);
    let f_high =
        simulation::mechanistic_cell_fitness(80.0, baseline, &pathways, damage_ic50, damage_hill_n);

    h.check_abs(
        "dose=0 → exact baseline (no pathways active)",
        f_zero,
        baseline,
        tolerances::MACHINE_EPSILON,
    );
    h.check_lower("low dose: repair > damage → hormesis", f_low, baseline);
    h.check_upper("high dose: damage > repair → toxicity", f_high, baseline);

    let (_, act_low, dam_low) = simulation::mechanistic_cell_fitness_detailed(
        3.0,
        baseline,
        &pathways,
        damage_ic50,
        damage_hill_n,
    );
    let (_, act_high, dam_high) = simulation::mechanistic_cell_fitness_detailed(
        80.0,
        baseline,
        &pathways,
        damage_ic50,
        damage_hill_n,
    );

    let repair_low: f64 = act_low.iter().sum();
    let repair_high: f64 = act_high.iter().sum();

    h.check_lower("low dose: total repair > damage", repair_low, dam_low);
    h.check_lower("high dose: damage > total repair", dam_high, repair_high);

    h.check_lower(
        "HSP activates first (lowest k_half)",
        simulation::pathway_activation(&pathways[0], 0.5),
        simulation::pathway_activation(&pathways[3], 0.5),
    );

    println!("  → The biphasic shape isn't a fit — it EMERGES from pathway competition");
    println!("  → HSP fires first (fast, broad), autophagy last (slow, deep)");
    println!("  → Each pathway saturates, but damage doesn't → eventually damage wins");
}

/// Study 2: Full causal chain — trace a single dose through every level.
///
/// dose → binding → pathways → cell fitness → tissue → organism → population
fn study_2_full_causal_chain(h: &mut ValidationHarness) {
    println!("\n─── Study 2: Full Causal Chain ───");

    let pathways = simulation::standard_eukaryotic_pathways();
    let chain_params = simulation::CausalChainParams {
        pathways: &pathways,
        damage_ic50: 50.0,
        damage_hill_n: 2.0,
        baseline_fitness: 100.0,
        tissue_sensitivity: 1.0,
        tissue_repair: 0.05,
        pop_k_base: 10_000.0,
    };

    println!("  Tracing causality at each dose level:");
    println!(
        "  {:>6} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Dose", "Repair%", "Damage%", "Cell", "Organ", "Pop_ss", "Zone"
    );

    let test_doses = [0.0, 1.0, 5.0, 15.0, 40.0, 70.0, 100.0];
    for &d in &test_doses {
        let out = simulation::causal_chain(d, &chain_params);
        let repair_pct: f64 = out.pathway_activations.iter().sum::<f64>() * 100.0;
        let zone = if out.is_hormetic { "hormetic" } else { "toxic" };
        println!(
            "  {:6.1} {:7.1}% {:7.1}% {:8.1} {:8.1} {:8.0} {:>8}",
            d,
            repair_pct,
            out.damage_fraction * 100.0,
            out.cell_fitness,
            out.organism_fitness,
            out.population_ss,
            zone,
        );
    }

    let hormetic = simulation::causal_chain(3.0, &chain_params);
    let toxic = simulation::causal_chain(70.0, &chain_params);

    h.check_bool(
        "hormetic zone: cell > baseline",
        hormetic.cell_fitness > chain_params.baseline_fitness,
    );
    h.check_bool(
        "hormetic zone: population grows",
        hormetic.population_ss > chain_params.pop_k_base,
    );
    h.check_bool(
        "toxic zone: cell < baseline",
        toxic.cell_fitness < chain_params.baseline_fitness,
    );
    h.check_bool(
        "toxic zone: population shrinks",
        toxic.population_ss < chain_params.pop_k_base,
    );
    h.check_lower(
        "damage increases from hormetic to toxic",
        toxic.damage_fraction,
        hormetic.damage_fraction,
    );

    let pop_increase_pct = (hormetic.population_ss / chain_params.pop_k_base - 1.0) * 100.0;
    let pop_decrease_pct = (1.0 - toxic.population_ss / chain_params.pop_k_base) * 100.0;
    println!("\n  Hormetic dose=3: population +{pop_increase_pct:.1}%");
    println!("  Toxic dose=70: population -{pop_decrease_pct:.1}%");
    println!("  → Every level is mechanistic: you can trace WHY the population changed");
}

fn resistant_pathways() -> Vec<simulation::StressPathway> {
    vec![
        simulation::StressPathway {
            name: "HSP",
            max_benefit: 0.20,
            k_half: 0.5,
            hill_n: 1.5,
        },
        simulation::StressPathway {
            name: "antioxidant",
            max_benefit: 0.15,
            k_half: 1.0,
            hill_n: 2.0,
        },
        simulation::StressPathway {
            name: "autophagy",
            max_benefit: 0.12,
            k_half: 2.0,
            hill_n: 1.0,
        },
    ]
}

fn sensitive_pathways() -> Vec<simulation::StressPathway> {
    vec![simulation::StressPathway {
        name: "HSP",
        max_benefit: 0.05,
        k_half: 3.0,
        hill_n: 1.5,
    }]
}

fn build_two_species() -> Vec<simulation::Species> {
    vec![
        simulation::Species {
            name: "resistant",
            population: 500.0,
            growth_rate: 0.2,
            k_base: 5000.0,
            damage_ic50: 80.0,
            pathways: resistant_pathways(),
        },
        simulation::Species {
            name: "sensitive",
            population: 500.0,
            growth_rate: 0.4,
            k_base: 5000.0,
            damage_ic50: 15.0,
            pathways: sensitive_pathways(),
        },
    ]
}

/// Study 3: Ecosystem reshaping — a pesticide changes who wins.
///
/// Two species: one resistant (strong stress pathways), one sensitive
/// (weak pathways but faster growth). Without pesticide, the fast grower
/// dominates. With pesticide, the resistant species takes over.
fn study_3_ecosystem_reshaping(h: &mut ValidationHarness) {
    println!("\n─── Study 3: Ecosystem Reshaping ───");

    let baseline = 100.0;
    let competition = 0.7;
    let dt = 0.1;
    let t_end = 100.0;

    println!("  No pesticide (dose=0):");
    let mut species_no_pest = build_two_species();
    let trajs_none = simulation::ecosystem_simulate(
        &mut species_no_pest,
        0.0,
        baseline,
        2.0,
        competition,
        t_end,
        dt,
    );
    let res_no = trajs_none[0].last().copied().unwrap_or(0.0);
    let sen_no = trajs_none[1].last().copied().unwrap_or(0.0);
    println!("    resistant: {res_no:.0}  |  sensitive: {sen_no:.0}");

    println!("  Moderate pesticide (dose=10):");
    let mut species_pest = build_two_species();
    let trajs_pest = simulation::ecosystem_simulate(
        &mut species_pest,
        10.0,
        baseline,
        2.0,
        competition,
        t_end,
        dt,
    );
    let res_pest = trajs_pest[0].last().copied().unwrap_or(0.0);
    let sen_pest = trajs_pest[1].last().copied().unwrap_or(0.0);
    println!("    resistant: {res_pest:.0}  |  sensitive: {sen_pest:.0}");

    h.check_lower(
        "without pesticide: fast grower (sensitive) dominates",
        sen_no,
        res_no,
    );
    h.check_lower(
        "with pesticide: resistant species takes over",
        res_pest,
        sen_pest,
    );

    let res_fitness =
        simulation::mechanistic_cell_fitness(10.0, baseline, &resistant_pathways(), 80.0, 2.0);
    let sen_fitness =
        simulation::mechanistic_cell_fitness(10.0, baseline, &sensitive_pathways(), 15.0, 2.0);
    println!(
        "\n  At dose=10: resistant fitness={res_fitness:.1}, sensitive fitness={sen_fitness:.1}"
    );
    h.check_lower(
        "resistant species has higher cellular fitness under stress",
        res_fitness,
        sen_fitness,
    );

    println!("  → Pesticide doesn't just kill — it reshapes the competitive landscape");
    println!("  → Microbes inside insects, QS in soil — every part of the terrarium responds");
    println!("  → groundSpring: soil community. wetSpring: gut flora. healthSpring: the organism.");
}

/// Study 4: Spring connectivity — mapping every causal layer.
///
/// Demonstrates that each level of the causal chain maps to an
/// existing or planned spring module.
fn study_4_spring_connectivity(h: &mut ValidationHarness) {
    println!("\n─── Study 4: Spring Connectivity Map ───");

    println!("  The terrarium's causal layers:");
    println!();
    println!("  ┌─────────────────────────────────────────────────────────────┐");
    println!("  │ Level 1: MOLECULAR BINDING          → healthSpring          │");
    println!("  │   discovery::affinity_landscape::fractional_occupancy       │");
    println!("  │   Hill dose-response, IC50, binding profile                 │");
    println!("  ├─────────────────────────────────────────────────────────────┤");
    println!("  │ Level 2: CELLULAR STRESS RESPONSE    → healthSpring         │");
    println!("  │   simulation::stress_pathways (HSP, SOD, p53, mTOR)        │");
    println!("  │   Mechanistic: repair saturates, damage doesn't             │");
    println!("  ├─────────────────────────────────────────────────────────────┤");
    println!("  │ Level 3: TISSUE INTEGRATION          → healthSpring         │");
    println!("  │   toxicology::compute_toxicity_landscape                    │");
    println!("  │   Anderson lattice: cells as sites, sensitivity as disorder │");
    println!("  ├─────────────────────────────────────────────────────────────┤");
    println!("  │ Level 4: ORGANISM (PK/PD)            → healthSpring         │");
    println!("  │   pkpd::compartment, pkpd::pbpk, pkpd::nonlinear           │");
    println!("  │   Absorption, distribution, metabolism, excretion           │");
    println!("  ├─────────────────────────────────────────────────────────────┤");
    println!("  │ Level 5: POPULATION DYNAMICS         → wetSpring/ground     │");
    println!("  │   simulation::population_dynamics, logistic growth          │");
    println!("  │   Fitness → carrying capacity → population size             │");
    println!("  ├─────────────────────────────────────────────────────────────┤");
    println!("  │ Level 6: ECOSYSTEM                   → all springs          │");
    println!("  │   simulation::ecosystem_simulate, Lotka-Volterra            │");
    println!("  │   Competition, mutualism, trophic cascades                  │");
    println!("  ├─────────────────────────────────────────────────────────────┤");
    println!("  │ Level 7: ENVIRONMENT                 → airSpring/ground     │");
    println!("  │   Dispersal, deposition, weathering, runoff                 │");
    println!("  │   The dose field that feeds back into Level 1               │");
    println!("  └─────────────────────────────────────────────────────────────┘");

    let pathways = simulation::standard_eukaryotic_pathways();
    h.check_exact(
        "4 standard stress pathways (HSP, SOD, p53, mTOR)",
        pathways.len() as u64,
        4,
    );

    let pathways = simulation::standard_eukaryotic_pathways();
    let chain_params = simulation::CausalChainParams {
        pathways: &pathways,
        damage_ic50: 50.0,
        damage_hill_n: 2.0,
        baseline_fitness: 100.0,
        tissue_sensitivity: 1.0,
        tissue_repair: 0.05,
        pop_k_base: 10_000.0,
    };
    let chain = simulation::causal_chain(5.0, &chain_params);
    h.check_exact(
        "causal chain activates all 4 pathways",
        chain.pathway_activations.len() as u64,
        4,
    );
    h.check_bool(
        "all pathway activations > 0 at dose=5",
        chain.pathway_activations.iter().all(|&a| a > 0.0),
    );
    h.check_bool(
        "causal chain produces population prediction",
        chain.population_ss > 0.0,
    );

    println!();
    println!("  At dose=5.0, the full chain reports:");
    println!(
        "    Pathway activations: {:?}",
        chain
            .pathway_activations
            .iter()
            .map(|a| format!("{a:.4}"))
            .collect::<Vec<_>>()
    );
    println!("    Damage fraction: {:.4}", chain.damage_fraction);
    println!("    Cell fitness: {:.1}", chain.cell_fitness);
    println!("    Organism fitness: {:.1}", chain.organism_fitness);
    println!("    Population steady state: {:.0}", chain.population_ss);
    println!("    Is hormetic: {}", chain.is_hormetic);
    println!();
    println!("  → Every number traces back to a mechanism");
    println!("  → Every mechanism maps to a spring");
    println!("  → The terrarium is already partially wired");
    println!("  → Build the missing layers → complete simulation of life");
}
