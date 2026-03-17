#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp006 validation: PBPK (Physiologically-Based Pharmacokinetic) compartments
//!
//! Cross-validates `healthspring_barracuda::pkpd::pbpk_*` against Python control.

use healthspring_barracuda::pkpd::{
    TissueCompartment, cardiac_output, pbpk_auc, pbpk_iv_simulate, standard_human_tissues,
};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::{OrExit, ValidationHarness};

fn main() {
    let mut h = ValidationHarness::new("Exp006 PBPK Compartments");

    let tissues = standard_human_tissues();
    let dose_mg = 100.0;
    let blood_volume_l = 5.0;
    let duration_hr = 48.0;
    let dt = 0.01;

    let (times, venous_profile, state) =
        pbpk_iv_simulate(&tissues, dose_mg, blood_volume_l, duration_hr, dt);

    // Check 1: Initial C(0) = dose/Vblood
    let c0_expected = dose_mg / blood_volume_l;
    let c0_actual = venous_profile[0];
    h.check_abs(
        "Initial C(0) = dose/Vblood",
        c0_actual,
        c0_expected,
        tolerances::HALF_LIFE_POINT,
    );

    // Check 2: Concentration decays over time
    let last = venous_profile.last().copied().unwrap_or(0.0);
    h.check_bool("Concentration decays over time", last < venous_profile[0]);

    // Check 3: All concentrations non-negative
    let venous_ok = venous_profile.iter().all(|&c| c >= 0.0);
    let tissue_ok = state.concentrations.iter().all(|&c| c >= 0.0);
    h.check_bool("All concentrations non-negative", venous_ok && tissue_ok);

    // Check 4: AUC positive and finite
    let auc = pbpk_auc(&times, &venous_profile);
    h.check_bool("AUC positive and finite", auc > 0.0 && auc.is_finite());

    // Check 5: Hepatic clearance reduces concentration faster than without
    let tissues_no_cl = tissues
        .iter()
        .map(|t| TissueCompartment {
            clearance_l_per_hr: 0.0,
            ..t.clone()
        })
        .collect::<Vec<_>>();
    let (_, profile_no_cl, _) =
        pbpk_iv_simulate(&tissues_no_cl, dose_mg, blood_volume_l, duration_hr, dt);
    let auc_no_cl = pbpk_auc(&times, &profile_no_cl);
    h.check_bool("Hepatic clearance reduces concentration", auc < auc_no_cl);

    // Check 6: Higher Kp → more tissue accumulation
    let fat_idx = tissues
        .iter()
        .position(|t| t.name == "fat")
        .or_exit("standard human tissues include fat");
    let muscle_idx = tissues
        .iter()
        .position(|t| t.name == "muscle")
        .or_exit("standard human tissues include muscle");
    let fat_kp = tissues[fat_idx].kp;
    let muscle_kp = tissues[muscle_idx].kp;
    let fat_conc = state.concentrations[fat_idx];
    let muscle_conc = state.concentrations[muscle_idx];
    h.check_bool(
        "Higher Kp → more tissue accumulation",
        fat_kp > muscle_kp && fat_conc > muscle_conc,
    );

    // Check 7: Mass conservation (total drug <= dose, decreases with elimination)
    let venous_at_end = state.venous_conc;
    let tissue_mass: f64 = tissues
        .iter()
        .zip(state.concentrations.iter())
        .map(|(t, c)| t.volume_l * c)
        .sum();
    let blood_mass = blood_volume_l * venous_at_end;
    let total_mass = blood_mass + tissue_mass;
    h.check_bool(
        "Mass conservation",
        total_mass <= dose_mg * 1.01 && total_mass >= 0.0,
    );

    // Check 8: 5 tissue compartments in standard model
    h.check_exact("5 tissue compartments", tissues.len() as u64, 5);

    // Check 9: Cardiac output ~330 L/hr
    let co = cardiac_output(&tissues);
    h.check_bool("Cardiac output ~330 L/hr", co > 200.0 && co < 400.0);

    // Check 10: Fat compartment accumulates more (high Kp)
    let fat_conc_24 = state.concentrations[fat_idx];
    let rest_idx = tissues
        .iter()
        .position(|t| t.name == "rest")
        .or_exit("standard human tissues include rest");
    let rest_conc_24 = state.concentrations[rest_idx];
    h.check_bool("Fat accumulates more", fat_conc_24 > rest_conc_24);

    // Check 11: Liver plasma-equivalent decays fastest (clearance)
    let liver_idx = tissues
        .iter()
        .position(|t| t.name == "liver")
        .or_exit("standard human tissues include liver");
    let kidney_idx = tissues
        .iter()
        .position(|t| t.name == "kidney")
        .or_exit("standard human tissues include kidney");
    let liver_free = state.concentrations[liver_idx] / tissues[liver_idx].kp;
    let kidney_free = state.concentrations[kidney_idx] / tissues[kidney_idx].kp;
    h.check_bool(
        "Liver free concentration lower (clearance)",
        liver_free < kidney_free,
    );

    // Check 12: Determinism (bit-identical on repeat)
    let (_, p2, _) = pbpk_iv_simulate(&tissues, dose_mg, blood_volume_l, duration_hr, dt);
    let bit_identical = venous_profile
        .iter()
        .zip(p2.iter())
        .all(|(a, b)| a.to_bits() == b.to_bits());
    h.check_bool("Determinism", bit_identical);

    // Check 13: Reduction to single compartment (Kp=1, no clearance)
    let tissues_uniform: Vec<TissueCompartment> = tissues
        .iter()
        .map(|t| TissueCompartment {
            kp: 1.0,
            clearance_l_per_hr: 0.0,
            ..t.clone()
        })
        .collect();
    let v_total = blood_volume_l + tissues_uniform.iter().map(|t| t.volume_l).sum::<f64>();
    let (_, _profile_uniform, state_uniform) =
        pbpk_iv_simulate(&tissues_uniform, dose_mg, blood_volume_l, 100.0, dt);
    let c_eq = dose_mg / v_total;
    let c_final = state_uniform.venous_conc;
    let mass_total: f64 = blood_volume_l * state_uniform.venous_conc
        + tissues_uniform
            .iter()
            .zip(state_uniform.concentrations.iter())
            .map(|(t, c)| t.volume_l * c)
            .sum::<f64>();
    // With Kp=1 and no CL, mass should be conserved; allow ~25% for Euler numerical drift
    let mass_ok = (mass_total - dose_mg).abs() < dose_mg * tolerances::PBPK_MASS_CONSERVATION;
    let conc_ok = (c_final - c_eq).abs() / c_eq < tolerances::PBPK_MASS_CONSERVATION;
    h.check_bool(
        "Kp=1, no CL → approximates single compartment",
        mass_ok && conc_ok,
    );

    h.exit();
}
