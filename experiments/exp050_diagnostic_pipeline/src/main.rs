// SPDX-License-Identifier: AGPL-3.0-or-later
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
use healthspring_barracuda::visualization::{assessment_to_scenario, scenario_to_json};

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

    // --- Male patient on TRT with gut data ---
    let mut male_trt = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    male_trt.testosterone_ng_dl = Some(450.0);
    male_trt.on_trt = true;
    male_trt.trt_months = 12.0;
    male_trt.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);

    let result = assess_patient(&male_trt);

    check!("pk_cmax_positive", result.pk.oral_cmax > 0.0);
    check!("pk_auc_positive", result.pk.oral_auc > 0.0);
    check!(
        "pk_hill_at_ec50_reasonable",
        (result.pk.hill_response_at_ec50 - 50.0).abs() < 1.0
    );
    check!(
        "pk_tmax_reasonable",
        result.pk.oral_tmax_hr > 0.5 && result.pk.oral_tmax_hr < 6.0
    );
    check!(
        "pk_allometric_cl_scaled",
        result.pk.allometric_cl > 0.1 && result.pk.allometric_cl < 1.0
    );

    check!(
        "microbiome_shannon_positive",
        result.microbiome.shannon > 0.0
    );
    check!(
        "microbiome_evenness_range",
        result.microbiome.pielou_evenness > 0.0 && result.microbiome.pielou_evenness <= 1.0
    );
    check!(
        "microbiome_resistance_range",
        result.microbiome.colonization_resistance >= 0.0
            && result.microbiome.colonization_resistance <= 1.0
    );

    check!(
        "biosignal_default_hr",
        result.biosignal.heart_rate_bpm > 50.0 && result.biosignal.heart_rate_bpm < 120.0
    );
    check!(
        "biosignal_spo2_default",
        result.biosignal.spo2_percent > 90.0 && result.biosignal.spo2_percent <= 100.0
    );
    check!(
        "biosignal_stress_range",
        result.biosignal.stress_index >= 0.0 && result.biosignal.stress_index <= 1.0
    );

    check!(
        "endocrine_testosterone_matches_input",
        (result.endocrine.predicted_testosterone - 450.0).abs() < 0.1
    );
    check!(
        "endocrine_hrv_improved_on_trt",
        result.endocrine.hrv_trt_sdnn > 50.0
    );
    check!(
        "endocrine_cardiac_risk_range",
        result.endocrine.cardiac_risk >= 0.0 && result.endocrine.cardiac_risk <= 1.0
    );
    check!(
        "endocrine_metabolic_negative_weight",
        result.endocrine.metabolic_response < 0.0
    );

    check!(
        "cross_gut_trt_positive",
        result.cross_track.gut_trt_response > 0.0
    );
    check!(
        "cross_hrv_cardiac_range",
        result.cross_track.hrv_cardiac_composite >= 0.0
            && result.cross_track.hrv_cardiac_composite <= 1.0
    );

    check!(
        "composite_risk_range",
        result.composite_risk >= 0.0 && result.composite_risk <= 1.0
    );

    // --- Female patient, minimal profile ---
    let female_min = PatientProfile::minimal(30.0, 60.0, Sex::Female);
    let result_f = assess_patient(&female_min);

    check!(
        "female_composite_valid",
        result_f.composite_risk >= 0.0 && result_f.composite_risk <= 1.0
    );
    check!("female_pk_auc_positive", result_f.pk.oral_auc > 0.0);

    // --- Enriched data checks ---
    check!("pk_curve_length", result.pk.curve_times_hr.len() == 101);
    check!(
        "pk_curve_concs_length",
        result.pk.curve_concs_mg_l.len() == 101
    );
    check!("pk_hill_sweep_length", result.pk.hill_concs.len() == 50);
    check!(
        "microbiome_abundances_passed",
        result.microbiome.abundances.len() == 7
    );

    // --- Scenario export ---
    let scenario = assessment_to_scenario(&result, "Exp050 Male TRT");
    check!(
        "scenario_nodes_count",
        scenario.ecosystem.primals.len() == 7
    );

    let json = scenario_to_json(&scenario);
    check!("json_has_name", json.contains("Exp050 Male TRT"));
    check!("json_has_version", json.contains("\"version\": \"2.0.0\""));
    check!("json_has_composite_risk", json.contains("composite_risk"));
    check!("json_has_data_channels", json.contains("data_channels"));
    check!("json_has_timeseries", json.contains("timeseries"));

    // --- Determinism ---
    let result2 = assess_patient(&male_trt);
    check!(
        "deterministic_composite",
        (result.composite_risk - result2.composite_risk).abs() < 1e-12
    );

    verify_scenario_json_structure(&json, &mut passed, &mut failed);

    println!(
        "\nExp050 Diagnostic Pipeline: {passed}/{} checks passed",
        passed + failed
    );
    if failed > 0 {
        std::process::exit(1);
    }
}

fn verify_scenario_json_structure(json: &str, passed: &mut u32, failed: &mut u32) {
    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                *passed += 1;
            } else {
                eprintln!("FAIL: {}", $name);
                *failed += 1;
            }
        };
    }

    check!("json_opens_brace", json.starts_with('{'));
    check!("json_has_ecosystem", json.contains("ecosystem"));
    check!("json_has_primals", json.contains("primals"));
    check!(
        "json_node_ids_present",
        json.contains("\"patient\"") && json.contains("\"pk\"") && json.contains("\"microbiome\"")
    );
}
