// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endocrine visualization nodes.

use crate::diagnostic::DiagnosticAssessment;
use crate::visualization::types::{ClinicalRange, DataChannel, ScenarioNode};

use super::health_to_status;

pub(super) fn build_endocrine_node(a: &DiagnosticAssessment, health: u8) -> ScenarioNode {
    ScenarioNode {
        id: "endocrine".into(),
        name: "Endocrine Outcomes".into(),
        node_type: "compute".into(),
        family: "healthspring".into(),
        status: health_to_status(health).into(),
        health,
        confidence: 97,
        position: None,
        capabilities: vec!["science.endocrine.testosterone_pk".into()],
        data_channels: vec![
            DataChannel::Gauge {
                id: "testosterone".into(),
                label: "Testosterone".into(),
                value: a.endocrine.predicted_testosterone,
                min: 0.0,
                max: 1200.0,
                unit: "ng/dL".into(),
                normal_range: [300.0, 1000.0],
                warning_range: [200.0, 300.0],
            },
            DataChannel::Gauge {
                id: "cardiac_risk".into(),
                label: "Cardiac Risk".into(),
                value: a.endocrine.cardiac_risk * 100.0,
                min: 0.0,
                max: 30.0,
                unit: "%".into(),
                normal_range: [0.0, 5.0],
                warning_range: [5.0, 15.0],
            },
            DataChannel::Gauge {
                id: "metabolic".into(),
                label: "Weight Change".into(),
                value: a.endocrine.metabolic_response,
                min: -15.0,
                max: 5.0,
                unit: "kg".into(),
                normal_range: [-10.0, 2.0],
                warning_range: [-15.0, -10.0],
            },
        ],
        clinical_ranges: vec![
            ClinicalRange {
                label: "T normal male".into(),
                min: 300.0,
                max: 1000.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "T low".into(),
                min: 0.0,
                max: 300.0,
                status: "warning".into(),
            },
        ],
    }
}
