// SPDX-License-Identifier: AGPL-3.0-or-later
//! PK/PD visualization nodes.

use crate::diagnostic::DiagnosticAssessment;
use crate::visualization::types::{ClinicalRange, DataChannel, ScenarioNode};

pub(super) fn build_pk_node(a: &DiagnosticAssessment) -> ScenarioNode {
    ScenarioNode {
        id: "pk".into(),
        name: "PK/PD Engine".into(),
        node_type: "compute".into(),
        family: "healthspring".into(),
        status: "healthy".into(),
        health: 100,
        confidence: 100,
        position: None,
        capabilities: vec![
            "science.pkpd.one_compartment_pk".into(),
            "science.pkpd.hill_dose_response".into(),
        ],
        data_channels: vec![
            DataChannel::TimeSeries {
                id: "pk_curve".into(),
                label: "Oral PK Concentration".into(),
                x_label: "Time (hr)".into(),
                y_label: "Concentration (mg/L)".into(),
                unit: "mg/L".into(),
                x_values: a.pk.curve_times_hr.clone(),
                y_values: a.pk.curve_concs_mg_l.clone(),
            },
            DataChannel::TimeSeries {
                id: "hill_curve".into(),
                label: "Hill Dose-Response".into(),
                x_label: "Concentration".into(),
                y_label: "Response (%)".into(),
                unit: "%".into(),
                x_values: a.pk.hill_concs.clone(),
                y_values: a.pk.hill_responses.clone(),
            },
            DataChannel::Gauge {
                id: "cmax".into(),
                label: "Cmax".into(),
                value: a.pk.oral_cmax,
                min: 0.0,
                max: 0.5,
                unit: "mg/L".into(),
                normal_range: [0.05, 0.3],
                warning_range: [0.3, 0.5],
            },
            DataChannel::Gauge {
                id: "auc".into(),
                label: "AUC\u{2080}\u{208b}\u{2082}\u{2084}".into(),
                value: a.pk.oral_auc,
                min: 0.0,
                max: 10.0,
                unit: "mg\u{b7}hr/L".into(),
                normal_range: [1.0, 6.0],
                warning_range: [6.0, 10.0],
            },
        ],
        clinical_ranges: vec![
            ClinicalRange {
                label: "Cmax therapeutic".into(),
                min: 0.05,
                max: 0.3,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Cmax high".into(),
                min: 0.3,
                max: 0.5,
                status: "warning".into(),
            },
        ],
    }
}
