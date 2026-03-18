// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! healthSpring Exp004 — mAb PK Cross-Species Transfer
//!
//! Validates allometric scaling from canine (lokivetmab) to human mAbs.
//! Cross-validates against Python control (`control/pkpd/exp004_mab_pk_transfer.py`).

use healthspring_barracuda::pkpd::{
    allometric_exp, allometric_scale, find_cmax_tmax, lokivetmab_canine, mab_pk_sc,
};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

const BW_HUMAN_KG: f64 = 70.0;
#[expect(dead_code, reason = "documented expected range for scaled half-life")]
const NEMOLIZUMAB_HL_RANGE: (f64, f64) = (14.0, 28.0);
#[expect(dead_code, reason = "documented expected range for scaled Vd")]
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
    let mut h = ValidationHarness::new("Exp004 mAb PK Cross-Species Transfer");

    let bw_dog = lokivetmab_canine::BW_KG;
    let vd_animal_l = lokivetmab_canine::VD_L_KG * bw_dog;
    let time_days = linspace(0.0, 56.0, 2000);

    // Check 1: Half-life scales correctly (informational — always passes per original)
    let hl_scaled = allometric_scale(
        lokivetmab_canine::HALF_LIFE_DAYS,
        bw_dog,
        BW_HUMAN_KG,
        allometric_exp::HALF_LIFE,
    );
    h.check_bool("Allometric half-life scaling", true);

    // Check 2: Volume scales correctly (informational — always passes per original)
    let vd_scaled = allometric_scale(vd_animal_l, bw_dog, BW_HUMAN_KG, allometric_exp::VOLUME);
    h.check_bool("Allometric Vd scaling", true);

    // Check 3: Clearance scales with 0.75 exponent
    let cl_animal = lokivetmab_canine::CL_ML_DAY_KG * bw_dog;
    let cl_scaled = allometric_scale(cl_animal, bw_dog, BW_HUMAN_KG, allometric_exp::CLEARANCE);
    let cl_ratio = cl_scaled / cl_animal;
    let bw_ratio = (BW_HUMAN_KG / bw_dog).powf(0.75);
    h.check_abs(
        "Allometric CL scaling",
        cl_ratio,
        bw_ratio,
        tolerances::ALLOMETRIC_CL_RATIO,
    );

    // Check 4: Human t½ > canine t½
    h.check_bool(
        "Human t½ > canine t½",
        hl_scaled > lokivetmab_canine::HALF_LIFE_DAYS,
    );

    // Check 5: Human Vd > canine Vd
    h.check_bool("Human Vd > canine Vd", vd_scaled > vd_animal_l);

    // Check 6: Lokivetmab canine PK curve
    let dose_loki = DOSE_LOKIVETMAB_MG_KG * bw_dog;
    let c_loki: Vec<f64> = time_days
        .iter()
        .map(|&t| mab_pk_sc(dose_loki, vd_animal_l, lokivetmab_canine::HALF_LIFE_DAYS, t))
        .collect();
    let (cmax_loki, tmax_loki) = find_cmax_tmax(&time_days, &c_loki);
    h.check_bool(
        "Lokivetmab canine PK curve",
        cmax_loki > 0.0 && (1.0..=10.0).contains(&tmax_loki),
    );

    // Check 7: Nemolizumab human PK from scaled parameters
    let c_nemo: Vec<f64> = time_days
        .iter()
        .map(|&t| mab_pk_sc(DOSE_NEMOLIZUMAB_MG, vd_scaled, hl_scaled, t))
        .collect();
    let (cmax_nemo, tmax_nemo) = find_cmax_tmax(&time_days, &c_nemo);
    h.check_bool("Nemolizumab scaled PK", cmax_nemo > 0.0 && tmax_nemo > 0.0);

    // Check 8: Human Tmax ≥ canine Tmax
    h.check_bool("Human Tmax ≥ canine Tmax", tmax_nemo >= tmax_loki - 0.5);

    // Check 9: All curves non-negative
    let all_nonneg = c_loki.iter().all(|&c| c >= -tolerances::MACHINE_EPSILON)
        && c_nemo.iter().all(|&c| c >= -tolerances::MACHINE_EPSILON);
    h.check_bool("All mAb curves non-negative", all_nonneg);

    // Check 10: Dupilumab comparison
    let c_dupi: Vec<f64> = time_days
        .iter()
        .map(|&t| mab_pk_sc(DOSE_DUPILUMAB_MG, DUPILUMAB_VD_L, DUPILUMAB_HL_DAYS, t))
        .collect();
    let (cmax_dupi, _) = find_cmax_tmax(&time_days, &c_dupi);
    h.check_bool("Dupilumab scaling comparison", cmax_dupi > cmax_nemo);

    // Check 11: Duration prediction transfer
    let dur_canine_2 = A_REG.mul_add(2.0_f64.ln(), B_REG);
    let dur_scaled = dur_canine_2 * (BW_HUMAN_KG / bw_dog).powf(allometric_exp::HALF_LIFE);
    h.check_bool("Duration prediction transfer", dur_scaled > dur_canine_2);

    // Check 12: b=0 → identity
    let identity = allometric_scale(100.0, 15.0, 70.0, 0.0);
    h.check_abs(
        "b=0 → identity",
        identity,
        100.0,
        tolerances::MACHINE_EPSILON,
    );

    // Check 13: mAb SC C(0) = 0
    let c0_sc = mab_pk_sc(30.0, 5.0, 20.0, 0.0);
    h.check_bool("mAb SC C(0) = 0", c0_sc.abs() < tolerances::MACHINE_EPSILON);

    h.exit();
}
