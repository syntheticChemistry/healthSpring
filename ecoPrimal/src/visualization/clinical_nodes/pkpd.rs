// SPDX-License-Identifier: AGPL-3.0-or-later
//! PK/PD clinical node builders for TRT protocols and population comparison.

use crate::endocrine::{self, testosterone_cypionate as tc};
use crate::visualization::clinical::{PatientTrtProfile, TrtProtocol};
use crate::visualization::scenarios::{gauge, node, timeseries};
use crate::visualization::types::{
    ClinicalRange, ClinicalStatus, DataChannel, NodeType, ScenarioNode,
};

#[expect(
    clippy::too_many_lines,
    reason = "three protocol variants with PK compute"
)]
pub fn protocol_node(p: &PatientTrtProfile) -> ScenarioNode {
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
        NodeType::Compute,
        &["clinical.treatment.testosterone_pk"],
        vec![
            timeseries(
                "pk_curve",
                &format!("Testosterone Level — {protocol_name}"),
                "Time (days)",
                "T (ng/mL)",
                "ng/mL",
                &days,
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
                status: ClinicalStatus::Normal,
            },
            ClinicalRange {
                label: "Supraphysiologic".into(),
                min: 35.0,
                max: 60.0,
                status: ClinicalStatus::Warning,
            },
        ],
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "population count ≤ 200 fits in f64"
)]
pub fn population_node(p: &PatientTrtProfile) -> ScenarioNode {
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
        NodeType::Storage,
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
            status: ClinicalStatus::Normal,
        }],
    )
}
