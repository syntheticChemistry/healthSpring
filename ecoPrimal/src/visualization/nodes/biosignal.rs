// SPDX-License-Identifier: AGPL-3.0-or-later
//! Biosignal visualization nodes.

use crate::diagnostic::DiagnosticAssessment;
use crate::visualization::types::{ClinicalRange, DataChannel, ScenarioNode};

use super::health_to_status;

pub(super) fn build_biosignal_node(a: &DiagnosticAssessment, health: u8) -> ScenarioNode {
    ScenarioNode {
        id: "biosignal".into(),
        name: "Biosignal Monitor".into(),
        node_type: "compute".into(),
        family: "healthspring".into(),
        status: health_to_status(health).into(),
        health,
        confidence: 92,
        position: None,
        capabilities: vec![
            "science.biosignal.pan_tompkins".into(),
            "science.biosignal.fuse_channels".into(),
        ],
        data_channels: build_biosignal_channels(a),
        clinical_ranges: vec![
            ClinicalRange {
                label: "HR normal".into(),
                min: 60.0,
                max: 100.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "SpO2 normal".into(),
                min: 95.0,
                max: 100.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "SDNN healthy".into(),
                min: 50.0,
                max: 200.0,
                status: "normal".into(),
            },
        ],
    }
}

fn build_biosignal_channels(a: &DiagnosticAssessment) -> Vec<DataChannel> {
    let mut channels = vec![
        DataChannel::Gauge {
            id: "heart_rate".into(),
            label: "Heart Rate".into(),
            value: a.biosignal.heart_rate_bpm,
            min: 40.0,
            max: 140.0,
            unit: "bpm".into(),
            normal_range: [60.0, 100.0],
            warning_range: [40.0, 60.0],
        },
        DataChannel::Gauge {
            id: "spo2".into(),
            label: "SpO\u{2082}".into(),
            value: a.biosignal.spo2_percent,
            min: 80.0,
            max: 100.0,
            unit: "%".into(),
            normal_range: [95.0, 100.0],
            warning_range: [90.0, 95.0],
        },
        DataChannel::Gauge {
            id: "sdnn".into(),
            label: "SDNN".into(),
            value: a.biosignal.sdnn_ms,
            min: 0.0,
            max: 200.0,
            unit: "ms".into(),
            normal_range: [50.0, 200.0],
            warning_range: [20.0, 50.0],
        },
        DataChannel::Gauge {
            id: "stress".into(),
            label: "Stress Index".into(),
            value: a.biosignal.stress_index * 100.0,
            min: 0.0,
            max: 100.0,
            unit: "%".into(),
            normal_range: [0.0, 30.0],
            warning_range: [30.0, 60.0],
        },
    ];

    if !a.biosignal.rr_intervals_ms.is_empty() {
        let beat_times: Vec<f64> = (0..a.biosignal.rr_intervals_ms.len())
            .enumerate()
            .map(|(i, _)| {
                #[expect(clippy::cast_precision_loss, reason = "beat count fits f64")]
                let v = i as f64;
                v
            })
            .collect();
        channels.push(DataChannel::TimeSeries {
            id: "rr_tachogram".into(),
            label: "RR Tachogram".into(),
            x_label: "Beat #".into(),
            y_label: "RR (ms)".into(),
            unit: "ms".into(),
            x_values: beat_times,
            y_values: a.biosignal.rr_intervals_ms.clone(),
        });
    }

    channels
}
