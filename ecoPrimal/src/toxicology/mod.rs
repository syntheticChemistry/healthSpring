// SPDX-License-Identifier: AGPL-3.0-or-later
//! Toxicology and cytotoxicity landscape modeling.
//!
//! Bridges the affinity landscape (what compounds bind) with the body's
//! capacity to handle that binding (clearance, repair, stress response).
//!
//! ## The Delocalization Hypothesis
//!
//! Traditional drug toxicity is **localized**: a strong binder saturates one
//! target or tissue, causing concentrated damage (hepatotoxicity, cardiotoxicity,
//! nephrotoxicity). The body's repair machinery at that site is overwhelmed.
//!
//! Weak, distributed binding creates **delocalized** toxicity: each tissue
//! bears a small fraction of the total burden. If the per-tissue burden stays
//! below the local repair capacity, the systemic exposure is tolerable even
//! though the total binding events are numerous.
//!
//! This maps directly to Anderson localization:
//! - **Strong binder** → localized wavefunction (IPR high) → organ-specific toxicity
//! - **Weak binder** → extended wavefunction (IPR low) → distributed, manageable load
//!
//! ## Clearance Regime
//!
//! Weak binders at low per-tissue concentrations stay in the **linear kinetics**
//! regime (C << Km), where clearance is first-order and predictable. Strong
//! binders at high local concentrations can **saturate** clearance
//! (Michaelis-Menten nonlinearity), causing unpredictable accumulation.
//!
//! ## References
//!
//! - Anderson PW (1958) *Phys Rev* — localization in disordered systems
//! - Rowland & Tozer — *Clinical Pharmacokinetics*, capacity-limited elimination
//! - Kärre K (1986) — NK cell integration of multiple weak signals
//! - Calabrese EJ, Baldwin LA (2003) *Annu Rev Pharmacol Toxicol* — hormesis

pub mod hormesis;
pub use hormesis::*;

#[cfg(test)]
mod wetspring_cross_validation;

use crate::tolerances;

/// Per-tissue toxicity profile: binding load vs repair capacity.
#[derive(Debug, Clone)]
pub struct TissueToxProfile {
    /// Human-readable tissue name.
    pub name: &'static str,
    /// Fractional occupancy of the compound at this tissue (0.0–1.0).
    pub occupancy: f64,
    /// Tissue sensitivity weight (higher = more vulnerable). Dimensionless.
    pub sensitivity: f64,
    /// Local repair capacity: fraction of occupancy the tissue can handle
    /// without adverse effect (0.0–1.0).
    pub repair_capacity: f64,
}

/// Systemic burden score.
///
/// Weighted sum of fractional occupancies across all tissues:
/// `SBS = Σ(occupancy_i × sensitivity_i)`
///
/// Strong binder: high SBS from one tissue. Weak distributed binder: SBS is
/// the sum of many small contributions.
#[must_use]
pub fn systemic_burden_score(tissues: &[TissueToxProfile]) -> f64 {
    tissues.iter().map(|t| t.occupancy * t.sensitivity).sum()
}

/// Tissue-excess burden: per-tissue burden that exceeds local repair capacity.
///
/// `excess_i = max(0, occupancy_i × sensitivity_i - repair_capacity_i)`
///
/// A tissue is stressed only when binding load exceeds its ability to cope.
/// Returns the vector of excess burdens and the total excess.
#[must_use]
pub fn tissue_excess_burden(tissues: &[TissueToxProfile]) -> (Vec<f64>, f64) {
    let excesses: Vec<f64> = tissues
        .iter()
        .map(|t| {
            t.occupancy
                .mul_add(t.sensitivity, -t.repair_capacity)
                .max(0.0)
        })
        .collect();
    let total = excesses.iter().sum();
    (excesses, total)
}

/// Toxicity IPR (Inverse Participation Ratio of the toxicity distribution).
///
/// Treats the weighted occupancy distribution across tissues as a "wavefunction"
/// and computes its IPR.
///
/// - High IPR → toxicity concentrated in few tissues (localized, dangerous)
/// - Low IPR → toxicity spread across many tissues (delocalized, manageable)
///
/// `IPR = Σ(p_i^4)` where `p_i = (occ_i × sens_i) / Σ(occ_j × sens_j)`
#[must_use]
pub fn toxicity_ipr(tissues: &[TissueToxProfile]) -> f64 {
    let weights: Vec<f64> = tissues
        .iter()
        .map(|t| t.occupancy * t.sensitivity)
        .collect();

    let total: f64 = weights.iter().sum();
    if total < tolerances::DIVISION_GUARD {
        return 0.0;
    }

    weights.iter().map(|&w| (w / total).powi(4)).sum()
}

/// Toxicity localization length: `xi = 1 / IPR`.
///
/// Number of tissues effectively sharing the toxic burden.
/// Higher xi → more tissues share the load → lower per-tissue damage.
#[must_use]
pub fn toxicity_localization_length(tissues: &[TissueToxProfile]) -> f64 {
    let ipr = toxicity_ipr(tissues);
    if ipr > tolerances::DIVISION_GUARD {
        1.0 / ipr
    } else {
        f64::INFINITY
    }
}

/// Delocalization advantage: ratio of localized to delocalized excess burden.
///
/// Compares a strong binder (same total binding concentrated in one tissue)
/// vs the distributed binding profile. Returns how many times worse the
/// localized scenario is.
///
/// If the distributed profile has zero excess, the advantage is infinite
/// (the body handles it entirely).
#[must_use]
pub fn delocalization_advantage(tissues: &[TissueToxProfile]) -> f64 {
    let total_binding: f64 = tissues.iter().map(|t| t.occupancy * t.sensitivity).sum();
    if total_binding < tolerances::DIVISION_GUARD {
        return 1.0;
    }

    let (_, distributed_excess) = tissue_excess_burden(tissues);

    let max_sensitivity = tissues
        .iter()
        .map(|t| t.sensitivity)
        .fold(0.0_f64, f64::max);
    let max_repair = tissues
        .iter()
        .map(|t| t.repair_capacity)
        .fold(0.0_f64, f64::max);

    let localized_excess = total_binding.mul_add(max_sensitivity, -max_repair).max(0.0);

    if distributed_excess < tolerances::DIVISION_GUARD {
        if localized_excess < tolerances::DIVISION_GUARD {
            return 1.0;
        }
        return f64::INFINITY;
    }

    localized_excess / distributed_excess
}

/// Clearance regime indicator.
///
/// Determines whether a compound at a given concentration is in the
/// linear (first-order) or nonlinear (capacity-limited) clearance regime.
///
/// `regime = C / Km`
///
/// - regime << 1: linear kinetics (safe, predictable clearance)
/// - regime ~ 1: transitional (dose-dependent behavior)
/// - regime >> 1: saturated (unpredictable accumulation, dangerous)
///
/// For weak binders at low tissue concentrations, we expect regime << 1
/// at every tissue site.
#[must_use]
pub fn clearance_regime(concentration: f64, km: f64) -> f64 {
    if km < tolerances::DIVISION_GUARD {
        return f64::INFINITY;
    }
    concentration / km
}

/// Fraction of elimination capacity utilized at a given concentration.
///
/// `utilization = C / (Km + C)` — the Michaelis-Menten saturation fraction.
///
/// - Near 0%: linear regime, plenty of clearance headroom
/// - Near 100%: saturated, no clearance headroom (accumulation risk)
#[must_use]
pub fn clearance_utilization(concentration: f64, km: f64) -> f64 {
    if concentration < 0.0 {
        return 0.0;
    }
    concentration / (km + concentration)
}

/// Systemic clearance safety margin.
///
/// For a multi-tissue binding profile, compute the maximum clearance
/// utilization across all tissues. If every tissue stays below a threshold
/// (e.g., 20%), clearance is predictably in the linear regime everywhere.
///
/// Returns `(max_utilization, all_below_threshold)`.
#[must_use]
pub fn clearance_safety_margin(
    tissue_concentrations: &[f64],
    km: f64,
    threshold: f64,
) -> (f64, bool) {
    let max_util = tissue_concentrations
        .iter()
        .map(|&c| clearance_utilization(c, km))
        .fold(0.0_f64, f64::max);
    (max_util, max_util < threshold)
}

/// Anderson toxicity landscape.
///
/// Models tissue sensitivities as a disordered lattice. The binding profile
/// of a compound determines whether the toxic response is localized
/// (concentrated in sensitive tissues) or delocalized (spread uniformly).
///
/// Given an array of tissue sensitivities (the disorder vector) and a
/// compound's affinity for each tissue (as IC50 values), computes:
/// - The fractional occupancy profile
/// - The toxicity IPR
/// - The effective localization length
/// - Whether the compound is in the delocalized (manageable) regime
#[derive(Debug, Clone)]
pub struct ToxicityLandscape {
    /// Number of tissue compartments.
    pub n_tissues: usize,
    /// Total systemic burden score.
    pub systemic_burden: f64,
    /// Total excess burden (above repair capacity).
    pub excess_burden: f64,
    /// Per-tissue excess burden.
    pub tissue_excesses: Vec<f64>,
    /// Toxicity IPR — localization of toxic load.
    pub tox_ipr: f64,
    /// Effective number of tissues sharing the burden.
    pub localization_length: f64,
    /// Maximum clearance utilization across tissues.
    pub max_clearance_utilization: f64,
    /// Whether all tissues are in the linear clearance regime.
    pub clearance_linear: bool,
    /// Delocalization advantage vs hypothetical concentrated exposure.
    pub delocalization_advantage: f64,
}

/// Compute the full toxicity landscape for a compound.
///
/// # Panics
///
/// Panics if `tissue_ic50s`, `tissue_sensitivities`, and `tissue_repair_capacities`
/// have different lengths.
///
/// # Arguments
///
/// - `concentration` — systemic concentration
/// - `tissue_ic50s` — IC50 at each tissue (higher = weaker binding)
/// - `tissue_sensitivities` — vulnerability weight per tissue
/// - `tissue_repair_capacities` — local repair capacity per tissue
/// - `hill_n` — Hill coefficient for binding
/// - `km` — Michaelis-Menten Km for clearance
/// - `clearance_threshold` — max utilization for linear regime
#[must_use]
pub fn compute_toxicity_landscape(
    concentration: f64,
    tissue_ic50s: &[f64],
    tissue_sensitivities: &[f64],
    tissue_repair_capacities: &[f64],
    hill_n: f64,
    km: f64,
    clearance_threshold: f64,
) -> ToxicityLandscape {
    let n = tissue_ic50s.len();
    assert!(
        tissue_sensitivities.len() == n && tissue_repair_capacities.len() == n,
        "tissue arrays must have equal length"
    );

    let names: Vec<&str> = (0..n).map(|_| "tissue").collect();

    let tissues: Vec<TissueToxProfile> = (0..n)
        .map(|i| {
            let occ =
                crate::discovery::fractional_occupancy(concentration, tissue_ic50s[i], hill_n);
            TissueToxProfile {
                name: names[i],
                occupancy: occ,
                sensitivity: tissue_sensitivities[i],
                repair_capacity: tissue_repair_capacities[i],
            }
        })
        .collect();

    let systemic_burden = systemic_burden_score(&tissues);
    let (tissue_excesses, excess_burden) = tissue_excess_burden(&tissues);
    let tox_ipr = toxicity_ipr(&tissues);
    let loc_len = toxicity_localization_length(&tissues);
    let deloc_adv = delocalization_advantage(&tissues);

    let tissue_concs: Vec<f64> = tissues
        .iter()
        .map(|t| t.occupancy * concentration)
        .collect();
    let (max_util, clearance_ok) = clearance_safety_margin(&tissue_concs, km, clearance_threshold);

    ToxicityLandscape {
        n_tissues: n,
        systemic_burden,
        excess_burden,
        tissue_excesses,
        tox_ipr,
        localization_length: loc_len,
        max_clearance_utilization: max_util,
        clearance_linear: clearance_ok,
        delocalization_advantage: deloc_adv,
    }
}

/// Hormesis check: does low-level distributed exposure fall in the hormetic zone?
///
/// Hormesis (Calabrese & Baldwin 2003): low doses can trigger adaptive stress
/// responses that are net beneficial. The hormetic zone is typically
/// 1/10 to 1/100 of the toxic threshold.
///
/// Returns `true` if the burden is in the hormetic range (between
/// `toxic_threshold / hormetic_high` and `toxic_threshold / hormetic_low`).
#[must_use]
pub fn in_hormetic_zone(
    burden: f64,
    toxic_threshold: f64,
    hormetic_low: f64,
    hormetic_high: f64,
) -> bool {
    if toxic_threshold < tolerances::DIVISION_GUARD {
        return false;
    }
    let low = toxic_threshold / hormetic_high;
    let high = toxic_threshold / hormetic_low;
    burden >= low && burden <= high
}

/// Disorder-modulated tissue sensitivity generator.
///
/// Generates a tissue sensitivity landscape with Anderson-type disorder.
/// Each tissue's sensitivity is sampled from a distribution parameterized
/// by a base sensitivity and a disorder width W.
///
/// `sensitivity_i = base × exp(W × z_i)` where `z_i ~ N(0,1)`
///
/// High W → wide spread of sensitivities (some tissues very vulnerable,
/// others very robust). Low W → uniform sensitivity.
#[must_use]
pub fn disorder_tissue_sensitivities(
    n_tissues: usize,
    base_sensitivity: f64,
    disorder_w: f64,
    seed: u64,
) -> Vec<f64> {
    use crate::rng::{lcg_step, normal_sample};

    let mut rng = seed;
    let mut sensitivities = Vec::with_capacity(n_tissues);
    for _ in 0..n_tissues {
        let (z, next) = normal_sample(rng);
        rng = next;
        sensitivities.push(base_sensitivity * (disorder_w * z).exp());
    }
    let _ = lcg_step(rng);
    sensitivities
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tissues(occupancies: &[f64], sensitivities: &[f64]) -> Vec<TissueToxProfile> {
        occupancies
            .iter()
            .zip(sensitivities)
            .map(|(&occ, &sens)| TissueToxProfile {
                name: "test",
                occupancy: occ,
                sensitivity: sens,
                repair_capacity: 0.05,
            })
            .collect()
    }

    #[test]
    fn systemic_burden_zero_for_no_binding() {
        let tissues = make_tissues(&[0.0; 5], &[1.0; 5]);
        assert!(systemic_burden_score(&tissues).abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn systemic_burden_weighted() {
        let tissues = make_tissues(&[0.1, 0.2], &[1.0, 2.0]);
        let sbs = systemic_burden_score(&tissues);
        let expected = 0.1f64.mul_add(1.0, 0.2 * 2.0);
        assert!(
            (sbs - expected).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "SBS: {sbs} vs {expected}"
        );
    }

    #[test]
    fn excess_burden_below_repair_is_zero() {
        let tissues = vec![TissueToxProfile {
            name: "liver",
            occupancy: 0.01,
            sensitivity: 1.0,
            repair_capacity: 0.1,
        }];
        let (_, total) = tissue_excess_burden(&tissues);
        assert!(total.abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn excess_burden_above_repair() {
        let tissues = vec![TissueToxProfile {
            name: "liver",
            occupancy: 0.5,
            sensitivity: 1.0,
            repair_capacity: 0.1,
        }];
        let (_, total) = tissue_excess_burden(&tissues);
        assert!(
            (total - 0.4).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "excess: {total}"
        );
    }

    #[test]
    fn ipr_concentrated_is_high() {
        let tissues = make_tissues(&[0.9, 0.0, 0.0, 0.0], &[1.0; 4]);
        let ipr = toxicity_ipr(&tissues);
        assert!(ipr > 0.9, "concentrated binding → high IPR: {ipr}");
    }

    #[test]
    fn ipr_distributed_is_low() {
        let tissues = make_tissues(&[0.1; 10], &[1.0; 10]);
        let ipr = toxicity_ipr(&tissues);
        assert!(ipr < 0.15, "distributed binding → low IPR: {ipr}");
    }

    #[test]
    fn localization_length_distributed() {
        let tissues = make_tissues(&[0.1; 10], &[1.0; 10]);
        let xi = toxicity_localization_length(&tissues);
        assert!(xi > 8.0, "10 equal tissues → xi ≈ 10: {xi}");
    }

    #[test]
    fn localization_length_concentrated() {
        let tissues = make_tissues(&[1.0, 0.0, 0.0, 0.0], &[1.0; 4]);
        let xi = toxicity_localization_length(&tissues);
        assert!(xi < 1.5, "concentrated → xi ≈ 1: {xi}");
    }

    #[test]
    fn delocalization_advantage_distributed_better() {
        let tissues: Vec<TissueToxProfile> = (0..10)
            .map(|_| TissueToxProfile {
                name: "tissue",
                occupancy: 0.03,
                sensitivity: 1.0,
                repair_capacity: 0.05,
            })
            .collect();
        let adv = delocalization_advantage(&tissues);
        assert!(adv > 1.0, "distributed should have advantage: {adv}");
    }

    #[test]
    fn clearance_utilization_low_concentration() {
        let util = clearance_utilization(0.1, 10.0);
        assert!(util < 0.02, "C << Km → low utilization: {util}");
    }

    #[test]
    fn clearance_utilization_high_concentration() {
        let util = clearance_utilization(100.0, 10.0);
        assert!(util > 0.9, "C >> Km → high utilization: {util}");
    }

    #[test]
    fn clearance_safety_margin_weak_binders() {
        let concs = vec![0.01; 10];
        let (max, safe) = clearance_safety_margin(&concs, 10.0, 0.2);
        assert!(safe, "all low conc → safe: max={max}");
    }

    #[test]
    fn clearance_regime_linear_at_low_c() {
        let regime = clearance_regime(0.1, 10.0);
        assert!(regime < 0.02, "C/Km << 1: {regime}");
    }

    #[test]
    fn hormetic_zone_inside() {
        assert!(in_hormetic_zone(0.05, 1.0, 10.0, 100.0));
    }

    #[test]
    fn hormetic_zone_outside_too_high() {
        assert!(!in_hormetic_zone(0.5, 1.0, 10.0, 100.0));
    }

    #[test]
    fn disorder_tissue_sensitivities_reproducible() {
        let s1 = disorder_tissue_sensitivities(20, 1.0, 0.5, 42);
        let s2 = disorder_tissue_sensitivities(20, 1.0, 0.5, 42);
        assert_eq!(s1, s2);
    }

    #[test]
    fn disorder_tissue_sensitivities_has_variation() {
        let s = disorder_tissue_sensitivities(50, 1.0, 1.0, 42);
        let min = s.iter().copied().fold(f64::INFINITY, f64::min);
        let max = s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max / min > 2.0,
            "disorder should spread sensitivities: [{min}, {max}]"
        );
    }

    #[test]
    fn full_landscape_weak_distributed() {
        let landscape =
            compute_toxicity_landscape(1.0, &[50.0; 8], &[1.0; 8], &[0.05; 8], 1.0, 10.0, 0.20);
        assert!(
            landscape.localization_length > 6.0,
            "distributed: xi={}",
            landscape.localization_length
        );
        assert!(
            landscape.clearance_linear,
            "weak binding → linear clearance"
        );
    }

    #[test]
    fn full_landscape_strong_localized() {
        let mut ic50s = vec![1000.0; 8];
        ic50s[0] = 0.5;
        let landscape =
            compute_toxicity_landscape(1.0, &ic50s, &[1.0; 8], &[0.05; 8], 1.0, 10.0, 0.20);
        assert!(
            landscape.localization_length < 2.0,
            "localized: xi={}",
            landscape.localization_length
        );
        assert!(
            landscape.excess_burden > 0.0,
            "strong binder exceeds repair capacity"
        );
    }
}
