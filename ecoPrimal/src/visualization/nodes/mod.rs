// SPDX-License-Identifier: AGPL-3.0-or-later
//! Node and edge construction for the healthSpring diagnostic scenario.
//!
//! Converts `DiagnosticAssessment` data into `ScenarioNode` and `ScenarioEdge`
//! collections for petalTongue rendering.

mod biosignal;
mod endocrine;
mod microbiome;
mod pkpd;

use biosignal::build_biosignal_node;
use endocrine::build_endocrine_node;
use microbiome::build_microbiome_node;
use pkpd::build_pk_node;

use crate::diagnostic::DiagnosticAssessment;

use super::types::{DataChannel, ScenarioEdge, ScenarioNode};
use crate::PRIMAL_NAME;

pub(super) fn risk_to_health(risk: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped [0,100] fits u8"
    )]
    let h = ((1.0 - risk.clamp(0.0, 1.0)) * 100.0) as u8;
    h
}

pub(super) const fn health_to_status(health: u8) -> &'static str {
    if health >= 90 {
        "healthy"
    } else if health >= 50 {
        "warning"
    } else {
        "critical"
    }
}

pub(super) fn build_nodes(a: &DiagnosticAssessment, patient_name: &str) -> Vec<ScenarioNode> {
    let patient_health = risk_to_health(a.composite_risk);
    let micro_health = risk_to_health(1.0 - a.microbiome.colonization_resistance);
    let bio_health = risk_to_health(a.biosignal.stress_index);
    let endo_health = risk_to_health(a.endocrine.cardiac_risk);
    let gut_health = risk_to_health(1.0 - a.cross_track.gut_trt_response);
    let hrv_health = risk_to_health(a.cross_track.hrv_cardiac_composite);

    vec![
        ScenarioNode {
            id: "patient".into(),
            name: patient_name.into(),
            node_type: "patient".into(),
            family: PRIMAL_NAME.into(),
            status: health_to_status(patient_health).into(),
            health: patient_health,
            confidence: 95,
            position: None,
            capabilities: vec!["science.diagnostic.assess_patient".into()],
            data_channels: vec![DataChannel::Gauge {
                id: "composite_risk".into(),
                label: "Composite Risk".into(),
                value: a.composite_risk * 100.0,
                min: 0.0,
                max: 100.0,
                unit: "%".into(),
                normal_range: [0.0, 25.0],
                warning_range: [25.0, 50.0],
            }],
            clinical_ranges: vec![],
        },
        build_pk_node(a),
        build_microbiome_node(a, micro_health),
        build_biosignal_node(a, bio_health),
        build_endocrine_node(a, endo_health),
        ScenarioNode {
            id: "gut-trt-axis".into(),
            name: "Gut\u{2013}TRT Axis".into(),
            node_type: "discovery".into(),
            family: PRIMAL_NAME.into(),
            status: health_to_status(gut_health).into(),
            health: gut_health,
            confidence: 80,
            position: None,
            capabilities: vec!["science.cross_track.gut_metabolic_response".into()],
            data_channels: vec![DataChannel::Gauge {
                id: "gut_trt_response".into(),
                label: "Gut-TRT Response".into(),
                value: a.cross_track.gut_trt_response * 100.0,
                min: 0.0,
                max: 100.0,
                unit: "%".into(),
                normal_range: [60.0, 100.0],
                warning_range: [30.0, 60.0],
            }],
            clinical_ranges: vec![],
        },
        ScenarioNode {
            id: "hrv-cardiac".into(),
            name: "HRV\u{2013}Cardiac".into(),
            node_type: "discovery".into(),
            family: PRIMAL_NAME.into(),
            status: health_to_status(hrv_health).into(),
            health: hrv_health,
            confidence: 95,
            position: None,
            capabilities: vec!["science.cross_track.hrv_cardiac_composite".into()],
            data_channels: vec![DataChannel::Gauge {
                id: "hrv_cardiac_composite".into(),
                label: "HRV-Cardiac Composite".into(),
                value: a.cross_track.hrv_cardiac_composite * 100.0,
                min: 0.0,
                max: 30.0,
                unit: "%".into(),
                normal_range: [0.0, 5.0],
                warning_range: [5.0, 15.0],
            }],
            clinical_ranges: vec![],
        },
    ]
}

pub(super) fn build_edges() -> Vec<ScenarioEdge> {
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
            to: "gut-trt-axis".into(),
            edge_type: "influences".into(),
            label: "evenness".into(),
        },
        ScenarioEdge {
            from: "endocrine".into(),
            to: "gut-trt-axis".into(),
            edge_type: "influences".into(),
            label: "TRT response".into(),
        },
        ScenarioEdge {
            from: "biosignal".into(),
            to: "hrv-cardiac".into(),
            edge_type: "influences".into(),
            label: "HRV".into(),
        },
        ScenarioEdge {
            from: "endocrine".into(),
            to: "hrv-cardiac".into(),
            edge_type: "influences".into(),
            label: "testosterone".into(),
        },
    ]
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions use expect for clarity")]
mod tests {
    use super::{build_edges, build_nodes, health_to_status, risk_to_health};
    use crate::diagnostic::{PatientProfile, Sex, assess_patient};
    use crate::visualization::DataChannel;

    fn sample_assessment() -> crate::diagnostic::DiagnosticAssessment {
        let mut p = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        p.testosterone_ng_dl = Some(450.0);
        p.on_trt = true;
        p.trt_months = 12.0;
        p.gut_abundances = Some(vec![0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02]);
        assess_patient(&p)
    }

    #[test]
    fn risk_to_health_bounds() {
        assert_eq!(risk_to_health(0.0), 100);
        assert_eq!(risk_to_health(1.0), 0);
        assert_eq!(risk_to_health(0.5), 50);
    }

    #[test]
    fn risk_to_health_clamps() {
        assert_eq!(risk_to_health(-0.5), 100);
        assert_eq!(risk_to_health(2.0), 0);
    }

    #[test]
    fn health_to_status_thresholds() {
        assert_eq!(health_to_status(100), "healthy");
        assert_eq!(health_to_status(90), "healthy");
        assert_eq!(health_to_status(89), "warning");
        assert_eq!(health_to_status(50), "warning");
        assert_eq!(health_to_status(49), "critical");
        assert_eq!(health_to_status(0), "critical");
    }

    #[test]
    fn build_nodes_count_and_ids() {
        let assessment = sample_assessment();
        let nodes = build_nodes(&assessment, "TestPatient");
        assert_eq!(nodes.len(), 7);
        let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"patient"));
        assert!(ids.contains(&"pk"));
        assert!(ids.contains(&"microbiome"));
        assert!(ids.contains(&"biosignal"));
        assert!(ids.contains(&"endocrine"));
        assert!(ids.contains(&"gut-trt-axis"));
        assert!(ids.contains(&"hrv-cardiac"));
    }

    #[test]
    fn build_nodes_patient_name() {
        let assessment = sample_assessment();
        let nodes = build_nodes(&assessment, "Alice");
        let patient = nodes
            .iter()
            .find(|n| n.id == "patient")
            .expect("patient node");
        assert_eq!(patient.name, "Alice");
    }

    #[test]
    fn build_edges_count_and_structure() {
        let edges = build_edges();
        assert_eq!(edges.len(), 8);
        assert_eq!(
            edges.iter().filter(|e| e.from == "patient").count(),
            4,
            "patient feeds 4 nodes"
        );
    }

    #[test]
    fn build_nodes_data_channels_serializable() {
        let assessment = sample_assessment();
        let nodes = build_nodes(&assessment, "Serial");
        for node in &nodes {
            let json = serde_json::to_string(&node.data_channels).expect("serialize");
            let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
            assert!(parsed.is_array());
        }
    }

    #[test]
    fn data_channel_gauge_roundtrip() {
        let ch = DataChannel::Gauge {
            id: "test".into(),
            label: "Test Gauge".into(),
            value: 50.0,
            min: 0.0,
            max: 100.0,
            unit: "%".into(),
            normal_range: [20.0, 80.0],
            warning_range: [10.0, 90.0],
        };
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"gauge\""));
        assert!(json.contains("\"value\":50"));
    }

    #[test]
    fn data_channel_timeseries_roundtrip() {
        let ch = DataChannel::TimeSeries {
            id: "ts".into(),
            label: "Time Series".into(),
            x_label: "Time".into(),
            y_label: "Value".into(),
            unit: "mg/L".into(),
            x_values: vec![0.0, 1.0, 2.0],
            y_values: vec![1.0, 2.0, 4.0],
        };
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"timeseries\""));
    }
}
