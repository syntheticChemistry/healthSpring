// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cellular stress response: pathways, damage, and mechanistic fitness.
//!
//! The MISSING PIECE that explains the biphasic curve mechanistically.
//! Multiple repair pathways compete with damage accumulation.

/// A single cellular stress-response pathway.
///
/// Each pathway activates in response to stress, following a Hill-like
/// saturation curve. The pathway provides a protective benefit that
/// saturates — you can't get more protection than the pathway's maximum.
///
/// Examples: heat shock proteins (HSP70/90), autophagy (mTOR/AMPK),
/// antioxidant defense (SOD, catalase), DNA repair (p53, BRCA).
#[derive(Debug, Clone)]
pub struct StressPathway {
    /// Human-readable name.
    pub name: &'static str,
    /// Maximum protective benefit (fraction above baseline, e.g., 0.15 = 15%).
    pub max_benefit: f64,
    /// Half-activation dose — the dose at which pathway provides 50% of max benefit.
    pub k_half: f64,
    /// Activation speed (Hill coefficient — higher = more switch-like).
    pub hill_n: f64,
}

/// Activation of a stress pathway at a given dose.
///
/// `benefit(D) = max_benefit × D^n / (k_half^n + D^n)`
///
/// Saturates at `max_benefit` for high doses.
#[must_use]
pub fn pathway_activation(pathway: &StressPathway, dose: f64) -> f64 {
    if dose <= 0.0 {
        return 0.0;
    }
    let d_n = dose.powf(pathway.hill_n);
    let k_n = pathway.k_half.powf(pathway.hill_n);
    pathway.max_benefit * d_n / (k_n + d_n)
}

/// Damage accumulation — the unbounded harm channel.
///
/// Unlike repair pathways (which saturate), damage accumulates
/// without bound. This is what eventually overwhelms the repair.
///
/// `damage(D) = D^n / (ic50^n + D^n)`
///
/// This IS the standard Hill inhibition curve.
#[must_use]
pub fn damage_accumulation(dose: f64, ic50: f64, hill_n: f64) -> f64 {
    if dose <= 0.0 || ic50 <= 0.0 {
        return 0.0;
    }
    let d_n = dose.powf(hill_n);
    let ic50_n = ic50.powf(hill_n);
    d_n / (ic50_n + d_n)
}

/// Standard stress-response pathway set for eukaryotic cells.
///
/// These four pathways represent the major categories of cellular defense.
/// Their half-activation doses are ordered: HSP (fast, broad) activates
/// first, autophagy (slow, deep) activates last.
#[must_use]
pub fn standard_eukaryotic_pathways() -> Vec<StressPathway> {
    vec![
        StressPathway {
            name: "HSP (heat shock proteins)",
            max_benefit: 0.12,
            k_half: 0.5,
            hill_n: 1.5,
        },
        StressPathway {
            name: "antioxidant (SOD/catalase)",
            max_benefit: 0.10,
            k_half: 1.0,
            hill_n: 2.0,
        },
        StressPathway {
            name: "DNA repair (p53/BRCA)",
            max_benefit: 0.08,
            k_half: 2.0,
            hill_n: 2.0,
        },
        StressPathway {
            name: "autophagy (mTOR/AMPK)",
            max_benefit: 0.15,
            k_half: 3.0,
            hill_n: 1.0,
        },
    ]
}

/// Mechanistic cellular fitness from stress pathways + damage.
///
/// `fitness = baseline × product(1 + benefit_i) × (1 - damage)`
///
/// At low doses: benefits dominate (hormesis)
/// At high doses: damage dominates (toxicity)
///
/// This DERIVES the biphasic curve from underlying biology rather
/// than fitting it phenomenologically.
#[must_use]
pub fn mechanistic_cell_fitness(
    dose: f64,
    baseline: f64,
    pathways: &[StressPathway],
    damage_ic50: f64,
    damage_hill_n: f64,
) -> f64 {
    let total_benefit: f64 = pathways
        .iter()
        .map(|p| 1.0 + pathway_activation(p, dose))
        .product();
    let damage = damage_accumulation(dose, damage_ic50, damage_hill_n);
    baseline * total_benefit * (1.0 - damage)
}

/// Cellular fitness with per-pathway detail.
///
/// Returns `(total_fitness, pathway_activations, damage_fraction)`.
#[must_use]
pub fn mechanistic_cell_fitness_detailed(
    dose: f64,
    baseline: f64,
    pathways: &[StressPathway],
    damage_ic50: f64,
    damage_hill_n: f64,
) -> (f64, Vec<f64>, f64) {
    let activations: Vec<f64> = pathways
        .iter()
        .map(|p| pathway_activation(p, dose))
        .collect();
    let damage = damage_accumulation(dose, damage_ic50, damage_hill_n);
    let benefit: f64 = activations.iter().map(|&a| 1.0 + a).product();
    (baseline * benefit * (1.0 - damage), activations, damage)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tolerances;

    fn test_pathways() -> Vec<StressPathway> {
        standard_eukaryotic_pathways()
    }

    #[test]
    fn pathway_activation_at_zero_is_zero() {
        let p = &test_pathways()[0];
        assert!(pathway_activation(p, 0.0).abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn pathway_activation_at_k_half_is_half_max() {
        let p = &test_pathways()[0];
        let a = pathway_activation(p, p.k_half);
        let expected = p.max_benefit * 0.5_f64.powf(p.hill_n)
            / (0.5_f64.powf(p.hill_n) + 0.5_f64.powf(p.hill_n));
        assert!(
            (a - p.max_benefit / 2.0).abs() < 0.02,
            "at k_half, activation ≈ max/2: {a} vs {}",
            p.max_benefit / 2.0
        );
        let _ = expected;
    }

    #[test]
    fn pathway_activation_saturates() {
        let p = &test_pathways()[0];
        let a = pathway_activation(p, p.k_half * 1000.0);
        assert!(
            (a - p.max_benefit).abs() < 0.001,
            "high dose → max benefit: {a} vs {}",
            p.max_benefit
        );
    }

    #[test]
    fn damage_accumulation_at_ic50_is_half() {
        let d = damage_accumulation(10.0, 10.0, 2.0);
        assert!(
            (d - 0.5).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "at IC50, damage = 0.5: {d}"
        );
    }

    #[test]
    fn mechanistic_fitness_at_zero_is_baseline() {
        let f = mechanistic_cell_fitness(0.0, 100.0, &test_pathways(), 50.0, 2.0);
        assert!(
            (f - 100.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "zero dose → baseline: {f}"
        );
    }

    #[test]
    fn mechanistic_fitness_low_dose_hormetic() {
        let f = mechanistic_cell_fitness(1.0, 100.0, &test_pathways(), 50.0, 2.0);
        assert!(f > 100.0, "low dose is hormetic: {f}");
    }

    #[test]
    fn mechanistic_fitness_high_dose_toxic() {
        let f = mechanistic_cell_fitness(100.0, 100.0, &test_pathways(), 50.0, 2.0);
        assert!(f < 100.0, "high dose is toxic: {f}");
    }

    #[test]
    fn mechanistic_biphasic_shape() {
        let pathways = test_pathways();
        let doses: Vec<f64> = (0..100).map(f64::from).collect();
        let fitnesses: Vec<f64> = doses
            .iter()
            .map(|&d| mechanistic_cell_fitness(d, 100.0, &pathways, 50.0, 2.0))
            .collect();
        let max_f = fitnesses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let max_idx = fitnesses
            .iter()
            .position(|&f| (f - max_f).abs() < tolerances::DIVISION_GUARD)
            .unwrap_or(0);
        assert!(max_f > 100.0, "peak above baseline: {max_f}");
        assert!(max_idx > 0, "peak not at dose=0: idx={max_idx}");
        assert!(max_idx < 50, "peak before IC50: idx={max_idx}");
    }
}
