// SPDX-License-Identifier: AGPL-3.0-or-later
//! `petalTongue`-compatible scenario export for healthSpring diagnostics.
//!
//! Serializes diagnostic assessments as JSON scenario files that `petalTongue`
//! can render as interactive health topology graphs. Each organ system, biomarker
//! track, and cross-track connection becomes a node or edge in the graph.

use crate::diagnostic::{DiagnosticAssessment, PopulationResult, RiskLevel};

/// A node in the `petalTongue` scenario graph.
#[derive(Debug, Clone)]
pub struct ScenarioNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub health: u8,
    pub properties: Vec<(String, String)>,
    pub x: f64,
    pub y: f64,
}

/// An edge in the `petalTongue` scenario graph.
#[derive(Debug, Clone)]
pub struct ScenarioEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub label: String,
}

/// Complete scenario for `petalTongue` rendering.
#[derive(Debug, Clone)]
pub struct HealthScenario {
    pub name: String,
    pub mode: String,
    pub nodes: Vec<ScenarioNode>,
    pub edges: Vec<ScenarioEdge>,
}

fn risk_to_health(risk: f64) -> u8 {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let h = ((1.0 - risk.clamp(0.0, 1.0)) * 100.0) as u8;
    h
}

fn risk_level_str(level: RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "low",
        RiskLevel::Moderate => "moderate",
        RiskLevel::High => "high",
        RiskLevel::Critical => "critical",
    }
}

/// Build a `petalTongue` scenario from a single patient diagnostic assessment.
#[must_use]
pub fn assessment_to_scenario(
    assessment: &DiagnosticAssessment,
    patient_name: &str,
) -> HealthScenario {
    HealthScenario {
        name: format!("healthSpring Diagnostic: {patient_name}"),
        mode: "live-ecosystem".into(),
        nodes: build_nodes(assessment, patient_name),
        edges: build_edges(),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "declarative node construction — each node is a struct literal"
)]
fn build_nodes(a: &DiagnosticAssessment, patient_name: &str) -> Vec<ScenarioNode> {
    vec![
        ScenarioNode {
            id: "patient".into(),
            name: patient_name.into(),
            node_type: "patient".into(),
            health: risk_to_health(a.composite_risk),
            properties: vec![("composite_risk".into(), format!("{:.3}", a.composite_risk))],
            x: 400.0,
            y: 50.0,
        },
        ScenarioNode {
            id: "pk".into(),
            name: "PK/PD Engine".into(),
            node_type: "track".into(),
            health: 100,
            properties: vec![
                ("cmax".into(), format!("{:.4}", a.pk.oral_cmax)),
                ("auc".into(), format!("{:.4}", a.pk.oral_auc)),
                ("tmax_hr".into(), format!("{:.2}", a.pk.oral_tmax_hr)),
                ("cl_l_hr".into(), format!("{:.4}", a.pk.allometric_cl)),
                ("vd_l".into(), format!("{:.2}", a.pk.allometric_vd)),
            ],
            x: 150.0,
            y: 200.0,
        },
        ScenarioNode {
            id: "microbiome".into(),
            name: "Microbiome Risk".into(),
            node_type: "track".into(),
            health: risk_to_health(1.0 - a.microbiome.colonization_resistance),
            properties: vec![
                ("shannon".into(), format!("{:.4}", a.microbiome.shannon)),
                ("simpson".into(), format!("{:.4}", a.microbiome.simpson)),
                (
                    "evenness".into(),
                    format!("{:.4}", a.microbiome.pielou_evenness),
                ),
                (
                    "resistance".into(),
                    format!("{:.4}", a.microbiome.colonization_resistance),
                ),
                (
                    "risk".into(),
                    risk_level_str(a.microbiome.risk_level).into(),
                ),
            ],
            x: 350.0,
            y: 200.0,
        },
        ScenarioNode {
            id: "biosignal".into(),
            name: "Biosignal Monitor".into(),
            node_type: "track".into(),
            health: risk_to_health(a.biosignal.stress_index),
            properties: vec![
                (
                    "hr_bpm".into(),
                    format!("{:.1}", a.biosignal.heart_rate_bpm),
                ),
                ("sdnn_ms".into(), format!("{:.1}", a.biosignal.sdnn_ms)),
                ("rmssd_ms".into(), format!("{:.1}", a.biosignal.rmssd_ms)),
                ("spo2".into(), format!("{:.1}", a.biosignal.spo2_percent)),
                ("stress".into(), format!("{:.3}", a.biosignal.stress_index)),
            ],
            x: 550.0,
            y: 200.0,
        },
        ScenarioNode {
            id: "endocrine".into(),
            name: "Endocrine Outcomes".into(),
            node_type: "track".into(),
            health: risk_to_health(a.endocrine.cardiac_risk),
            properties: vec![
                (
                    "testosterone".into(),
                    format!("{:.1}", a.endocrine.predicted_testosterone),
                ),
                (
                    "hrv_sdnn".into(),
                    format!("{:.1}", a.endocrine.hrv_trt_sdnn),
                ),
                (
                    "cardiac_risk".into(),
                    format!("{:.4}", a.endocrine.cardiac_risk),
                ),
                (
                    "metabolic".into(),
                    format!("{:.2}", a.endocrine.metabolic_response),
                ),
            ],
            x: 750.0,
            y: 200.0,
        },
        ScenarioNode {
            id: "gut_trt_axis".into(),
            name: "Gut-TRT Axis".into(),
            node_type: "cross_track".into(),
            health: risk_to_health(1.0 - a.cross_track.gut_trt_response),
            properties: vec![(
                "response".into(),
                format!("{:.4}", a.cross_track.gut_trt_response),
            )],
            x: 350.0,
            y: 400.0,
        },
        ScenarioNode {
            id: "hrv_cardiac".into(),
            name: "HRV-Cardiac".into(),
            node_type: "cross_track".into(),
            health: risk_to_health(a.cross_track.hrv_cardiac_composite),
            properties: vec![(
                "composite".into(),
                format!("{:.4}", a.cross_track.hrv_cardiac_composite),
            )],
            x: 650.0,
            y: 400.0,
        },
    ]
}

fn build_edges() -> Vec<ScenarioEdge> {
    vec![
        ScenarioEdge {
            from: "patient".into(),
            to: "pk".into(),
            edge_type: "feeds".into(),
            label: "demographics".into(),
        },
        ScenarioEdge {
            from: "patient".into(),
            to: "microbiome".into(),
            edge_type: "feeds".into(),
            label: "gut sample".into(),
        },
        ScenarioEdge {
            from: "patient".into(),
            to: "biosignal".into(),
            edge_type: "feeds".into(),
            label: "ECG/PPG/EDA".into(),
        },
        ScenarioEdge {
            from: "patient".into(),
            to: "endocrine".into(),
            edge_type: "feeds".into(),
            label: "labs".into(),
        },
        ScenarioEdge {
            from: "microbiome".into(),
            to: "gut_trt_axis".into(),
            edge_type: "influences".into(),
            label: "evenness → disorder".into(),
        },
        ScenarioEdge {
            from: "endocrine".into(),
            to: "gut_trt_axis".into(),
            edge_type: "influences".into(),
            label: "TRT response".into(),
        },
        ScenarioEdge {
            from: "biosignal".into(),
            to: "hrv_cardiac".into(),
            edge_type: "influences".into(),
            label: "HRV".into(),
        },
        ScenarioEdge {
            from: "endocrine".into(),
            to: "hrv_cardiac".into(),
            edge_type: "influences".into(),
            label: "testosterone".into(),
        },
    ]
}

/// Serialize a scenario to `petalTongue`-compatible JSON.
#[must_use]
pub fn scenario_to_json(scenario: &HealthScenario) -> String {
    use std::fmt::Write;

    let mut json = String::with_capacity(4096);
    let _ = writeln!(json, "{{");
    let _ = writeln!(json, "  \"name\": \"{}\",", scenario.name);
    let _ = writeln!(json, "  \"mode\": \"{}\",", scenario.mode);
    json.push_str("  \"ecosystem\": {\n    \"primals\": [\n");

    for (i, node) in scenario.nodes.iter().enumerate() {
        write_node(&mut json, node);
        if i + 1 < scenario.nodes.len() {
            json.push(',');
        }
        json.push('\n');
    }

    json.push_str("    ],\n    \"topology\": [\n");
    for (i, edge) in scenario.edges.iter().enumerate() {
        write_edge(&mut json, edge);
        if i + 1 < scenario.edges.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("    ]\n  }\n}\n");
    json
}

fn write_node(out: &mut String, node: &ScenarioNode) {
    use std::fmt::Write;

    out.push_str("      {\n");
    let _ = writeln!(out, "        \"id\": \"{}\",", node.id);
    let _ = writeln!(out, "        \"name\": \"{}\",", node.name);
    let _ = writeln!(out, "        \"type\": \"{}\",", node.node_type);
    let _ = writeln!(out, "        \"health\": {},", node.health);
    let _ = writeln!(
        out,
        "        \"position\": {{ \"x\": {}, \"y\": {} }},",
        node.x, node.y
    );
    out.push_str("        \"properties\": {");
    for (j, (k, v)) in node.properties.iter().enumerate() {
        let _ = write!(out, " \"{k}\": \"{v}\"");
        if j + 1 < node.properties.len() {
            out.push(',');
        }
    }
    out.push_str(" }\n      }");
}

fn write_edge(out: &mut String, edge: &ScenarioEdge) {
    use std::fmt::Write;
    let _ = write!(
        out,
        "      {{ \"from\": \"{}\", \"to\": \"{}\", \"edge_type\": \"{}\", \"label\": \"{}\" }}",
        edge.from, edge.to, edge.edge_type, edge.label
    );
}

/// Append population percentile annotation to a scenario.
#[must_use]
pub fn annotate_population(mut scenario: HealthScenario, pop: &PopulationResult) -> HealthScenario {
    scenario.nodes.push(ScenarioNode {
        id: "population".into(),
        name: format!("Population (n={})", pop.n_patients),
        node_type: "population".into(),
        health: 100,
        properties: vec![
            ("mean_risk".into(), format!("{:.4}", pop.mean_risk)),
            ("std_risk".into(), format!("{:.4}", pop.std_risk)),
            (
                "patient_percentile".into(),
                format!("{:.1}", pop.patient_percentile),
            ),
        ],
        x: 400.0,
        y: 550.0,
    });
    scenario.edges.push(ScenarioEdge {
        from: "patient".into(),
        to: "population".into(),
        edge_type: "context".into(),
        label: format!("{:.0}th percentile", pop.patient_percentile),
    });
    scenario
}

#[cfg(test)]
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
        assert_eq!(scenario.nodes.len(), 7);
        assert_eq!(scenario.edges.len(), 8);
    }

    #[test]
    fn scenario_json_valid_structure() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Test Patient");
        let json = scenario_to_json(&scenario);

        assert!(json.contains("\"name\": \"healthSpring Diagnostic: Test Patient\""));
        assert!(json.contains("\"mode\": \"live-ecosystem\""));
        assert!(json.contains("\"primals\""));
        assert!(json.contains("\"topology\""));
    }

    #[test]
    fn scenario_node_health_in_range() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Patient");
        for node in &scenario.nodes {
            assert!(
                node.health <= 100,
                "node {} health {} > 100",
                node.id,
                node.health
            );
        }
    }

    #[test]
    fn population_annotation_adds_node() {
        let mut p = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        p.testosterone_ng_dl = Some(450.0);
        let assessment = assess_patient(&p);
        let pop = population_montecarlo(&p, 100, 42);

        let scenario = assessment_to_scenario(&assessment, "MC Patient");
        let annotated = annotate_population(scenario, &pop);

        assert_eq!(annotated.nodes.len(), 8);
        assert_eq!(annotated.edges.len(), 9);
        assert!(annotated.nodes.iter().any(|n| n.id == "population"));
    }

    #[test]
    fn json_roundtrip_contains_all_properties() {
        let assessment = sample_assessment();
        let scenario = assessment_to_scenario(&assessment, "Props Test");
        let json = scenario_to_json(&scenario);

        assert!(json.contains("composite_risk"));
        assert!(json.contains("shannon"));
        assert!(json.contains("hr_bpm"));
        assert!(json.contains("testosterone"));
        assert!(json.contains("response"));
    }
}
