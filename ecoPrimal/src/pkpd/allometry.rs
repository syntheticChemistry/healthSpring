// SPDX-License-Identifier: AGPL-3.0-only
//! Allometric scaling, mAb PK, cross-species transfer.

use core::f64::consts::LN_2;

/// Lokivetmab canine reference constants.
pub mod lokivetmab_canine {
    pub const BW_KG: f64 = 15.0;
    pub const HALF_LIFE_DAYS: f64 = 14.0;
    pub const VD_L_KG: f64 = 0.07;
    pub const CL_ML_DAY_KG: f64 = 3.5;
}

/// Standard allometric exponents for mAb PK.
pub mod allometric_exp {
    pub const CLEARANCE: f64 = 0.75;
    pub const VOLUME: f64 = 1.0;
    pub const HALF_LIFE: f64 = 0.25;
}

// ═══════════════════════════════════════════════════════════════════════
// Allometric scaling (Exp004)
// ═══════════════════════════════════════════════════════════════════════

/// Allometric scaling: `P_human = P_animal * (BW_human / BW_animal)^b`.
#[must_use]
pub fn allometric_scale(param_animal: f64, bw_animal: f64, bw_human: f64, exponent: f64) -> f64 {
    param_animal * (bw_human / bw_animal).powf(exponent)
}

/// mAb PK curve with SC absorption (Bateman-like for subcutaneous mAb).
///
/// Models typical mAb SC absorption: `k_a = ln2 / 2 days` (Tmax ≈ 3-8 days).
#[must_use]
pub fn mab_pk_sc(dose_mg: f64, vd_l: f64, half_life_days: f64, t_days: f64) -> f64 {
    if half_life_days <= 0.0 || vd_l <= 0.0 {
        return 0.0;
    }
    let k_e = LN_2 / half_life_days;
    let k_a = LN_2 / 2.0;
    if (k_a - k_e).abs() < 1e-12 {
        return 0.0;
    }
    let coeff = (dose_mg / vd_l) * k_a / (k_a - k_e);
    coeff * ((-k_e * t_days).exp() - (-k_a * t_days).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkpd::find_cmax_tmax;

    const TOL: f64 = 1e-10;

    #[test]
    fn allometric_identity_at_b0() {
        let val = allometric_scale(100.0, 15.0, 70.0, 0.0);
        assert!((val - 100.0).abs() < TOL, "b=0 → identity");
    }

    #[test]
    fn allometric_hl_scaling() {
        let hl = allometric_scale(
            lokivetmab_canine::HALF_LIFE_DAYS,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::HALF_LIFE,
        );
        assert!(hl > lokivetmab_canine::HALF_LIFE_DAYS, "human t½ > canine");
        assert!((14.0..=28.0).contains(&hl), "scaled t½ in expected range");
    }

    #[test]
    fn allometric_vd_scaling() {
        let vd_animal = lokivetmab_canine::VD_L_KG * lokivetmab_canine::BW_KG;
        let vd_human = allometric_scale(
            vd_animal,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::VOLUME,
        );
        assert!(vd_human > vd_animal, "human Vd > canine");
    }

    #[test]
    fn allometric_cl_ratio() {
        let bw_dog = lokivetmab_canine::BW_KG;
        let bw_human = 70.0_f64;
        let cl_animal = lokivetmab_canine::CL_ML_DAY_KG * bw_dog;
        let cl_human = allometric_scale(cl_animal, bw_dog, bw_human, allometric_exp::CLEARANCE);
        let expected_ratio = (bw_human / bw_dog).powf(0.75);
        let actual_ratio = cl_human / cl_animal;
        assert!((actual_ratio - expected_ratio).abs() < 1e-6);
    }

    #[test]
    fn mab_pk_sc_shape() {
        let vd = allometric_scale(
            lokivetmab_canine::VD_L_KG * lokivetmab_canine::BW_KG,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::VOLUME,
        );
        let hl = allometric_scale(
            lokivetmab_canine::HALF_LIFE_DAYS,
            lokivetmab_canine::BW_KG,
            70.0,
            allometric_exp::HALF_LIFE,
        );
        let times: Vec<f64> = (0..2000).map(|i| 56.0 * f64::from(i) / 1999.0).collect();
        let concs: Vec<f64> = times.iter().map(|&t| mab_pk_sc(30.0, vd, hl, t)).collect();
        assert!(concs.iter().all(|&c| c >= -1e-12), "non-negative");
        let (cmax, tmax) = find_cmax_tmax(&times, &concs);
        assert!(cmax > 0.0, "Cmax > 0");
        assert!(tmax > 0.0, "Tmax > 0");
    }

    #[test]
    fn mab_pk_sc_zero_at_t0() {
        let c = mab_pk_sc(30.0, 5.0, 20.0, 0.0);
        assert!(c.abs() < TOL, "C(0) = 0 for SC dosing");
    }
}
