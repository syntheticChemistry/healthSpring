// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp104: Cross-species allometric PK bridge validation (CM-005)
//!
//! Validates allometric scaling across species: canine → human → feline → equine → murine.

use healthspring_barracuda::comparative::species_params::{
    Species, SpeciesPkParams, allometric_clearance, allometric_half_life, allometric_volume,
    scale_across_species,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    ALLOMETRIC_CL_RATIO, ALLOMETRIC_ROUNDTRIP, DETERMINISM, MACHINE_EPSILON,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp104_cross_species_pk");

    // Provenance: Allometric scaling
    log_analytical(&AnalyticalProvenance {
        formula: "CL = CL_ref × (BW/BW_ref)^0.75",
        reference: "Mahmood 2006, J Pharm Sci 95:1810",
        doi: Some("10.1002/jps.20590"),
    });

    let canine = Species::Canine.oclacitinib_pk();
    let human = Species::Human.oclacitinib_pk();
    let equine = Species::Equine.oclacitinib_pk();
    let murine = Species::Murine.oclacitinib_pk();

    // Check 1: allometric_clearance: same weight → same CL (identity)
    let cl_same = allometric_clearance(10.0, 70.0, 70.0);
    h.check_abs(
        "allometric_clearance: same weight → same CL",
        cl_same,
        10.0,
        MACHINE_EPSILON,
    );

    // Check 2: allometric_clearance: 10× weight → CL × 10^0.75 = 5.623×
    let cl_10x = allometric_clearance(1.0, 1.0, 10.0);
    let expected_10_075 = 10.0_f64.powf(0.75);
    h.check_abs(
        "allometric_clearance: 10× weight → CL × 10^0.75",
        cl_10x,
        expected_10_075,
        ALLOMETRIC_CL_RATIO,
    );

    // Check 3: allometric_volume: same weight → same Vd (identity)
    let vd_same = allometric_volume(10.0, 70.0, 70.0);
    h.check_abs(
        "allometric_volume: same weight → same Vd",
        vd_same,
        10.0,
        MACHINE_EPSILON,
    );

    // Check 4: allometric_volume: 2× weight → 2× Vd (linear)
    let vd_2x = allometric_volume(10.0, 70.0, 140.0);
    h.check_abs(
        "allometric_volume: 2× weight → 2× Vd",
        vd_2x,
        20.0,
        MACHINE_EPSILON,
    );

    // Check 5: allometric_half_life: t½ = ln(2) × Vd / CL
    let vd = 10.0;
    let cl = 2.0;
    let t_half = allometric_half_life(vd, cl);
    let expected_t_half = core::f64::consts::LN_2 * vd / cl;
    h.check_abs(
        "allometric_half_life: t½ = ln(2) × Vd / CL",
        t_half,
        expected_t_half,
        MACHINE_EPSILON,
    );

    // Check 6: allometric_half_life: zero CL → infinity
    let t_half_inf = allometric_half_life(10.0, 0.0);
    h.check_bool(
        "allometric_half_life: zero CL → infinity",
        t_half_inf.is_infinite(),
    );

    // Check 7: scale_across_species: canine to canine (roundtrip identity)
    let scaled_canine = scale_across_species(&canine, Species::Canine, canine.body_weight_kg);
    h.check_abs(
        "scale_across_species: canine to canine (roundtrip identity)",
        scaled_canine.clearance_l_hr_kg,
        canine.clearance_l_hr_kg,
        ALLOMETRIC_ROUNDTRIP,
    );

    // Check 8: scale_across_species: canine → human CL is reduced per kg (larger animals clear less per kg)
    let scaled_human = scale_across_species(&canine, Species::Human, human.body_weight_kg);
    h.check_bool(
        "scale_across_species: canine → human CL per kg reduced",
        scaled_human.clearance_l_hr_kg < canine.clearance_l_hr_kg,
    );

    // Check 9: scale_across_species: murine → human CL per kg is much lower (mice clear faster per kg)
    let scaled_from_murine = scale_across_species(&murine, Species::Human, human.body_weight_kg);
    h.check_bool(
        "scale_across_species: murine → human CL per kg lower than murine",
        scaled_from_murine.clearance_l_hr_kg < murine.clearance_l_hr_kg,
    );

    // Check 10: scale_across_species: all species have positive CL
    let all_species: Vec<SpeciesPkParams> = [
        Species::Canine,
        Species::Human,
        Species::Feline,
        Species::Equine,
        Species::Murine,
    ]
    .iter()
    .map(|s| s.oclacitinib_pk())
    .collect();
    h.check_bool(
        "scale_across_species: all species have positive CL",
        all_species.iter().all(|p| p.clearance_l_hr_kg > 0.0),
    );

    // Check 11: scale_across_species: all species have positive Vd
    h.check_bool(
        "scale_across_species: all species have positive Vd",
        all_species.iter().all(|p| p.volume_distribution_l_kg > 0.0),
    );

    // Check 12: Canine PK params: BW=15 kg
    h.check_abs(
        "Canine PK params: BW=15 kg",
        canine.body_weight_kg,
        15.0,
        MACHINE_EPSILON,
    );

    // Check 13: Human PK params: BW=70 kg
    h.check_abs(
        "Human PK params: BW=70 kg",
        human.body_weight_kg,
        70.0,
        MACHINE_EPSILON,
    );

    // Check 14: Half-life: larger species have longer half-lives (canine < equine)
    let t_half_canine = allometric_half_life(
        canine.volume_distribution_l_kg * canine.body_weight_kg,
        canine.clearance_l_hr_kg * canine.body_weight_kg,
    );
    let t_half_equine = allometric_half_life(
        equine.volume_distribution_l_kg * equine.body_weight_kg,
        equine.clearance_l_hr_kg * equine.body_weight_kg,
    );
    h.check_bool(
        "Half-life: canine < equine (larger species longer t½)",
        t_half_canine < t_half_equine,
    );

    // Check 15: Determinism: same scaling → identical results
    let r1 = allometric_clearance(10.0, 70.0, 140.0);
    let r2 = allometric_clearance(10.0, 70.0, 140.0);
    h.check_abs(
        "Determinism: same scaling → identical results",
        r1,
        r2,
        DETERMINISM,
    );

    h.exit();
}
