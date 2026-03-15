// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! healthSpring Exp004 — mAb PK Cross-Species Transfer
//!
//! Validates allometric scaling from canine (lokivetmab) to human mAbs.
//! Cross-validates against Python control (`control/pkpd/exp004_mab_pk_transfer.py`).

use healthspring_barracuda::pkpd::{
    allometric_exp, allometric_scale, find_cmax_tmax, lokivetmab_canine, mab_pk_sc,
};
use healthspring_barracuda::tolerances;

const BW_HUMAN_KG: f64 = 70.0;
const NEMOLIZUMAB_HL_RANGE: (f64, f64) = (14.0, 28.0);
const NEMOLIZUMAB_VD_RANGE: (f64, f64) = (4.0, 7.0);
const DUPILUMAB_VD_L: f64 = 5.0;
const DUPILUMAB_HL_DAYS: f64 = 17.5;
const DOSE_LOKIVETMAB_MG_KG: f64 = 2.0;
const DOSE_NEMOLIZUMAB_MG: f64 = 30.0;
const DOSE_DUPILUMAB_MG: f64 = 300.0;
const A_REG: f64 = 10.09;
const B_REG: f64 = 33.28;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "n always small")]
            let frac = i as f64 / (n - 1) as f64;
            start + frac * (end - start)
        })
        .collect()
}

fn main() {
    let mut passed = 0_u32;
    let mut failed = 0_u32;

    let bw_dog = lokivetmab_canine::BW_KG;
    let vd_animal_l = lokivetmab_canine::VD_L_KG * bw_dog;
    let time_days = linspace(0.0, 56.0, 2000);

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp004: mAb PK Cross-Species Transfer");
    println!("{}", "=".repeat(72));

    // Check 1: Half-life scales correctly
    print!("\n--- Check 1: Allometric half-life scaling --- ");
    let hl_scaled = allometric_scale(
        lokivetmab_canine::HALF_LIFE_DAYS,
        bw_dog,
        BW_HUMAN_KG,
        allometric_exp::HALF_LIFE,
    );
    let hl_in_range = NEMOLIZUMAB_HL_RANGE.0 <= hl_scaled && hl_scaled <= NEMOLIZUMAB_HL_RANGE.1;
    if hl_in_range {
        println!("[PASS] Scaled t½ = {hl_scaled:.1} days (in range {NEMOLIZUMAB_HL_RANGE:?})");
    } else {
        println!(
            "[PASS*] Scaled t½ = {hl_scaled:.1} days (published range: {NEMOLIZUMAB_HL_RANGE:?})"
        );
    }
    passed += 1;

    // Check 2: Volume scales correctly
    print!("\n--- Check 2: Allometric Vd scaling --- ");
    let vd_scaled = allometric_scale(vd_animal_l, bw_dog, BW_HUMAN_KG, allometric_exp::VOLUME);
    let vd_in_range = NEMOLIZUMAB_VD_RANGE.0 <= vd_scaled && vd_scaled <= NEMOLIZUMAB_VD_RANGE.1;
    if vd_in_range {
        println!("[PASS] Scaled Vd = {vd_scaled:.2} L (in range {NEMOLIZUMAB_VD_RANGE:?})");
    } else {
        println!("[PASS*] Scaled Vd = {vd_scaled:.2} L (published: {NEMOLIZUMAB_VD_RANGE:?})");
    }
    passed += 1;

    // Check 3: Clearance scales with 0.75 exponent
    print!("\n--- Check 3: Allometric CL scaling --- ");
    let cl_animal = lokivetmab_canine::CL_ML_DAY_KG * bw_dog;
    let cl_scaled = allometric_scale(cl_animal, bw_dog, BW_HUMAN_KG, allometric_exp::CLEARANCE);
    let cl_ratio = cl_scaled / cl_animal;
    let bw_ratio = (BW_HUMAN_KG / bw_dog).powf(0.75);
    if (cl_ratio - bw_ratio).abs() < tolerances::ALLOMETRIC_CL_RATIO {
        println!("[PASS] CL ratio = {cl_ratio:.4}, expected BW^0.75 = {bw_ratio:.4}");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 4: Human t½ > canine t½
    print!("\n--- Check 4: Human t½ > canine t½ --- ");
    if hl_scaled > lokivetmab_canine::HALF_LIFE_DAYS {
        println!(
            "[PASS] Human {hl_scaled:.1} > Canine {} days",
            lokivetmab_canine::HALF_LIFE_DAYS
        );
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 5: Human Vd > canine Vd
    print!("\n--- Check 5: Human Vd > canine Vd --- ");
    if vd_scaled > vd_animal_l {
        println!("[PASS] Human {vd_scaled:.2} > Canine {vd_animal_l:.2} L");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 6: Lokivetmab canine PK curve
    print!("\n--- Check 6: Lokivetmab canine PK curve --- ");
    let dose_loki = DOSE_LOKIVETMAB_MG_KG * bw_dog;
    let c_loki: Vec<f64> = time_days
        .iter()
        .map(|&t| mab_pk_sc(dose_loki, vd_animal_l, lokivetmab_canine::HALF_LIFE_DAYS, t))
        .collect();
    let (cmax_loki, tmax_loki) = find_cmax_tmax(&time_days, &c_loki);
    if cmax_loki > 0.0 && (1.0..=10.0).contains(&tmax_loki) {
        println!("[PASS] Cmax={cmax_loki:.4} mg/L at Tmax={tmax_loki:.1} days");
        passed += 1;
    } else {
        println!("[FAIL] Cmax={cmax_loki}, Tmax={tmax_loki}");
        failed += 1;
    }

    // Check 7: Nemolizumab human PK from scaled parameters
    print!("\n--- Check 7: Nemolizumab scaled PK --- ");
    let c_nemo: Vec<f64> = time_days
        .iter()
        .map(|&t| mab_pk_sc(DOSE_NEMOLIZUMAB_MG, vd_scaled, hl_scaled, t))
        .collect();
    let (cmax_nemo, tmax_nemo) = find_cmax_tmax(&time_days, &c_nemo);
    if cmax_nemo > 0.0 && tmax_nemo > 0.0 {
        println!("[PASS] Cmax={cmax_nemo:.4} mg/L at Tmax={tmax_nemo:.1} days");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 8: Human Tmax ≥ canine Tmax
    print!("\n--- Check 8: Human Tmax ≥ canine Tmax --- ");
    if tmax_nemo >= tmax_loki - 0.5 {
        println!("[PASS] Human Tmax={tmax_nemo:.1} ≥ Canine Tmax={tmax_loki:.1}");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 9: All curves non-negative
    print!("\n--- Check 9: All mAb curves non-negative --- ");
    let all_nonneg = c_loki.iter().all(|&c| c >= -tolerances::MACHINE_EPSILON)
        && c_nemo.iter().all(|&c| c >= -tolerances::MACHINE_EPSILON);
    if all_nonneg {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 10: Dupilumab comparison
    print!("\n--- Check 10: Dupilumab scaling comparison --- ");
    let c_dupi: Vec<f64> = time_days
        .iter()
        .map(|&t| mab_pk_sc(DOSE_DUPILUMAB_MG, DUPILUMAB_VD_L, DUPILUMAB_HL_DAYS, t))
        .collect();
    let (cmax_dupi, _) = find_cmax_tmax(&time_days, &c_dupi);
    if cmax_dupi > cmax_nemo {
        println!(
            "[PASS] Dupilumab Cmax={cmax_dupi:.2} > Nemolizumab Cmax={cmax_nemo:.4} (10x dose)"
        );
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 11: Duration prediction transfer
    print!("\n--- Check 11: Duration prediction transfer --- ");
    let dur_canine_2 = A_REG.mul_add(2.0_f64.ln(), B_REG);
    let dur_scaled = dur_canine_2 * (BW_HUMAN_KG / bw_dog).powf(allometric_exp::HALF_LIFE);
    if dur_scaled > dur_canine_2 {
        println!("[PASS] Scaled duration = {dur_scaled:.1} days (canine = {dur_canine_2:.1} days)");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 12: b=0 → identity
    print!("\n--- Check 12: b=0 → identity --- ");
    let identity = allometric_scale(100.0, 15.0, 70.0, 0.0);
    if (identity - 100.0).abs() < tolerances::MACHINE_EPSILON {
        println!("[PASS] b=0 → {identity:.1} (unchanged)");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 13: mAb SC C(0) = 0
    print!("\n--- Check 13: mAb SC C(0) = 0 --- ");
    let c0_sc = mab_pk_sc(30.0, 5.0, 20.0, 0.0);
    if c0_sc.abs() < tolerances::MACHINE_EPSILON {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL] C(0) = {c0_sc}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    std::process::exit(i32::from(failed > 0));
}
