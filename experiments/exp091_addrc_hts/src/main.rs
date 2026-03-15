// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp091: ADDRC high-throughput screening analysis (Lisabeth 2024)
//!
//! Validates Z'-factor, SSMD, percent inhibition, and hit classification
//! for a synthetic 96-well plate with known controls.

use healthspring_barracuda::discovery::{
    classify_hits, percent_inhibition, ssmd, z_prime_factor, HitClass,
};
use healthspring_barracuda::provenance::{log_analytical, AnalyticalProvenance};
use healthspring_barracuda::tolerances::{DETERMINISM, HTS_PERCENT_INHIBITION, HTS_SSMD, HTS_Z_PRIME};
use healthspring_barracuda::validation::ValidationHarness;

const HTS_PROV: AnalyticalProvenance = AnalyticalProvenance {
    formula: "Z' = 1 - 3(σ_p + σ_n)/|μ_p - μ_n| (Zhang 1999)",
    reference: "Zhang JH et al. J Biomol Screen 4:67",
    doi: None,
};

fn main() {
    let mut h = ValidationHarness::new("exp091_addrc_hts");
    log_analytical(&HTS_PROV);

    let pos_mean = 10.0;
    let pos_std = 2.0;
    let neg_mean = 90.0;
    let neg_std = 3.0;

    // 1. Z'-factor for excellent plate: Z' = 1 - 3(2+3)/|10-90| = 0.8125
    let zp = z_prime_factor(pos_mean, pos_std, neg_mean, neg_std);
    let expected_zp = 1.0 - 3.0 * (pos_std + neg_std) / (neg_mean - pos_mean).abs();
    h.check_abs("Z'-factor: excellent plate formula", zp, expected_zp, HTS_Z_PRIME);

    // 2. Z'-factor > 0.5 (excellent)
    h.check_lower("Z'-factor > 0.5", zp, 0.5);

    // 3. Z'-factor with equal means → negative infinity
    let zp_equal = z_prime_factor(50.0, 1.0, 50.0, 1.0);
    h.check_bool(
        "Z'-factor: equal means → -inf",
        zp_equal.is_infinite() && zp_equal < 0.0,
    );

    // 4. SSMD for strong hit: |SSMD| > 3
    let ssmd_strong = ssmd(15.0, 1.0, neg_mean, neg_std);
    h.check_bool("SSMD: strong hit |SSMD| > 3", ssmd_strong.abs() > 3.0);

    // 5. SSMD for inactive: |SSMD| < 1
    let ssmd_inactive = ssmd(88.0, 2.0, neg_mean, neg_std);
    h.check_bool("SSMD: inactive |SSMD| < 1", ssmd_inactive.abs() < 1.0);

    // 6. SSMD with zero variance → 0.0
    let ssmd_zero_var = ssmd(10.0, 0.0, 10.0, 0.0);
    h.check_abs("SSMD: zero variance → 0", ssmd_zero_var, 0.0, HTS_SSMD);

    // 7. Percent inhibition: signal=10 (pos_mean) → 100%
    let pct_full = percent_inhibition(pos_mean, pos_mean, neg_mean);
    h.check_abs("percent_inhibition: pos → 100%", pct_full, 100.0, HTS_PERCENT_INHIBITION);

    // 8. Percent inhibition: signal=90 (neg_mean) → 0%
    let pct_zero = percent_inhibition(neg_mean, pos_mean, neg_mean);
    h.check_abs("percent_inhibition: neg → 0%", pct_zero, 0.0, HTS_PERCENT_INHIBITION);

    // 9. Percent inhibition: signal=50 → 50%
    let pct_half = percent_inhibition(50.0, pos_mean, neg_mean);
    h.check_abs("percent_inhibition: 50 → 50%", pct_half, 50.0, HTS_PERCENT_INHIBITION);

    // Synthetic plate: strong (15), moderate (45), weak (70), inactive (88)
    let signals = vec![15.0, 45.0, 70.0, 88.0];
    let compound_stds = vec![1.0, 2.0, 2.0, 2.0];

    // 10. classify_hits: strong hit classified Strong
    let results = classify_hits(&signals, &compound_stds, pos_mean, neg_mean, neg_std);
    h.check_bool(
        "classify_hits: strong hit → Strong",
        results[0].classification == HitClass::Strong,
    );

    // 11. classify_hits: inactive classified Inactive
    h.check_bool(
        "classify_hits: inactive → Inactive",
        results[3].classification == HitClass::Inactive,
    );

    // 12. Determinism: same plate → same results
    let results2 = classify_hits(&signals, &compound_stds, pos_mean, neg_mean, neg_std);
    let identical = results
        .iter()
        .zip(results2.iter())
        .all(|(a, b)| {
            (a.percent_inhibition - b.percent_inhibition).abs() < DETERMINISM
                && (a.ssmd_value - b.ssmd_value).abs() < DETERMINISM
                && a.classification == b.classification
        });
    h.check_bool("determinism: same plate → same results", identical);

    h.exit();
}
