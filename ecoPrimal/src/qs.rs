// SPDX-License-Identifier: AGPL-3.0-or-later
//! Quorum sensing (QS) gene profiling for functional Anderson disorder.
//!
//! Extends the structural disorder parameter `W` (derived from Pielou evenness)
//! with a functional dimension based on inter-species signaling capacity.
//! Two communities with identical Pielou evenness but different QS gene
//! repertoires would have the same structural `W` but different effective
//! disorder — their signaling landscapes differ fundamentally.
//!
//! ## QS Families
//!
//! Nine QS gene families relevant to the human gut and cross-site microbiome:
//!
//! | Family | Signal | Gut relevance |
//! |--------|--------|---------------|
//! | `LuxI`/`LuxR` | AHL | Gram-negative cell-density |
//! | `LuxS` (AI-2) | DPD-derived | Universal inter-species |
//! | `Agr` | AIP | *C. difficile* virulence |
//! | `Com` | CSP | *Streptococcus* competence |
//! | `Las`/`Rhl` | HSL cascade | *Pseudomonas* (dysbiosis marker) |
//! | `Fsr` | GBAP | *Enterococcus* virulence |
//! | `QseBC` | Epinephrine/NE | Inter-kingdom signaling |
//! | `VqsM` | Vibrio QS | Cholera-associated dysbiosis |
//! | `PqsABCDE` | PQS | *Pseudomonas* pathogenesis |
//!
//! ## Integration with Anderson model
//!
//! ```text
//! Community abundances → Pielou(J) → W_structural
//! Community + QS matrix → qs_profile() → W_functional
//! W_effective = α·W_structural + (1-α)·W_functional
//!     → Hamiltonian H(W_effective) → eigensolve → IPR → ξ → CR
//! ```
//!
//! ## Data source
//!
//! The QS gene matrix is built from NCBI Gene/Protein by
//! `data/fetch_qs_genes.py` and deserialized from [`crate::QS_GENE_MATRIX_FILE`].
//! See `specs/QS_GENE_PROFILING.md` for the full design.

use serde::Deserialize;

/// Number of tracked QS gene families.
pub const NUM_FAMILIES: usize = 9;

/// QS gene family identifiers.
///
/// Order matches the column layout of [`QsGeneMatrix::presence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum QsFamily {
    /// `LuxI`/`LuxR` — N-acyl homoserine lactones (Gram-negative)
    LuxIR,
    /// `LuxS` / AI-2 — universal inter-species signal
    LuxS,
    /// Agr — autoinducing peptides (Gram-positive, *C. difficile*)
    Agr,
    /// Com — competence-stimulating peptide (*Streptococcus*)
    Com,
    /// Las/Rhl — hierarchical HSL cascade (*Pseudomonas*)
    LasRhl,
    /// Fsr — gelatinase biosynthesis pheromone (*Enterococcus*)
    Fsr,
    /// `QseBC` — epinephrine/norepinephrine inter-kingdom signaling
    QseBC,
    /// `VqsM` — Vibrio quorum signal
    VqsM,
    /// `PqsABCDE` — Pseudomonas quinolone signal
    PqsABCDE,
}

impl QsFamily {
    /// All families in canonical order (matches matrix columns).
    pub const ALL: [Self; NUM_FAMILIES] = [
        Self::LuxIR,
        Self::LuxS,
        Self::Agr,
        Self::Com,
        Self::LasRhl,
        Self::Fsr,
        Self::QseBC,
        Self::VqsM,
        Self::PqsABCDE,
    ];

    /// Index into the presence matrix columns.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::LuxIR => 0,
            Self::LuxS => 1,
            Self::Agr => 2,
            Self::Com => 3,
            Self::LasRhl => 4,
            Self::Fsr => 5,
            Self::QseBC => 6,
            Self::VqsM => 7,
            Self::PqsABCDE => 8,
        }
    }
}

/// QS gene presence/absence matrix: species × families.
///
/// Deserialized from [`crate::QS_GENE_MATRIX_FILE`] produced by `data/fetch_qs_genes.py`.
/// Each row is a genus/species, each column a [`QsFamily`].
#[derive(Debug, Clone, Deserialize)]
pub struct QsGeneMatrix {
    /// Species/genus names (row labels).
    pub species: Vec<String>,
    /// Family names as strings (column labels) — for JSON round-trip.
    pub families: Vec<String>,
    /// Presence/absence: `presence[species_idx][family_idx]`.
    pub presence: Vec<Vec<bool>>,
}

impl QsGeneMatrix {
    /// Number of species in the matrix.
    #[must_use]
    pub const fn num_species(&self) -> usize {
        self.species.len()
    }

    /// Look up a species index by name (case-insensitive prefix match on genus).
    #[must_use]
    pub fn find_species(&self, name: &str) -> Option<usize> {
        let lower = name.to_lowercase();
        self.species
            .iter()
            .position(|s| s.to_lowercase().starts_with(&lower))
    }

    /// Check if a species has a specific QS family gene.
    ///
    /// Returns `false` if the species index is out of range.
    #[must_use]
    pub fn has_gene(&self, species_idx: usize, family: QsFamily) -> bool {
        self.presence
            .get(species_idx)
            .and_then(|row| row.get(family.index()))
            .copied()
            .unwrap_or(false)
    }
}

/// QS signaling profile for a microbial community.
///
/// Computed by [`qs_profile`] from community abundances and a [`QsGeneMatrix`].
#[derive(Debug, Clone)]
pub struct QsProfile {
    /// Fraction of community abundance carrying each QS family gene.
    pub family_densities: [f64; NUM_FAMILIES],
    /// Fraction of community abundance carrying ANY QS gene.
    pub total_qs_density: f64,
    /// Shannon entropy across QS family densities (signaling diversity).
    pub signaling_diversity: f64,
}

/// Fraction of community abundance carrying genes from a specific QS family.
///
/// For each species present in both `abundances` and `qs_matrix`, sums the
/// abundance of those species that have the given QS family gene.
///
/// `abundances` and `qs_matrix.species` must be aligned (same length, same order).
#[must_use]
pub fn qs_gene_density(abundances: &[f64], qs_matrix: &QsGeneMatrix, family: QsFamily) -> f64 {
    let n = abundances.len().min(qs_matrix.num_species());
    abundances
        .iter()
        .take(n)
        .enumerate()
        .filter(|&(i, _)| qs_matrix.has_gene(i, family))
        .map(|(_, &a)| a)
        .sum()
}

/// Compute the full QS signaling profile for a community.
///
/// Returns per-family densities, total QS density, and Shannon entropy
/// of the family density distribution (signaling diversity).
#[must_use]
pub fn qs_profile(abundances: &[f64], qs_matrix: &QsGeneMatrix) -> QsProfile {
    let mut family_densities = [0.0_f64; NUM_FAMILIES];
    for family in QsFamily::ALL {
        family_densities[family.index()] = qs_gene_density(abundances, qs_matrix, family);
    }

    let total_qs_density: f64 = {
        let n = abundances.len().min(qs_matrix.num_species());
        abundances
            .iter()
            .take(n)
            .enumerate()
            .filter(|&(i, _)| QsFamily::ALL.iter().any(|&f| qs_matrix.has_gene(i, f)))
            .map(|(_, &a)| a)
            .sum()
    };

    let signaling_diversity = {
        let sum: f64 = family_densities.iter().sum();
        if sum <= 0.0 {
            0.0
        } else {
            let mut h = 0.0;
            for &d in &family_densities {
                if d > 0.0 {
                    let p = d / sum;
                    h -= p * p.ln();
                }
            }
            h
        }
    };

    QsProfile {
        family_densities,
        total_qs_density,
        signaling_diversity,
    }
}

/// Effective disorder incorporating both structural and functional dimensions.
///
/// `W_effective = alpha * W_structural + (1 - alpha) * W_functional`
///
/// where:
/// - `W_structural = evenness_to_disorder(pielou_j, w_scale)` (existing)
/// - `W_functional` = QS-weighted disorder: higher QS density → lower disorder
///   (more signaling → more coordination → more "extended" community behavior)
/// - `alpha` = mixing parameter (expected ~0.6–0.8 structural-dominant)
///
/// The functional disorder is derived as: `W_functional = w_scale * (1 - total_qs_density)`
/// — communities with 100% QS coverage have zero functional disorder;
/// communities with no QS genes have maximum functional disorder.
///
/// # Panics
///
/// Does not panic. Clamps `alpha` to `[0, 1]` and `total_qs_density` to `[0, 1]`.
#[must_use]
pub fn effective_disorder(pielou_j: f64, profile: &QsProfile, alpha: f64, w_scale: f64) -> f64 {
    let alpha = alpha.clamp(0.0, 1.0);
    let qs_density = profile.total_qs_density.clamp(0.0, 1.0);

    let w_structural = pielou_j * w_scale;
    let w_functional = (1.0 - qs_density) * w_scale;

    alpha.mul_add(w_structural, (1.0 - alpha) * w_functional)
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_matrix() -> QsGeneMatrix {
        QsGeneMatrix {
            species: vec![
                "Bacteroides".into(),
                "Clostridioides".into(),
                "Escherichia".into(),
                "Faecalibacterium".into(),
                "Enterococcus".into(),
                "Pseudomonas".into(),
            ],
            families: vec![
                "LuxIR".into(),
                "LuxS".into(),
                "Agr".into(),
                "Com".into(),
                "LasRhl".into(),
                "Fsr".into(),
                "QseBC".into(),
                "VqsM".into(),
                "PqsABCDE".into(),
            ],
            // Bacteroides:      LuxIR=T, LuxS=T
            // Clostridioides:   LuxS=T, Agr=T
            // Escherichia:      LuxS=T, QseBC=T
            // Faecalibacterium: LuxS=T
            // Enterococcus:     Fsr=T
            // Pseudomonas:      LasRhl=T, PqsABCDE=T
            presence: vec![
                vec![true, true, false, false, false, false, false, false, false],
                vec![false, true, true, false, false, false, false, false, false],
                vec![false, true, false, false, false, false, true, false, false],
                vec![false, true, false, false, false, false, false, false, false],
                vec![false, false, false, false, false, true, false, false, false],
                vec![false, false, false, false, true, false, false, false, true],
            ],
        }
    }

    #[test]
    fn family_index_is_canonical() {
        assert_eq!(QsFamily::LuxIR.index(), 0);
        assert_eq!(QsFamily::LuxS.index(), 1);
        assert_eq!(QsFamily::Agr.index(), 2);
        assert_eq!(QsFamily::Com.index(), 3);
        assert_eq!(QsFamily::LasRhl.index(), 4);
        assert_eq!(QsFamily::Fsr.index(), 5);
        assert_eq!(QsFamily::QseBC.index(), 6);
        assert_eq!(QsFamily::VqsM.index(), 7);
        assert_eq!(QsFamily::PqsABCDE.index(), 8);
    }

    #[test]
    fn has_gene_lookup() {
        let m = test_matrix();
        assert!(m.has_gene(0, QsFamily::LuxIR));
        assert!(m.has_gene(0, QsFamily::LuxS));
        assert!(!m.has_gene(0, QsFamily::Agr));
        assert!(m.has_gene(1, QsFamily::Agr));
        assert!(m.has_gene(4, QsFamily::Fsr));
        assert!(m.has_gene(5, QsFamily::LasRhl));
        assert!(m.has_gene(2, QsFamily::QseBC)); // Escherichia
        assert!(m.has_gene(5, QsFamily::PqsABCDE)); // Pseudomonas
        assert!(!m.has_gene(0, QsFamily::VqsM)); // No Vibrio in test matrix
        assert!(!m.has_gene(99, QsFamily::LuxS));
    }

    #[test]
    fn find_species_case_insensitive() {
        let m = test_matrix();
        assert_eq!(m.find_species("bacteroides"), Some(0));
        assert_eq!(m.find_species("Clostridioides"), Some(1));
        assert_eq!(m.find_species("ESCHERICHIA"), Some(2));
        assert_eq!(m.find_species("nonexistent"), None);
    }

    #[test]
    fn qs_gene_density_uniform() {
        let m = test_matrix();
        let uniform = [1.0 / 6.0; 6];

        let luxs_density = qs_gene_density(&uniform, &m, QsFamily::LuxS);
        // Bacteroides, Clostridioides, Escherichia, Faecalibacterium = 4/6
        assert!((luxs_density - 4.0 / 6.0).abs() < 1e-10);

        let agr_density = qs_gene_density(&uniform, &m, QsFamily::Agr);
        // Only Clostridioides = 1/6
        assert!((agr_density - 1.0 / 6.0).abs() < 1e-10);

        let com_density = qs_gene_density(&uniform, &m, QsFamily::Com);
        assert!((com_density).abs() < 1e-10);

        let qsebc_density = qs_gene_density(&uniform, &m, QsFamily::QseBC);
        // Escherichia only = 1/6
        assert!((qsebc_density - 1.0 / 6.0).abs() < 1e-10);

        let pqs_density = qs_gene_density(&uniform, &m, QsFamily::PqsABCDE);
        // Pseudomonas only = 1/6
        assert!((pqs_density - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn qs_profile_healthy_community() {
        let m = test_matrix();
        // Healthy: dominated by Bacteroides + Faecalibacterium (commensals)
        let healthy = [0.30, 0.05, 0.15, 0.30, 0.05, 0.15];
        let prof = qs_profile(&healthy, &m);

        // All species have at least one QS gene → total_qs_density = 1.0
        assert!(
            (prof.total_qs_density - 1.0).abs() < 1e-10,
            "all species have QS genes"
        );

        // LuxS density: Bacteroides(0.30) + Clostridioides(0.05) +
        //               Escherichia(0.15) + Faecalibacterium(0.30) = 0.80
        assert!((prof.family_densities[QsFamily::LuxS.index()] - 0.80).abs() < 1e-10);

        assert!(
            prof.signaling_diversity > 0.0,
            "multiple families → positive H'"
        );
    }

    #[test]
    fn qs_profile_dysbiotic_monoculture() {
        let m = test_matrix();
        // Dysbiotic: Clostridioides dominates (like C. diff overgrowth)
        let dysbiotic = [0.01, 0.90, 0.01, 0.01, 0.01, 0.06];
        let prof = qs_profile(&dysbiotic, &m);

        // Still high total QS density (all species have some QS)
        assert!(prof.total_qs_density > 0.9);

        // But Agr dominance marker should be high
        let agr = prof.family_densities[QsFamily::Agr.index()];
        assert!(agr > 0.85, "Clostridioides Agr dominates: {agr}");
    }

    #[test]
    fn effective_disorder_pure_structural() {
        let m = test_matrix();
        let abundances = [1.0 / 6.0; 6];
        let prof = qs_profile(&abundances, &m);

        let w_eff = effective_disorder(0.8, &prof, 1.0, 20.0);
        let w_struct = 0.8 * 20.0;
        assert!(
            (w_eff - w_struct).abs() < 1e-10,
            "alpha=1.0 → pure structural: {w_eff} vs {w_struct}"
        );
    }

    #[test]
    fn effective_disorder_pure_functional() {
        let m = test_matrix();
        let abundances = [1.0 / 6.0; 6];
        let prof = qs_profile(&abundances, &m);

        // alpha=0 → pure functional. total_qs_density=1.0 → W_func = 0
        let w_eff = effective_disorder(0.8, &prof, 0.0, 20.0);
        let w_func = (1.0 - prof.total_qs_density) * 20.0;
        assert!(
            (w_eff - w_func).abs() < 1e-10,
            "alpha=0.0 → pure functional: {w_eff} vs {w_func}"
        );
    }

    #[test]
    fn effective_disorder_mixed() {
        let m = test_matrix();
        let abundances = [1.0 / 6.0; 6];
        let prof = qs_profile(&abundances, &m);

        let alpha = 0.7;
        let w_scale = 20.0;
        let w_eff = effective_disorder(0.8, &prof, alpha, w_scale);

        let w_struct = 0.8 * w_scale;
        let w_func = (1.0 - prof.total_qs_density) * w_scale;
        let expected = alpha.mul_add(w_struct, (1.0 - alpha) * w_func);

        assert!(
            (w_eff - expected).abs() < 1e-10,
            "mixed: {w_eff} vs {expected}"
        );
    }

    #[test]
    fn effective_disorder_healthy_vs_depleted() {
        let m = test_matrix();
        let w_scale = 20.0;
        let alpha = 0.7;

        // Healthy gut: high evenness, full QS coverage
        let healthy = [1.0 / 6.0; 6];
        let pielou_healthy = 1.0; // perfect evenness
        let prof_healthy = qs_profile(&healthy, &m);
        let w_healthy = effective_disorder(pielou_healthy, &prof_healthy, alpha, w_scale);

        // Depleted gut: low evenness (one species dominates), less QS diversity
        // Only 2 species with nonzero abundance, low Pielou
        let depleted = [0.95, 0.05, 0.0, 0.0, 0.0, 0.0];
        let pielou_depleted = 0.3;
        let prof_depleted = qs_profile(&depleted, &m);
        let w_depleted = effective_disorder(pielou_depleted, &prof_depleted, alpha, w_scale);

        assert!(
            w_healthy > w_depleted,
            "healthy gut should have higher effective W (more extended): \
             healthy={w_healthy}, depleted={w_depleted}"
        );
    }

    #[test]
    fn qs_profile_empty_community() {
        let m = test_matrix();
        let empty: [f64; 0] = [];
        let prof = qs_profile(&empty, &m);
        assert!((prof.total_qs_density).abs() < 1e-10);
        assert!((prof.signaling_diversity).abs() < 1e-10);
        for &d in &prof.family_densities {
            assert!((d).abs() < 1e-10);
        }
    }

    #[test]
    fn qs_gene_matrix_json_roundtrip() {
        let m = test_matrix();
        let json = serde_json::to_string(&serde_json::json!({
            "species": m.species,
            "families": m.families,
            "presence": m.presence,
        }))
        .ok();
        assert!(json.is_some(), "serialization should succeed");

        if let Some(ref s) = json {
            let parsed: Result<QsGeneMatrix, _> = serde_json::from_str(s);
            assert!(parsed.is_ok(), "deserialization should succeed");
            if let Ok(p) = parsed {
                assert_eq!(p.species.len(), 6);
                assert_eq!(p.families.len(), 9);
                assert_eq!(p.presence.len(), 6);
            }
        }
    }

    #[test]
    fn effective_disorder_alpha_clamped() {
        let m = test_matrix();
        let abundances = [1.0 / 6.0; 6];
        let prof = qs_profile(&abundances, &m);

        let w_over = effective_disorder(0.8, &prof, 1.5, 20.0);
        let w_at_1 = effective_disorder(0.8, &prof, 1.0, 20.0);
        assert!(
            (w_over - w_at_1).abs() < 1e-10,
            "alpha>1 should clamp to 1.0"
        );

        let w_under = effective_disorder(0.8, &prof, -0.5, 20.0);
        let w_at_0 = effective_disorder(0.8, &prof, 0.0, 20.0);
        assert!(
            (w_under - w_at_0).abs() < 1e-10,
            "alpha<0 should clamp to 0.0"
        );
    }

    #[test]
    fn determinism_qs_profile() {
        let m = test_matrix();
        let abundances = [0.20, 0.15, 0.10, 0.25, 0.10, 0.20];
        let p1 = qs_profile(&abundances, &m);
        let p2 = qs_profile(&abundances, &m);

        assert_eq!(
            p1.total_qs_density.to_bits(),
            p2.total_qs_density.to_bits(),
            "QS profile must be bit-identical across runs"
        );
        for i in 0..NUM_FAMILIES {
            assert_eq!(
                p1.family_densities[i].to_bits(),
                p2.family_densities[i].to_bits(),
                "family density {i} must be bit-identical"
            );
        }
    }
}
