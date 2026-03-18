// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! healthSpring Exp063 — Clinical TRT Scenario Validation
//!
//! Validates that patient-parameterized TRT scenarios produce correct
//! structure, meaningful data ranges, and protocol-appropriate results
//! for multiple patient archetypes.

use healthspring_barracuda::validation::ValidationHarness;
use healthspring_barracuda::visualization::DataChannel;
use healthspring_barracuda::visualization::clinical::{
    PatientTrtProfile, TrtProtocol, trt_clinical_json, trt_clinical_scenario,
};

#[expect(
    clippy::too_many_lines,
    clippy::collapsible_if,
    reason = "validation binary — patient archetype TRT scenario checks"
)]
fn main() {
    let mut h = ValidationHarness::new("exp063_clinical_trt_scenarios");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp063: Clinical TRT Scenarios (Rust)");
    println!("{}", "=".repeat(72));

    // Patient archetypes
    let patients = vec![
        {
            let mut p =
                PatientTrtProfile::new("Young Low-T", 35.0, 180.0, 250.0, TrtProtocol::ImWeekly);
            p.gut_diversity = Some(0.85);
            p.hba1c = Some(5.8);
            p.sdnn_ms = Some(45.0);
            p
        },
        {
            let mut p = PatientTrtProfile::new(
                "Middle-Aged Obese",
                52.0,
                280.0,
                220.0,
                TrtProtocol::Pellet,
            );
            p.gut_diversity = Some(0.40);
            p.hba1c = Some(7.8);
            p.sdnn_ms = Some(28.0);
            p
        },
        {
            let mut p =
                PatientTrtProfile::new("Senior Lean", 68.0, 170.0, 310.0, TrtProtocol::ImBiweekly);
            p.gut_diversity = Some(0.70);
            p.sdnn_ms = Some(32.0);
            p
        },
        PatientTrtProfile::new("Minimal Input", 45.0, 200.0, 280.0, TrtProtocol::Pellet),
    ];

    for patient in &patients {
        println!("\n--- Patient: {} ---", patient.name);
        let (scenario, edges) = trt_clinical_scenario(patient);

        // Check 1: 8 nodes
        h.check_exact(
            &format!("{}: 8 nodes", patient.name),
            scenario.ecosystem.primals.len() as u64,
            8,
        );

        // Check 2: 8 edges
        h.check_exact(&format!("{}: 8 edges", patient.name), edges.len() as u64, 8);

        // Check 3: every node has data channels
        let all_have_data = scenario
            .ecosystem
            .primals
            .iter()
            .all(|n| !n.data_channels.is_empty());
        h.check_bool(
            &format!("{}: all nodes have data", patient.name),
            all_have_data,
        );

        // Check 4: every node has clinical ranges
        let all_have_ranges = scenario
            .ecosystem
            .primals
            .iter()
            .all(|n| !n.clinical_ranges.is_empty());
        h.check_bool(
            &format!("{}: all nodes have ranges", patient.name),
            all_have_ranges,
        );

        // Check 5: scenario name contains patient name
        h.check_bool(
            &format!("{}: name personalized", patient.name),
            scenario.name.contains(&patient.name),
        );

        // Check 6: assessment node baseline T matches patient
        let assess = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "assessment");
        h.check_bool(
            &format!("{}: assessment node found", patient.name),
            assess.is_some(),
        );
        let Some(assess) = assess else {
            continue;
        };
        let baseline_gauge = assess.data_channels.iter().find_map(|ch| {
            if let DataChannel::Gauge { id, value, .. } = ch {
                if id == "baseline_t" {
                    return Some(*value);
                }
            }
            None
        });
        if let Some(val) = baseline_gauge {
            h.check_abs(
                &format!("{}: baseline T correct", patient.name),
                val,
                patient.baseline_t_ng_dl,
                0.01,
            );
        } else {
            h.check_bool(
                &format!("{}: baseline_t gauge present", patient.name),
                false,
            );
        }

        // Check 7: protocol node name matches protocol type
        let prot = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "protocol");
        h.check_bool(
            &format!("{}: protocol node found", patient.name),
            prot.is_some(),
        );
        let Some(prot) = prot else {
            continue;
        };
        let matches = match patient.protocol {
            TrtProtocol::ImWeekly => prot.name.contains("Weekly"),
            TrtProtocol::ImBiweekly => prot.name.contains("Biweekly"),
            TrtProtocol::Pellet => prot.name.contains("Pellet"),
        };
        h.check_bool(&format!("{}: protocol matches", patient.name), matches);

        // Check 8: population distribution has 100 values
        let pop = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "population");
        h.check_bool(
            &format!("{}: population node found", patient.name),
            pop.is_some(),
        );
        let Some(pop) = pop else {
            continue;
        };
        let has_100 = pop.data_channels.iter().any(|ch| {
            if let DataChannel::Distribution { values, .. } = ch {
                values.len() == 100
            } else {
                false
            }
        });
        h.check_bool(&format!("{}: population n=100", patient.name), has_100);

        // Check 9: JSON roundtrip
        let json = trt_clinical_json(patient);
        let json_ok = serde_json::from_str::<serde_json::Value>(&json)
            .is_ok_and(|v| v["ecosystem"]["primals"].is_array() && v["edges"].is_array());
        h.check_bool(&format!("{}: JSON valid", patient.name), json_ok);

        // Check 10: all timeseries have non-empty x/y of equal length
        let ts_ok = scenario.ecosystem.primals.iter().all(|n| {
            n.data_channels.iter().all(|ch| {
                if let DataChannel::TimeSeries {
                    x_values, y_values, ..
                } = ch
                {
                    !x_values.is_empty() && x_values.len() == y_values.len()
                } else {
                    true
                }
            })
        });
        h.check_bool(&format!("{}: timeseries valid", patient.name), ts_ok);
    }

    // Cross-patient checks
    println!("\n--- Cross-Patient Validation ---");

    // Check: heavier patient gets larger pellet dose
    let heavy = PatientTrtProfile::new("Heavy", 50.0, 300.0, 280.0, TrtProtocol::Pellet);
    let light = PatientTrtProfile::new("Light", 50.0, 150.0, 280.0, TrtProtocol::Pellet);
    let (sh, _) = trt_clinical_scenario(&heavy);
    let (sl, _) = trt_clinical_scenario(&light);
    let pellet_ok = match (
        sh.ecosystem.primals.iter().find(|n| n.id == "protocol"),
        sl.ecosystem.primals.iter().find(|n| n.id == "protocol"),
    ) {
        (Some(ph), Some(pl)) => ph.name.contains("3000") && pl.name.contains("1500"),
        _ => false,
    };
    h.check_bool("pellet dose scales", pellet_ok);

    // Check: low baseline T → critical assessment
    let low_t = PatientTrtProfile::new("LowT", 60.0, 200.0, 180.0, TrtProtocol::ImWeekly);
    let (s_low, _) = trt_clinical_scenario(&low_t);
    let low_t_ok = s_low
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "assessment")
        .is_some_and(|a_low| a_low.status == "critical" && a_low.health <= 50);
    h.check_bool("low T flags critical", low_t_ok);

    // Check: high vs low gut diversity → different response predictions
    let high_gut = {
        let mut p = PatientTrtProfile::new("HG", 50.0, 200.0, 280.0, TrtProtocol::Pellet);
        p.gut_diversity = Some(0.95);
        p
    };
    let low_gut = {
        let mut p = PatientTrtProfile::new("LG", 50.0, 200.0, 280.0, TrtProtocol::Pellet);
        p.gut_diversity = Some(0.20);
        p
    };
    let (scenario_high_gut, _) = trt_clinical_scenario(&high_gut);
    let (scenario_low_gut, _) = trt_clinical_scenario(&low_gut);
    let get_resp = |n: &healthspring_barracuda::visualization::ScenarioNode| -> f64 {
        n.data_channels
            .iter()
            .find_map(|ch| {
                if let DataChannel::Gauge { id, value, .. } = ch {
                    if id == "patient_response" {
                        return Some(*value);
                    }
                }
                None
            })
            .unwrap_or(0.0)
    };
    let gut_mod_ok = match (
        scenario_high_gut
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "gut_health"),
        scenario_low_gut
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "gut_health"),
    ) {
        (Some(gh), Some(gl)) => get_resp(gh) > get_resp(gl),
        _ => false,
    };
    h.check_bool("gut diversity modulates", gut_mod_ok);

    println!("\n{}", "=".repeat(72));
    h.exit();
}
