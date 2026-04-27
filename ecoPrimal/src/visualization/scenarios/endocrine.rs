// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{
    ClinicalRange, ClinicalStatus, HealthScenario, NodeType, ScenarioEdge,
};
use super::{bar, edge, gauge, node, scaffold, timeseries};
use crate::endocrine;

/// Build a complete endocrinology study scenario with real computed data.
#[must_use]
#[expect(clippy::too_many_lines, reason = "9 sub-studies, each compact")]
pub fn endocrine_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold(
        "healthSpring Endocrinology Study",
        "Testosterone PK, age decline, TRT outcomes, population, cross-track — 9 experiments",
    );

    // Testosterone IM PK (exp030)
    let days: Vec<f64> = (0..=280).map(|i| f64::from(i) / 10.0).collect();
    let im = endocrine::ImRegimen {
        dose_mg: endocrine::testosterone_cypionate::DOSE_WEEKLY_MG,
        f: endocrine::testosterone_cypionate::F_IM,
        vd: endocrine::testosterone_cypionate::VD_L,
        ka: endocrine::testosterone_cypionate::K_A_IM,
        ke: endocrine::testosterone_cypionate::K_E,
        interval: endocrine::testosterone_cypionate::INTERVAL_WEEKLY,
        n_doses: 4,
    };
    let single_curve: Vec<f64> = days
        .iter()
        .map(|&t| endocrine::pk_im_depot(im.dose_mg, im.f, im.vd, im.ka, im.ke, t))
        .collect();
    let (ss_cmax, ss_trough) = endocrine::im_steady_state_metrics(&im, &days);
    s.ecosystem.primals.push(node(
        "t_im",
        "Testosterone IM PK",
        NodeType::Compute,
        &["science.endocrine.testosterone_im"],
        vec![
            timeseries(
                "im_single",
                "Single IM Dose",
                "Time (days)",
                "C (ng/mL)",
                "ng/mL",
                &days,
                single_curve,
            ),
            gauge(
                "cmax",
                "Steady-State Cmax",
                ss_cmax,
                0.0,
                50.0,
                "ng/mL",
                [10.0, 35.0],
                [35.0, 45.0],
            ),
            gauge(
                "trough",
                "Steady-State Trough",
                ss_trough,
                0.0,
                20.0,
                "ng/mL",
                [3.0, 10.0],
                [1.0, 3.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Therapeutic T".into(),
            min: 10.0,
            max: 35.0,
            status: ClinicalStatus::Normal,
        }],
    ));

    // Pellet PK (exp031)
    let pellet_days: Vec<f64> = (0..=1800).map(|i| f64::from(i) / 10.0).collect();
    let pellet_curve: Vec<f64> = pellet_days
        .iter()
        .map(|&t| {
            endocrine::pellet_concentration(
                t,
                endocrine::pellet_params::RELEASE_RATE,
                endocrine::testosterone_cypionate::K_E,
                endocrine::testosterone_cypionate::VD_L,
                endocrine::pellet_params::DURATION_DAYS,
            )
        })
        .collect();
    s.ecosystem.primals.push(node(
        "t_pellet",
        "Testosterone Pellet PK",
        NodeType::Compute,
        &["science.endocrine.testosterone_pellet"],
        vec![timeseries(
            "pellet_pk",
            "Pellet Concentration",
            "Time (days)",
            "C (ng/mL)",
            "ng/mL",
            &pellet_days,
            pellet_curve,
        )],
        vec![],
    ));

    // Age decline (exp032)
    let ages: Vec<f64> = (300..=900).map(|i| f64::from(i) / 10.0).collect();
    let t0 = endocrine::decline_params::T0_MEAN_NGDL;
    let t_low: Vec<f64> = ages
        .iter()
        .map(|&a| endocrine::testosterone_decline(t0, endocrine::decline_params::RATE_LOW, a, 30.0))
        .collect();
    let t_mid: Vec<f64> = ages
        .iter()
        .map(|&a| endocrine::testosterone_decline(t0, endocrine::decline_params::RATE_MID, a, 30.0))
        .collect();
    let t_high: Vec<f64> = ages
        .iter()
        .map(|&a| {
            endocrine::testosterone_decline(t0, endocrine::decline_params::RATE_HIGH, a, 30.0)
        })
        .collect();
    let age_threshold = endocrine::age_at_threshold(
        t0,
        endocrine::decline_params::RATE_MID,
        endocrine::decline_params::THRESHOLD_CLINICAL,
        30.0,
    );
    s.ecosystem.primals.push(node(
        "age_decline",
        "Age-Related T Decline",
        NodeType::Compute,
        &["science.endocrine.testosterone_decline"],
        vec![
            timeseries(
                "t_low_rate",
                "T Decline (1%/yr)",
                "Age (yr)",
                "T (ng/dL)",
                "ng/dL",
                &ages,
                t_low,
            ),
            timeseries(
                "t_mid_rate",
                "T Decline (1.6%/yr)",
                "Age (yr)",
                "T (ng/dL)",
                "ng/dL",
                &ages,
                t_mid,
            ),
            timeseries(
                "t_high_rate",
                "T Decline (3%/yr)",
                "Age (yr)",
                "T (ng/dL)",
                "ng/dL",
                &ages,
                t_high,
            ),
            gauge(
                "age_threshold",
                "Age at Clinical Low T",
                age_threshold,
                40.0,
                90.0,
                "years",
                [60.0, 80.0],
                [50.0, 60.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Clinical low T".into(),
            min: 0.0,
            max: 300.0,
            status: ClinicalStatus::Critical,
        }],
    ));

    // TRT weight trajectory (exp033)
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
    s.ecosystem.primals.push(node(
        "trt_weight",
        "TRT Weight Trajectory",
        NodeType::Compute,
        &["science.endocrine.trt_weight"],
        vec![
            timeseries(
                "weight_loss",
                "Weight Loss",
                "Month",
                "ΔWeight (kg)",
                "kg",
                &months,
                weight_curve,
            ),
            timeseries(
                "waist_loss",
                "Waist Loss",
                "Month",
                "ΔWaist (cm)",
                "cm",
                &months,
                waist_curve,
            ),
        ],
        vec![],
    ));

    // TRT cardiovascular (exp034)
    let ldl_curve: Vec<f64> = months
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
    let sbp_curve: Vec<f64> = months
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
    s.ecosystem.primals.push(node(
        "trt_cardio",
        "TRT Cardiovascular",
        NodeType::Compute,
        &["science.endocrine.trt_cardiovascular"],
        vec![
            timeseries(
                "ldl",
                "LDL Cholesterol",
                "Month",
                "LDL (mg/dL)",
                "mg/dL",
                &months,
                ldl_curve,
            ),
            timeseries(
                "sbp",
                "Systolic BP",
                "Month",
                "SBP (mmHg)",
                "mmHg",
                &months,
                sbp_curve,
            ),
        ],
        vec![
            ClinicalRange {
                label: "LDL optimal".into(),
                min: 0.0,
                max: 130.0,
                status: ClinicalStatus::Normal,
            },
            ClinicalRange {
                label: "SBP normal".into(),
                min: 90.0,
                max: 130.0,
                status: ClinicalStatus::Normal,
            },
        ],
    ));

    // TRT diabetes (exp035)
    let dm_months: Vec<f64> = (0..=120).map(|i| f64::from(i) / 10.0).collect();
    let hba1c_curve: Vec<f64> = dm_months
        .iter()
        .map(|&m| {
            endocrine::hba1c_trajectory(
                m,
                endocrine::diabetes_params::HBA1C_BASELINE,
                endocrine::diabetes_params::HBA1C_DELTA,
                endocrine::diabetes_params::TAU_MONTHS,
            )
        })
        .collect();
    s.ecosystem.primals.push(node(
        "trt_diabetes",
        "TRT Diabetes Outcomes",
        NodeType::Compute,
        &["science.endocrine.trt_diabetes"],
        vec![timeseries(
            "hba1c",
            "HbA1c",
            "Month",
            "HbA1c (%)",
            "%",
            &dm_months,
            hba1c_curve,
        )],
        vec![ClinicalRange {
            label: "HbA1c target".into(),
            min: 4.0,
            max: 7.0,
            status: ClinicalStatus::Normal,
        }],
    ));

    // Gut-TRT axis (exp037)
    let gut_communities = [("Even", 0.95_f64), ("Moderate", 0.7), ("Dominated", 0.3)];
    let mut gut_cats = Vec::new();
    let mut gut_resp = Vec::new();
    for &(name, j) in &gut_communities {
        let w = endocrine::evenness_to_disorder(j, endocrine::gut_axis_params::DISORDER_SCALE);
        let xi =
            endocrine::anderson_localization_length(w, endocrine::gut_axis_params::LATTICE_SIZE);
        let resp = endocrine::gut_metabolic_response(
            xi,
            endocrine::gut_axis_params::LATTICE_SIZE,
            endocrine::gut_axis_params::BASE_RESPONSE_KG,
        );
        gut_cats.push(name.to_string());
        gut_resp.push(resp);
    }
    s.ecosystem.primals.push(node(
        "gut_axis",
        "Testosterone-Gut Axis",
        NodeType::Compute,
        &["science.endocrine.gut_trt_axis"],
        vec![bar(
            "gut_response",
            "Metabolic Response by Gut Health",
            &gut_cats,
            gut_resp,
            "kg weight change",
        )],
        vec![],
    ));

    // HRV-TRT cardiac (exp038)
    let hrv_months: Vec<f64> = (0..=1200).map(|i| f64::from(i) / 10.0).collect();
    let sdnn_curve: Vec<f64> = hrv_months
        .iter()
        .map(|&m| endocrine::hrv_trt_response(40.0, 20.0, 24.0, m))
        .collect();
    let risk_pre = endocrine::cardiac_risk_composite(40.0, 300.0, 1.0);
    let risk_post = endocrine::cardiac_risk_composite(55.0, 500.0, 1.0);
    s.ecosystem.primals.push(node(
        "hrv_cardiac",
        "HRV × TRT Cardiovascular",
        NodeType::Compute,
        &["science.endocrine.hrv_trt"],
        vec![
            timeseries(
                "sdnn_trt",
                "SDNN on TRT",
                "Month",
                "SDNN (ms)",
                "ms",
                &hrv_months,
                sdnn_curve,
            ),
            bar(
                "risk_compare",
                "Cardiac Risk Pre/Post TRT",
                vec!["Pre-TRT".into(), "Post-TRT".into()],
                vec![risk_pre, risk_post],
                "composite",
            ),
            gauge(
                "risk_reduction",
                "Risk Reduction",
                (1.0 - risk_post / risk_pre) * 100.0,
                0.0,
                100.0,
                "%",
                [10.0, 50.0],
                [0.0, 10.0],
            ),
        ],
        vec![],
    ));

    let edges = vec![
        edge("t_im", "t_pellet", "IM → pellet comparison"),
        edge("t_im", "age_decline", "PK → age context"),
        edge("age_decline", "trt_weight", "decline → TRT outcomes"),
        edge("trt_weight", "trt_cardio", "metabolic → cardiovascular"),
        edge("trt_weight", "trt_diabetes", "metabolic → glycemic"),
        edge("trt_weight", "gut_axis", "metabolic → gut-mediated"),
        edge("trt_cardio", "hrv_cardiac", "CV → HRV composite"),
    ];
    (s, edges)
}
