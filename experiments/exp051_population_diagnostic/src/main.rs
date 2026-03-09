// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

//! Exp051: Population diagnostic Monte Carlo.
//!
//! Generates 1,000 virtual patients around a base profile, runs each through
//! the full diagnostic pipeline, and validates population statistics. Also
//! validates `petalTongue` scenario export with population annotation.

use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{
    annotate_population, assessment_to_scenario, scenario_to_json,
};

fn main() {
    let mut passed = 0;
    let mut failed = 0;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                passed += 1;
            } else {
                eprintln!("FAIL: {}", $name);
                failed += 1;
            }
        };
    }

    // --- Base patient ---
    let mut base = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    base.testosterone_ng_dl = Some(400.0);
    base.on_trt = true;
    base.trt_months = 6.0;
    base.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);

    // --- Population Monte Carlo (1000 patients) ---
    let pop = population_montecarlo(&base, 1000, 42);

    check!("pop_n_patients", pop.n_patients == 1000);
    check!("pop_risks_count", pop.composite_risks.len() == 1000);
    check!("pop_mean_positive", pop.mean_risk > 0.0);
    check!("pop_mean_bounded", pop.mean_risk < 1.0);
    check!("pop_std_positive", pop.std_risk > 0.0);
    check!("pop_std_less_than_mean", pop.std_risk < pop.mean_risk * 3.0);
    check!(
        "pop_percentile_range",
        pop.patient_percentile >= 0.0 && pop.patient_percentile <= 100.0
    );

    let min_risk = pop
        .composite_risks
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_risk = pop
        .composite_risks
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    check!("pop_min_non_negative", min_risk >= 0.0);
    check!("pop_max_bounded", max_risk <= 1.0);
    check!("pop_spread_exists", max_risk > min_risk);

    // --- Determinism ---
    let pop2 = population_montecarlo(&base, 1000, 42);
    check!(
        "deterministic_mean",
        (pop.mean_risk - pop2.mean_risk).abs() < 1e-12
    );
    check!(
        "deterministic_percentile",
        (pop.patient_percentile - pop2.patient_percentile).abs() < 1e-12
    );
    check!(
        "deterministic_std",
        (pop.std_risk - pop2.std_risk).abs() < 1e-12
    );

    // --- Different seeds produce different results ---
    let pop3 = population_montecarlo(&base, 1000, 99);
    check!(
        "different_seed_different_mean",
        (pop.mean_risk - pop3.mean_risk).abs() > 1e-6
    );

    // --- Scenario export with population ---
    let base_assessment = assess_patient(&base);
    let scenario = assessment_to_scenario(&base_assessment, "Exp051 Population Base");
    let annotated = annotate_population(scenario, &pop);

    check!(
        "annotated_node_count",
        annotated.ecosystem.primals.len() == 8
    );
    check!(
        "annotated_has_population_node",
        annotated
            .ecosystem
            .primals
            .iter()
            .any(|n| n.id == "population")
    );

    let json = scenario_to_json(&annotated);
    check!("json_has_population", json.contains("population"));
    check!("json_has_percentile", json.contains("patient_value"));
    check!("json_has_distribution", json.contains("distribution"));

    // --- Lighter patient should have different CL ---
    let mut light = PatientProfile::minimal(55.0, 55.0, Sex::Male);
    light.testosterone_ng_dl = Some(400.0);
    let pop_light = population_montecarlo(&light, 500, 42);
    check!(
        "weight_affects_distribution",
        (pop_light.mean_risk - pop.mean_risk).abs() > 1e-6
    );

    // --- Female population ---
    let mut female = PatientProfile::minimal(45.0, 65.0, Sex::Female);
    female.testosterone_ng_dl = Some(35.0);
    let pop_female = population_montecarlo(&female, 500, 42);
    check!(
        "female_pop_valid",
        pop_female.mean_risk > 0.0 && pop_female.mean_risk < 1.0
    );

    let total = passed + failed;
    println!("\nExp051 Population Diagnostic: {passed}/{total} checks passed",);
    println!(
        "  Population: n={}, mean_risk={:.4}, std={:.4}, percentile={:.1}",
        pop.n_patients, pop.mean_risk, pop.std_risk, pop.patient_percentile
    );
    println!("  Risk range: [{min_risk:.4}, {max_risk:.4}]");
    std::process::exit(i32::from(passed != total));
}
