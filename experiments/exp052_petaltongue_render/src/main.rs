// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! Exp052: petalTongue scenario schema validation.
//!
//! Validates that healthSpring's enriched diagnostic data produces
//! petalTongue-compatible JSON with data channels, and that the
//! schema round-trips through serde correctly.

use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{
    annotate_population, assessment_to_scenario, full_scenario_json, scenario_to_json,
};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

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

    // --- Patient with full data ---
    let mut patient = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    patient.testosterone_ng_dl = Some(450.0);
    patient.on_trt = true;
    patient.trt_months = 12.0;
    patient.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);
    patient.ecg_peaks = Some(vec![0, 320, 630, 950, 1260, 1580, 1900, 2210, 2530, 2850]);
    patient.ecg_fs = 360.0;
    patient.ppg_spo2 = Some(97.5);

    let assessment = assess_patient(&patient);

    // --- PK enriched data ---
    check!(
        "pk_curve_101_points",
        assessment.pk.curve_times_hr.len() == 101
    );
    check!(
        "pk_curve_starts_at_zero",
        assessment.pk.curve_times_hr[0] == 0.0
    );
    check!(
        "pk_curve_ends_at_24h",
        (assessment.pk.curve_times_hr[100] - 24.0).abs() < 0.01
    );
    check!("pk_hill_50_points", assessment.pk.hill_concs.len() == 50);
    check!(
        "pk_hill_responses_match",
        assessment.pk.hill_responses.len() == 50
    );

    // --- Microbiome enriched ---
    check!(
        "microbiome_abundances_passed",
        assessment.microbiome.abundances.len() == 7
    );
    check!(
        "microbiome_abundances_sum_near_1",
        (assessment.microbiome.abundances.iter().sum::<f64>() - 1.0).abs() < 0.01
    );

    // --- Biosignal enriched ---
    check!(
        "biosignal_rr_intervals_from_ecg",
        assessment.biosignal.rr_intervals_ms.len() == 9
    );
    check!(
        "biosignal_rr_positive",
        assessment
            .biosignal
            .rr_intervals_ms
            .iter()
            .all(|&r| r > 0.0)
    );

    // --- Scenario schema compatibility ---
    let scenario = assessment_to_scenario(&assessment, "Exp052 Schema Test");
    let json = scenario_to_json(&scenario);

    check!("json_has_version", json.contains("\"version\": \"2.0.0\""));
    check!(
        "json_has_sensory_config",
        json.contains("\"sensory_config\"")
    );
    check!("json_has_ui_config", json.contains("\"ui_config\""));
    check!("json_has_neural_api", json.contains("\"neural_api\""));
    check!("json_has_ecosystem_primals", json.contains("\"primals\""));
    check!("json_has_data_channels", json.contains("\"data_channels\""));
    check!(
        "json_has_timeseries",
        json.contains("\"channel_type\": \"timeseries\"")
    );
    check!(
        "json_has_gauge",
        json.contains("\"channel_type\": \"gauge\"")
    );
    check!(
        "json_has_clinical_ranges",
        json.contains("\"clinical_ranges\"")
    );

    // --- JSON round-trip ---
    let scenario_val: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL: valid JSON: {e}");
            std::process::exit(1);
        }
    };
    let Some(primals) = scenario_val["ecosystem"]["primals"].as_array() else {
        eprintln!("FAIL: primals not array");
        std::process::exit(1);
    };
    check!("round_trip_7_primals", primals.len() == 7);

    let Some(pk) = primals.iter().find(|p| p["id"] == "pk") else {
        eprintln!("FAIL: pk primal not found");
        std::process::exit(1);
    };
    check!("pk_has_type_compute", pk["type"] == "compute");
    check!("pk_has_family_healthspring", pk["family"] == "healthspring");
    check!(
        "pk_position_optional",
        pk.get("position").is_none() || pk["position"].is_null() || pk["position"]["x"].is_f64()
    );
    let Some(pk_channels) = pk["data_channels"].as_array() else {
        eprintln!("FAIL: pk data_channels not array");
        std::process::exit(1);
    };
    check!("pk_has_4_channels", pk_channels.len() == 4);
    check!(
        "pk_first_is_timeseries",
        pk_channels[0]["channel_type"] == "timeseries"
    );

    // --- Full scenario with population ---
    let pop = population_montecarlo(&patient, 500, 42);
    let full_json = full_scenario_json(&assessment, &pop, "Exp052 Full");
    let full: serde_json::Value = match serde_json::from_str(&full_json) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL: full JSON: {e}");
            std::process::exit(1);
        }
    };
    check!("full_has_edges", full["edges"].is_array());
    let Some(full_primals) = full["ecosystem"]["primals"].as_array() else {
        eprintln!("FAIL: full primals not array");
        std::process::exit(1);
    };
    check!("full_8_primals_with_pop", full_primals.len() == 8);
    let Some(pop_node) = full_primals.iter().find(|p| p["id"] == "population") else {
        eprintln!("FAIL: population primal not found");
        std::process::exit(1);
    };
    let Some(pop_channels) = pop_node["data_channels"].as_array() else {
        eprintln!("FAIL: pop data_channels not array");
        std::process::exit(1);
    };
    check!(
        "pop_has_distribution",
        pop_channels[0]["channel_type"] == "distribution"
    );
    check!(
        "pop_distribution_has_values",
        pop_channels[0]["values"].as_array().map_or(0, Vec::len) == 500
    );

    // --- Population with annotated scenario ---
    let scenario2 = assessment_to_scenario(&assessment, "Annotate Test");
    let annotated = annotate_population(scenario2, &pop);
    check!("annotated_8_nodes", annotated.ecosystem.primals.len() == 8);

    // --- Biosignal node has tachogram ---
    let Some(bio_node) = primals.iter().find(|p| p["id"] == "biosignal") else {
        eprintln!("FAIL: biosignal primal not found");
        std::process::exit(1);
    };
    let Some(bio_channels) = bio_node["data_channels"].as_array() else {
        eprintln!("FAIL: biosignal data_channels not array");
        std::process::exit(1);
    };
    let has_tachogram = bio_channels.iter().any(|c| c["id"] == "rr_tachogram");
    check!("biosignal_has_rr_tachogram", has_tachogram);

    // --- Microbiome node has bar chart ---
    let Some(micro_node) = primals.iter().find(|p| p["id"] == "microbiome") else {
        eprintln!("FAIL: microbiome primal not found");
        std::process::exit(1);
    };
    let Some(micro_channels) = micro_node["data_channels"].as_array() else {
        eprintln!("FAIL: microbiome data_channels not array");
        std::process::exit(1);
    };
    check!(
        "microbiome_has_bar_chart",
        micro_channels[0]["channel_type"] == "bar"
    );

    let total = passed + failed;
    println!("\nExp052 petalTongue Render: {passed}/{total} checks passed",);
    std::process::exit(i32::from(passed != total));
}
