// SPDX-License-Identifier: AGPL-3.0-or-later
//! Species PK parameter registry and allometric scaling.
//!
//! The same compartmental ODE governs drug disposition in all mammals.
//! What changes between species is the parameters: clearance, volume of
//! distribution, bioavailability. Allometric scaling bridges species:
//!
//! - `CL_target = CL_ref × (BW_target / BW_ref)^0.75`
//! - `Vd_target = Vd_ref × (BW_target / BW_ref)^1.0`
//! - `t½ = 0.693 × Vd / CL`
//!
//! References:
//! - Mahmood I (2006) *J Pharm Sci* 95:1810 — allometric exponents
//! - Boxenbaum H (1982) *J Pharmacokinet Biopharm* 10:201 — interspecies scaling

use serde::{Deserialize, Serialize};

/// Supported species for cross-species PK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    /// Human (default clinical scaling).
    Human,
    /// Dog.
    Canine,
    /// Cat.
    Feline,
    /// Horse.
    Equine,
    /// Mouse / rat surrogate.
    Murine,
}

/// PK parameters for a species.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesPkParams {
    /// Species these parameters describe.
    pub species: Species,
    /// Typical body weight in kg.
    pub body_weight_kg: f64,
    /// Clearance in L/hr/kg.
    pub clearance_l_hr_kg: f64,
    /// Volume of distribution in L/kg.
    pub volume_distribution_l_kg: f64,
    /// Oral bioavailability (fraction, 0–1).
    pub bioavailability: f64,
}

impl Species {
    /// Reference PK parameters for oclacitinib (JAK1 inhibitor).
    ///
    /// Sources:
    /// - Canine: Gonzales 2014, JVPT 37:317 (FDA CVM approval data)
    /// - Human: Tofacitinib FDA label (cross-validated, same target)
    /// - Feline: Estimated from allometric scaling (no published feline JAK data)
    #[must_use]
    pub const fn oclacitinib_pk(self) -> SpeciesPkParams {
        match self {
            Self::Canine => SpeciesPkParams {
                species: self,
                body_weight_kg: 15.0,
                clearance_l_hr_kg: 0.82,
                volume_distribution_l_kg: 2.9,
                bioavailability: 0.89,
            },
            Self::Human => SpeciesPkParams {
                species: self,
                body_weight_kg: 70.0,
                clearance_l_hr_kg: 0.38,
                volume_distribution_l_kg: 1.24,
                bioavailability: 0.74,
            },
            Self::Feline => SpeciesPkParams {
                species: self,
                body_weight_kg: 4.5,
                clearance_l_hr_kg: 1.1,
                volume_distribution_l_kg: 3.2,
                bioavailability: 0.70,
            },
            Self::Equine => SpeciesPkParams {
                species: self,
                body_weight_kg: 500.0,
                clearance_l_hr_kg: 0.25,
                volume_distribution_l_kg: 0.9,
                bioavailability: 0.60,
            },
            Self::Murine => SpeciesPkParams {
                species: self,
                body_weight_kg: 0.025,
                clearance_l_hr_kg: 4.0,
                volume_distribution_l_kg: 5.0,
                bioavailability: 0.50,
            },
        }
    }
}

/// Allometric clearance scaling.
///
/// `CL_target = CL_ref × (BW_target / BW_ref)^0.75`
///
/// The 0.75 exponent is the standard allometric exponent for clearance
/// (Mahmood 2006). Returns clearance in the same units as `cl_ref`.
#[must_use]
pub fn allometric_clearance(cl_ref: f64, bw_ref: f64, bw_target: f64) -> f64 {
    cl_ref * (bw_target / bw_ref).powf(0.75)
}

/// Allometric volume of distribution scaling.
///
/// `Vd_target = Vd_ref × (BW_target / BW_ref)^1.0`
///
/// Volume scales linearly with body weight (exponent = 1.0).
#[must_use]
pub fn allometric_volume(vd_ref: f64, bw_ref: f64, bw_target: f64) -> f64 {
    vd_ref * (bw_target / bw_ref)
}

/// Half-life from clearance and volume of distribution.
///
/// `t½ = ln(2) × Vd / CL`
#[must_use]
pub fn allometric_half_life(vd: f64, cl: f64) -> f64 {
    if cl <= 0.0 {
        return f64::INFINITY;
    }
    core::f64::consts::LN_2 * vd / cl
}

/// Scale PK parameters from one species to another via allometry.
///
/// Returns a new `SpeciesPkParams` with clearance and volume scaled by body
/// weight exponents. Bioavailability is copied (no allometric basis for
/// bioavailability scaling).
#[must_use]
pub fn scale_across_species(
    reference: &SpeciesPkParams,
    target_species: Species,
    target_bw_kg: f64,
) -> SpeciesPkParams {
    let cl = allometric_clearance(
        reference.clearance_l_hr_kg * reference.body_weight_kg,
        reference.body_weight_kg,
        target_bw_kg,
    ) / target_bw_kg;

    let vd = allometric_volume(
        reference.volume_distribution_l_kg * reference.body_weight_kg,
        reference.body_weight_kg,
        target_bw_kg,
    ) / target_bw_kg;

    SpeciesPkParams {
        species: target_species,
        body_weight_kg: target_bw_kg,
        clearance_l_hr_kg: cl,
        volume_distribution_l_kg: vd,
        bioavailability: reference.bioavailability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn allometric_cl_same_weight() {
        let cl = allometric_clearance(10.0, 70.0, 70.0);
        assert!((cl - 10.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn allometric_cl_exponent() {
        let cl_ratio = allometric_clearance(1.0, 1.0, 10.0);
        let expected = 10.0_f64.powf(0.75);
        assert!((cl_ratio - expected).abs() < tolerances::ALLOMETRIC_CL_RATIO);
    }

    #[test]
    fn allometric_vd_linear() {
        let vd = allometric_volume(10.0, 70.0, 140.0);
        assert!((vd - 20.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn half_life_formula() {
        let t_half = allometric_half_life(10.0, 2.0);
        let expected = core::f64::consts::LN_2 * 10.0 / 2.0;
        assert!((t_half - expected).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn half_life_zero_clearance() {
        assert!(allometric_half_life(10.0, 0.0).is_infinite());
    }

    #[test]
    fn canine_pk_params_exist() {
        let pk = Species::Canine.oclacitinib_pk();
        assert!(pk.body_weight_kg > 0.0);
        assert!(pk.clearance_l_hr_kg > 0.0);
        assert!(pk.bioavailability > 0.0 && pk.bioavailability <= 1.0);
    }

    #[test]
    fn scale_canine_to_human() {
        let canine = Species::Canine.oclacitinib_pk();
        let human = scale_across_species(&canine, Species::Human, 70.0);
        assert_eq!(human.species, Species::Human);
        assert!((human.body_weight_kg - 70.0).abs() < tolerances::MACHINE_EPSILON);
        assert!(human.clearance_l_hr_kg > 0.0);
    }

    #[test]
    fn scale_roundtrip_identity() {
        let canine = Species::Canine.oclacitinib_pk();
        let scaled = scale_across_species(&canine, Species::Canine, canine.body_weight_kg);
        assert!(
            (scaled.clearance_l_hr_kg - canine.clearance_l_hr_kg).abs()
                < tolerances::ALLOMETRIC_CL_RATIO
        );
    }
}
