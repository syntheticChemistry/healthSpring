// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! Exp094: Rho/MRTF/SRF fibrosis pathway scoring (Neubig, DD-005)
//!
//! Validates fibrosis pathway scoring and Anderson geometry for anti-fibrotic compounds.

use healthspring_barracuda::discovery::fibrosis::{
    ccg_1423, ccg_203971, fibrosis_matrix_score, fibrotic_geometry_factor, fractional_inhibition,
    score_anti_fibrotic,
};
use healthspring_barracuda::discovery::tissue_geometry_factor;
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    ANTI_FIBROTIC_SCORE, DETERMINISM, FIBROTIC_GEOMETRY, FRACTIONAL_AT_IC50,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp094_rho_mrtf_fibrosis");

    log_analytical(&AnalyticalProvenance {
        formula: "score = 0.2×rho + 0.5×mrtf + 0.3×srf",
        reference: "Haak 2014, JPET 349:480",
        doi: None,
    });

    // 1. fractional_inhibition at 0 → 0
    let frac_at_0 = fractional_inhibition(0.0, 5.0);
    h.check_abs(
        "fractional_inhibition at 0 → 0",
        frac_at_0,
        0.0,
        FRACTIONAL_AT_IC50,
    );

    // 2. fractional_inhibition at IC50 → 0.5
    let frac_at_ic50 = fractional_inhibition(5.0, 5.0);
    h.check_abs(
        "fractional_inhibition at IC50 → 0.5",
        frac_at_ic50,
        0.5,
        FRACTIONAL_AT_IC50,
    );

    // 3. fractional_inhibition at 10000× → >0.99
    let frac_at_10k = fractional_inhibition(50_000.0, 5.0);
    h.check_bool(
        "fractional_inhibition at 10000× → >0.99",
        frac_at_10k > 0.99,
    );

    // 4. fractional_inhibition with zero IC50 → 0
    let frac_zero_ic50 = fractional_inhibition(5.0, 0.0);
    h.check_abs(
        "fractional_inhibition with zero IC50 → 0",
        frac_zero_ic50,
        0.0,
        FRACTIONAL_AT_IC50,
    );

    // 5. CCG-1423 score at 0 → 0
    let ccg1423 = ccg_1423();
    let score_ccg_at_0 = score_anti_fibrotic(&ccg1423, 0.0);
    h.check_abs(
        "CCG-1423 score at 0 → 0",
        score_ccg_at_0.anti_fibrotic_score,
        0.0,
        ANTI_FIBROTIC_SCORE,
    );

    // 6. CCG-1423 at 10 µM: all components >0
    let score_ccg_10um = score_anti_fibrotic(&ccg1423, 10.0);
    h.check_bool(
        "CCG-1423 at 10 µM: all components >0",
        score_ccg_10um.rho_inhibition > 0.0
            && score_ccg_10um.mrtf_block > 0.0
            && score_ccg_10um.srf_reduction > 0.0,
    );

    // 7. CCG-203971 has better MRTF than CCG-1423 at 5 µM (lower MRTF IC50)
    let ccg203971 = ccg_203971();
    let score_203971_at_5 = score_anti_fibrotic(&ccg203971, 5.0);
    let score_1423_at_5 = score_anti_fibrotic(&ccg1423, 5.0);
    h.check_bool(
        "CCG-203971 has better MRTF than CCG-1423 at 5 µM",
        score_203971_at_5.mrtf_block > score_1423_at_5.mrtf_block,
    );

    // 8. Anti-fibrotic score monotonic with concentration
    let score_lo = score_anti_fibrotic(&ccg1423, 1.0).anti_fibrotic_score;
    let score_mid = score_anti_fibrotic(&ccg1423, 5.0).anti_fibrotic_score;
    let score_hi = score_anti_fibrotic(&ccg1423, 20.0).anti_fibrotic_score;
    h.check_bool(
        "Anti-fibrotic score monotonic with concentration",
        score_lo
            .partial_cmp(&score_mid)
            .unwrap_or(core::cmp::Ordering::Equal)
            == core::cmp::Ordering::Less
            && score_mid
                .partial_cmp(&score_hi)
                .unwrap_or(core::cmp::Ordering::Equal)
                == core::cmp::Ordering::Less,
    );

    // 9. fibrotic_geometry_factor + standard geometry ≈ 1.0 (complementary)
    let xi = 10.0;
    let l = 1.0;
    let fibrotic_geom = fibrotic_geometry_factor(xi, l);
    let standard_geom = tissue_geometry_factor(xi, l);
    h.check_abs(
        "fibrotic_geometry_factor + standard geometry ≈ 1.0",
        fibrotic_geom + standard_geom,
        1.0,
        FIBROTIC_GEOMETRY,
    );

    // 10. fibrotic_geometry_factor: zero thickness → 0
    let fibrotic_zero_thick = fibrotic_geometry_factor(10.0, 0.0);
    h.check_abs(
        "fibrotic_geometry: zero thickness → 0",
        fibrotic_zero_thick,
        0.0,
        FIBROTIC_GEOMETRY,
    );

    // 11. fibrotic_geometry: small ξ → high factor (good for anti-fibrotic)
    let factor_small_xi = fibrotic_geometry_factor(0.1, 1.0);
    let factor_large_xi = fibrotic_geometry_factor(10.0, 1.0);
    h.check_bool(
        "fibrotic_geometry: small ξ → high factor",
        factor_small_xi > factor_large_xi,
    );

    // 12. fibrotic_geometry: large ξ → low factor
    h.check_bool(
        "fibrotic_geometry: large ξ → low factor",
        factor_large_xi < 0.1,
    );

    // 13. fibrosis_matrix_score: product identity
    let anti_fib = 0.7;
    let fib_geom = 0.4;
    let disorder_fac = 1.1;
    let combined = fibrosis_matrix_score(anti_fib, fib_geom, disorder_fac);
    let expected = anti_fib * fib_geom * disorder_fac;
    h.check_abs(
        "fibrosis_matrix_score: product identity",
        combined,
        expected,
        FIBROTIC_GEOMETRY,
    );

    // 14. Determinism: same compound, same concentration → identical score
    let run1 = score_anti_fibrotic(&ccg1423, 5.0).anti_fibrotic_score;
    let run2 = score_anti_fibrotic(&ccg1423, 5.0).anti_fibrotic_score;
    h.check_abs(
        "Determinism: same compound, same concentration → identical score",
        run1,
        run2,
        DETERMINISM,
    );

    h.exit();
}
