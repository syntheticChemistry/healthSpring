// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp098: Toxicity landscape — the body's burden of low-affinity binding.
//!
//! Computational evidence that **delocalized toxicity** (from many weak binders)
//! is more manageable than **localized toxicity** (from one strong binder),
//! using Anderson localization as the analytical framework.
//!
//! Five studies:
//! 1. Localized vs delocalized toxicity IPR
//! 2. Repair capacity buffer — weak binding stays below tissue thresholds
//! 3. Clearance regime — weak binders stay in linear kinetics
//! 4. Delocalization advantage — quantified benefit of distributed binding
//! 5. Disorder-modulated tissue landscape — Anderson on organ sensitivity

use healthspring_barracuda::tolerances;
use healthspring_barracuda::toxicology;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp098_toxicity_landscape");

    study_1_ipr_comparison(&mut h);
    study_2_repair_capacity_buffer(&mut h);
    study_3_clearance_regime(&mut h);
    study_4_delocalization_advantage(&mut h);
    study_5_disorder_modulated_landscape(&mut h);

    h.exit();
}

const TISSUE_NAMES: [&str; 8] = [
    "liver", "kidney", "heart", "muscle", "brain", "gut", "skin", "lung",
];
const SENSITIVITIES: [f64; 8] = [1.0, 1.0, 1.5, 0.5, 2.0, 0.8, 0.3, 1.0];
const REPAIRS: [f64; 8] = [0.05, 0.05, 0.03, 0.10, 0.02, 0.08, 0.10, 0.05];

fn build_tissue_profiles(occupancies: &[f64; 8]) -> Vec<toxicology::TissueToxProfile> {
    TISSUE_NAMES
        .iter()
        .zip(occupancies.iter())
        .zip(SENSITIVITIES.iter().zip(REPAIRS.iter()))
        .map(
            |((&name, &occ), (&sens, &repair))| toxicology::TissueToxProfile {
                name,
                occupancy: occ,
                sensitivity: sens,
                repair_capacity: repair,
            },
        )
        .collect()
}

/// Study 1: Toxicity IPR — localized vs delocalized.
///
/// Strong binder concentrates toxic burden in one tissue (high IPR).
/// Weak distributed binder spreads burden across many tissues (low IPR).
fn study_1_ipr_comparison(h: &mut ValidationHarness) {
    println!("\n─── Study 1: Toxicity IPR Comparison ───");

    let strong_tissues = build_tissue_profiles(&[0.85, 0.02, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01]);
    let weak_tissues = build_tissue_profiles(&[0.04, 0.04, 0.03, 0.04, 0.02, 0.04, 0.05, 0.04]);

    let strong_ipr = toxicology::toxicity_ipr(&strong_tissues);
    let weak_ipr = toxicology::toxicity_ipr(&weak_tissues);
    let strong_xi = toxicology::toxicity_localization_length(&strong_tissues);
    let weak_xi = toxicology::toxicity_localization_length(&weak_tissues);

    println!("  Strong binder: IPR={strong_ipr:.4}, ξ={strong_xi:.1} tissues");
    println!("  Weak binder:   IPR={weak_ipr:.4}, ξ={weak_xi:.1} tissues");

    h.check_lower(
        "strong binder: high IPR (localized toxicity)",
        strong_ipr,
        tolerances::TOX_IPR_LOCALIZED,
    );
    h.check_upper(
        "weak binder: low IPR (delocalized toxicity)",
        weak_ipr,
        tolerances::TOX_IPR_DELOCALIZED,
    );
    h.check_lower(
        "weak binder spans more tissues than strong binder",
        weak_xi,
        strong_xi,
    );

    println!("  → Strong binder: toxicity concentrated in liver (hepatotoxicity risk)");
    println!("  → Weak binder: toxicity spread across {weak_xi:.0} tissues (manageable load)");
}

/// Study 2: Repair capacity buffer.
///
/// Weak binders produce per-tissue burden below repair capacity.
/// Strong binders exceed repair capacity at the target tissue.
fn study_2_repair_capacity_buffer(h: &mut ValidationHarness) {
    println!("\n─── Study 2: Repair Capacity Buffer ───");

    let concentration = 1.0;
    let hill_n = 1.0;
    let repair = [0.05; 8];
    let sensitivities = [1.0, 1.0, 1.5, 0.5, 2.0, 0.8, 0.3, 1.0];

    let weak_ic50s = [30.0; 8];
    let weak_landscape = toxicology::compute_toxicity_landscape(
        concentration,
        &weak_ic50s,
        &sensitivities,
        &repair,
        hill_n,
        10.0,
        0.20,
    );

    println!(
        "  Weak distributed: excess burden = {:.6} (systemic = {:.4})",
        weak_landscape.excess_burden, weak_landscape.systemic_burden
    );

    let mut strong_ic50s = [500.0; 8];
    strong_ic50s[0] = 0.3;
    let strong_landscape = toxicology::compute_toxicity_landscape(
        concentration,
        &strong_ic50s,
        &sensitivities,
        &repair,
        hill_n,
        10.0,
        0.20,
    );

    println!(
        "  Strong localized: excess burden = {:.6} (systemic = {:.4})",
        strong_landscape.excess_burden, strong_landscape.systemic_burden
    );

    h.check_upper(
        "weak binding: low excess burden (within repair capacity)",
        weak_landscape.excess_burden,
        0.05,
    );
    h.check_lower(
        "strong binding: high excess burden (exceeds repair at liver)",
        strong_landscape.excess_burden,
        0.5,
    );

    let tissues_stressed = weak_landscape
        .tissue_excesses
        .iter()
        .filter(|&&e| e > tolerances::DIVISION_GUARD)
        .count();
    println!("  Weak binder: {tissues_stressed}/8 tissues stressed (excess > 0)");
}

/// Study 3: Clearance regime — linear vs saturated.
///
/// Weak binders at low tissue concentrations stay in first-order kinetics.
/// Strong binders at high local concentrations saturate clearance.
fn study_3_clearance_regime(h: &mut ValidationHarness) {
    println!("\n─── Study 3: Clearance Regime ───");

    let km = 10.0;

    let weak_concs: Vec<f64> = vec![0.03; 8];
    let strong_concs: Vec<f64> = {
        let mut v = vec![0.001; 8];
        v[0] = 25.0;
        v
    };

    let (weak_max_util, weak_linear) = toxicology::clearance_safety_margin(
        &weak_concs,
        km,
        tolerances::CLEARANCE_LINEAR_THRESHOLD,
    );
    let (strong_max_util, strong_linear) = toxicology::clearance_safety_margin(
        &strong_concs,
        km,
        tolerances::CLEARANCE_LINEAR_THRESHOLD,
    );

    println!(
        "  Weak binder: max utilization = {weak_max_util:.4} ({:.1}%), linear = {weak_linear}",
        weak_max_util * 100.0
    );
    println!(
        "  Strong binder: max utilization = {strong_max_util:.4} ({:.1}%), linear = {strong_linear}",
        strong_max_util * 100.0
    );

    h.check_bool("weak binder in linear clearance regime", weak_linear);
    h.check_bool("strong binder saturates clearance", !strong_linear);

    for &c in &weak_concs {
        let regime = toxicology::clearance_regime(c, km);
        h.check_upper("weak binder C/Km < 0.01 (deep linear)", regime, 0.01);
    }

    let strong_regime = toxicology::clearance_regime(strong_concs[0], km);
    h.check_lower("strong binder C/Km > 1 (saturated)", strong_regime, 1.0);

    println!("  → Weak binder: predictable first-order clearance at every tissue");
    println!("  → Strong binder: hepatic clearance saturated (nonlinear PK, accumulation risk)");
}

/// Study 4: Delocalization advantage — quantified.
///
/// How many times worse is localized toxicity vs the same total
/// binding distributed across tissues?
fn study_4_delocalization_advantage(h: &mut ValidationHarness) {
    println!("\n─── Study 4: Delocalization Advantage ───");

    let distributed: Vec<toxicology::TissueToxProfile> = (0..10)
        .map(|_| toxicology::TissueToxProfile {
            name: "tissue",
            occupancy: 0.04,
            sensitivity: 1.0,
            repair_capacity: 0.05,
        })
        .collect();

    let adv = toxicology::delocalization_advantage(&distributed);
    let sbs = toxicology::systemic_burden_score(&distributed);
    let (_, excess) = toxicology::tissue_excess_burden(&distributed);

    println!("  10 tissues at 4% occupancy each:");
    println!("    Systemic burden = {sbs:.3}");
    println!("    Excess burden   = {excess:.6}");
    println!("    Delocalization advantage = {adv:.1}x");

    h.check_lower(
        "delocalization advantage > 1 (distributed is safer)",
        adv,
        1.0,
    );

    let tight_distributed: Vec<toxicology::TissueToxProfile> = (0..10)
        .map(|_| toxicology::TissueToxProfile {
            name: "tissue",
            occupancy: 0.03,
            sensitivity: 1.0,
            repair_capacity: 0.05,
        })
        .collect();

    let tight_adv = toxicology::delocalization_advantage(&tight_distributed);
    let (_, tight_excess) = toxicology::tissue_excess_burden(&tight_distributed);
    println!("\n  10 tissues at 3% occupancy (within repair capacity):");
    println!("    Excess burden = {tight_excess:.6}");
    println!("    Delocalization advantage = {tight_adv}");

    h.check_bool(
        "at 3% occupancy: zero excess (fully within repair capacity)",
        tight_excess < tolerances::DIVISION_GUARD,
    );

    println!("  → Below repair threshold: distributed binding has ZERO toxic cost");
    println!("  → The body handles it entirely — localized equivalent still has excess");
}

/// Study 5: Disorder-modulated tissue landscape.
///
/// Anderson disorder on tissue sensitivities creates a realistic organ
/// vulnerability profile. The toxicity localization length of a weak
/// binder should remain high even with sensitivity disorder.
fn study_5_disorder_modulated_landscape(h: &mut ValidationHarness) {
    println!("\n─── Study 5: Disorder-Modulated Tissue Landscape ───");

    let n_tissues = 12;
    let sensitivities = toxicology::disorder_tissue_sensitivities(n_tissues, 1.0, 0.8, 42);

    let min_s = sensitivities.iter().copied().fold(f64::INFINITY, f64::min);
    let max_s = sensitivities
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    println!("  Tissue sensitivity range: [{min_s:.3}, {max_s:.3}] (disorder W=0.8)");

    let concentration = 1.0;
    let weak_ic50s: Vec<f64> = vec![40.0; n_tissues];
    let repair: Vec<f64> = vec![tolerances::TISSUE_REPAIR_CAPACITY; n_tissues];

    let landscape = toxicology::compute_toxicity_landscape(
        concentration,
        &weak_ic50s,
        &sensitivities,
        &repair,
        1.0,
        10.0,
        0.20,
    );

    println!(
        "  Weak binder on disordered landscape: ξ={:.1}, IPR={:.4}",
        landscape.localization_length, landscape.tox_ipr
    );
    println!(
        "    Systemic burden = {:.4}, excess = {:.6}",
        landscape.systemic_burden, landscape.excess_burden
    );
    println!(
        "    Max clearance utilization = {:.4} ({:.1}%)",
        landscape.max_clearance_utilization,
        landscape.max_clearance_utilization * 100.0
    );

    h.check_lower(
        "delocalized even with disordered sensitivities (ξ > 5)",
        landscape.localization_length,
        5.0,
    );
    h.check_bool(
        "clearance stays linear even with disorder",
        landscape.clearance_linear,
    );

    let very_weak_ic50s: Vec<f64> = vec![200.0; n_tissues];
    let hormetic_landscape = toxicology::compute_toxicity_landscape(
        concentration,
        &very_weak_ic50s,
        &sensitivities,
        &repair,
        1.0,
        10.0,
        0.20,
    );

    let hormetic = toxicology::in_hormetic_zone(
        hormetic_landscape.systemic_burden,
        1.0,
        tolerances::HORMETIC_LOW_DIVISOR,
        tolerances::HORMETIC_HIGH_DIVISOR,
    );
    println!(
        "  Very weak binder (IC50=200µM): burden = {:.4}, in hormetic zone = {hormetic}",
        hormetic_landscape.systemic_burden
    );
    h.check_bool(
        "very weak distributed burden falls in hormetic zone (adaptive benefit)",
        hormetic,
    );
    h.check_bool(
        "moderate weak binder (IC50=40µM) is above hormetic zone (still safe, not hormetic)",
        !toxicology::in_hormetic_zone(
            landscape.systemic_burden,
            1.0,
            tolerances::HORMETIC_LOW_DIVISOR,
            tolerances::HORMETIC_HIGH_DIVISOR,
        ),
    );

    println!("  → Even with disordered tissue sensitivities, weak binding remains delocalized");
    println!("  → Anderson disorder on organs doesn't break the delocalization advantage");
    println!(
        "  → Very weak distributed exposure enters the hormetic zone — potential adaptive benefit"
    );
}
