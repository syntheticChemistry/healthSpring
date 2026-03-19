// SPDX-License-Identifier: AGPL-3.0-or-later
//! Affinity landscape modeling for the low-affinity binding regime.
//!
//! Traditional HTS optimizes for strong, specific binders (IC50 < 1 µM).
//! This module explores the full affinity spectrum — including the
//! deliberately weak (IC50 > 10 µM) regime that standard screens discard.
//!
//! ## Computation as Preprocessor
//!
//! Instead of analyzing hits post-screen, these models predict binding
//! landscapes *before* the experiment, turning the plate screener into
//! a computational prediction validator.
//!
//! ## Key Concepts
//!
//! - **Composite binding**: Multiple weak interactions that individually
//!   achieve < 20% occupancy but collectively create selective targeting
//!   through coincidence detection.
//! - **Affinity distribution**: Analyzing the full IC50 distribution of a
//!   compound across a target panel, not just the best hit.
//! - **Cross-reactivity matrix**: Pairwise fractional occupancies across
//!   all compound × target combinations.
//! - **Colonization occupancy**: Cumulative weak binding that creates
//!   competitive exclusion (probiotic adhesion model).
//!
//! References:
//! - Kärre K (1986) *Immunol Today* — "missing self" hypothesis, NK integration
//! - Anderson PW (1958) *Phys Rev* — localization in disordered systems
//! - Lisabeth et al. (2024) *Front Microbiol* — Brucella screen (full dose-response)

use crate::tolerances;

/// Hill fractional occupancy at a given concentration and IC50.
///
/// `f(C) = C^n / (IC50^n + C^n)`
///
/// This is the building block for all binding landscape models.
/// At low affinity (high IC50), fractional occupancy is small but nonzero.
#[must_use]
pub fn fractional_occupancy(concentration: f64, ic50: f64, hill_n: f64) -> f64 {
    if ic50 <= 0.0 || concentration < 0.0 {
        return 0.0;
    }
    let c_n = concentration.powf(hill_n);
    let ic50_n = ic50.powf(hill_n);
    c_n / (ic50_n + c_n)
}

/// Composite binding score for multi-target coincidence detection.
///
/// Models the NK cell integration analogy: a compound (or treatment)
/// interacts weakly with N surface markers. Each individual binding event
/// achieves fractional occupancy `f_i`. The composite signal is:
///
/// `S = 1 - product(1 - f_i)` for i in targets
///
/// This is the probability that **at least one** target is occupied,
/// which for many weak binders approximates the sum of occupancies
/// (for small `f_i`). But it correctly saturates at 1.0 for strong
/// cumulative binding.
///
/// For cancer targeting: normal cells have low composite S (few markers
/// match); cancer cells have high composite S (many markers match due
/// to surface disorder).
#[must_use]
pub fn composite_binding_score(occupancies: &[f64]) -> f64 {
    if occupancies.is_empty() {
        return 0.0;
    }
    let product_complement: f64 = occupancies
        .iter()
        .map(|&f| 1.0 - f.clamp(0.0, 1.0))
        .product();
    1.0 - product_complement
}

/// Binding profile across a panel of targets at a fixed concentration.
///
/// Returns the vector of fractional occupancies — the compound's
/// "fingerprint" across the target landscape. This is the full
/// binding profile, not just the best hit.
#[must_use]
pub fn binding_profile(concentration: f64, ic50s: &[f64], hill_n: f64) -> Vec<f64> {
    ic50s
        .iter()
        .map(|&ic50| fractional_occupancy(concentration, ic50, hill_n))
        .collect()
}

/// Affinity distribution statistics for a compound across a target panel.
///
/// Instead of reporting only the best IC50, characterizes the full
/// distribution: mean, median, breadth (fraction of targets with
/// measurable occupancy), and the critical "low-affinity tail" metrics.
#[derive(Debug, Clone)]
pub struct AffinityDistribution {
    /// Number of targets in the panel.
    pub n_targets: usize,
    /// Fraction of targets where occupancy > threshold at test concentration.
    pub breadth: f64,
    /// Mean fractional occupancy across all targets.
    pub mean_occupancy: f64,
    /// Maximum fractional occupancy (traditional "best hit").
    pub max_occupancy: f64,
    /// Composite binding score (coincidence model).
    pub composite_score: f64,
    /// Gini coefficient of occupancy distribution (0 = uniform, 1 = single target).
    /// Low Gini = broad weak binding; high Gini = specific strong binding.
    pub gini: f64,
}

/// Analyze the affinity distribution of a compound across a target panel.
///
/// `threshold` is the minimum fractional occupancy to count as "measurable"
/// (e.g., 0.05 for 5% occupancy).
#[must_use]
pub fn analyze_affinity_distribution(
    concentration: f64,
    ic50s: &[f64],
    hill_n: f64,
    threshold: f64,
) -> AffinityDistribution {
    let profile = binding_profile(concentration, ic50s, hill_n);
    let n = profile.len();

    if n == 0 {
        return AffinityDistribution {
            n_targets: 0,
            breadth: 0.0,
            mean_occupancy: 0.0,
            max_occupancy: 0.0,
            composite_score: 0.0,
            gini: 0.0,
        };
    }

    #[expect(clippy::cast_precision_loss, reason = "target count fits f64")]
    let n_f = n as f64;

    let above_threshold = profile.iter().filter(|&&f| f >= threshold).count();
    #[expect(clippy::cast_precision_loss, reason = "count fits f64")]
    let breadth = above_threshold as f64 / n_f;

    let mean_occupancy = profile.iter().sum::<f64>() / n_f;
    let max_occupancy = profile.iter().copied().fold(0.0_f64, f64::max);
    let composite_score = composite_binding_score(&profile);

    let gini = compute_gini(&profile);

    AffinityDistribution {
        n_targets: n,
        breadth,
        mean_occupancy,
        max_occupancy,
        composite_score,
        gini,
    }
}

/// Gini coefficient of a distribution (0 = perfectly equal, 1 = maximally unequal).
///
/// For binding profiles: low Gini means broad weak binding across many targets;
/// high Gini means concentrated strong binding at few targets.
fn compute_gini(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let sum: f64 = sorted.iter().sum();
    if sum < tolerances::DIVISION_GUARD {
        return 0.0;
    }

    #[expect(clippy::cast_precision_loss, reason = "index/count fits f64")]
    let numerator: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, &v)| (2.0f64.mul_add((i + 1) as f64, -(n as f64)) - 1.0) * v)
        .sum();

    #[expect(clippy::cast_precision_loss, reason = "count fits f64")]
    let denom = n as f64 * sum;
    (numerator / denom).clamp(0.0, 1.0)
}

/// Cross-reactivity matrix: fractional occupancy for each compound × target pair.
///
/// Returns a `n_compounds × n_targets` matrix where entry (i, j) is the
/// fractional occupancy of compound i at target j.
#[must_use]
pub fn cross_reactivity_matrix(
    concentration: f64,
    compound_ic50s: &[Vec<f64>],
    hill_n: f64,
) -> Vec<Vec<f64>> {
    compound_ic50s
        .iter()
        .map(|ic50s| binding_profile(concentration, ic50s, hill_n))
        .collect()
}

/// Low-affinity selectivity index.
///
/// For traditional drug discovery: `selectivity = IC50_off / IC50_on` (higher = more selective).
///
/// For low-affinity targeting: selectivity comes from the **composite score
/// differential** between target cells and non-target cells, not from
/// individual binding strength.
///
/// `selectivity = composite_target / max(composite_nontarget, guard)`
///
/// A compound with many weak interactions on cancer cells but few on
/// normal cells has high low-affinity selectivity even though no
/// individual binding is strong.
#[must_use]
pub fn low_affinity_selectivity(target_occupancies: &[f64], nontarget_occupancies: &[f64]) -> f64 {
    let s_target = composite_binding_score(target_occupancies);
    let s_nontarget = composite_binding_score(nontarget_occupancies);
    if s_nontarget < tolerances::DIVISION_GUARD {
        if s_target < tolerances::DIVISION_GUARD {
            return 1.0;
        }
        return f64::INFINITY;
    }
    s_target / s_nontarget
}

/// Colonization occupancy model for probiotic adhesion.
///
/// Models N bacterial species each weakly binding to M epithelial sites.
/// Each species i has adhesion affinity `K_i` at site j. The cumulative
/// occupancy at each site determines colonization resistance.
///
/// `site_occupancy_j = 1 - product(1 - f_ij)` for i in species
///
/// Returns the fraction of sites where cumulative occupancy exceeds the
/// colonization threshold (e.g., 0.5 = site is resistant to pathogen).
///
/// This connects to the Anderson disorder model: epithelial sites with
/// high glycosylation variation (disorder) create a heterogeneous binding
/// landscape where multiple weak binders achieve better coverage than
/// single strong binders.
#[must_use]
pub fn colonization_resistance(
    species_adhesion_profiles: &[Vec<f64>],
    resistance_threshold: f64,
) -> f64 {
    if species_adhesion_profiles.is_empty() {
        return 0.0;
    }

    let n_sites = species_adhesion_profiles
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0);
    if n_sites == 0 {
        return 0.0;
    }

    let mut resistant_count = 0_usize;
    for site in 0..n_sites {
        let site_occupancies: Vec<f64> = species_adhesion_profiles
            .iter()
            .map(|profile| profile.get(site).copied().unwrap_or(0.0))
            .collect();

        let cumulative = composite_binding_score(&site_occupancies);
        if cumulative >= resistance_threshold {
            resistant_count += 1;
        }
    }

    #[expect(clippy::cast_precision_loss, reason = "site count fits f64")]
    let fraction = resistant_count as f64 / n_sites as f64;
    fraction
}

/// Disorder-dependent adhesion profile generator.
///
/// Given a base adhesion strength and disorder parameter (Anderson W),
/// generates a site-specific adhesion profile where each site's affinity
/// is modulated by the local disorder.
///
/// `K_j = K_base * exp(disorder * noise_j)`
///
/// High disorder → wide spread of site affinities → some sites strongly
/// bound, others weakly bound. This is the Anderson lattice analog
/// for epithelial binding.
#[must_use]
pub fn disorder_adhesion_profile(
    n_sites: usize,
    base_affinity: f64,
    disorder_w: f64,
    seed: u64,
) -> Vec<f64> {
    use crate::rng::{lcg_step, normal_sample};

    let mut rng = seed;
    let mut profile = Vec::with_capacity(n_sites);

    for _ in 0..n_sites {
        let (z, next) = normal_sample(rng);
        rng = next;
        let site_affinity = base_affinity * (disorder_w * z).exp();
        profile.push(fractional_occupancy(1.0, site_affinity, 1.0));
    }
    // Consume rng to avoid lint about unused assignment
    let _ = lcg_step(rng);

    profile
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn fractional_occupancy_at_ic50_is_half() {
        let f = fractional_occupancy(10.0, 10.0, 1.0);
        assert!(
            (f - 0.5).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "at C=IC50, f should be 0.5, got {f}"
        );
    }

    #[test]
    fn fractional_occupancy_low_affinity_is_small() {
        let f = fractional_occupancy(1.0, 100.0, 1.0);
        assert!(f < 0.02, "at C=1, IC50=100, f should be ~0.01, got {f}");
    }

    #[test]
    fn composite_binding_single_weak() {
        let score = composite_binding_score(&[0.05]);
        assert!(
            (score - 0.05).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "single weak binder: {score}"
        );
    }

    #[test]
    fn composite_binding_many_weak_accumulates() {
        let occupancies = vec![0.05; 20];
        let score = composite_binding_score(&occupancies);
        assert!(
            score > 0.5,
            "20 sites at 5% each should accumulate: {score}"
        );
    }

    #[test]
    fn composite_binding_empty_is_zero() {
        assert!(composite_binding_score(&[]).abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn gini_uniform_is_zero() {
        let g = compute_gini(&[1.0, 1.0, 1.0, 1.0]);
        assert!(g < 0.01, "uniform distribution gini ~ 0: {g}");
    }

    #[test]
    fn gini_skewed_is_high() {
        let g = compute_gini(&[0.0, 0.0, 0.0, 1.0]);
        assert!(g > 0.5, "maximally skewed gini should be high: {g}");
    }

    #[test]
    fn affinity_distribution_broad_weak() {
        let ic50s = vec![50.0; 10];
        let dist = analyze_affinity_distribution(1.0, &ic50s, 1.0, 0.01);
        assert!(
            dist.breadth > 0.9,
            "all targets above threshold: {}",
            dist.breadth
        );
        assert!(
            dist.gini < 0.1,
            "uniform binding has low gini: {}",
            dist.gini
        );
        assert!(
            dist.max_occupancy < 0.1,
            "weak binding: {}",
            dist.max_occupancy
        );
    }

    #[test]
    fn affinity_distribution_narrow_strong() {
        let mut ic50s = vec![1000.0; 10];
        ic50s[0] = 0.1;
        let dist = analyze_affinity_distribution(1.0, &ic50s, 1.0, 0.01);
        assert!(
            dist.gini > 0.5,
            "single strong hit has high gini: {}",
            dist.gini
        );
        assert!(
            dist.max_occupancy > 0.9,
            "strong hit: {}",
            dist.max_occupancy
        );
    }

    #[test]
    fn low_affinity_selectivity_cancer_vs_normal() {
        let cancer_occupancies = vec![0.05; 20];
        let normal_occupancies = vec![0.02; 5];
        let sel = low_affinity_selectivity(&cancer_occupancies, &normal_occupancies);
        assert!(sel > 5.0, "cancer should be > 5x more targeted: {sel}");
    }

    #[test]
    fn colonization_resistance_many_weak_binders() {
        let species: Vec<Vec<f64>> = (0..5).map(|_| vec![0.15; 20]).collect();
        let cr = colonization_resistance(&species, 0.5);
        assert!(
            cr > 0.8,
            "5 species at 15% each should cover most sites: {cr}"
        );
    }

    #[test]
    fn colonization_resistance_single_strong_binder() {
        let species = vec![vec![0.6; 20]];
        let cr = colonization_resistance(&species, 0.5);
        assert!(
            (cr - 1.0).abs() < tolerances::DIVISION_GUARD,
            "single strong binder covers all: {cr}"
        );
    }

    #[test]
    fn disorder_adhesion_profile_reproducible() {
        let p1 = disorder_adhesion_profile(100, 10.0, 0.5, 42);
        let p2 = disorder_adhesion_profile(100, 10.0, 0.5, 42);
        assert_eq!(p1, p2, "same seed should produce same profile");
    }

    #[test]
    fn disorder_adhesion_profile_has_variation() {
        let profile = disorder_adhesion_profile(100, 10.0, 1.0, 42);
        let min = profile.iter().copied().fold(f64::INFINITY, f64::min);
        let max = profile.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max - min > 0.1,
            "disorder should create variation: [{min}, {max}]"
        );
    }

    #[test]
    fn cross_reactivity_matrix_dimensions() {
        let ic50s = vec![vec![10.0, 20.0, 30.0]; 4];
        let matrix = cross_reactivity_matrix(5.0, &ic50s, 1.0);
        assert_eq!(matrix.len(), 4);
        assert_eq!(matrix[0].len(), 3);
    }
}
