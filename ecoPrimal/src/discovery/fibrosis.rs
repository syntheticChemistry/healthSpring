// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fibrosis pathway scoring for drug discovery.
//!
//! The Rho/MRTF/SRF signaling pathway is a master regulator of fibrosis.
//! Anti-fibrotic compounds that inhibit this pathway are relevant to
//! atopic dermatitis (skin barrier fibrosis) and other fibrotic diseases.
//!
//! Anderson geometry scoring for fibrotic tissue: fibrosis increases lattice
//! order (reduces disorder W), creating extended states that paradoxically
//! trap drug penetration — the drug distributes broadly but cannot concentrate
//! at the fibrotic site.
//!
//! References:
//! - Olson EN & Nordheim A (2010) *Nat Rev Mol Cell Biol* 11:353 — SRF review
//! - Haak AJ et al. (2014) *J Pharmacol Exp Ther* 349:480 — CCG-1423 MRTF inhibition
//! - Small EM (2012) *Trends Cell Biol* 22:97 — MRTF in fibrosis
//! - Neubig RR (2015) — Rho pathway small molecule inhibitors

use serde::{Deserialize, Serialize};

/// Fibrosis pathway components scored for drug targeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibrosisPathwayScore {
    pub compound: String,
    /// Rho `GTPase` inhibition (0 = none, 1 = complete).
    pub rho_inhibition: f64,
    /// MRTF nuclear translocation block (0 = none, 1 = complete).
    pub mrtf_block: f64,
    /// SRF transcriptional activity reduction (0 = none, 1 = complete).
    pub srf_reduction: f64,
    /// Combined anti-fibrotic score.
    pub anti_fibrotic_score: f64,
}

/// Anti-fibrotic compound profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiFibroticCompound {
    pub name: String,
    /// IC50 for Rho inhibition (µM).
    pub rho_ic50_um: f64,
    /// IC50 for MRTF nuclear translocation block (µM).
    pub mrtf_ic50_um: f64,
    /// IC50 for SRF reporter reduction (µM).
    pub srf_ic50_um: f64,
}

/// CCG-1423: first-in-class MRTF/SRF pathway inhibitor (Evelyn 2007).
#[must_use]
pub fn ccg_1423() -> AntiFibroticCompound {
    AntiFibroticCompound {
        name: "CCG-1423".into(),
        rho_ic50_um: 10.0,
        mrtf_ic50_um: 3.2,
        srf_ic50_um: 5.0,
    }
}

#[must_use]
pub fn ccg_203971() -> AntiFibroticCompound {
    AntiFibroticCompound {
        name: "CCG-203971".into(),
        rho_ic50_um: 15.0,
        mrtf_ic50_um: 1.5,
        srf_ic50_um: 2.8,
    }
}

/// Fractional inhibition from Hill dose-response at a given concentration.
///
/// `E = C^n / (IC50^n + C^n)` with `n = 1` (standard Hill).
#[must_use]
pub fn fractional_inhibition(concentration_um: f64, ic50_um: f64) -> f64 {
    if ic50_um <= 0.0 || concentration_um <= 0.0 {
        return 0.0;
    }
    concentration_um / (ic50_um + concentration_um)
}

/// Score a compound's anti-fibrotic activity at a given concentration.
///
/// The Rho → MRTF → SRF cascade is sequential: blocking any node
/// reduces downstream activity. The combined score weights MRTF highest
/// (rate-limiting step for nuclear translocation):
///
/// `score = 0.2 × rho + 0.5 × mrtf + 0.3 × srf`
#[must_use]
pub fn score_anti_fibrotic(compound: &AntiFibroticCompound, concentration_um: f64) -> FibrosisPathwayScore {
    let rho = fractional_inhibition(concentration_um, compound.rho_ic50_um);
    let mrtf = fractional_inhibition(concentration_um, compound.mrtf_ic50_um);
    let srf = fractional_inhibition(concentration_um, compound.srf_ic50_um);

    let score = 0.2_f64.mul_add(rho, 0.5_f64.mul_add(mrtf, 0.3 * srf));

    FibrosisPathwayScore {
        compound: compound.name.clone(),
        rho_inhibition: rho,
        mrtf_block: mrtf,
        srf_reduction: srf,
        anti_fibrotic_score: score,
    }
}

/// Anderson geometry factor for fibrotic tissue.
///
/// Fibrosis reduces tissue disorder (collagen deposition creates ordered
/// lattice). This paradoxically increases localization length — the drug
/// spreads broadly but cannot concentrate.
///
/// For anti-fibrotic drugs, we want short localization length (localized
/// delivery to the fibrotic site). The geometry factor inverts the standard
/// tissue geometry: lower ξ = better for anti-fibrotic drugs.
///
/// `factor = exp(-xi / L)` (opposite of standard MATRIX geometry)
///
/// - Small ξ/L → drug concentrates locally → good for anti-fibrotic → high factor
/// - Large ξ/L → drug spreads broadly → poor for anti-fibrotic → low factor
#[must_use]
pub fn fibrotic_geometry_factor(localization_length: f64, tissue_thickness: f64) -> f64 {
    if tissue_thickness <= 0.0 || localization_length <= 0.0 {
        return 0.0;
    }
    (-localization_length / tissue_thickness).exp()
}

/// Combined fibrosis + MATRIX score for anti-fibrotic compound.
///
/// `combined = anti_fibrotic × fibrotic_geometry × disorder_factor`
#[must_use]
pub fn fibrosis_matrix_score(
    anti_fibrotic: f64,
    fibrotic_geometry: f64,
    disorder_factor: f64,
) -> f64 {
    anti_fibrotic * fibrotic_geometry * disorder_factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn fractional_zero_conc() {
        assert!(fractional_inhibition(0.0, 5.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn fractional_at_ic50() {
        let e = fractional_inhibition(5.0, 5.0);
        assert!((e - 0.5).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn fractional_saturation() {
        let e = fractional_inhibition(10000.0, 5.0);
        assert!(e > 0.99);
    }

    #[test]
    fn score_at_zero() {
        let compound = ccg_1423();
        let score = score_anti_fibrotic(&compound, 0.0);
        assert!(score.anti_fibrotic_score.abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn ccg203971_better_mrtf_than_ccg1423() {
        let c1 = ccg_1423();
        let c2 = ccg_203971();
        let s1 = score_anti_fibrotic(&c1, 5.0);
        let s2 = score_anti_fibrotic(&c2, 5.0);
        assert!(s2.mrtf_block > s1.mrtf_block);
    }

    #[test]
    fn fibrotic_geometry_inverted() {
        let standard = crate::discovery::tissue_geometry_factor(10.0, 1.0);
        let fibrotic = fibrotic_geometry_factor(10.0, 1.0);
        assert!((standard + fibrotic - 1.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn fibrotic_geometry_zero_thickness() {
        assert_eq!(fibrotic_geometry_factor(10.0, 0.0), 0.0);
    }

    #[test]
    fn score_product_identity() {
        let combined = fibrosis_matrix_score(0.7, 0.4, 1.1);
        assert!((combined - 0.7 * 0.4 * 1.1).abs() < tolerances::MACHINE_EPSILON);
    }
}
