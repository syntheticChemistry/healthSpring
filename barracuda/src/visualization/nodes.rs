// SPDX-License-Identifier: AGPL-3.0-or-later
//! Node and edge construction for the healthSpring diagnostic scenario.
//!
//! Converts `DiagnosticAssessment` data into `ScenarioNode` and `ScenarioEdge`
//! collections for petalTongue rendering.

use crate::diagnostic::DiagnosticAssessment;

use super::types::{ClinicalRange, DataChannel, ScenarioEdge, ScenarioNode};

pub(super) fn risk_to_health(risk: f64) -> u8 {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
            family: "healthspring".into(),
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
            family: "healthspring".into(),
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
            family: "healthspring".into(),
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

fn build_pk_node(a: &DiagnosticAssessment) -> ScenarioNode {
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

fn build_microbiome_node(a: &DiagnosticAssessment, health: u8) -> ScenarioNode {
    ScenarioNode {
        id: "microbiome".into(),
        name: "Microbiome Risk".into(),
        node_type: "data".into(),
        family: "healthspring".into(),
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

fn build_biosignal_node(a: &DiagnosticAssessment, health: u8) -> ScenarioNode {
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

fn build_endocrine_node(a: &DiagnosticAssessment, health: u8) -> ScenarioNode {
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
