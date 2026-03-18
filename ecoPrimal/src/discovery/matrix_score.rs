// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fajgenbaum MATRIX drug repurposing framework with Anderson geometry augmentation.
//!
//! References:
//! - Fajgenbaum DC et al. (2018) *NEJM* 379:1941 — MATRIX concept
//! - Fajgenbaum DC et al. (2019) *JCI* 129:4857 — scoring methodology
//! - Anderson PW (1958) *Phys Rev* 109:1492 — localization
//!
//! The combined score extends MATRIX with two physics-based dimensions:
//! - **Tissue geometry**: Anderson localization predicts drug penetration depth
//! - **Disorder impact**: drug effect on microbiome diversity (colonization resistance)

use serde::{Deserialize, Serialize};

/// A scored drug-disease pair with pathway, geometry, and disorder dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEntry {
    pub compound: String,
    pub disease: String,
    pub pathway_score: f64,
    pub tissue_geometry: f64,
    pub disorder_factor: f64,
    pub combined_score: f64,
}

/// Pathway selectivity score from IC50 profile.
///
/// Measures how selective a compound is for the disease-relevant target
/// relative to off-target kinases. Uses geometric mean of off-target IC50s
/// normalized by on-target IC50:
///
/// `score = geomean(IC50_off) / (geomean(IC50_off) + IC50_on)`
///
/// Returns a value in `[0, 1]`. A perfectly selective compound (off-targets
/// infinitely weak) scores 1.0. A non-selective compound scores ~0.5.
///
/// # Panics
///
/// Returns 0.0 if `off_target_ic50s` is empty (no selectivity measurable).
#[must_use]
pub fn pathway_selectivity_score(on_target_ic50: f64, off_target_ic50s: &[f64]) -> f64 {
    if off_target_ic50s.is_empty() {
        return 0.0;
    }
    let log_sum: f64 = off_target_ic50s.iter().map(|x| x.ln()).sum();
    #[expect(clippy::cast_precision_loss, reason = "off-target count ≪ 2^52")]
    let geomean = (log_sum / off_target_ic50s.len() as f64).exp();
    geomean / (geomean + on_target_ic50)
}

/// Anderson tissue geometry factor.
///
/// Predicts drug penetration through a tissue barrier of thickness `L`
/// given the Anderson localization length `xi` in that tissue:
///
/// `factor = 1 - exp(-xi / L)`
///
/// - Large ξ/L → extended states → good penetration → factor → 1
/// - Small ξ/L → localized states → poor penetration → factor → 0
///
/// This is the transmission probability for a wave in the Anderson regime.
#[must_use]
pub fn tissue_geometry_factor(localization_length: f64, tissue_thickness: f64) -> f64 {
    if tissue_thickness <= 0.0 || localization_length <= 0.0 {
        return 0.0;
    }
    1.0 - (-localization_length / tissue_thickness).exp()
}

/// Disorder impact factor from drug effect on microbiome diversity.
///
/// Measures how a compound changes the Anderson disorder parameter W
/// (derived from gut Pielou diversity). The factor is:
///
/// `factor = 1 + (W_treated - W_baseline) / W_baseline`, clamped to `[0, 2]`
///
/// - Neutral (no change): 1.0
/// - Beneficial (increases diversity): > 1.0 (capped at 2.0)
/// - Harmful (reduces diversity, e.g., antibiotics): < 1.0 (floored at 0.0)
#[must_use]
pub fn disorder_impact_factor(w_baseline: f64, w_treated: f64) -> f64 {
    if w_baseline <= 0.0 {
        return 1.0;
    }
    let raw = 1.0 + (w_treated - w_baseline) / w_baseline;
    raw.clamp(0.0, 2.0)
}

/// Combined MATRIX-Anderson score.
///
/// `combined = pathway × geometry × disorder`
///
/// All three factors are in `[0, 2]`, so the combined score is in `[0, 4]`.
/// In practice, pathway and geometry are in `[0, 1]`, and disorder in `[0, 2]`,
/// giving a practical range of `[0, 2]`.
#[must_use]
pub fn matrix_combined_score(pathway: f64, geometry: f64, disorder: f64) -> f64 {
    pathway * geometry * disorder
}

/// Tissue and microbiome context shared across a scoring panel.
#[derive(Debug, Clone)]
pub struct TissueContext {
    /// Anderson localization length in the tissue (arbitrary units).
    pub localization_length: f64,
    /// Tissue barrier thickness (same units as `localization_length`).
    pub tissue_thickness: f64,
    /// Baseline Anderson disorder parameter W (from gut Pielou diversity).
    pub w_baseline: f64,
    /// Post-treatment disorder parameter W.
    pub w_treated: f64,
}

/// Score a compound against a disease target, producing a full [`MatrixEntry`].
#[must_use]
pub fn score_compound(
    compound: &str,
    disease: &str,
    on_target_ic50: f64,
    off_target_ic50s: &[f64],
    ctx: &TissueContext,
) -> MatrixEntry {
    let localization_length = ctx.localization_length;
    let tissue_thickness = ctx.tissue_thickness;
    let w_baseline = ctx.w_baseline;
    let w_treated = ctx.w_treated;
    let pathway = pathway_selectivity_score(on_target_ic50, off_target_ic50s);
    let geometry = tissue_geometry_factor(localization_length, tissue_thickness);
    let disorder = disorder_impact_factor(w_baseline, w_treated);
    let combined = matrix_combined_score(pathway, geometry, disorder);

    MatrixEntry {
        compound: compound.to_owned(),
        disease: disease.to_owned(),
        pathway_score: pathway,
        tissue_geometry: geometry,
        disorder_factor: disorder,
        combined_score: combined,
    }
}

/// Score a panel of compounds against a shared disease target.
///
/// Each tuple contains `(name, on_target_ic50, off_target_ic50s)`.
/// Tissue geometry and disorder parameters are shared across the panel
/// (same disease, same tissue, same baseline microbiome).
#[must_use]
pub fn score_panel(
    compounds: &[(&str, f64, &[f64])],
    disease: &str,
    ctx: &TissueContext,
) -> Vec<MatrixEntry> {
    compounds
        .iter()
        .map(|(name, ic50_on, ic50_off)| score_compound(name, disease, *ic50_on, ic50_off, ctx))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn pathway_perfect_selectivity() {
        let score = pathway_selectivity_score(1.0, &[10000.0, 10000.0, 10000.0]);
        assert!((score - 10000.0 / 10001.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pathway_no_selectivity() {
        let score = pathway_selectivity_score(10.0, &[10.0, 10.0]);
        assert!((score - 0.5).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pathway_empty_off_targets() {
        assert!((pathway_selectivity_score(10.0, &[])).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn tissue_geometry_large_xi() {
        let factor = tissue_geometry_factor(100.0, 1.0);
        assert!(factor > 0.99);
    }

    #[test]
    fn tissue_geometry_small_xi() {
        let factor = tissue_geometry_factor(0.01, 1.0);
        assert!(factor < 0.01);
    }

    #[test]
    fn tissue_geometry_zero_thickness() {
        assert!((tissue_geometry_factor(10.0, 0.0)).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn disorder_neutral() {
        let factor = disorder_impact_factor(5.0, 5.0);
        assert!((factor - 1.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn disorder_beneficial() {
        let factor = disorder_impact_factor(5.0, 7.5);
        assert!((factor - 1.5).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn disorder_harmful_clamped() {
        let factor = disorder_impact_factor(5.0, 0.0);
        assert!((factor - 0.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn disorder_caps_at_two() {
        let factor = disorder_impact_factor(1.0, 100.0);
        assert!((factor - 2.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn combined_product_identity() {
        let combined = matrix_combined_score(0.8, 0.6, 1.2);
        assert!((0.8_f64 * 0.6).mul_add(-1.2, combined).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn panel_ordering() {
        let ctx = TissueContext {
            localization_length: 10.0,
            tissue_thickness: 1.0,
            w_baseline: 5.0,
            w_treated: 5.0,
        };
        let panel = score_panel(
            &[
                ("selective", 1.0, &[1000.0, 1000.0]),
                ("nonselective", 500.0, &[1000.0, 1000.0]),
            ],
            "AD",
            &ctx,
        );
        assert!(panel[0].combined_score > panel[1].combined_score);
    }

    #[test]
    fn geometry_reranks_panel() {
        let good_ctx = TissueContext {
            localization_length: 50.0,
            tissue_thickness: 1.0,
            w_baseline: 5.0,
            w_treated: 5.0,
        };
        let poor_ctx = TissueContext {
            localization_length: 0.1,
            tissue_thickness: 1.0,
            w_baseline: 5.0,
            w_treated: 5.0,
        };
        let good_tissue = score_compound("drug_a", "AD", 5.0, &[500.0], &good_ctx);
        let poor_tissue = score_compound("drug_b", "AD", 5.0, &[500.0], &poor_ctx);
        assert!(good_tissue.combined_score > poor_tissue.combined_score);
        assert!(
            (good_tissue.pathway_score - poor_tissue.pathway_score).abs()
                < tolerances::MACHINE_EPSILON
        );
    }
}
