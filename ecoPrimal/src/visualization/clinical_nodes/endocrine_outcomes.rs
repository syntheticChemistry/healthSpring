// SPDX-License-Identifier: AGPL-3.0-only
//! Endocrine outcome nodes: metabolic, cardiovascular, glycemic (Saad, Sharma, Kapoor).

use crate::endocrine;
use crate::visualization::scenarios::{gauge, node, timeseries};
use crate::visualization::types::{ClinicalRange, ScenarioNode};

pub fn metabolic_node() -> ScenarioNode {
    let months: Vec<f64> = (0..=600).map(|i| f64::from(i) / 10.0).collect();

    let weight_curve: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::weight_trajectory(
                m,
                endocrine::weight_params::WEIGHT_LOSS_5YR_KG,
                endocrine::weight_params::TAU_MONTHS,
                endocrine::weight_params::TOTAL_MONTHS,
            )
        })
        .collect();

    let waist_curve: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::weight_trajectory(
                m,
                endocrine::weight_params::WAIST_LOSS_5YR_CM,
                endocrine::weight_params::TAU_MONTHS,
                endocrine::weight_params::TOTAL_MONTHS,
            )
        })
        .collect();

    let bmi_curve: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::weight_trajectory(
                m,
                endocrine::weight_params::BMI_LOSS_5YR,
                endocrine::weight_params::TAU_MONTHS,
                endocrine::weight_params::TOTAL_MONTHS,
            )
        })
        .collect();

    node(
        "metabolic",
        "Metabolic Response (Saad 2013, n=411)",
        "compute",
        &[
            "clinical.outcome.weight",
            "clinical.outcome.waist",
            "clinical.outcome.bmi",
        ],
        vec![
            timeseries(
                "weight",
                "Expected Weight Change",
                "Month",
                "Weight Change (kg)",
                "kg",
                months.clone(),
                weight_curve,
            ),
            timeseries(
                "waist",
                "Expected Waist Change",
                "Month",
                "Waist Change (cm)",
                "cm",
                months.clone(),
                waist_curve,
            ),
            timeseries(
                "bmi",
                "Expected BMI Change",
                "Month",
                "BMI Change",
                "units",
                months,
                bmi_curve,
            ),
        ],
        vec![
            ClinicalRange {
                label: "Weight loss on track".into(),
                min: -20.0,
                max: -5.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Waist target (<102cm men)".into(),
                min: 0.0,
                max: 102.0,
                status: "normal".into(),
            },
        ],
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "assembles cardiovascular node with LDL, HbA1c, weight, and blood-pressure channels"
)]
pub fn cardiovascular_node() -> ScenarioNode {
    let months: Vec<f64> = (0..=600).map(|i| f64::from(i) / 10.0).collect();

    let ldl: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::biomarker_trajectory(
                m,
                endocrine::cv_params::LDL_BASELINE,
                endocrine::cv_params::LDL_ENDPOINT,
                endocrine::cv_params::TAU_MONTHS,
            )
        })
        .collect();
    let hdl: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::biomarker_trajectory(
                m,
                endocrine::cv_params::HDL_BASELINE,
                endocrine::cv_params::HDL_ENDPOINT,
                endocrine::cv_params::TAU_MONTHS,
            )
        })
        .collect();
    let crp: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::biomarker_trajectory(
                m,
                endocrine::cv_params::CRP_BASELINE,
                endocrine::cv_params::CRP_ENDPOINT,
                endocrine::cv_params::TAU_MONTHS,
            )
        })
        .collect();
    let sbp: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::biomarker_trajectory(
                m,
                endocrine::cv_params::SBP_BASELINE,
                endocrine::cv_params::SBP_ENDPOINT,
                endocrine::cv_params::TAU_MONTHS,
            )
        })
        .collect();

    let hr = endocrine::hazard_ratio_model(500.0, 300.0, 0.44);

    node(
        "cardiovascular",
        "Cardiovascular (Sharma 2015, n=83,010)",
        "compute",
        &[
            "clinical.outcome.lipids",
            "clinical.outcome.bp",
            "clinical.outcome.crp",
        ],
        vec![
            timeseries(
                "ldl",
                "LDL Cholesterol",
                "Month",
                "LDL (mg/dL)",
                "mg/dL",
                months.clone(),
                ldl,
            ),
            timeseries(
                "hdl",
                "HDL Cholesterol",
                "Month",
                "HDL (mg/dL)",
                "mg/dL",
                months.clone(),
                hdl,
            ),
            timeseries(
                "crp",
                "C-Reactive Protein",
                "Month",
                "CRP (mg/dL)",
                "mg/dL",
                months.clone(),
                crp,
            ),
            timeseries(
                "sbp",
                "Systolic Blood Pressure",
                "Month",
                "SBP (mmHg)",
                "mmHg",
                months,
                sbp,
            ),
            gauge(
                "hazard_ratio",
                "MI/Stroke Hazard Ratio (Normalized T)",
                hr,
                0.0,
                2.0,
                "HR",
                [0.3, 0.7],
                [0.7, 1.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "LDL optimal".into(),
                min: 0.0,
                max: 130.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "LDL borderline".into(),
                min: 130.0,
                max: 160.0,
                status: "warning".into(),
            },
            ClinicalRange {
                label: "HDL protective".into(),
                min: 40.0,
                max: 100.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "SBP normal".into(),
                min: 90.0,
                max: 130.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "CRP low risk".into(),
                min: 0.0,
                max: 1.0,
                status: "normal".into(),
            },
        ],
    )
}

pub fn diabetes_node(hba1c_baseline: f64) -> ScenarioNode {
    let months: Vec<f64> = (0..=240).map(|i| f64::from(i) / 10.0).collect();

    let hba1c_curve: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::hba1c_trajectory(
                m,
                hba1c_baseline,
                endocrine::diabetes_params::HBA1C_DELTA,
                endocrine::diabetes_params::TAU_MONTHS,
            )
        })
        .collect();

    let homa_curve: Vec<f64> = months
        .iter()
        .map(|&m| {
            endocrine::biomarker_trajectory(
                m,
                endocrine::diabetes_params::HOMA_BASELINE,
                endocrine::diabetes_params::HOMA_ENDPOINT,
                endocrine::diabetes_params::TAU_MONTHS,
            )
        })
        .collect();

    let projected_a1c = endocrine::hba1c_trajectory(
        12.0,
        hba1c_baseline,
        endocrine::diabetes_params::HBA1C_DELTA,
        endocrine::diabetes_params::TAU_MONTHS,
    );

    node(
        "diabetes",
        "Glycemic Response (Kapoor 2006 RCT)",
        "compute",
        &["clinical.outcome.hba1c", "clinical.outcome.homa"],
        vec![
            timeseries(
                "hba1c",
                "HbA1c Trajectory",
                "Month",
                "HbA1c (%)",
                "%",
                months.clone(),
                hba1c_curve,
            ),
            timeseries(
                "homa",
                "Insulin Sensitivity (HOMA-IR)",
                "Month",
                "HOMA-IR",
                "index",
                months,
                homa_curve,
            ),
            gauge(
                "a1c_12mo",
                "Projected HbA1c at 12 Months",
                projected_a1c,
                4.0,
                10.0,
                "%",
                [4.0, 7.0],
                [7.0, 8.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "HbA1c target".into(),
                min: 4.0,
                max: 7.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "HbA1c prediabetic".into(),
                min: 7.0,
                max: 8.0,
                status: "warning".into(),
            },
        ],
    )
}
