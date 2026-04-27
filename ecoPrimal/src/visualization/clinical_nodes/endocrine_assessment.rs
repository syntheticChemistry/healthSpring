// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endocrine assessment node: patient testosterone baseline and age projection.

use crate::endocrine;
use crate::visualization::clinical::PatientTrtProfile;
use crate::visualization::scenarios::{gauge, timeseries};
use crate::visualization::types::{
    ClinicalRange, ClinicalStatus, DataChannel, NodeStatus, NodeType, ScenarioNode,
};

pub fn assessment_node(p: &PatientTrtProfile) -> (ScenarioNode, Vec<DataChannel>) {
    let t0 = endocrine::decline_params::T0_MEAN_NGDL;
    let ages: Vec<f64> = (300..=900).map(|i| f64::from(i) / 10.0).collect();

    let decline_curve: Vec<f64> = ages
        .iter()
        .map(|&a| endocrine::testosterone_decline(t0, endocrine::decline_params::RATE_MID, a, 30.0))
        .collect();

    let age_at_low = endocrine::age_at_threshold(
        t0,
        endocrine::decline_params::RATE_MID,
        endocrine::decline_params::THRESHOLD_CLINICAL,
        30.0,
    );

    let patient_projected_t =
        endocrine::testosterone_decline(t0, endocrine::decline_params::RATE_MID, p.age, 30.0);

    let health = if p.baseline_t_ng_dl < 300.0 {
        40
    } else if p.baseline_t_ng_dl < 400.0 {
        60
    } else {
        80
    };

    let channels = vec![
        timeseries(
            "age_decline",
            "Average T Decline With Age",
            "Age (years)",
            "Testosterone (ng/dL)",
            "ng/dL",
            &ages,
            decline_curve,
        ),
        gauge(
            "baseline_t",
            "Patient Baseline T",
            p.baseline_t_ng_dl,
            0.0,
            1000.0,
            "ng/dL",
            [300.0, 900.0],
            [200.0, 300.0],
        ),
        gauge(
            "projected_t",
            "Age-Projected T (Population Average)",
            patient_projected_t,
            0.0,
            1000.0,
            "ng/dL",
            [300.0, 900.0],
            [200.0, 300.0],
        ),
        gauge(
            "age_at_clinical",
            "Projected Age at Clinical Low T",
            age_at_low,
            40.0,
            90.0,
            "years",
            [65.0, 85.0],
            [50.0, 65.0],
        ),
    ];

    let n = ScenarioNode {
        id: "assessment".into(),
        name: format!("Patient Assessment: {}", p.name),
        node_type: NodeType::Sensor,
        family: "healthspring-clinical".into(),
        status: NodeStatus::from_aggregate_health(health),
        health,
        confidence: 90,
        position: None,
        capabilities: vec![
            "clinical.assessment.testosterone".into(),
            "clinical.assessment.age_projection".into(),
        ],
        data_channels: channels.clone(),
        clinical_ranges: vec![
            ClinicalRange {
                label: "Normal T".into(),
                min: 300.0,
                max: 1000.0,
                status: ClinicalStatus::Normal,
            },
            ClinicalRange {
                label: "Borderline low T".into(),
                min: 200.0,
                max: 300.0,
                status: ClinicalStatus::Warning,
            },
            ClinicalRange {
                label: "Clinical hypogonadism".into(),
                min: 0.0,
                max: 200.0,
                status: ClinicalStatus::Critical,
            },
        ],
    };

    (n, channels)
}
