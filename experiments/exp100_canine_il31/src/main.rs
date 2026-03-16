// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! Exp100: Canine IL-31 serum kinetics in atopic dermatitis (Gonzales 2013, CM-001)
//!
//! Validates IL-31 kinetics in AD dogs and pruritus VAS response.

use healthspring_barracuda::comparative::canine::{
    CanineIl31Treatment, il31_serum_kinetics, pruritus_vas_response,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    DETERMINISM, IL31_INITIAL, IL31_STEADY_STATE, MACHINE_EPSILON, MACHINE_EPSILON_STRICT,
    PRURITUS_AT_EC50,
};
use healthspring_barracuda::validation::ValidationHarness;

const BASELINE_PG_ML: f64 = 44.5;

fn main() {
    let mut h = ValidationHarness::new("exp100_canine_il31");

    // Provenance: IL-31 kinetics
    log_analytical(&AnalyticalProvenance {
        formula: "dC/dt = k_prod - k_el*C",
        reference: "Gonzales 2013, Vet Dermatol 24:48",
        doi: Some("10.1111/j.1365-3164.2012.01098.x"),
    });

    // Provenance: Pruritus VAS
    log_analytical(&AnalyticalProvenance {
        formula: "VAS = VAS_max * C^n / (EC50^n + C^n)",
        reference: "Gonzales 2016, Vet Dermatol 27:34",
        doi: None,
    });

    // Check 1: Untreated at t=0: C = baseline (44.5)
    let c_untreated_0 = il31_serum_kinetics(BASELINE_PG_ML, 0.0, CanineIl31Treatment::Untreated);
    h.check_abs(
        "Untreated at t=0: C = baseline",
        c_untreated_0,
        BASELINE_PG_ML,
        IL31_INITIAL,
    );

    // Check 2: Untreated at steady-state (t=1000hr): C ≈ baseline (first-order kinetics)
    let c_untreated_ss =
        il31_serum_kinetics(BASELINE_PG_ML, 1000.0, CanineIl31Treatment::Untreated);
    h.check_abs(
        "Untreated at steady-state (t=1000hr): C ≈ baseline",
        c_untreated_ss,
        BASELINE_PG_ML,
        IL31_STEADY_STATE,
    );

    // Check 3: Oclacitinib at t=0: C = baseline (drug hasn't acted yet)
    let c_ocla_0 = il31_serum_kinetics(BASELINE_PG_ML, 0.0, CanineIl31Treatment::Oclacitinib);
    h.check_abs(
        "Oclacitinib at t=0: C = baseline",
        c_ocla_0,
        BASELINE_PG_ML,
        IL31_INITIAL,
    );

    // Check 4: Oclacitinib reduces IL-31 vs untreated at t=200hr
    let c_untreated_200 =
        il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Untreated);
    let c_ocla_200 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib);
    h.check_bool(
        "Oclacitinib reduces IL-31 vs untreated at t=200hr",
        c_ocla_200 < c_untreated_200,
    );

    // Check 5: Lokivetmab reduces IL-31 vs untreated at t=200hr
    let c_loki_200 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Lokivetmab);
    h.check_bool(
        "Lokivetmab reduces IL-31 vs untreated at t=200hr",
        c_loki_200 < c_untreated_200,
    );

    // Check 6: Lokivetmab reduces faster than oclacitinib at t=24hr
    let c_ocla_24 = il31_serum_kinetics(BASELINE_PG_ML, 24.0, CanineIl31Treatment::Oclacitinib);
    let c_loki_24 = il31_serum_kinetics(BASELINE_PG_ML, 24.0, CanineIl31Treatment::Lokivetmab);
    h.check_bool(
        "Lokivetmab reduces faster than oclacitinib at t=24hr",
        c_loki_24 < c_ocla_24,
    );

    // Check 7: All concentrations non-negative
    let c_vals = [
        il31_serum_kinetics(BASELINE_PG_ML, 0.0, CanineIl31Treatment::Untreated),
        il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib),
        il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Lokivetmab),
    ];
    h.check_bool(
        "All concentrations non-negative",
        c_vals.iter().all(|&c| c >= 0.0),
    );

    // Check 8: Untreated monotonic (stays at steady state)
    let c_0 = il31_serum_kinetics(BASELINE_PG_ML, 0.0, CanineIl31Treatment::Untreated);
    let c_500 = il31_serum_kinetics(BASELINE_PG_ML, 500.0, CanineIl31Treatment::Untreated);
    let c_1000 = il31_serum_kinetics(BASELINE_PG_ML, 1000.0, CanineIl31Treatment::Untreated);
    h.check_bool(
        "Untreated monotonic (stays at steady state)",
        (c_0 - c_500).abs() < IL31_STEADY_STATE && (c_500 - c_1000).abs() < IL31_STEADY_STATE,
    );

    // Check 9: Pruritus VAS at IL-31=0: VAS=0
    let vas_at_zero = pruritus_vas_response(0.0);
    h.check_abs(
        "Pruritus VAS at IL-31=0: VAS=0",
        vas_at_zero,
        0.0,
        MACHINE_EPSILON,
    );

    // Check 10: Pruritus VAS at IL-31=EC50 (25): VAS=5.0 (half-max, analytical Hill identity)
    let vas_ec50 = pruritus_vas_response(25.0);
    h.check_abs(
        "Pruritus VAS at IL-31=EC50 (25): VAS=5.0",
        vas_ec50,
        5.0,
        PRURITUS_AT_EC50,
    );

    // Check 11: Pruritus VAS monotonic: higher IL-31 → higher VAS
    let vas_at_10 = pruritus_vas_response(10.0);
    let vas_at_50 = pruritus_vas_response(50.0);
    h.check_bool(
        "Pruritus VAS monotonic: higher IL-31 → higher VAS",
        vas_at_50 > vas_at_10 + MACHINE_EPSILON_STRICT,
    );

    // Check 12: Pruritus VAS saturates: VAS at 1000 pg/mL > 9.5
    let vas_1000 = pruritus_vas_response(1000.0);
    h.check_bool(
        "Pruritus VAS saturates: VAS at 1000 pg/mL > 9.5",
        vas_1000 > 9.5,
    );

    // Check 13: Treatment reduces pruritus: VAS(treated IL-31) < VAS(untreated IL-31) at t=200hr
    let vas_untreated = pruritus_vas_response(c_untreated_200);
    let vas_treated = pruritus_vas_response(c_ocla_200);
    h.check_bool(
        "Treatment reduces pruritus at t=200hr",
        vas_treated < vas_untreated,
    );

    // Check 14: Determinism: same inputs → identical results
    let r1 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib);
    let r2 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib);
    h.check_abs(
        "Determinism: same inputs → identical results",
        r1,
        r2,
        DETERMINISM,
    );

    h.exit();
}
