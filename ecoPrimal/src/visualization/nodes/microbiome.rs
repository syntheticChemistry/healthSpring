// SPDX-License-Identifier: AGPL-3.0-or-later
//! Microbiome visualization nodes.

use crate::diagnostic::DiagnosticAssessment;
use crate::visualization::types::{ClinicalRange, DataChannel, ScenarioNode};

use super::health_to_status;
use crate::PRIMAL_NAME;

pub(super) fn build_microbiome_node(a: &DiagnosticAssessment, health: u8) -> ScenarioNode {
    ScenarioNode {
        id: "microbiome".into(),
        name: "Microbiome Risk".into(),
        node_type: "data".into(),
        family: PRIMAL_NAME.into(),
        status: health_to_status(health).into(),
        health,
        confidence: 88,
        position: None,
        capabilities: vec!["science.microbiome.shannon_index".into()],
        data_channels: vec![
            DataChannel::Bar {
                id: "gut_abundances".into(),
                label: "Genus Relative Abundance".into(),
                categories: (0..a.microbiome.abundances.len())
                    .map(|i| format!("Genus {}", i + 1))
                    .collect(),
                values: a.microbiome.abundances.clone(),
                unit: "relative".into(),
            },
            DataChannel::Gauge {
                id: "shannon".into(),
                label: "Shannon H'".into(),
                value: a.microbiome.shannon,
                min: 0.0,
                max: 4.0,
                unit: "nats".into(),
                normal_range: [2.5, 4.0],
                warning_range: [1.5, 2.5],
            },
            DataChannel::Gauge {
                id: "colonization_resistance".into(),
                label: "Colonization Resistance".into(),
                value: a.microbiome.colonization_resistance,
                min: 0.0,
                max: 1.0,
                unit: String::new(),
                normal_range: [0.7, 1.0],
                warning_range: [0.4, 0.7],
            },
        ],
        clinical_ranges: vec![
            ClinicalRange {
                label: "Shannon healthy".into(),
                min: 2.5,
                max: 4.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Shannon dysbiotic".into(),
                min: 0.0,
                max: 1.5,
                status: "critical".into(),
            },
        ],
    }
}
