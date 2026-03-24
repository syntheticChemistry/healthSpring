// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
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
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::{OrExit, ValidationHarness};
use healthspring_barracuda::visualization::{
    annotate_population, assessment_to_scenario, full_scenario_json, scenario_to_json,
};

fn main() {
    let mut h = ValidationHarness::new("exp052_petaltongue_render");

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
    h.check_exact(
        "pk_curve_101_points",
        assessment.pk.curve_times_hr.len() as u64,
        101,
    );
    h.check_abs(
        "pk_curve_starts_at_zero",
        assessment.pk.curve_times_hr[0],
        0.0,
        tolerances::MACHINE_EPSILON_STRICT,
    );
    h.check_abs(
        "pk_curve_ends_at_24h",
        assessment.pk.curve_times_hr[100],
        24.0,
        tolerances::TEST_ASSERTION_LOOSE,
    );
    h.check_exact(
        "pk_hill_50_points",
        assessment.pk.hill_concs.len() as u64,
        50,
    );
    h.check_exact(
        "pk_hill_responses_match",
        assessment.pk.hill_responses.len() as u64,
        50,
    );

    // --- Microbiome enriched ---
    h.check_exact(
        "microbiome_abundances_passed",
        assessment.microbiome.abundances.len() as u64,
        7,
    );
    h.check_abs(
        "microbiome_abundances_sum_near_1",
        assessment.microbiome.abundances.iter().sum::<f64>(),
        1.0,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    // --- Biosignal enriched ---
    h.check_exact(
        "biosignal_rr_intervals_from_ecg",
        assessment.biosignal.rr_intervals_ms.len() as u64,
        9,
    );
    h.check_bool(
        "biosignal_rr_positive",
        assessment
            .biosignal
            .rr_intervals_ms
            .iter()
            .all(|&r| r > 0.0),
    );

    // --- Scenario schema compatibility ---
    let scenario = assessment_to_scenario(&assessment, "Exp052 Schema Test");
    let json = scenario_to_json(&scenario);

    h.check_bool("json_has_version", json.contains("\"version\": \"2.0.0\""));
    h.check_bool(
        "json_has_sensory_config",
        json.contains("\"sensory_config\""),
    );
    h.check_bool("json_has_ui_config", json.contains("\"ui_config\""));
    h.check_bool("json_has_neural_api", json.contains("\"neural_api\""));
    h.check_bool("json_has_ecosystem_primals", json.contains("\"primals\""));
    h.check_bool("json_has_data_channels", json.contains("\"data_channels\""));
    h.check_bool(
        "json_has_timeseries",
        json.contains("\"channel_type\": \"timeseries\""),
    );
    h.check_bool(
        "json_has_gauge",
        json.contains("\"channel_type\": \"gauge\""),
    );
    h.check_bool(
        "json_has_clinical_ranges",
        json.contains("\"clinical_ranges\""),
    );

    // --- JSON round-trip ---
    let scenario_val: serde_json::Value = serde_json::from_str(&json).or_exit("valid JSON");
    let primals = scenario_val["ecosystem"]["primals"]
        .as_array()
        .or_exit("primals not array");
    h.check_exact("round_trip_7_primals", primals.len() as u64, 7);

    let pk = primals
        .iter()
        .find(|p| p["id"] == "pk")
        .or_exit("pk primal not found");
    h.check_bool("pk_has_type_compute", pk["type"] == "compute");
    h.check_bool("pk_has_family_healthspring", pk["family"] == "healthspring");
    h.check_bool(
        "pk_position_optional",
        pk.get("position").is_none() || pk["position"].is_null() || pk["position"]["x"].is_f64(),
    );
    let pk_channels = pk["data_channels"]
        .as_array()
        .or_exit("pk data_channels not array");
    h.check_exact("pk_has_4_channels", pk_channels.len() as u64, 4);
    h.check_bool(
        "pk_first_is_timeseries",
        pk_channels[0]["channel_type"] == "timeseries",
    );

    // --- Full scenario with population ---
    let pop = population_montecarlo(&patient, 500, 42);
    let full_json = full_scenario_json(&assessment, &pop, "Exp052 Full");
    let full: serde_json::Value = serde_json::from_str(&full_json).or_exit("full JSON");
    h.check_bool("full_has_edges", full["edges"].is_array());
    let full_primals = full["ecosystem"]["primals"]
        .as_array()
        .or_exit("full primals not array");
    h.check_exact("full_8_primals_with_pop", full_primals.len() as u64, 8);
    let pop_node = full_primals
        .iter()
        .find(|p| p["id"] == "population")
        .or_exit("population primal not found");
    let pop_channels = pop_node["data_channels"]
        .as_array()
        .or_exit("pop data_channels not array");
    h.check_bool(
        "pop_has_distribution",
        pop_channels[0]["channel_type"] == "distribution",
    );
    h.check_exact(
        "pop_distribution_has_values",
        pop_channels[0]["values"].as_array().map_or(0, Vec::len) as u64,
        500,
    );

    // --- Population with annotated scenario ---
    let scenario2 = assessment_to_scenario(&assessment, "Annotate Test");
    let annotated = annotate_population(scenario2, &pop);
    h.check_exact(
        "annotated_8_nodes",
        annotated.ecosystem.primals.len() as u64,
        8,
    );

    // --- Biosignal node has tachogram ---
    let bio_node = primals
        .iter()
        .find(|p| p["id"] == "biosignal")
        .or_exit("biosignal primal not found");
    let bio_channels = bio_node["data_channels"]
        .as_array()
        .or_exit("biosignal data_channels not array");
    let has_tachogram = bio_channels.iter().any(|c| c["id"] == "rr_tachogram");
    h.check_bool("biosignal_has_rr_tachogram", has_tachogram);

    // --- Microbiome node has bar chart ---
    let micro_node = primals
        .iter()
        .find(|p| p["id"] == "microbiome")
        .or_exit("microbiome primal not found");
    let micro_channels = micro_node["data_channels"]
        .as_array()
        .or_exit("microbiome data_channels not array");
    h.check_bool(
        "microbiome_has_bar_chart",
        micro_channels[0]["channel_type"] == "bar",
    );

    h.exit();
}
