// SPDX-License-Identifier: AGPL-3.0-or-later
//! Node builders for patient-parameterized TRT clinical scenarios.
//!
//! Each function produces a single [`ScenarioNode`] populated with
//! validated endocrine models (Mok, Saad, Sharma, Kapoor).

use super::clinical::{PatientTrtProfile, TrtProtocol};
use super::scenarios::{bar, gauge, node, timeseries};
use super::types::{ClinicalRange, DataChannel, ScenarioNode};
use crate::endocrine::{self, testosterone_cypionate as tc};

#[expect(
    clippy::too_many_lines,
    reason = "assembles patient assessment node with multiple data channels — splitting would fragment a single logical unit"
)]
pub(super) fn assessment_node(p: &PatientTrtProfile) -> (ScenarioNode, Vec<DataChannel>) {
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
            ages,
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
        node_type: "sensor".into(),
        family: "healthspring-clinical".into(),
        status: if p.baseline_t_ng_dl < 300.0 {
            "critical"
        } else {
            "active"
        }
        .into(),
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
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Borderline low T".into(),
                min: 200.0,
                max: 300.0,
                status: "warning".into(),
            },
            ClinicalRange {
                label: "Clinical hypogonadism".into(),
                min: 0.0,
                max: 200.0,
                status: "critical".into(),
            },
        ],
    };

    (n, channels)
}

#[expect(
    clippy::too_many_lines,
    reason = "three protocol variants with PK compute"
)]
pub(super) fn protocol_node(p: &PatientTrtProfile) -> ScenarioNode {
    let weight_kg = p.weight_lb * 0.453_592;
    let vd = tc::VD_L * (weight_kg / 70.0);

    let days: Vec<f64> = (0..=560).map(|i| f64::from(i) / 10.0).collect();

    let (protocol_name, pk_curve, trough_val, cmax_val) = match p.protocol {
        TrtProtocol::ImWeekly => {
            let reg = endocrine::ImRegimen {
                dose_mg: tc::DOSE_WEEKLY_MG,
                f: tc::F_IM,
                vd,
                ka: tc::K_A_IM,
                ke: tc::K_E,
                interval: tc::INTERVAL_WEEKLY,
                n_doses: 8,
            };
            let curve: Vec<f64> = days
                .iter()
                .map(|&t| endocrine::pk_im_depot(reg.dose_mg, reg.f, reg.vd, reg.ka, reg.ke, t))
                .collect();
            let (cmax, trough) = endocrine::im_steady_state_metrics(&reg, &days);
            ("Weekly IM (100mg)".to_string(), curve, trough, cmax)
        }
        TrtProtocol::ImBiweekly => {
            let reg = endocrine::ImRegimen {
                dose_mg: tc::DOSE_BIWEEKLY_MG,
                f: tc::F_IM,
                vd,
                ka: tc::K_A_IM,
                ke: tc::K_E,
                interval: tc::INTERVAL_BIWEEKLY,
                n_doses: 4,
            };
            let curve: Vec<f64> = days
                .iter()
                .map(|&t| endocrine::pk_im_depot(reg.dose_mg, reg.f, reg.vd, reg.ka, reg.ke, t))
                .collect();
            let (cmax, trough) = endocrine::im_steady_state_metrics(&reg, &days);
            ("Biweekly IM (200mg)".to_string(), curve, trough, cmax)
        }
        TrtProtocol::Pellet => {
            let dose_mg = 10.0 * p.weight_lb;
            let release_rate = dose_mg / endocrine::pellet_params::DURATION_DAYS;
            let curve: Vec<f64> = days
                .iter()
                .map(|&t| {
                    endocrine::pellet_concentration(
                        t,
                        release_rate,
                        tc::K_E,
                        vd,
                        endocrine::pellet_params::DURATION_DAYS,
                    )
                })
                .collect();
            let ss = release_rate / (vd * tc::K_E);
            (
                format!("Pellet ({dose_mg:.0}mg, 10mg/lb)"),
                curve,
                ss * 0.95,
                ss,
            )
        }
    };

    node(
        "protocol",
        &format!("Treatment: {protocol_name}"),
        "compute",
        &["clinical.treatment.testosterone_pk"],
        vec![
            timeseries(
                "pk_curve",
                &format!("Testosterone Level — {protocol_name}"),
                "Time (days)",
                "T (ng/mL)",
                "ng/mL",
                days,
                pk_curve,
            ),
            gauge(
                "steady_trough",
                "Projected Trough",
                trough_val,
                0.0,
                40.0,
                "ng/mL",
                [3.0, 10.0],
                [1.0, 3.0],
            ),
            gauge(
                "steady_cmax",
                "Projected Peak",
                cmax_val,
                0.0,
                60.0,
                "ng/mL",
                [10.0, 35.0],
                [35.0, 50.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Therapeutic window".into(),
                min: 3.0,
                max: 35.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Supraphysiologic".into(),
                min: 35.0,
                max: 60.0,
                status: "warning".into(),
            },
        ],
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "population count ≤ 200 fits in f64"
)]
pub(super) fn population_node(p: &PatientTrtProfile) -> ScenarioNode {
    let weight_kg = p.weight_lb * 0.453_592;
    let n_pop: usize = 100;
    let times: Vec<f64> = (0..500).map(|i| f64::from(i) * 56.0 / 499.0).collect();

    let (mu_vd, sig_vd) =
        endocrine::lognormal_params(endocrine::pop_trt::VD_TYPICAL, endocrine::pop_trt::VD_CV);
    let (mu_ke, sig_ke) =
        endocrine::lognormal_params(endocrine::pop_trt::KE_TYPICAL, endocrine::pop_trt::KE_CV);

    let pop_denom = (n_pop - 1) as f64;
    let mut trough_values = Vec::with_capacity(n_pop);

    for i in 0..n_pop {
        let z = -2.0 + 4.0 * (i as f64) / pop_denom;
        let vd_i = sig_vd.mul_add(z, mu_vd).exp();
        let ke_i = sig_ke.mul_add(z, mu_ke).exp();

        let reg = endocrine::ImRegimen {
            dose_mg: tc::DOSE_WEEKLY_MG,
            f: tc::F_IM,
            vd: vd_i,
            ka: tc::K_A_IM,
            ke: ke_i,
            interval: tc::INTERVAL_WEEKLY,
            n_doses: 8,
        };
        let (_, trough) = endocrine::im_steady_state_metrics(&reg, &times);
        trough_values.push(trough);
    }

    let mean_trough: f64 = trough_values.iter().sum::<f64>() / n_pop as f64;
    let var: f64 = trough_values
        .iter()
        .map(|&t| (t - mean_trough).powi(2))
        .sum::<f64>()
        / n_pop as f64;
    let std_trough = var.sqrt();

    let patient_vd = tc::VD_L * (weight_kg / 70.0);
    let patient_reg = endocrine::ImRegimen {
        dose_mg: tc::DOSE_WEEKLY_MG,
        f: tc::F_IM,
        vd: patient_vd,
        ka: tc::K_A_IM,
        ke: tc::K_E,
        interval: tc::INTERVAL_WEEKLY,
        n_doses: 8,
    };
    let (_, patient_trough) = endocrine::im_steady_state_metrics(&patient_reg, &times);

    node(
        "population",
        "Population Comparison (100 patients)",
        "storage",
        &["clinical.population.pk_comparison"],
        vec![
            DataChannel::Distribution {
                id: "trough_dist".into(),
                label: "Trough Level Distribution".into(),
                unit: "ng/mL".into(),
                values: trough_values,
                mean: mean_trough,
                std: std_trough,
                patient_value: patient_trough,
            },
            gauge(
                "patient_trough",
                "Your Projected Trough",
                patient_trough,
                0.0,
                20.0,
                "ng/mL",
                [3.0, 10.0],
                [1.0, 3.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Population therapeutic range".into(),
            min: 3.0,
            max: 15.0,
            status: "normal".into(),
        }],
    )
}

pub(super) fn metabolic_node() -> ScenarioNode {
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
pub(super) fn cardiovascular_node() -> ScenarioNode {
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

pub(super) fn diabetes_node(hba1c_baseline: f64) -> ScenarioNode {
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

pub(super) fn cardiac_monitor_node(sdnn_base: f64) -> ScenarioNode {
    let delta = 20.0;
    let tau = 6.0;
    let months: Vec<f64> = (0..=240).map(|i| f64::from(i) / 10.0).collect();

    let sdnn_curve: Vec<f64> = months
        .iter()
        .map(|&m| endocrine::hrv_trt_response(sdnn_base, delta, tau, m))
        .collect();

    let risk_pre = endocrine::cardiac_risk_composite(sdnn_base, 280.0, 1.0);
    let risk_post = endocrine::cardiac_risk_composite(sdnn_base + delta, 500.0, 1.0);
    let reduction_pct = (1.0 - risk_post / risk_pre) * 100.0;

    node(
        "cardiac",
        "Cardiac Monitoring (HRV + Composite Risk)",
        "compute",
        &["clinical.monitor.hrv", "clinical.monitor.cardiac_risk"],
        vec![
            timeseries(
                "sdnn",
                "SDNN on TRT",
                "Month",
                "SDNN (ms)",
                "ms",
                months,
                sdnn_curve,
            ),
            bar(
                "risk_compare",
                "Cardiac Risk: Pre vs Post TRT",
                vec!["Pre-TRT".into(), "12-Month TRT".into()],
                vec![risk_pre, risk_post],
                "composite score",
            ),
            gauge(
                "risk_reduction",
                "Projected Risk Reduction",
                reduction_pct,
                0.0,
                100.0,
                "%",
                [15.0, 60.0],
                [5.0, 15.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "SDNN healthy".into(),
                min: 50.0,
                max: 200.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "SDNN reduced".into(),
                min: 20.0,
                max: 50.0,
                status: "warning".into(),
            },
        ],
    )
}

pub(super) fn gut_health_node(diversity: f64) -> ScenarioNode {
    let communities = [
        ("High Diversity", 0.95),
        ("Moderate", 0.70),
        ("Low Diversity", 0.30),
        ("Patient", diversity),
    ];

    let mut cats = Vec::new();
    let mut responses = Vec::new();
    let mut xi_max: f64 = 0.0;

    let mut xis = Vec::new();
    for &(_, j) in &communities {
        let w = endocrine::evenness_to_disorder(j, endocrine::gut_axis_params::DISORDER_SCALE);
        let xi =
            endocrine::anderson_localization_length(w, endocrine::gut_axis_params::LATTICE_SIZE);
        xis.push(xi);
        if xi > xi_max {
            xi_max = xi;
        }
    }

    for (i, &(name, _)) in communities.iter().enumerate() {
        let resp = endocrine::gut_metabolic_response(
            xis[i],
            xi_max,
            endocrine::gut_axis_params::BASE_RESPONSE_KG,
        );
        cats.push(name.to_string());
        responses.push(resp);
    }

    let patient_resp = responses.last().copied().unwrap_or(0.0);

    node(
        "gut_health",
        "Gut Health Factor (Cross-Track)",
        "compute",
        &[
            "clinical.predictor.gut_diversity",
            "clinical.predictor.metabolic_response",
        ],
        vec![
            bar(
                "gut_response",
                "Expected Weight Loss by Gut Diversity",
                cats,
                responses,
                "kg",
            ),
            gauge(
                "patient_gut",
                "Patient Gut Diversity (Pielou J)",
                diversity,
                0.0,
                1.0,
                "J",
                [0.6, 1.0],
                [0.3, 0.6],
            ),
            gauge(
                "patient_response",
                "Patient Predicted Weight Loss",
                patient_resp.abs(),
                0.0,
                20.0,
                "kg",
                [8.0, 20.0],
                [4.0, 8.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Diverse gut".into(),
                min: 0.6,
                max: 1.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Low diversity".into(),
                min: 0.0,
                max: 0.4,
                status: "critical".into(),
            },
        ],
    )
}
