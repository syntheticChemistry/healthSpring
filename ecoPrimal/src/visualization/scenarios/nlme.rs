// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{ClinicalRange, DataChannel, HealthScenario, ScenarioEdge};
use super::{bar, edge, gauge, node, scaffold, scatter3d, timeseries};
use crate::pkpd;

/// Build a complete NLME population PK scenario with real FOCE estimation,
/// NCA metrics, and full diagnostics (CWRES, VPC, GOF).
///
/// Runs FOCE on a 30-subject synthetic testosterone cypionate dataset,
/// then computes NCA on the population-predicted profile, and produces
/// all standard pharmacometric diagnostics.
#[must_use]
pub fn nlme_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut scenario = scaffold(
        "healthSpring NLME Population PK",
        "FOCE estimation, NCA metrics, CWRES, VPC, GOF — sovereign NONMEM replacement",
    );

    let (subjects, result, theta_true, omega_true) = run_foce_estimation();

    build_population_node(&mut scenario, &subjects, &result, &theta_true, &omega_true);
    build_nca_node(&mut scenario);
    build_cwres_node(&mut scenario, &subjects, &result);
    build_vpc_node(&mut scenario, &subjects, &result);
    build_gof_node(&mut scenario, &subjects, &result);

    let edges = vec![
        edge("nlme_population", "nca_metrics", "model → NCA"),
        edge("nlme_population", "cwres_diagnostics", "fit → CWRES"),
        edge("nlme_population", "vpc_check", "fit → VPC"),
        edge("nlme_population", "gof_fit", "fit → GOF"),
        edge("cwres_diagnostics", "gof_fit", "residuals → GOF"),
    ];
    (scenario, edges)
}

/// Shared FOCE run for all downstream nodes.
fn run_foce_estimation() -> (Vec<pkpd::Subject>, pkpd::NlmeResult, Vec<f64>, Vec<f64>) {
    let theta_true = vec![-2.44, 4.25, -0.77];
    let omega_true = vec![0.0625, 0.04, 0.1225];
    let sigma_true = 0.01;
    let times: Vec<f64> = (0..14).map(f64::from).collect();

    let subjects = pkpd::generate_synthetic_population(&pkpd::SyntheticPopConfig {
        model: pkpd::oral_one_compartment_model,
        theta: &theta_true,
        omega: &omega_true,
        sigma: sigma_true,
        n_subjects: 30,
        times: &times,
        dose: 100.0,
        seed: 42,
    });

    let config = pkpd::NlmeConfig {
        n_theta: 3,
        n_eta: 3,
        max_iter: 150,
        tol: crate::tolerances::NLME_DEFAULT_TOL,
        seed: 12_345,
    };

    let result = pkpd::foce(
        pkpd::oral_one_compartment_model,
        &subjects,
        &theta_true,
        &omega_true,
        sigma_true,
        &config,
    );

    (subjects, result, theta_true, omega_true)
}

#[expect(
    clippy::too_many_lines,
    reason = "constructs 10 IPRED curves + 2 bar + 3 distribution + 2 gauge + scatter3d"
)]
fn build_population_node(
    scenario: &mut HealthScenario,
    subjects: &[pkpd::Subject],
    result: &pkpd::NlmeResult,
    theta_true: &[f64],
    omega_true: &[f64],
) {
    let mut channels = Vec::new();

    for (idx, subj) in subjects.iter().enumerate().take(10) {
        let eta = &result.individual_etas[idx];
        let pred: Vec<f64> = subj
            .times
            .iter()
            .map(|&t| pkpd::oral_one_compartment_model(&result.theta, eta, subj.dose, t))
            .collect();
        channels.push(timeseries(
            &format!("ipred_subj_{idx}"),
            &format!("Subject {idx} IPRED"),
            "Time (hr)",
            "C (mg/L)",
            "mg/L",
            subj.times.clone(),
            pred,
        ));
    }

    let param_names = ["ln(CL)", "ln(Vd)", "ln(ka)"];
    channels.push(bar(
        "theta_est",
        "Estimated vs True Theta",
        param_names
            .iter()
            .flat_map(|&n| [format!("{n} est"), format!("{n} true")])
            .collect(),
        result
            .theta
            .iter()
            .zip(theta_true.iter())
            .flat_map(|(&est, &truth)| [est, truth])
            .collect(),
        "log-scale",
    ));

    channels.push(bar(
        "omega_est",
        "Estimated vs True Omega (BSV)",
        param_names
            .iter()
            .flat_map(|&n| [format!("{n} est"), format!("{n} true")])
            .collect(),
        result
            .omega_diag
            .iter()
            .zip(omega_true.iter())
            .flat_map(|(&est, &truth)| [est, truth])
            .collect(),
        "variance",
    ));

    for (dim, &name) in param_names.iter().enumerate() {
        let etas: Vec<f64> = result.individual_etas.iter().map(|e| e[dim]).collect();
        #[expect(clippy::cast_precision_loss, reason = "subject count ≤ 30")]
        let n_f = etas.len() as f64;
        let mean_eta = etas.iter().sum::<f64>() / n_f;
        let std_eta = (etas.iter().map(|&e| (e - mean_eta).powi(2)).sum::<f64>() / n_f).sqrt();
        channels.push(DataChannel::Distribution {
            id: format!("eta_{dim}"),
            label: format!("{name} Individual Etas"),
            unit: "deviation".into(),
            values: etas,
            mean: mean_eta,
            std: std_eta,
            patient_value: mean_eta,
        });
    }

    channels.push(gauge(
        "objective",
        "Objective Function (-2LL)",
        result.objective,
        0.0,
        5000.0,
        "-2LL",
        [0.0, 2000.0],
        [2000.0, 4000.0],
    ));
    channels.push(gauge(
        "sigma",
        "Residual Error (sigma)",
        result.sigma,
        0.0,
        1.0,
        "variance",
        [0.0, 0.05],
        [0.05, 0.2],
    ));

    let cls: Vec<f64> = result
        .individual_etas
        .iter()
        .map(|e| (result.theta[0] + e[0]).exp())
        .collect();
    let vds: Vec<f64> = result
        .individual_etas
        .iter()
        .map(|e| (result.theta[1] + e[1]).exp())
        .collect();
    let aucs: Vec<f64> = cls
        .iter()
        .map(|&cl| 100.0 / cl.max(crate::tolerances::NLME_DEFAULT_TOL))
        .collect();
    channels.push(scatter3d(
        "pop_param_space",
        "Individual CL x Vd x AUC",
        cls,
        vds,
        aucs,
        vec![],
        "mixed",
    ));

    scenario.ecosystem.primals.push(node(
        "nlme_population",
        "NLME Population PK (FOCE)",
        "compute",
        &["science.pkpd.nlme_foce", "science.pkpd.population_pk"],
        channels,
        vec![],
    ));
}

fn build_nca_node(scenario: &mut HealthScenario) {
    let ke = 0.087 / 70.0;
    let vd = 70.0;
    let dose = 100.0;
    let c0 = dose / vd;
    let times: Vec<f64> = (0..1000).map(|i| 500.0 * f64::from(i) / 999.0).collect();
    let concs: Vec<f64> = times.iter().map(|&t| c0 * (-ke * t).exp()).collect();

    let nca = pkpd::nca_iv(&times, &concs, dose, 3);

    scenario.ecosystem.primals.push(node(
        "nca_metrics",
        "Non-Compartmental Analysis",
        "compute",
        &["science.pkpd.nca"],
        vec![
            timeseries(
                "nca_ct_curve",
                "Concentration-Time Profile",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                times,
                concs,
            ),
            gauge(
                "nca_cmax",
                "Cmax",
                nca.cmax,
                0.0,
                5.0,
                "mg/L",
                [0.5, 2.0],
                [2.0, 4.0],
            ),
            gauge(
                "nca_tmax",
                "Tmax",
                nca.tmax,
                0.0,
                10.0,
                "hr",
                [0.0, 2.0],
                [2.0, 5.0],
            ),
            gauge(
                "nca_auc_inf",
                "AUC(0-inf)",
                nca.auc_inf,
                0.0,
                2000.0,
                "mg·hr/L",
                [100.0, 1500.0],
                [1500.0, 1900.0],
            ),
            gauge(
                "nca_lambda_z",
                "Lambda-z",
                nca.lambda_z,
                0.0,
                0.01,
                "1/hr",
                [0.0005, 0.005],
                [0.005, 0.009],
            ),
            gauge(
                "nca_half_life",
                "Half-Life",
                nca.half_life,
                0.0,
                1000.0,
                "hr",
                [100.0, 700.0],
                [700.0, 900.0],
            ),
            gauge(
                "nca_cl",
                "Clearance",
                nca.cl_obs,
                0.0,
                0.2,
                "L/hr",
                [0.01, 0.1],
                [0.1, 0.18],
            ),
            gauge(
                "nca_vss",
                "Vss",
                nca.vss_obs,
                0.0,
                100.0,
                "L",
                [10.0, 80.0],
                [80.0, 95.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Therapeutic testosterone".into(),
            min: 300.0,
            max: 1000.0,
            status: "normal".into(),
        }],
    ));
}

fn build_cwres_node(
    scenario: &mut HealthScenario,
    subjects: &[pkpd::Subject],
    result: &pkpd::NlmeResult,
) {
    let cwres_data = pkpd::compute_cwres(pkpd::oral_one_compartment_model, subjects, result);
    let summary = pkpd::cwres_summary(&cwres_data);

    scenario.ecosystem.primals.push(node(
        "cwres_diagnostics",
        "CWRES Diagnostics",
        "compute",
        &["science.pkpd.nlme_diagnostics"],
        vec![
            timeseries(
                "cwres_vs_time",
                "CWRES vs Time",
                "Time (hr)",
                "CWRES",
                "standardized",
                summary.all_times.clone(),
                summary.all_residuals.clone(),
            ),
            DataChannel::Distribution {
                id: "cwres_histogram".into(),
                label: "CWRES Distribution".into(),
                unit: "standardized".into(),
                values: summary.all_residuals,
                mean: summary.mean,
                std: summary.std_dev,
                patient_value: 0.0,
            },
            gauge(
                "cwres_mean",
                "CWRES Mean",
                summary.mean,
                -3.0,
                3.0,
                "standardized",
                [-0.5, 0.5],
                [-1.5, -0.5],
            ),
            gauge(
                "cwres_std",
                "CWRES Std Dev",
                summary.std_dev,
                0.0,
                3.0,
                "standardized",
                [0.8, 1.2],
                [0.5, 0.8],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Well-specified model".into(),
                min: -2.0,
                max: 2.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Model misspecification".into(),
                min: -4.0,
                max: -2.0,
                status: "warning".into(),
            },
        ],
    ));
}

fn build_vpc_node(
    scenario: &mut HealthScenario,
    subjects: &[pkpd::Subject],
    result: &pkpd::NlmeResult,
) {
    let vpc = pkpd::compute_vpc(
        pkpd::oral_one_compartment_model,
        subjects,
        result,
        &pkpd::VpcConfig {
            n_simulations: 50,
            n_bins: 8,
            seed: 42,
        },
    );

    scenario.ecosystem.primals.push(node(
        "vpc_check",
        "Visual Predictive Check",
        "compute",
        &["science.pkpd.nlme_diagnostics"],
        vec![
            timeseries(
                "vpc_obs_p5",
                "Observed 5th Percentile",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                vpc.time_bins.clone(),
                vpc.obs_p5,
            ),
            timeseries(
                "vpc_obs_p50",
                "Observed Median",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                vpc.time_bins.clone(),
                vpc.obs_p50,
            ),
            timeseries(
                "vpc_obs_p95",
                "Observed 95th Percentile",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                vpc.time_bins.clone(),
                vpc.obs_p95,
            ),
            timeseries(
                "vpc_sim_p5",
                "Simulated 5th Percentile",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                vpc.time_bins.clone(),
                vpc.sim_p5,
            ),
            timeseries(
                "vpc_sim_p50",
                "Simulated Median",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                vpc.time_bins.clone(),
                vpc.sim_p50,
            ),
            timeseries(
                "vpc_sim_p95",
                "Simulated 95th Percentile",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                vpc.time_bins,
                vpc.sim_p95,
            ),
        ],
        vec![],
    ));
}

fn build_gof_node(
    scenario: &mut HealthScenario,
    subjects: &[pkpd::Subject],
    result: &pkpd::NlmeResult,
) {
    let gof = pkpd::compute_gof(pkpd::oral_one_compartment_model, subjects, result);

    scenario.ecosystem.primals.push(node(
        "gof_fit",
        "Goodness-of-Fit",
        "compute",
        &["science.pkpd.nlme_diagnostics"],
        vec![
            timeseries(
                "gof_obs_vs_ipred",
                "Observed vs Individual Predicted",
                "IPRED (mg/L)",
                "Observed (mg/L)",
                "mg/L",
                gof.individual_predicted.clone(),
                gof.observed.clone(),
            ),
            timeseries(
                "gof_obs_vs_ppred",
                "Observed vs Population Predicted",
                "PPRED (mg/L)",
                "Observed (mg/L)",
                "mg/L",
                gof.population_predicted,
                gof.observed,
            ),
            timeseries(
                "gof_iwres_vs_time",
                "Individual Residuals vs Time",
                "Time (hr)",
                "IWRES",
                "standardized",
                gof.times,
                gof.individual_residuals,
            ),
            gauge(
                "gof_r2_ind",
                "R² (Individual)",
                gof.r_squared_individual,
                0.0,
                1.0,
                "R²",
                [0.8, 1.0],
                [0.5, 0.8],
            ),
            gauge(
                "gof_r2_pop",
                "R² (Population)",
                gof.r_squared_population,
                0.0,
                1.0,
                "R²",
                [0.5, 1.0],
                [0.2, 0.5],
            ),
        ],
        vec![],
    ));
}
