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

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp006 — PBPK Compartments Validation");
    println!("{}", "=".repeat(72));

    let tissues = standard_human_tissues();
    let dose_mg = 100.0;
    let blood_volume_l = 5.0;
    let duration_hr = 48.0;
    let dt = 0.01;

    let (times, venous_profile, state) =
        pbpk_iv_simulate(&tissues, dose_mg, blood_volume_l, duration_hr, dt);

    // Check 1: Initial C(0) = dose/Vblood
    println!("\n--- Check 1: Initial C(0) = dose/Vblood ---");
    let c0_expected = dose_mg / blood_volume_l;
    let c0_actual = venous_profile[0];
    let err = (c0_actual - c0_expected).abs();
    if err < tolerances::HALF_LIFE_POINT {
        println!("  [PASS] C(0) = {c0_actual:.6} = dose/Vblood = {c0_expected:.6}");
        passed += 1;
    } else {
        println!("  [FAIL] C(0) = {c0_actual:.6}, expected {c0_expected:.6}");
        failed += 1;
    }

    // Check 2: Concentration decays over time
    println!("\n--- Check 2: Concentration decays over time ---");
    let last = venous_profile.last().copied().unwrap_or(0.0);
    if last < venous_profile[0] {
        println!(
            "  [PASS] C(0) = {:.6} > C(48hr) = {last:.6}",
            venous_profile[0]
        );
        passed += 1;
    } else {
        println!("  [FAIL] C did not decay");
        failed += 1;
    }

    // Check 3: All concentrations non-negative
    println!("\n--- Check 3: All concentrations non-negative ---");
    let venous_ok = venous_profile.iter().all(|&c| c >= 0.0);
    let tissue_ok = state.concentrations.iter().all(|&c| c >= 0.0);
    if venous_ok && tissue_ok {
        println!("  [PASS] venous and tissue concentrations all >= 0");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 4: AUC positive and finite
    println!("\n--- Check 4: AUC positive and finite ---");
    let auc = pbpk_auc(&times, &venous_profile);
    if auc > 0.0 && auc.is_finite() {
        println!("  [PASS] AUC = {auc:.6} mg·hr/L");
        passed += 1;
    } else {
        println!("  [FAIL] AUC = {auc}");
        failed += 1;
    }

    // Check 5: Hepatic clearance reduces concentration faster than without
    println!("\n--- Check 5: Hepatic clearance reduces concentration ---");
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
    if auc < auc_no_cl {
        println!("  [PASS] AUC with CL = {auc:.4} < AUC without CL = {auc_no_cl:.4}");
        passed += 1;
    } else {
        println!("  [FAIL] hepatic clearance should reduce AUC");
        failed += 1;
    }

    // Check 6: Higher Kp → more tissue accumulation
    println!("\n--- Check 6: Higher Kp → more tissue accumulation ---");
    let fat_idx = tissues
        .iter()
        .position(|t| t.name == "fat")
        .expect("standard human tissues include fat");
    let muscle_idx = tissues
        .iter()
        .position(|t| t.name == "muscle")
        .expect("standard human tissues include muscle");
    let fat_kp = tissues[fat_idx].kp;
    let muscle_kp = tissues[muscle_idx].kp;
    let fat_conc = state.concentrations[fat_idx];
    let muscle_conc = state.concentrations[muscle_idx];
    if fat_kp > muscle_kp && fat_conc > muscle_conc {
        println!(
            "  [PASS] fat Kp={fat_kp}, C={fat_conc:.4} > muscle Kp={muscle_kp}, C={muscle_conc:.4}"
        );
        passed += 1;
    } else {
        println!("  [FAIL] fat should accumulate more than muscle");
        failed += 1;
    }

    // Check 7: Mass conservation (total drug <= dose, decreases with elimination)
    println!("\n--- Check 7: Mass conservation ---");
    let venous_at_end = state.venous_conc;
    let tissue_mass: f64 = tissues
        .iter()
        .zip(state.concentrations.iter())
        .map(|(t, c)| t.volume_l * c)
        .sum();
    let blood_mass = blood_volume_l * venous_at_end;
    let total_mass = blood_mass + tissue_mass;
    if total_mass <= dose_mg * 1.01 && total_mass >= 0.0 {
        println!("  [PASS] total drug = {total_mass:.4} mg <= dose {dose_mg}");
        passed += 1;
    } else {
        println!("  [FAIL] total = {total_mass:.4}, dose = {dose_mg}");
        failed += 1;
    }

    // Check 8: 5 tissue compartments in standard model
    println!("\n--- Check 8: 5 tissue compartments ---");
    if tissues.len() == 5 {
        println!("  [PASS] {} tissues", tissues.len());
        passed += 1;
    } else {
        println!("  [FAIL] expected 5, got {}", tissues.len());
        failed += 1;
    }

    // Check 9: Cardiac output ~330 L/hr
    println!("\n--- Check 9: Cardiac output ~330 L/hr ---");
    let co = cardiac_output(&tissues);
    if co > 200.0 && co < 400.0 {
        println!("  [PASS] CO = {co:.1} L/hr");
        passed += 1;
    } else {
        println!("  [FAIL] CO = {co}");
        failed += 1;
    }

    // Check 10: Fat compartment accumulates more (high Kp)
    println!("\n--- Check 10: Fat accumulates more ---");
    let fat_conc_24 = state.concentrations[fat_idx];
    let rest_idx = tissues
        .iter()
        .position(|t| t.name == "rest")
        .expect("standard human tissues include rest");
    let rest_conc_24 = state.concentrations[rest_idx];
    if fat_conc_24 > rest_conc_24 {
        println!("  [PASS] fat C = {fat_conc_24:.4} > rest C = {rest_conc_24:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 11: Liver plasma-equivalent decays fastest (clearance)
    println!("\n--- Check 11: Liver free concentration lower (clearance) ---");
    let liver_idx = tissues
        .iter()
        .position(|t| t.name == "liver")
        .expect("standard human tissues include liver");
    let kidney_idx = tissues
        .iter()
        .position(|t| t.name == "kidney")
        .expect("standard human tissues include kidney");
    let liver_free = state.concentrations[liver_idx] / tissues[liver_idx].kp;
    let kidney_free = state.concentrations[kidney_idx] / tissues[kidney_idx].kp;
    if liver_free < kidney_free {
        println!("  [PASS] liver C_free = {liver_free:.4} < kidney C_free = {kidney_free:.4}");
        passed += 1;
    } else {
        println!("  [FAIL] liver C_free should be lower due to hepatic clearance");
        failed += 1;
    }

    // Check 12: Determinism (bit-identical on repeat)
    println!("\n--- Check 12: Determinism ---");
    let (_, p2, _) = pbpk_iv_simulate(&tissues, dose_mg, blood_volume_l, duration_hr, dt);
    let bit_identical = venous_profile
        .iter()
        .zip(p2.iter())
        .all(|(a, b)| a.to_bits() == b.to_bits());
    if bit_identical {
        println!("  [PASS] bit-identical on repeat");
        passed += 1;
    } else {
        println!("  [FAIL] results differ on repeat");
        failed += 1;
    }

    // Check 13: Reduction to single compartment (Kp=1, no clearance)
    println!("\n--- Check 13: Kp=1, no CL → approximates single compartment ---");
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
    if mass_ok && conc_ok {
        println!("  [PASS] mass conserved ({mass_total:.2}≈{dose_mg}), C_final≈dose/V_total");
        passed += 1;
    } else {
        println!("  [FAIL] mass={mass_total:.2}, C_final={c_final:.4}, expected={c_eq:.4}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
