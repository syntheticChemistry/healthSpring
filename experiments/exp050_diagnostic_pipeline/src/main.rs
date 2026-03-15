// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! Exp050: Integrated diagnostic pipeline validation.
//!
//! Runs a synthetic patient through all four tracks (PK/PD, microbiome,
//! biosignal, endocrine) plus cross-track models, verifying that the
//! composed diagnostic produces physiologically reasonable results and
//! the `petalTongue` scenario export generates valid JSON.

use healthspring_barracuda::diagnostic::{PatientProfile, Sex, assess_patient};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;
use healthspring_barracuda::visualization::{assessment_to_scenario, scenario_to_json};

fn main() {
    let mut h = ValidationHarness::new("exp050_diagnostic_pipeline");

    // --- Male patient on TRT with gut data ---
    let mut male_trt = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    male_trt.testosterone_ng_dl = Some(450.0);
    male_trt.on_trt = true;
    male_trt.trt_months = 12.0;
    male_trt.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);

    let result = assess_patient(&male_trt);

    h.check_bool("pk_cmax_positive", result.pk.oral_cmax > 0.0);
    h.check_bool("pk_auc_positive", result.pk.oral_auc > 0.0);
    h.check_abs(
        "pk_hill_at_ec50_reasonable",
        result.pk.hill_response_at_ec50,
        50.0,
        tolerances::HILL_AT_EC50,
    );
    h.check_bool(
        "pk_tmax_reasonable",
        result.pk.oral_tmax_hr > 0.5 && result.pk.oral_tmax_hr < 6.0,
    );
    h.check_bool(
        "pk_allometric_cl_scaled",
        result.pk.allometric_cl > 0.1 && result.pk.allometric_cl < 1.0,
    );

    h.check_bool(
        "microbiome_shannon_positive",
        result.microbiome.shannon > 0.0,
    );
    h.check_bool(
        "microbiome_evenness_range",
        result.microbiome.pielou_evenness > 0.0 && result.microbiome.pielou_evenness <= 1.0,
    );
    h.check_bool(
        "microbiome_resistance_range",
        result.microbiome.colonization_resistance >= 0.0
            && result.microbiome.colonization_resistance <= 1.0,
    );

    h.check_bool(
        "biosignal_default_hr",
        result.biosignal.heart_rate_bpm > 50.0 && result.biosignal.heart_rate_bpm < 120.0,
    );
    h.check_bool(
        "biosignal_spo2_default",
        result.biosignal.spo2_percent > 90.0 && result.biosignal.spo2_percent <= 100.0,
    );
    h.check_bool(
        "biosignal_stress_range",
        result.biosignal.stress_index >= 0.0 && result.biosignal.stress_index <= 1.0,
    );

    h.check_abs(
        "endocrine_testosterone_matches_input",
        result.endocrine.predicted_testosterone,
        450.0,
        0.1,
    );
    h.check_bool(
        "endocrine_hrv_improved_on_trt",
        result.endocrine.hrv_trt_sdnn > 50.0,
    );
    h.check_bool(
        "endocrine_cardiac_risk_range",
        result.endocrine.cardiac_risk >= 0.0 && result.endocrine.cardiac_risk <= 1.0,
    );
    h.check_bool(
        "endocrine_metabolic_negative_weight",
        result.endocrine.metabolic_response < 0.0,
    );

    h.check_bool(
        "cross_gut_trt_positive",
        result.cross_track.gut_trt_response > 0.0,
    );
    h.check_bool(
        "cross_hrv_cardiac_range",
        result.cross_track.hrv_cardiac_composite >= 0.0
            && result.cross_track.hrv_cardiac_composite <= 1.0,
    );

    h.check_bool(
        "composite_risk_range",
        result.composite_risk >= 0.0 && result.composite_risk <= 1.0,
    );

    // --- Female patient, minimal profile ---
    let female_min = PatientProfile::minimal(30.0, 60.0, Sex::Female);
    let result_f = assess_patient(&female_min);

    h.check_bool(
        "female_composite_valid",
        result_f.composite_risk >= 0.0 && result_f.composite_risk <= 1.0,
    );
    h.check_bool("female_pk_auc_positive", result_f.pk.oral_auc > 0.0);

    // --- Enriched data checks ---
    h.check_exact(
        "pk_curve_length",
        result.pk.curve_times_hr.len() as u64,
        101,
    );
    h.check_exact(
        "pk_curve_concs_length",
        result.pk.curve_concs_mg_l.len() as u64,
        101,
    );
    h.check_exact(
        "pk_hill_sweep_length",
        result.pk.hill_concs.len() as u64,
        50,
    );
    h.check_exact(
        "microbiome_abundances_passed",
        result.microbiome.abundances.len() as u64,
        7,
    );

    // --- Scenario export ---
    let scenario = assessment_to_scenario(&result, "Exp050 Male TRT");
    h.check_exact(
        "scenario_nodes_count",
        scenario.ecosystem.primals.len() as u64,
        7,
    );

    let json = scenario_to_json(&scenario);
    h.check_bool("json_has_name", json.contains("Exp050 Male TRT"));
    h.check_bool("json_has_version", json.contains("\"version\": \"2.0.0\""));
    h.check_bool("json_has_composite_risk", json.contains("composite_risk"));
    h.check_bool("json_has_data_channels", json.contains("data_channels"));
    h.check_bool("json_has_timeseries", json.contains("timeseries"));

    // --- Determinism ---
    let result2 = assess_patient(&male_trt);
    h.check_abs(
        "deterministic_composite",
        result.composite_risk,
        result2.composite_risk,
        tolerances::DETERMINISM,
    );

    verify_scenario_json_structure(&json, &mut h);

    h.exit();
}

fn verify_scenario_json_structure(json: &str, h: &mut ValidationHarness) {
    h.check_bool("json_opens_brace", json.starts_with('{'));
    h.check_bool("json_has_ecosystem", json.contains("ecosystem"));
    h.check_bool("json_has_primals", json.contains("primals"));
    h.check_bool(
        "json_node_ids_present",
        json.contains("\"patient\"") && json.contains("\"pk\"") && json.contains("\"microbiome\""),
    );
}
