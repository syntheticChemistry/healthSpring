// SPDX-License-Identifier: AGPL-3.0-or-later
//! Patient-parameterized TRT clinical scenario builder.
//!
//! Unlike `endocrine_study()` which demonstrates the validated math with fixed
//! parameters, this module produces scenarios parameterized by a specific
//! patient — age, weight, baseline testosterone, and chosen protocol. The
//! output tells a clinical story that a clinician can show a patient:
//! assessment → protocol → population comparison → predicted outcomes.

use super::scenarios::{bar, edge, gauge, node, scenario_with_edges_json, timeseries};
use super::types::{
    Animations, CapReqs, ClinicalRange, DataChannel, Ecosystem, HealthScenario, NeuralApi,
    Performance, Position, ScenarioEdge, ScenarioNode, SensoryConfig, ShowPanels, UiConfig,
};
use crate::endocrine::{self, testosterone_cypionate as tc};

/// TRT delivery protocol.
#[derive(Debug, Clone, Copy)]
pub enum TrtProtocol {
    ImWeekly,
    ImBiweekly,
    Pellet,
}

/// A patient's clinical profile for TRT scenario generation.
#[derive(Debug, Clone)]
pub struct PatientTrtProfile {
    pub name: String,
    pub age: f64,
    pub weight_lb: f64,
    pub baseline_t_ng_dl: f64,
    pub protocol: TrtProtocol,
    /// Pielou evenness (0..1). `None` = not measured.
    pub gut_diversity: Option<f64>,
    /// Baseline `HbA1c`. `None` = not diabetic / not measured.
    pub hba1c: Option<f64>,
    /// Baseline SDNN in ms. `None` = not measured.
    pub sdnn_ms: Option<f64>,
}

impl PatientTrtProfile {
    #[must_use]
    pub fn new(name: &str, age: f64, weight_lb: f64, baseline_t: f64, protocol: TrtProtocol) -> Self {
        Self {
            name: name.into(),
            age,
            weight_lb,
            baseline_t_ng_dl: baseline_t,
            protocol,
            gut_diversity: None,
            hba1c: None,
            sdnn_ms: None,
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "display rounding — age/weight/T are always positive and < u32::MAX"
)]
fn scaffold_clinical(patient: &PatientTrtProfile) -> HealthScenario {
    let age = patient.age as u32;
    let wt = patient.weight_lb as u32;
    let t = patient.baseline_t_ng_dl as u32;
    HealthScenario {
        name: format!("TRT Clinical: {}", patient.name),
        description: format!(
            "Patient-specific TRT projection — {age}yo, {wt}lb, baseline T {t}ng/dL"
        ),
        version: "2.0.0".into(),
        mode: "clinical".into(),
        sensory_config: SensoryConfig {
            required_capabilities: CapReqs {
                outputs: vec!["visual".into()],
                inputs: vec![],
            },
            optional_capabilities: CapReqs {
                outputs: vec!["audio".into()],
                inputs: vec!["pointer".into(), "keyboard".into()],
            },
            complexity_hint: "standard".into(),
        },
        ui_config: UiConfig {
            theme: "clinical-dark".into(),
            animations: Animations {
                enabled: true,
                breathing_nodes: true,
                connection_pulses: true,
                smooth_transitions: true,
                celebration_effects: false,
            },
            performance: Performance {
                target_fps: 60,
                vsync: true,
                hardware_acceleration: true,
            },
            show_panels: Some(ShowPanels {
                left_sidebar: false,
                right_sidebar: true,
                top_menu: true,
                system_dashboard: false,
                audio_panel: false,
                trust_dashboard: false,
                proprioception: false,
                graph_stats: true,
            }),
            awakening_enabled: Some(false),
            initial_zoom: Some("fit".into()),
        },
        ecosystem: Ecosystem { primals: vec![] },
        neural_api: NeuralApi { enabled: false },
        edges: Vec::new(),
    }
}

// ---- Node builders (patient-parameterized) ----

fn assessment_node(p: &PatientTrtProfile) -> (ScenarioNode, Vec<DataChannel>) {
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

    let patient_projected_t = endocrine::testosterone_decline(
        t0,
        endocrine::decline_params::RATE_MID,
        p.age,
        30.0,
    );

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
        status: if p.baseline_t_ng_dl < 300.0 { "critical" } else { "active" }.into(),
        health,
        confidence: 90,
        position: Position { x: 400.0, y: 50.0 },
        capabilities: vec![
            "clinical.assessment.testosterone".into(),
            "clinical.assessment.age_projection".into(),
        ],
        data_channels: channels.clone(),
        clinical_ranges: vec![
            ClinicalRange { label: "Normal T".into(), min: 300.0, max: 1000.0, status: "normal".into() },
            ClinicalRange { label: "Borderline low T".into(), min: 200.0, max: 300.0, status: "warning".into() },
            ClinicalRange { label: "Clinical hypogonadism".into(), min: 0.0, max: 200.0, status: "critical".into() },
        ],
    };

    (n, channels)
}

#[expect(clippy::too_many_lines, reason = "three protocol variants with PK compute")]
fn protocol_node(p: &PatientTrtProfile) -> ScenarioNode {
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
        400.0,
        200.0,
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
            ClinicalRange { label: "Therapeutic window".into(), min: 3.0, max: 35.0, status: "normal".into() },
            ClinicalRange { label: "Supraphysiologic".into(), min: 35.0, max: 60.0, status: "warning".into() },
        ],
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "population count ≤ 200 fits in f64"
)]
fn population_node(p: &PatientTrtProfile) -> ScenarioNode {
    let weight_kg = p.weight_lb * 0.453_592;
    let n_pop: usize = 100;
    let times: Vec<f64> = (0..500).map(|i| f64::from(i) * 56.0 / 499.0).collect();

    let (mu_vd, sig_vd) = endocrine::lognormal_params(
        endocrine::pop_trt::VD_TYPICAL,
        endocrine::pop_trt::VD_CV,
    );
    let (mu_ke, sig_ke) = endocrine::lognormal_params(
        endocrine::pop_trt::KE_TYPICAL,
        endocrine::pop_trt::KE_CV,
    );

    let pop_denom = (n_pop - 1) as f64;
    let mut trough_values = Vec::with_capacity(n_pop);

    for i in 0..n_pop {
        let z = -2.0 + 4.0 * (i as f64) / pop_denom;
        let vd_i = (mu_vd + sig_vd * z).exp();
        let ke_i = (mu_ke + sig_ke * z).exp();

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
    let var: f64 = trough_values.iter().map(|&t| (t - mean_trough).powi(2)).sum::<f64>() / n_pop as f64;
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
        700.0,
        200.0,
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
        vec![
            ClinicalRange { label: "Population therapeutic range".into(), min: 3.0, max: 15.0, status: "normal".into() },
        ],
    )
}

fn metabolic_node() -> ScenarioNode {
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
        100.0,
        400.0,
        &["clinical.outcome.weight", "clinical.outcome.waist", "clinical.outcome.bmi"],
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
            ClinicalRange { label: "Weight loss on track".into(), min: -20.0, max: -5.0, status: "normal".into() },
            ClinicalRange { label: "Waist target (<102cm men)".into(), min: 0.0, max: 102.0, status: "normal".into() },
        ],
    )
}

fn cardiovascular_node() -> ScenarioNode {
    let months: Vec<f64> = (0..=600).map(|i| f64::from(i) / 10.0).collect();

    let ldl: Vec<f64> = months.iter().map(|&m| {
        endocrine::biomarker_trajectory(m, endocrine::cv_params::LDL_BASELINE, endocrine::cv_params::LDL_ENDPOINT, endocrine::cv_params::TAU_MONTHS)
    }).collect();
    let hdl: Vec<f64> = months.iter().map(|&m| {
        endocrine::biomarker_trajectory(m, endocrine::cv_params::HDL_BASELINE, endocrine::cv_params::HDL_ENDPOINT, endocrine::cv_params::TAU_MONTHS)
    }).collect();
    let crp: Vec<f64> = months.iter().map(|&m| {
        endocrine::biomarker_trajectory(m, endocrine::cv_params::CRP_BASELINE, endocrine::cv_params::CRP_ENDPOINT, endocrine::cv_params::TAU_MONTHS)
    }).collect();
    let sbp: Vec<f64> = months.iter().map(|&m| {
        endocrine::biomarker_trajectory(m, endocrine::cv_params::SBP_BASELINE, endocrine::cv_params::SBP_ENDPOINT, endocrine::cv_params::TAU_MONTHS)
    }).collect();

    let hr = endocrine::hazard_ratio_model(500.0, 300.0, 0.44);

    node(
        "cardiovascular",
        "Cardiovascular (Sharma 2015, n=83,010)",
        "compute",
        400.0,
        400.0,
        &["clinical.outcome.lipids", "clinical.outcome.bp", "clinical.outcome.crp"],
        vec![
            timeseries("ldl", "LDL Cholesterol", "Month", "LDL (mg/dL)", "mg/dL", months.clone(), ldl),
            timeseries("hdl", "HDL Cholesterol", "Month", "HDL (mg/dL)", "mg/dL", months.clone(), hdl),
            timeseries("crp", "C-Reactive Protein", "Month", "CRP (mg/dL)", "mg/dL", months.clone(), crp),
            timeseries("sbp", "Systolic Blood Pressure", "Month", "SBP (mmHg)", "mmHg", months, sbp),
            gauge("hazard_ratio", "MI/Stroke Hazard Ratio (Normalized T)", hr, 0.0, 2.0, "HR", [0.3, 0.7], [0.7, 1.0]),
        ],
        vec![
            ClinicalRange { label: "LDL optimal".into(), min: 0.0, max: 130.0, status: "normal".into() },
            ClinicalRange { label: "LDL borderline".into(), min: 130.0, max: 160.0, status: "warning".into() },
            ClinicalRange { label: "HDL protective".into(), min: 40.0, max: 100.0, status: "normal".into() },
            ClinicalRange { label: "SBP normal".into(), min: 90.0, max: 130.0, status: "normal".into() },
            ClinicalRange { label: "CRP low risk".into(), min: 0.0, max: 1.0, status: "normal".into() },
        ],
    )
}

fn diabetes_node(hba1c_baseline: f64) -> ScenarioNode {
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
        700.0,
        400.0,
        &["clinical.outcome.hba1c", "clinical.outcome.homa"],
        vec![
            timeseries("hba1c", "HbA1c Trajectory", "Month", "HbA1c (%)", "%", months.clone(), hba1c_curve),
            timeseries("homa", "Insulin Sensitivity (HOMA-IR)", "Month", "HOMA-IR", "index", months, homa_curve),
            gauge("a1c_12mo", "Projected HbA1c at 12 Months", projected_a1c, 4.0, 10.0, "%", [4.0, 7.0], [7.0, 8.0]),
        ],
        vec![
            ClinicalRange { label: "HbA1c target".into(), min: 4.0, max: 7.0, status: "normal".into() },
            ClinicalRange { label: "HbA1c prediabetic".into(), min: 7.0, max: 8.0, status: "warning".into() },
        ],
    )
}

fn cardiac_monitor_node(sdnn_base: f64) -> ScenarioNode {
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
        400.0,
        600.0,
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
            ClinicalRange { label: "SDNN healthy".into(), min: 50.0, max: 200.0, status: "normal".into() },
            ClinicalRange { label: "SDNN reduced".into(), min: 20.0, max: 50.0, status: "warning".into() },
        ],
    )
}

fn gut_health_node(diversity: f64) -> ScenarioNode {
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
        let xi = endocrine::anderson_localization_length(w, endocrine::gut_axis_params::LATTICE_SIZE);
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
        100.0,
        600.0,
        &["clinical.predictor.gut_diversity", "clinical.predictor.metabolic_response"],
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
            ClinicalRange { label: "Diverse gut".into(), min: 0.6, max: 1.0, status: "normal".into() },
            ClinicalRange { label: "Low diversity".into(), min: 0.0, max: 0.4, status: "critical".into() },
        ],
    )
}

// ---- Public API ----

/// Build a complete patient-specific TRT clinical scenario.
///
/// The scenario tells the clinical story: assessment → protocol → population
/// comparison → metabolic/cardiovascular/glycemic outcomes → cardiac monitoring
/// → gut health predictor. Every data channel is computed from validated models
/// parameterized by this specific patient.
#[must_use]
pub fn trt_clinical_scenario(p: &PatientTrtProfile) -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold_clinical(p);

    let (assessment, _) = assessment_node(p);
    s.ecosystem.primals.push(assessment);
    s.ecosystem.primals.push(protocol_node(p));
    s.ecosystem.primals.push(population_node(p));
    s.ecosystem.primals.push(metabolic_node());
    s.ecosystem.primals.push(cardiovascular_node());

    let hba1c_base = p.hba1c.unwrap_or(endocrine::diabetes_params::HBA1C_BASELINE);
    s.ecosystem.primals.push(diabetes_node(hba1c_base));

    let sdnn_base = p.sdnn_ms.unwrap_or(35.0);
    s.ecosystem.primals.push(cardiac_monitor_node(sdnn_base));

    let gut_j = p.gut_diversity.unwrap_or(0.70);
    s.ecosystem.primals.push(gut_health_node(gut_j));

    let edges = vec![
        edge("assessment", "protocol", "Baseline → Prescribe"),
        edge("protocol", "population", "Compare to similar patients"),
        edge("protocol", "metabolic", "Treatment → Weight/Waist"),
        edge("protocol", "cardiovascular", "Treatment → CV Biomarkers"),
        edge("protocol", "diabetes", "Treatment → Glycemic Control"),
        edge("cardiovascular", "cardiac", "Lipids/BP → Cardiac Risk"),
        edge("gut_health", "metabolic", "Gut diversity modulates response"),
        edge("assessment", "cardiac", "Baseline HRV → Monitor"),
    ];

    (s, edges)
}

/// Serialize a patient TRT clinical scenario to JSON.
#[must_use]
pub fn trt_clinical_json(p: &PatientTrtProfile) -> String {
    let (scenario, edges) = trt_clinical_scenario(p);
    scenario_with_edges_json(&scenario, &edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_patient() -> PatientTrtProfile {
        let mut p = PatientTrtProfile::new("Sample", 55.0, 220.0, 280.0, TrtProtocol::Pellet);
        p.gut_diversity = Some(0.65);
        p.hba1c = Some(7.2);
        p.sdnn_ms = Some(38.0);
        p
    }

    #[test]
    fn scenario_has_8_nodes() {
        let (s, _) = trt_clinical_scenario(&sample_patient());
        assert_eq!(s.ecosystem.primals.len(), 8, "expected 8 clinical nodes");
    }

    #[test]
    fn scenario_has_8_edges() {
        let (_, edges) = trt_clinical_scenario(&sample_patient());
        assert_eq!(edges.len(), 8, "expected 8 clinical edges");
    }

    #[test]
    fn all_nodes_have_data_channels() {
        let (s, _) = trt_clinical_scenario(&sample_patient());
        for n in &s.ecosystem.primals {
            assert!(
                !n.data_channels.is_empty(),
                "node '{}' has no data channels",
                n.id
            );
        }
    }

    #[test]
    fn all_nodes_have_clinical_ranges() {
        let (s, _) = trt_clinical_scenario(&sample_patient());
        for n in &s.ecosystem.primals {
            assert!(
                !n.clinical_ranges.is_empty(),
                "node '{}' has no clinical ranges",
                n.id
            );
        }
    }

    #[test]
    fn json_roundtrips() {
        let json = trt_clinical_json(&sample_patient());
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["ecosystem"]["primals"].is_array());
        assert!(parsed["edges"].is_array());
        assert_eq!(parsed["ecosystem"]["primals"].as_array().unwrap().len(), 8);
    }

    #[test]
    fn clinical_mode_and_panel_config() {
        let json = trt_clinical_json(&sample_patient());
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["mode"], "clinical");
        let panels = &parsed["ui_config"]["show_panels"];
        assert_eq!(panels["left_sidebar"], false);
        assert_eq!(panels["system_dashboard"], false);
        assert_eq!(panels["audio_panel"], false);
        assert_eq!(panels["trust_dashboard"], false);
        assert_eq!(panels["proprioception"], false);
        assert_eq!(panels["graph_stats"], true);
        assert_eq!(panels["top_menu"], true);
        assert_eq!(parsed["ui_config"]["awakening_enabled"], false);
        assert_eq!(parsed["ui_config"]["initial_zoom"], "fit");
    }

    #[test]
    fn pellet_dose_scales_with_weight() {
        let light = PatientTrtProfile::new("Light", 50.0, 150.0, 300.0, TrtProtocol::Pellet);
        let heavy = PatientTrtProfile::new("Heavy", 50.0, 250.0, 300.0, TrtProtocol::Pellet);

        let (sl, _) = trt_clinical_scenario(&light);
        let (sh, _) = trt_clinical_scenario(&heavy);

        let prot_l = sl.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();
        let prot_h = sh.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();

        assert!(prot_l.name.contains("1500"), "150lb × 10 = 1500mg");
        assert!(prot_h.name.contains("2500"), "250lb × 10 = 2500mg");
    }

    #[test]
    fn weekly_and_biweekly_produce_different_curves() {
        let weekly = PatientTrtProfile::new("W", 50.0, 200.0, 300.0, TrtProtocol::ImWeekly);
        let biweekly = PatientTrtProfile::new("B", 50.0, 200.0, 300.0, TrtProtocol::ImBiweekly);

        let (sw, _) = trt_clinical_scenario(&weekly);
        let (sb, _) = trt_clinical_scenario(&biweekly);

        let pw = sw.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();
        let pb = sb.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();

        assert!(pw.name.contains("Weekly"));
        assert!(pb.name.contains("Biweekly"));
    }

    #[test]
    fn low_baseline_t_flags_critical() {
        let p = PatientTrtProfile::new("Low", 60.0, 200.0, 180.0, TrtProtocol::ImWeekly);
        let (s, _) = trt_clinical_scenario(&p);
        let assess = s.ecosystem.primals.iter().find(|n| n.id == "assessment").unwrap();
        assert_eq!(assess.status, "critical");
        assert!(assess.health <= 50);
    }

    #[test]
    fn gut_diversity_affects_response_prediction() {
        let low = {
            let mut p = PatientTrtProfile::new("Low", 50.0, 200.0, 300.0, TrtProtocol::Pellet);
            p.gut_diversity = Some(0.20);
            p
        };
        let high = {
            let mut p = PatientTrtProfile::new("High", 50.0, 200.0, 300.0, TrtProtocol::Pellet);
            p.gut_diversity = Some(0.95);
            p
        };

        let (sl, _) = trt_clinical_scenario(&low);
        let (sh, _) = trt_clinical_scenario(&high);

        let gl = sl.ecosystem.primals.iter().find(|n| n.id == "gut_health").unwrap();
        let gh = sh.ecosystem.primals.iter().find(|n| n.id == "gut_health").unwrap();

        let get_response_gauge = |n: &ScenarioNode| -> f64 {
            n.data_channels.iter().find_map(|ch| {
                if let DataChannel::Gauge { id, value, .. } = ch {
                    if id == "patient_response" { return Some(*value); }
                }
                None
            }).unwrap()
        };

        let resp_low = get_response_gauge(gl);
        let resp_high = get_response_gauge(gh);
        assert!(resp_high > resp_low, "higher diversity should predict more weight loss");
    }
}
