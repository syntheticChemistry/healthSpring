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

fn validate_il31_kinetics(h: &mut ValidationHarness) -> (f64, f64) {
    let c_untreated_0 = il31_serum_kinetics(BASELINE_PG_ML, 0.0, CanineIl31Treatment::Untreated);
    h.check_abs(
        "Untreated at t=0: C = baseline",
        c_untreated_0,
        BASELINE_PG_ML,
        IL31_INITIAL,
    );

    let c_untreated_ss =
        il31_serum_kinetics(BASELINE_PG_ML, 1000.0, CanineIl31Treatment::Untreated);
    h.check_abs(
        "Untreated at steady-state (t=1000hr): C ≈ baseline",
        c_untreated_ss,
        BASELINE_PG_ML,
        IL31_STEADY_STATE,
    );

    let c_ocla_0 = il31_serum_kinetics(BASELINE_PG_ML, 0.0, CanineIl31Treatment::Oclacitinib);
    h.check_abs(
        "Oclacitinib at t=0: C = baseline",
        c_ocla_0,
        BASELINE_PG_ML,
        IL31_INITIAL,
    );

    let c_untreated_200 =
        il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Untreated);
    let c_ocla_200 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib);
    h.check_bool(
        "Oclacitinib reduces IL-31 vs untreated at t=200hr",
        c_ocla_200 < c_untreated_200,
    );

    let c_loki_200 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Lokivetmab);
    h.check_bool(
        "Lokivetmab reduces IL-31 vs untreated at t=200hr",
        c_loki_200 < c_untreated_200,
    );

    let c_ocla_24 = il31_serum_kinetics(BASELINE_PG_ML, 24.0, CanineIl31Treatment::Oclacitinib);
    let c_loki_24 = il31_serum_kinetics(BASELINE_PG_ML, 24.0, CanineIl31Treatment::Lokivetmab);
    h.check_bool(
        "Lokivetmab reduces faster than oclacitinib at t=24hr",
        c_loki_24 < c_ocla_24,
    );

    h.check_bool(
        "All concentrations non-negative",
        [c_untreated_0, c_ocla_200, c_loki_200]
            .iter()
            .all(|&c| c >= 0.0),
    );

    let c_500 = il31_serum_kinetics(BASELINE_PG_ML, 500.0, CanineIl31Treatment::Untreated);
    h.check_bool(
        "Untreated monotonic (stays at steady state)",
        (c_untreated_0 - c_500).abs() < IL31_STEADY_STATE
            && (c_500 - c_untreated_ss).abs() < IL31_STEADY_STATE,
    );

    (c_untreated_200, c_ocla_200)
}

fn validate_pruritus_vas(h: &mut ValidationHarness, c_untreated_200: f64, c_ocla_200: f64) {
    h.check_abs(
        "Pruritus VAS at IL-31=0: VAS=0",
        pruritus_vas_response(0.0),
        0.0,
        MACHINE_EPSILON,
    );
    h.check_abs(
        "Pruritus VAS at IL-31=EC50 (25): VAS=5.0",
        pruritus_vas_response(25.0),
        5.0,
        PRURITUS_AT_EC50,
    );
    h.check_bool(
        "Pruritus VAS monotonic: higher IL-31 → higher VAS",
        pruritus_vas_response(50.0) > pruritus_vas_response(10.0) + MACHINE_EPSILON_STRICT,
    );
    h.check_bool(
        "Pruritus VAS saturates: VAS at 1000 pg/mL > 9.5",
        pruritus_vas_response(1000.0) > 9.5,
    );
    h.check_bool(
        "Treatment reduces pruritus at t=200hr",
        pruritus_vas_response(c_ocla_200) < pruritus_vas_response(c_untreated_200),
    );

    let r1 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib);
    let r2 = il31_serum_kinetics(BASELINE_PG_ML, 200.0, CanineIl31Treatment::Oclacitinib);
    h.check_abs(
        "Determinism: same inputs → identical results",
        r1,
        r2,
        DETERMINISM,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp100_canine_il31");

    log_analytical(&AnalyticalProvenance {
        formula: "dC/dt = k_prod - k_el*C",
        reference: "Gonzales 2013, Vet Dermatol 24:48",
        doi: Some("10.1111/j.1365-3164.2012.01098.x"),
    });
    log_analytical(&AnalyticalProvenance {
        formula: "VAS = VAS_max * C^n / (EC50^n + C^n)",
        reference: "Gonzales 2016, Vet Dermatol 27:34",
        doi: None,
    });

    let (c_untreated_200, c_ocla_200) = validate_il31_kinetics(&mut h);
    validate_pruritus_vas(&mut h, c_untreated_200, c_ocla_200);

    h.exit();
}
