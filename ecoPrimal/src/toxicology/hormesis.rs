// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hormesis and biphasic dose-response models.
//!
//! Hormesis is the biphasic response where low-dose stress improves fitness
//! and high-dose stress destroys it. The same shape appears in:
//! - Pesticide effects on insects (weak pesticide → more grasshoppers)
//! - Caloric restriction (mild hunger → longevity)
//! - Mithridatism (self-dosing poison → tolerance, at a metabolic cost)
//! - Hygiene hypothesis (microbial exposure → immune calibration)
//! - Plant growth under mild herbicide exposure
//!
//! Mathematically: `R(D) = baseline × stimulation(D) × survival(D)`
//! where stimulation saturates (Hill-like) and survival declines (Hill-like).
//!
//! ## References
//!
//! - Calabrese EJ, Baldwin LA (2003) *Annu Rev Pharmacol Toxicol* 43:175
//! - Mattson MP (2008) *Ageing Res Rev* 7:1 — hormesis and disease resistance
//! - Strachan DP (1989) *BMJ* — hygiene hypothesis
//! - Fontana L, Partridge L (2015) *Cell* 161:106 — CR and longevity

use crate::tolerances;

use super::{TissueToxProfile, disorder_tissue_sensitivities, toxicity_ipr};

/// Biphasic (hormetic) dose-response.
///
/// Models the two competing forces:
/// - **Stimulation**: adaptive stress response, saturating at `s_max`.
///   `S(D) = s_max × D / (k_stim + D)`
/// - **Inhibition**: toxicity, Hill-shaped.
///   `I(D) = D^n / (ic50^n + D^n)`
///
/// Net fitness: `R(D) = baseline × (1 + S(D)) × (1 - I(D))`
///
/// At D=0: R = baseline (no stress)
/// At low D: stimulation dominates → R > baseline (hormesis)
/// At high D: inhibition dominates → R → 0 (toxicity)
///
/// There's a peak somewhere in between — the hormetic optimum.
#[must_use]
pub fn biphasic_dose_response(
    dose: f64,
    baseline: f64,
    s_max: f64,
    k_stim: f64,
    ic50: f64,
    hill_n: f64,
) -> f64 {
    if dose <= 0.0 {
        return baseline;
    }
    let stimulation = s_max * dose / (k_stim + dose);
    let inhibition = dose.powf(hill_n) / (ic50.powf(hill_n) + dose.powf(hill_n));
    baseline * (1.0 + stimulation) * (1.0 - inhibition)
}

/// Find the hormetic optimum — the dose that maximizes fitness.
///
/// Scans a dose range and returns `(optimal_dose, peak_fitness)`.
/// Uses grid search; precision depends on `n_steps`.
#[must_use]
pub fn hormetic_optimum(
    baseline: f64,
    s_max: f64,
    k_stim: f64,
    ic50: f64,
    hill_n: f64,
    dose_max: f64,
    n_steps: usize,
) -> (f64, f64) {
    let mut best_dose = 0.0;
    let mut best_fitness = baseline;

    for i in 0..=n_steps {
        #[expect(clippy::cast_precision_loss, reason = "step count small")]
        let dose = dose_max * (i as f64) / (n_steps as f64);
        let fitness = biphasic_dose_response(dose, baseline, s_max, k_stim, ic50, hill_n);
        if fitness > best_fitness {
            best_fitness = fitness;
            best_dose = dose;
        }
    }
    (best_dose, best_fitness)
}

/// Mithridatism: adaptive tolerance through repeated low-dose exposure.
///
/// Each exposure shifts the effective IC50 upward (the organism becomes
/// more tolerant). But tolerance has a metabolic cost — maintaining the
/// detoxification machinery reduces baseline fitness.
///
/// `IC50_adapted = IC50_naive × (1 + adaptation × n / (k_adapt + n))`
/// `baseline_adapted = baseline × (1 - cost × n / (k_cost + n))`
///
/// where `n` is number of prior exposures.
///
/// The adaptation itself follows a Hill-like saturation: early exposures
/// build tolerance quickly, later exposures have diminishing returns.
///
/// Historical reference: Mithridates VI of Pontus (c. 120-63 BCE)
/// reportedly self-dosed with poisons to build immunity.
#[derive(Debug, Clone)]
pub struct MithridatismParams {
    /// Naive IC50 (no prior exposure).
    pub ic50_naive: f64,
    /// Maximum fold-increase in IC50 from full adaptation.
    pub max_adaptation: f64,
    /// Half-saturation: exposures needed for 50% of max adaptation.
    pub k_adapt: f64,
    /// Maximum fitness cost from full adaptation (fraction of baseline lost).
    pub max_cost: f64,
    /// Half-saturation for cost accumulation.
    pub k_cost: f64,
}

/// Compute adapted IC50 and fitness cost after `n` low-dose exposures.
///
/// Returns `(ic50_adapted, cost_fraction)`.
#[must_use]
pub fn mithridatism_adaptation(params: &MithridatismParams, n_exposures: f64) -> (f64, f64) {
    let adaptation = params.max_adaptation * n_exposures / (params.k_adapt + n_exposures);
    let cost = params.max_cost * n_exposures / (params.k_cost + n_exposures);
    let ic50_adapted = params.ic50_naive * (1.0 + adaptation);
    (ic50_adapted, cost)
}

/// Net fitness after mithridatism: adapted biphasic response minus cost.
///
/// The adapted organism has higher IC50 (more tolerant) but lower baseline
/// (metabolic cost of tolerance). The question: at what dose is the
/// adapted organism better off than the naive one?
#[must_use]
pub fn mithridatism_fitness(
    dose: f64,
    baseline: f64,
    s_max: f64,
    k_stim: f64,
    hill_n: f64,
    params: &MithridatismParams,
    n_exposures: f64,
) -> f64 {
    let (ic50_adapted, cost) = mithridatism_adaptation(params, n_exposures);
    let adapted_baseline = baseline * (1.0 - cost);
    biphasic_dose_response(dose, adapted_baseline, s_max, k_stim, ic50_adapted, hill_n)
}

/// Hygiene threshold: minimum microbial exposure for immune calibration.
///
/// The immune system needs calibration through exposure. Below the
/// hygiene threshold, the immune system is uncalibrated and over-reactive
/// (allergies, autoimmune). Above it, the system is properly trained.
/// Far above it, the system is overwhelmed (infection).
///
/// This is itself a biphasic (hormetic) response applied to immune
/// competence rather than organism fitness.
///
/// Returns the immune competence score (0.0 = uncalibrated, 1.0 = optimal).
///
/// References:
/// - Strachan DP (1989) *BMJ* — hygiene hypothesis
/// - Du Toit G et al. (2015) *NEJM* — LEAP study (peanut allergy)
/// - Rook GA (2010) *Clin Exp Immunol* — "old friends" hypothesis
#[must_use]
pub fn immune_calibration(
    microbial_exposure: f64,
    baseline_competence: f64,
    calibration_gain: f64,
    k_calibration: f64,
    overwhelm_ic50: f64,
    hill_n: f64,
) -> f64 {
    biphasic_dose_response(
        microbial_exposure,
        baseline_competence,
        calibration_gain,
        k_calibration,
        overwhelm_ic50,
        hill_n,
    )
}

/// Caloric restriction fitness model.
///
/// Mild caloric deficit triggers autophagy, sirtuin activation, and
/// mitochondrial efficiency improvements. Severe deficit causes
/// malnutrition. The dose is "restriction fraction" (0.0 = ad libitum,
/// 1.0 = starvation).
///
/// References:
/// - Fontana L, Partridge L (2015) *Cell* 161:106 — CR and longevity
/// - Mattison JA et al. (2017) *Nat Commun* — rhesus monkey CR study
#[must_use]
pub fn caloric_restriction_fitness(
    restriction_fraction: f64,
    baseline_lifespan: f64,
    longevity_gain: f64,
    k_autophagy: f64,
    starvation_ic50: f64,
    hill_n: f64,
) -> f64 {
    biphasic_dose_response(
        restriction_fraction,
        baseline_lifespan,
        longevity_gain,
        k_autophagy,
        starvation_ic50,
        hill_n,
    )
}

/// Ecological hormesis: effect of pesticide on non-target species.
///
/// A weak pesticide might increase grasshopper populations through:
/// 1. Suppression of competitors/predators (indirect ecological effect)
/// 2. Direct hormetic stimulation of reproduction under mild stress
/// 3. Release from density-dependent constraints
///
/// This models the direct hormetic channel. The indirect (ecological)
/// channel would require the full groundSpring/airSpring ecosystem model.
///
/// The dose is pesticide concentration in the environment.
#[must_use]
pub fn ecological_hormesis(
    pesticide_concentration: f64,
    baseline_population: f64,
    stress_response_gain: f64,
    k_stress: f64,
    lethal_ic50: f64,
    hill_n: f64,
) -> f64 {
    biphasic_dose_response(
        pesticide_concentration,
        baseline_population,
        stress_response_gain,
        k_stress,
        lethal_ic50,
        hill_n,
    )
}

/// Parameters for the Anderson hormesis-localization model.
#[derive(Debug, Clone)]
pub struct HormesisLocalizationParams {
    /// Biphasic baseline fitness.
    pub baseline: f64,
    /// Maximum stimulation gain (fraction above baseline).
    pub s_max: f64,
    /// Half-saturation concentration for stimulation.
    pub k_stim: f64,
    /// IC50 for inhibition.
    pub ic50: f64,
    /// Hill coefficient.
    pub hill_n: f64,
    /// Number of repair/stress pathways.
    pub n_pathways: usize,
    /// Anderson disorder width for pathway sensitivities.
    pub disorder_w: f64,
    /// RNG seed for reproducible disorder.
    pub seed: u64,
}

/// Anderson interpretation of hormesis.
///
/// At the hormetic optimum, the stress response is **delocalized** —
/// broadly distributed across cellular repair pathways (autophagy,
/// heat shock proteins, antioxidants, DNA repair). This widespread
/// activation is why mild stress improves overall fitness.
///
/// Above the hormetic threshold, damage **localizes** — concentrated
/// at vulnerable sites (mitochondria, membrane, DNA) faster than
/// repair can distribute. The transition from hormetic to toxic is
/// an Anderson localization transition.
///
/// Returns `(stress_ipr, interpretation)`:
/// - IPR < 0.15 → delocalized stress (hormetic, beneficial)
/// - IPR > 0.50 → localized damage (toxic, harmful)
#[must_use]
pub fn hormesis_localization(
    dose: f64,
    params: &HormesisLocalizationParams,
) -> (f64, &'static str) {
    let HormesisLocalizationParams {
        baseline,
        s_max,
        k_stim,
        ic50,
        hill_n,
        n_pathways,
        disorder_w,
        seed,
    } = *params;
    let sensitivities = disorder_tissue_sensitivities(n_pathways, 1.0, disorder_w, seed);

    let profiles: Vec<TissueToxProfile> = sensitivities
        .iter()
        .map(|&s| {
            let pathway_dose = dose * s;
            let occupancy =
                pathway_dose.powf(hill_n) / (ic50.powf(hill_n) + pathway_dose.powf(hill_n));
            TissueToxProfile {
                name: "pathway",
                occupancy,
                sensitivity: s,
                repair_capacity: 0.1,
            }
        })
        .collect();

    let ipr = toxicity_ipr(&profiles);
    let fitness = biphasic_dose_response(dose, baseline, s_max, k_stim, ic50, hill_n);

    let interpretation = if fitness > baseline && ipr < tolerances::TOX_IPR_DELOCALIZED {
        "hormetic: delocalized stress, fitness above baseline"
    } else if fitness > baseline {
        "stimulated but localizing: approaching transition"
    } else if ipr > tolerances::TOX_IPR_LOCALIZED {
        "toxic: localized damage, fitness below baseline"
    } else {
        "declining: damage spreading, fitness below baseline"
    };

    (ipr, interpretation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn biphasic_at_zero_returns_baseline() {
        let r = biphasic_dose_response(0.0, 100.0, 0.5, 1.0, 10.0, 2.0);
        assert!(
            (r - 100.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "at dose=0, fitness = baseline: {r}"
        );
    }

    #[test]
    fn biphasic_low_dose_exceeds_baseline() {
        let r = biphasic_dose_response(0.5, 100.0, 0.5, 1.0, 50.0, 2.0);
        assert!(r > 100.0, "low dose should stimulate: {r}");
    }

    #[test]
    fn biphasic_high_dose_below_baseline() {
        let r = biphasic_dose_response(100.0, 100.0, 0.5, 1.0, 10.0, 2.0);
        assert!(r < 100.0, "high dose should inhibit: {r}");
    }

    #[test]
    fn biphasic_very_high_dose_near_zero() {
        let r = biphasic_dose_response(1000.0, 100.0, 0.5, 1.0, 10.0, 2.0);
        assert!(r < 1.0, "very high dose → near zero: {r}");
    }

    #[test]
    fn hormetic_optimum_exists() {
        let (opt_dose, peak) = hormetic_optimum(100.0, 0.5, 1.0, 50.0, 2.0, 100.0, 10000);
        assert!(opt_dose > 0.0, "optimum dose > 0: {opt_dose}");
        assert!(peak > 100.0, "peak fitness > baseline: {peak}");
        assert!(opt_dose < 50.0, "optimum dose < IC50: {opt_dose}");
    }

    #[test]
    fn mithridatism_increases_ic50() {
        let params = MithridatismParams {
            ic50_naive: 10.0,
            max_adaptation: 5.0,
            k_adapt: 10.0,
            max_cost: 0.15,
            k_cost: 20.0,
        };
        let (ic50_naive, _) = mithridatism_adaptation(&params, 0.0);
        let (ic50_adapted, _) = mithridatism_adaptation(&params, 20.0);
        assert!(
            (ic50_naive - 10.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "no exposure → naive IC50: {ic50_naive}"
        );
        assert!(
            ic50_adapted > ic50_naive,
            "adaptation increases IC50: {ic50_adapted} > {ic50_naive}"
        );
    }

    #[test]
    fn mithridatism_has_cost() {
        let params = MithridatismParams {
            ic50_naive: 10.0,
            max_adaptation: 5.0,
            k_adapt: 10.0,
            max_cost: 0.15,
            k_cost: 20.0,
        };
        let (_, cost_naive) = mithridatism_adaptation(&params, 0.0);
        let (_, cost_adapted) = mithridatism_adaptation(&params, 50.0);
        assert!(
            cost_naive.abs() < tolerances::TEST_ASSERTION_TIGHT,
            "no cost at 0: {cost_naive}"
        );
        assert!(
            cost_adapted > 0.05,
            "significant cost at 50 exposures: {cost_adapted}"
        );
    }

    #[test]
    fn mithridatism_adapted_survives_higher_dose() {
        let params = MithridatismParams {
            ic50_naive: 10.0,
            max_adaptation: 5.0,
            k_adapt: 10.0,
            max_cost: 0.10,
            k_cost: 20.0,
        };
        let dose = 15.0;
        let naive_fitness = biphasic_dose_response(dose, 100.0, 0.3, 1.0, 10.0, 2.0);
        let adapted_fitness = mithridatism_fitness(dose, 100.0, 0.3, 1.0, 2.0, &params, 30.0);
        assert!(
            adapted_fitness > naive_fitness,
            "adapted survives better at high dose: {adapted_fitness} > {naive_fitness}"
        );
    }

    #[test]
    fn immune_calibration_biphasic() {
        let low = immune_calibration(0.01, 0.3, 2.0, 0.5, 100.0, 2.0);
        let mid = immune_calibration(5.0, 0.3, 2.0, 0.5, 100.0, 2.0);
        let high = immune_calibration(500.0, 0.3, 2.0, 0.5, 100.0, 2.0);
        assert!(mid > low, "moderate exposure > minimal: {mid} > {low}");
        assert!(
            mid > high,
            "moderate exposure > overwhelming: {mid} > {high}"
        );
    }

    #[test]
    fn caloric_restriction_biphasic() {
        let ad_lib = caloric_restriction_fitness(0.0, 80.0, 0.3, 0.15, 0.7, 3.0);
        let mild_cr = caloric_restriction_fitness(0.2, 80.0, 0.3, 0.15, 0.7, 3.0);
        let severe = caloric_restriction_fitness(0.9, 80.0, 0.3, 0.15, 0.7, 3.0);
        assert!(
            (ad_lib - 80.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "no restriction → baseline: {ad_lib}"
        );
        assert!(mild_cr > 80.0, "mild CR extends lifespan: {mild_cr}");
        assert!(severe < 80.0, "severe restriction is harmful: {severe}");
    }

    #[test]
    fn ecological_hormesis_grasshopper() {
        let no_pesticide = ecological_hormesis(0.0, 1000.0, 0.4, 0.5, 20.0, 2.0);
        let weak_pesticide = ecological_hormesis(1.0, 1000.0, 0.4, 0.5, 20.0, 2.0);
        let strong_pesticide = ecological_hormesis(50.0, 1000.0, 0.4, 0.5, 20.0, 2.0);
        assert!(
            (no_pesticide - 1000.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "no pesticide → baseline: {no_pesticide}"
        );
        assert!(
            weak_pesticide > 1000.0,
            "weak pesticide → MORE grasshoppers: {weak_pesticide}"
        );
        assert!(
            strong_pesticide < 1000.0,
            "strong pesticide → fewer grasshoppers: {strong_pesticide}"
        );
    }

    #[test]
    fn hormesis_localization_low_dose_delocalized() {
        let params = HormesisLocalizationParams {
            baseline: 100.0,
            s_max: 0.5,
            k_stim: 1.0,
            ic50: 50.0,
            hill_n: 2.0,
            n_pathways: 10,
            disorder_w: 0.5,
            seed: 42,
        };
        let (ipr, interp) = hormesis_localization(0.5, &params);
        assert!(
            ipr < tolerances::TOX_IPR_DELOCALIZED,
            "low dose → delocalized: IPR={ipr}"
        );
        assert!(interp.contains("hormetic"), "should be hormetic: {interp}");
    }

    #[test]
    fn hormesis_localization_high_dose_localized() {
        let params = HormesisLocalizationParams {
            baseline: 100.0,
            s_max: 0.5,
            k_stim: 1.0,
            ic50: 10.0,
            hill_n: 2.0,
            n_pathways: 10,
            disorder_w: 0.8,
            seed: 42,
        };
        let (ipr, interp) = hormesis_localization(80.0, &params);
        assert!(
            interp.contains("toxic") || interp.contains("declining"),
            "high dose should be toxic or declining: {interp}, IPR={ipr}"
        );
    }
}
