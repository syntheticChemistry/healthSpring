// SPDX-License-Identifier: AGPL-3.0-or-later
//! `petalTongue`-compatible scenario export for healthSpring diagnostics.
//!
//! Produces two output formats:
//! 1. **Native petalTongue JSON** — loads directly in `petalTongue --scenario`
//! 2. **Enriched scenario** — native format plus `DataChannel` extensions for
//!    time-series, distributions, bar charts, and gauges. petalTongue absorbs
//!    these extensions to evolve from topology viewer to universal data UI.

pub mod capabilities;
pub mod clinical;
mod clinical_nodes;
pub mod ipc_push;
mod nodes;
pub mod scenarios;
pub mod stream;
mod types;

pub use types::*;

use crate::diagnostic::{DiagnosticAssessment, PopulationResult, RiskLevel};

/// Map a clinical risk level to a display string.
#[must_use]
pub const fn risk_level_str(level: RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "low",
        RiskLevel::Moderate => "moderate",
        RiskLevel::High => "high",
        RiskLevel::Critical => "critical",
    }
}

/// Build a full `petalTongue`-compatible scenario from a diagnostic assessment.
#[must_use]
pub fn assessment_to_scenario(
    assessment: &DiagnosticAssessment,
    patient_name: &str,
) -> HealthScenario {
    HealthScenario {
        name: format!("healthSpring Diagnostic: {patient_name}"),
        description: format!(
            "Integrated 4-track patient diagnostic with cross-track fusion for {patient_name}"
        ),
        version: "2.0.0".into(),
        mode: "live-ecosystem".into(),
        sensory_config: SensoryConfig {
            required_capabilities: CapReqs {
                outputs: vec!["visual".into()],
                inputs: vec![],
            },
            optional_capabilities: CapReqs {
                outputs: vec!["audio".into()],
                inputs: vec!["pointer".into(), "keyboard".into()],
            },
            complexity_hint: "standard".into(),
        },
        ui_config: UiConfig {
            theme: "benchtop-dark".into(),
            animations: Animations {
                enabled: true,
                breathing_nodes: true,
                connection_pulses: true,
                smooth_transitions: true,
                celebration_effects: false,
            },
            performance: Performance {
                target_fps: 60,
                vsync: true,
                hardware_acceleration: true,
            },
            show_panels: None,
            awakening_enabled: None,
            initial_zoom: None,
        },
        ecosystem: Ecosystem {
            primals: nodes::build_nodes(assessment, patient_name),
        },
        neural_api: NeuralApi { enabled: false },
        edges: Vec::new(),
    }
}

/// Serialize a scenario to JSON via serde.
///
/// # Panics
/// Cannot panic — all scenario types implement `Serialize` deterministically.
#[must_use]
pub fn scenario_to_json(scenario: &HealthScenario) -> String {
    serde_json::to_string_pretty(scenario)
        .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

/// Append population Monte Carlo annotation to a scenario.
#[must_use]
pub fn annotate_population(mut scenario: HealthScenario, pop: &PopulationResult) -> HealthScenario {
    let pop_node = ScenarioNode {
        id: "population".into(),
        name: format!("Population (n={})", pop.n_patients),
        node_type: "storage".into(),
        family: "healthspring".into(),
        status: "healthy".into(),
        health: 100,
        confidence: 99,
        position: None,
        capabilities: vec!["science.diagnostic.population_montecarlo".into()],
        data_channels: vec![
            DataChannel::Distribution {
                id: "risk_distribution".into(),
                label: "Population Risk Distribution".into(),
                unit: "composite risk".into(),
                values: pop.composite_risks.clone(),
                mean: pop.mean_risk,
                std: pop.std_risk,
                patient_value: pop.patient_percentile / 100.0,
            },
            DataChannel::Gauge {
                id: "percentile".into(),
                label: "Patient Percentile".into(),
                value: pop.patient_percentile,
                min: 0.0,
                max: 100.0,
                unit: "th".into(),
                normal_range: [25.0, 75.0],
                warning_range: [10.0, 25.0],
            },
        ],
        clinical_ranges: vec![],
    };

    scenario.ecosystem.primals.push(pop_node);
    scenario
}

/// Build the complete enriched scenario (nodes + edges) as a JSON string.
/// This is the primary export for the standalone UI and petalTongue integration.
///
/// # Panics
/// Cannot panic — all types implement `Serialize` deterministically.
#[must_use]
pub fn full_scenario_json(
    assessment: &DiagnosticAssessment,
    pop: &PopulationResult,
    patient_name: &str,
) -> String {
    let scenario = assessment_to_scenario(assessment, patient_name);
    let mut annotated = annotate_population(scenario, pop);
    annotated.edges = nodes::build_edges();
    serde_json::to_string_pretty(&annotated)
        .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::diagnostic::{PatientProfile, Sex, assess_patient, population_montecarlo};

    fn sample_assessment() -> DiagnosticAssessment {
        let mut p = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        p.testosterone_ng_dl = Some(450.0);
        p.on_trt = true;
        p.trt_months = 12.0;
        p.gut_abundances = Some(vec![0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02]);
        assess_patient(&p)
    }

    #[test]
    fn scenario_has_correct_node_count() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Test Patient");
        assert_eq!(scenario.ecosystem.primals.len(), 7);
    }

    #[test]
    fn scenario_json_valid_structure() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Test Patient");
        let json = scenario_to_json(&scenario);

        assert!(json.contains("\"name\": \"healthSpring Diagnostic: Test Patient\""));
        assert!(json.contains("\"mode\": \"live-ecosystem\""));
        assert!(json.contains("\"version\": \"2.0.0\""));
        assert!(json.contains("\"primals\""));
        assert!(json.contains("\"sensory_config\""));

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["ecosystem"]["primals"].is_array());
    }

    #[test]
    fn scenario_node_health_in_range() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Patient");
        for node in &scenario.ecosystem.primals {
            assert!(
                node.health <= 100,
                "node {} health {} > 100",
                node.id,
                node.health
            );
        }
    }

    #[test]
    fn population_annotation_adds_node_and_distribution() {
        let mut p = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        p.testosterone_ng_dl = Some(450.0);
        let assessment = assess_patient(&p);
        let pop = population_montecarlo(&p, 100, 42);

        let scenario = assessment_to_scenario(&assessment, "MC Patient");
        let annotated = annotate_population(scenario, &pop);

        assert_eq!(annotated.ecosystem.primals.len(), 8);
        let pop_node = annotated
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "population")
            .unwrap();
        assert_eq!(pop_node.data_channels.len(), 2);
    }

    #[test]
    fn pk_node_has_timeseries_channels() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "PK Test");
        let pk_node = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "pk")
            .unwrap();

        assert!(pk_node.data_channels.len() >= 2);
        let json = serde_json::to_string(&pk_node.data_channels[0]).unwrap();
        assert!(json.contains("\"channel_type\":\"timeseries\""));
    }

    #[test]
    fn full_scenario_json_roundtrips() {
        let assessment = sample_assessment();
        let pop = population_montecarlo(&PatientProfile::minimal(55.0, 85.0, Sex::Male), 50, 42);
        let json = full_scenario_json(&assessment, &pop, "Roundtrip");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["edges"].is_array());
        assert!(parsed["ecosystem"]["primals"].is_array());
    }

    #[test]
    fn microbiome_has_bar_channel() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Bar Test");
        let micro = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "microbiome")
            .unwrap();
        let json = serde_json::to_string(&micro.data_channels[0]).unwrap();
        assert!(json.contains("\"channel_type\":\"bar\""));
    }
}
