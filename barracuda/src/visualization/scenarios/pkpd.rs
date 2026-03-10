// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{ClinicalRange, DataChannel, HealthScenario, ScenarioEdge};
use super::{bar, edge, gauge, node, scaffold, scatter3d, timeseries};
use crate::pkpd;

/// Build a complete PK/PD study scenario with real computed data.
#[must_use]
#[expect(clippy::too_many_lines, reason = "6 sub-studies, each compact")]
#[expect(clippy::cast_precision_loss, reason = "population size ≤ 200")]
pub fn pkpd_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold(
        "healthSpring PK/PD Study",
        "Hill dose-response, compartmental PK, population PK, PBPK — all 6 experiments",
    );

    // Hill dose-response (exp001)
    let concs: Vec<f64> = (0..50)
        .map(|i| 10_f64.powf(-1.0 + 5.0 * f64::from(i) / 49.0))
        .collect();
    let mut hill_channels = Vec::new();
    let mut ec_cats = Vec::new();
    let mut ec50_vals = Vec::new();
    for drug in pkpd::ALL_INHIBITORS {
        let responses = pkpd::hill_sweep(drug.ic50_jak1_nm, drug.hill_n, 1.0, &concs);
        hill_channels.push(timeseries(
            &format!("hill_{}", drug.name.to_lowercase()),
            &format!("{} Hill Curve", drug.name),
            "Concentration (nM)",
            "Response",
            "fractional",
            concs.clone(),
            responses,
        ));
        let ec = pkpd::compute_ec_values(drug.ic50_jak1_nm, drug.hill_n);
        ec_cats.push(drug.name.to_string());
        ec50_vals.push(ec.ec50);
    }
    hill_channels.push(bar(
        "ec50_compare",
        "EC50 Comparison",
        ec_cats,
        ec50_vals,
        "nM",
    ));
    s.ecosystem.primals.push(node(
        "hill",
        "Hill Dose-Response",
        "compute",
        &["science.pkpd.hill_dose_response"],
        hill_channels,
        vec![],
    ));

    // One-compartment oral PK (exp002)
    let times: Vec<f64> = (0..=480).map(|i| f64::from(i) / 10.0).collect();
    let oral_concs: Vec<f64> = times
        .iter()
        .map(|&t| pkpd::pk_oral_one_compartment(4.0, 0.8, 80.0, 1.5, 0.15, t))
        .collect();
    let (cmax, tmax) = pkpd::find_cmax_tmax(&times, &oral_concs);
    let auc = pkpd::auc_trapezoidal(&times, &oral_concs);
    s.ecosystem.primals.push(node(
        "one_comp",
        "One-Compartment Oral PK",
        "compute",
        &["science.pkpd.one_compartment_pk"],
        vec![
            timeseries(
                "oral_pk",
                "Oral Concentration",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                times,
                oral_concs,
            ),
            gauge(
                "cmax",
                "Cmax",
                cmax,
                0.0,
                0.1,
                "mg/L",
                [0.01, 0.05],
                [0.05, 0.08],
            ),
            gauge(
                "auc",
                "AUC",
                auc,
                0.0,
                5.0,
                "mg·hr/L",
                [0.5, 3.0],
                [3.0, 4.5],
            ),
            gauge(
                "tmax",
                "Tmax",
                tmax,
                0.0,
                10.0,
                "hr",
                [0.5, 3.0],
                [3.0, 6.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Therapeutic".into(),
                min: 0.01,
                max: 0.05,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Supratherapeutic".into(),
                min: 0.05,
                max: 0.08,
                status: "warning".into(),
            },
        ],
    ));

    // Two-compartment IV PK (exp003)
    let t2c: Vec<f64> = (0..=1680).map(|i| f64::from(i) / 10.0).collect();
    let (k10, k12, k21) = (0.087, 0.06, 0.05);
    let central: Vec<f64> = t2c
        .iter()
        .map(|&t| pkpd::pk_two_compartment_iv(500.0, 18.0, k10, k12, k21, t))
        .collect();
    let (alpha, beta) = pkpd::micro_to_macro(k10, k12, k21);
    s.ecosystem.primals.push(node(
        "two_comp",
        "Two-Compartment IV PK",
        "compute",
        &["science.pkpd.two_compartment_pk"],
        vec![
            timeseries(
                "central_pk",
                "Central Compartment",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                t2c,
                central,
            ),
            gauge(
                "alpha",
                "α (distribution)",
                alpha,
                0.0,
                0.5,
                "1/hr",
                [0.05, 0.3],
                [0.3, 0.45],
            ),
            gauge(
                "beta",
                "β (elimination)",
                beta,
                0.0,
                0.1,
                "1/hr",
                [0.005, 0.05],
                [0.05, 0.08],
            ),
        ],
        vec![],
    ));

    // mAb cross-species (exp004)
    let t_days: Vec<f64> = (0..=560).map(|i| f64::from(i) / 10.0).collect();
    let loki_vd = pkpd::allometric_scale(
        pkpd::lokivetmab_canine::VD_L_KG * pkpd::lokivetmab_canine::BW_KG,
        pkpd::lokivetmab_canine::BW_KG,
        70.0,
        pkpd::allometric_exp::VOLUME,
    );
    let loki_hl = pkpd::allometric_scale(
        pkpd::lokivetmab_canine::HALF_LIFE_DAYS,
        pkpd::lokivetmab_canine::BW_KG,
        70.0,
        pkpd::allometric_exp::HALF_LIFE,
    );
    let mab_concs: Vec<f64> = t_days
        .iter()
        .map(|&t| pkpd::mab_pk_sc(200.0, loki_vd, loki_hl, t))
        .collect();
    let (mab_cmax, _mab_tmax) = pkpd::find_cmax_tmax(&t_days, &mab_concs);
    s.ecosystem.primals.push(node(
        "mab",
        "mAb Cross-Species Transfer",
        "compute",
        &["science.pkpd.allometric_scaling"],
        vec![
            timeseries(
                "mab_pk",
                "Scaled mAb (Lokivetmab→Human)",
                "Time (days)",
                "C (mg/L)",
                "mg/L",
                t_days,
                mab_concs,
            ),
            gauge(
                "mab_cmax",
                "Cmax",
                mab_cmax,
                0.0,
                50.0,
                "mg/L",
                [5.0, 30.0],
                [30.0, 45.0],
            ),
        ],
        vec![],
    ));

    // Population PK (exp005)
    let pop_times: Vec<f64> = (0..=240).map(|i| f64::from(i) / 10.0).collect();
    let n_patients: u32 = 200;
    let cl_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            let frac = f64::from(i) / f64::from(n_patients - 1);
            pkpd::pop_baricitinib::CL.typical * 0.8f64.mul_add(frac, 0.6)
        })
        .collect();
    let vd_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            let frac = f64::from(i) / f64::from(n_patients - 1);
            pkpd::pop_baricitinib::VD.typical * 0.6f64.mul_add(frac, 0.7)
        })
        .collect();
    let ka_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            let frac = f64::from(i) / f64::from(n_patients - 1);
            pkpd::pop_baricitinib::KA.typical * 0.4f64.mul_add(frac, 0.8)
        })
        .collect();
    let pop = pkpd::population_pk_cpu(
        n_patients as usize,
        &cl_vals,
        &vd_vals,
        &ka_vals,
        pkpd::pop_baricitinib::DOSE_MG,
        pkpd::pop_baricitinib::F_BIOAVAIL,
        &pop_times,
    );
    let aucs: Vec<f64> = pop.iter().map(|p| p.auc).collect();
    let cmaxs: Vec<f64> = pop.iter().map(|p| p.cmax).collect();
    let n_pop = aucs.len() as f64;
    let auc_mean = aucs.iter().sum::<f64>() / n_pop;
    let auc_std = (aucs.iter().map(|a| (a - auc_mean).powi(2)).sum::<f64>() / n_pop).sqrt();
    let cmax_mean = cmaxs.iter().sum::<f64>() / n_pop;
    let cmax_std = (cmaxs.iter().map(|c| (c - cmax_mean).powi(2)).sum::<f64>() / n_pop).sqrt();
    s.ecosystem.primals.push(node(
        "pop_pk",
        "Population PK (n=200)",
        "storage",
        &["science.pkpd.population_pk"],
        vec![
            DataChannel::Distribution {
                id: "pop_auc".into(),
                label: "AUC Distribution".into(),
                unit: "mg·hr/L".into(),
                values: aucs.clone(),
                mean: auc_mean,
                std: auc_std,
                patient_value: auc_mean,
            },
            DataChannel::Distribution {
                id: "pop_cmax".into(),
                label: "Cmax Distribution".into(),
                unit: "mg/L".into(),
                values: cmaxs,
                mean: cmax_mean,
                std: cmax_std,
                patient_value: cmax_mean,
            },
            scatter3d(
                "pop_pk_3d",
                "Population PK: CL × Vd × AUC",
                cl_vals,
                vd_vals,
                aucs,
                vec![],
                "mixed",
            ),
        ],
        vec![],
    ));

    // PBPK (exp006)
    let tissues = pkpd::standard_human_tissues();
    let (pbpk_times, pbpk_venous, pbpk_state) =
        pkpd::pbpk_iv_simulate(&tissues, 100.0, 5.0, 24.0, 0.01);
    let pbpk_auc = pkpd::pbpk_auc(&pbpk_times, &pbpk_venous);
    let tissue_names: Vec<String> = tissues.iter().map(|t| t.name.into()).collect();
    let tissue_concs = pbpk_state.concentrations;
    let co = pkpd::cardiac_output(&tissues);
    let ts_step = pbpk_times.len().max(1) / 500;
    let ts_step = ts_step.max(1);
    let pbpk_times_ds: Vec<f64> = pbpk_times.iter().step_by(ts_step).copied().collect();
    let pbpk_venous_ds: Vec<f64> = pbpk_venous.iter().step_by(ts_step).copied().collect();

    // Per-tissue concentration profiles over time
    let tp = pkpd::pbpk_iv_tissue_profiles(&tissues, 100.0, 5.0, 24.0, 0.01, 300);
    let mut pbpk_channels = vec![timeseries(
        "pbpk_venous",
        "Venous Concentration",
        "Time (hr)",
        "C (mg/L)",
        "mg/L",
        pbpk_times_ds,
        pbpk_venous_ds,
    )];
    for (i, name) in tp.tissue_names.iter().enumerate() {
        pbpk_channels.push(timeseries(
            &format!("pbpk_{name}"),
            &format!("{}{} Concentration", name[..1].to_uppercase(), &name[1..]),
            "Time (hr)",
            "C (mg/L)",
            "mg/L",
            tp.times.clone(),
            tp.profiles[i].clone(),
        ));
    }
    pbpk_channels.push(bar(
        "tissue_concs",
        "Tissue Concentrations (24 hr)",
        tissue_names,
        tissue_concs,
        "mg/L",
    ));
    pbpk_channels.push(gauge(
        "pbpk_auc",
        "AUC (venous)",
        pbpk_auc,
        0.0,
        500.0,
        "mg·hr/L",
        [10.0, 200.0],
        [200.0, 400.0],
    ));
    pbpk_channels.push(gauge(
        "cardiac_output",
        "Cardiac Output",
        co,
        0.0,
        500.0,
        "L/hr",
        [250.0, 400.0],
        [200.0, 250.0],
    ));

    s.ecosystem.primals.push(node(
        "pbpk",
        "PBPK 5-Tissue Model",
        "compute",
        &["science.pkpd.pbpk"],
        pbpk_channels,
        vec![],
    ));

    let edges = vec![
        edge("hill", "one_comp", "dose-response → PK"),
        edge("one_comp", "two_comp", "1-comp → 2-comp"),
        edge("one_comp", "mab", "allometric scaling"),
        edge("one_comp", "pop_pk", "population variability"),
        edge("one_comp", "pbpk", "PBPK tissue distribution"),
    ];
    (s, edges)
}
