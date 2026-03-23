// SPDX-License-Identifier: AGPL-3.0-or-later
//! High-throughput screening (HTS) analysis pipeline.
//!
//! References:
//! - Lisabeth et al. (2024) *Front Microbiol* — Brucella host-cellular small molecule screen
//! - Zhang JH et al. (1999) *J Biomol Screen* 4:67-73 — Z'-factor
//!
//! Provides plate quality metrics (Z'-factor), hit identification (SSMD, percent
//! inhibition), and classification for compound screening data from ADDRC and
//! similar HTS pipelines.

use crate::tolerances;

use serde::{Deserialize, Serialize};

/// Hit classification from SSMD magnitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitClass {
    /// Strong effect (|SSMD| > 3).
    Strong,
    /// Moderate effect (2 < |SSMD| ≤ 3).
    Moderate,
    /// Weak effect (1 < |SSMD| ≤ 2).
    Weak,
    /// No meaningful effect (|SSMD| ≤ 1).
    Inactive,
}

/// Result of hit classification for a single well/compound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitResult {
    /// Well or compound index in the input slice.
    pub index: usize,
    /// Percent inhibition vs plate controls.
    pub percent_inhibition: f64,
    /// Strictly standardized mean difference vs negative control.
    pub ssmd_value: f64,
    /// Discrete hit strength bucket from `classify_ssmd`.
    pub classification: HitClass,
}

/// Z'-factor for plate quality assessment.
///
/// `Z' = 1 - 3(σ_p + σ_n) / |μ_p - μ_n|`
///
/// Interpretation:
/// - Z' > 0.5: excellent assay (clear separation between controls)
/// - 0.0 ≤ Z' ≤ 0.5: marginal (usable with caveats)
/// - Z' < 0.0: unacceptable (control distributions overlap)
///
/// Returns `f64::NEG_INFINITY` if positive and negative means are equal.
#[must_use]
pub fn z_prime_factor(pos_mean: f64, pos_std: f64, neg_mean: f64, neg_std: f64) -> f64 {
    let separation = (pos_mean - neg_mean).abs();
    if separation < tolerances::DIVISION_GUARD {
        return f64::NEG_INFINITY;
    }
    1.0 - 3.0 * (pos_std + neg_std) / separation
}

/// Strictly standardized mean difference (SSMD).
///
/// `SSMD = (μ_s - μ_n) / sqrt(σ_s² + σ_n²)`
///
/// Classification (by |SSMD|):
/// - > 3.0: strong effect
/// - 2.0–3.0: moderate effect
/// - 1.0–2.0: weak effect
/// - < 1.0: no effect
///
/// Returns 0.0 if both standard deviations are zero.
#[must_use]
pub fn ssmd(sample_mean: f64, sample_std: f64, neg_mean: f64, neg_std: f64) -> f64 {
    let denom = sample_std.hypot(neg_std);
    if denom < tolerances::DIVISION_GUARD {
        return 0.0;
    }
    (sample_mean - neg_mean) / denom
}

/// Percent inhibition from raw signal relative to plate controls.
///
/// `%inhib = 100 × (μ_neg - signal) / (μ_neg - μ_pos)`
///
/// Assumes lower signal = more inhibition (e.g., cell viability assay where
/// positive control kills cells).
///
/// Returns 0.0 if positive and negative means are equal.
#[must_use]
pub fn percent_inhibition(signal: f64, pos_mean: f64, neg_mean: f64) -> f64 {
    let range = neg_mean - pos_mean;
    if range.abs() < tolerances::DIVISION_GUARD {
        return 0.0;
    }
    100.0 * (neg_mean - signal) / range
}

/// Classify SSMD value into hit strength.
#[must_use]
pub fn classify_ssmd(ssmd_abs: f64) -> HitClass {
    if ssmd_abs > 3.0 {
        HitClass::Strong
    } else if ssmd_abs > 2.0 {
        HitClass::Moderate
    } else if ssmd_abs > 1.0 {
        HitClass::Weak
    } else {
        HitClass::Inactive
    }
}

/// Classify all compounds from an HTS plate.
///
/// Each element in `signals` is a single-well measurement. `neg_mean`/`neg_std`
/// are the plate negative control statistics. `pos_mean` is the positive control
/// mean (used for percent inhibition normalization).
///
/// `compound_std` is the per-compound replicate standard deviation. If replicates
/// are unavailable, pass 0.0 (SSMD will use only the negative control variance).
#[must_use]
pub fn classify_hits(
    signals: &[f64],
    compound_stds: &[f64],
    pos_mean: f64,
    neg_mean: f64,
    neg_std: f64,
) -> Vec<HitResult> {
    signals
        .iter()
        .zip(compound_stds.iter())
        .enumerate()
        .map(|(i, (&signal, &cstd))| {
            let pct = percent_inhibition(signal, pos_mean, neg_mean);
            let ssmd_val = ssmd(signal, cstd, neg_mean, neg_std);
            let classification = classify_ssmd(ssmd_val.abs());
            HitResult {
                index: i,
                percent_inhibition: pct,
                ssmd_value: ssmd_val,
                classification,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn z_prime_excellent() {
        let zp = z_prime_factor(10.0, 1.0, 90.0, 1.0);
        assert!((zp - (1.0 - 6.0 / 80.0)).abs() < tolerances::MACHINE_EPSILON);
        assert!(zp > 0.5);
    }

    #[test]
    fn z_prime_equal_means() {
        let zp = z_prime_factor(50.0, 1.0, 50.0, 1.0);
        assert!(zp.is_infinite() && zp < 0.0);
    }

    #[test]
    fn z_prime_marginal() {
        let zp = z_prime_factor(30.0, 5.0, 70.0, 5.0);
        // Z' = 1 - 3*(5+5)/40 = 1 - 0.75 = 0.25
        assert!(zp > 0.0);
        assert!(zp < 0.5);
    }

    #[test]
    fn ssmd_strong_hit() {
        let val = ssmd(10.0, 1.0, 90.0, 1.0);
        assert!(val.abs() > 3.0);
    }

    #[test]
    fn ssmd_zero_variance() {
        assert!((ssmd(10.0, 0.0, 10.0, 0.0)).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pct_inhibition_full() {
        let pct = percent_inhibition(10.0, 10.0, 90.0);
        assert!((pct - 100.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pct_inhibition_zero() {
        let pct = percent_inhibition(90.0, 10.0, 90.0);
        assert!(pct.abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pct_inhibition_half() {
        let pct = percent_inhibition(50.0, 10.0, 90.0);
        assert!((pct - 50.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn classify_ssmd_boundaries() {
        assert_eq!(classify_ssmd(3.5), HitClass::Strong);
        assert_eq!(classify_ssmd(2.5), HitClass::Moderate);
        assert_eq!(classify_ssmd(1.5), HitClass::Weak);
        assert_eq!(classify_ssmd(0.5), HitClass::Inactive);
    }

    #[test]
    fn classify_hits_mixed_plate() {
        let signals = vec![15.0, 50.0, 88.0];
        let stds = vec![1.0, 5.0, 2.0];
        let results = classify_hits(&signals, &stds, 10.0, 90.0, 3.0);
        assert_eq!(results.len(), 3);
        assert!(results[0].percent_inhibition > 90.0);
        assert!(results[2].percent_inhibition < 5.0);
    }
}
