// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-track petalTongue scenario builders.
//!
//! Each builder calls real healthSpring math and wraps the outputs in
//! `DataChannel` / `ScenarioNode` / `HealthScenario` so petalTongue can
//! render them directly.

use super::types::{
    Animations, CapReqs, ClinicalRange, DataChannel, Ecosystem, HealthScenario, NeuralApi,
    Performance, Position, ScenarioEdge, ScenarioNode, SensoryConfig, UiConfig,
};
use crate::{biosignal, endocrine, microbiome, pkpd};

fn scaffold(name: &str, description: &str) -> HealthScenario {
    HealthScenario {
        name: name.into(),
        description: description.into(),
        version: "2.0.0".into(),
        mode: "live-ecosystem".into(),
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
            theme: "benchtop-dark".into(),
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
        },
        ecosystem: Ecosystem { primals: vec![] },
        neural_api: NeuralApi { enabled: false },
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "internal helper — all args have clear roles"
)]
fn gauge(
    id: &str,
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    normal: [f64; 2],
    warn: [f64; 2],
) -> DataChannel {
    DataChannel::Gauge {
        id: id.into(),
        label: label.into(),
        value,
        min,
        max,
        unit: unit.into(),
        normal_range: normal,
        warning_range: warn,
    }
}

fn timeseries(
    id: &str,
    label: &str,
    x_label: &str,
    y_label: &str,
    unit: &str,
    xs: Vec<f64>,
    ys: Vec<f64>,
) -> DataChannel {
    DataChannel::TimeSeries {
        id: id.into(),
        label: label.into(),
        x_label: x_label.into(),
        y_label: y_label.into(),
        unit: unit.into(),
        x_values: xs,
        y_values: ys,
    }
}

fn bar(id: &str, label: &str, cats: Vec<String>, vals: Vec<f64>, unit: &str) -> DataChannel {
    DataChannel::Bar {
        id: id.into(),
        label: label.into(),
        categories: cats,
        values: vals,
        unit: unit.into(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "internal helper — all args have clear roles"
)]
fn node(
    id: &str,
    name: &str,
    node_type: &str,
    x: f64,
    y: f64,
    caps: &[&str],
    channels: Vec<DataChannel>,
    ranges: Vec<ClinicalRange>,
) -> ScenarioNode {
    ScenarioNode {
        id: id.into(),
        name: name.into(),
        node_type: node_type.into(),
        family: "healthspring".into(),
        status: "healthy".into(),
        health: 100,
        confidence: 95,
        position: Position { x, y },
        capabilities: caps.iter().map(|s| (*s).into()).collect(),
        data_channels: channels,
        clinical_ranges: ranges,
    }
}

fn edge(from: &str, to: &str, label: &str) -> ScenarioEdge {
    ScenarioEdge {
        from: from.into(),
        to: to.into(),
        edge_type: "data-flow".into(),
        label: label.into(),
    }
}

// ---------------------------------------------------------------------------
// Track 1: PK/PD
// ---------------------------------------------------------------------------

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
        100.0,
        100.0,
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
        300.0,
        100.0,
        &["science.pkpd.one_compartment_pk"],
        vec![
            timeseries(
                "oral_pk",
                "Oral Concentration",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                times.clone(),
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
        500.0,
        100.0,
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
        700.0,
        100.0,
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
            pkpd::pop_baricitinib::CL.typical * (0.6 + 0.8 * frac)
        })
        .collect();
    let vd_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            let frac = f64::from(i) / f64::from(n_patients - 1);
            pkpd::pop_baricitinib::VD.typical * (0.7 + 0.6 * frac)
        })
        .collect();
    let ka_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            let frac = f64::from(i) / f64::from(n_patients - 1);
            pkpd::pop_baricitinib::KA.typical * (0.8 + 0.4 * frac)
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
        300.0,
        300.0,
        &["science.pkpd.population_pk"],
        vec![
            DataChannel::Distribution {
                id: "pop_auc".into(),
                label: "AUC Distribution".into(),
                unit: "mg·hr/L".into(),
                values: aucs,
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
        ],
        vec![],
    ));

    // PBPK (exp006)
    let tissues = pkpd::standard_human_tissues();
    let (pbpk_times, pbpk_venous, pbpk_state) =
        pkpd::pbpk_iv_simulate(&tissues, 100.0, 5.0, 24.0, 0.01);
    let pbpk_auc = pkpd::pbpk_auc(&pbpk_times, &pbpk_venous);
    let tissue_names: Vec<String> = tissues.iter().map(|t| t.name.into()).collect();
    let tissue_concs = pbpk_state.concentrations.clone();
    let co = pkpd::cardiac_output(&tissues);
    let ts_step = pbpk_times.len().max(1) / 500;
    let ts_step = ts_step.max(1);
    let pbpk_times_ds: Vec<f64> = pbpk_times.iter().step_by(ts_step).copied().collect();
    let pbpk_venous_ds: Vec<f64> = pbpk_venous.iter().step_by(ts_step).copied().collect();
    s.ecosystem.primals.push(node(
        "pbpk",
        "PBPK 5-Tissue Model",
        "compute",
        500.0,
        300.0,
        &["science.pkpd.pbpk"],
        vec![
            timeseries(
                "pbpk_venous",
                "Venous Concentration",
                "Time (hr)",
                "C (mg/L)",
                "mg/L",
                pbpk_times_ds,
                pbpk_venous_ds,
            ),
            bar(
                "tissue_concs",
                "Tissue Concentrations (24 hr)",
                tissue_names,
                tissue_concs,
                "mg/L",
            ),
            gauge(
                "pbpk_auc",
                "AUC (venous)",
                pbpk_auc,
                0.0,
                500.0,
                "mg·hr/L",
                [10.0, 200.0],
                [200.0, 400.0],
            ),
            gauge(
                "cardiac_output",
                "Cardiac Output",
                co,
                0.0,
                500.0,
                "L/hr",
                [250.0, 400.0],
                [200.0, 250.0],
            ),
        ],
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

// ---------------------------------------------------------------------------
// Track 2: Microbiome
// ---------------------------------------------------------------------------

/// Build a complete microbiome study scenario with real computed data.
#[must_use]
#[expect(clippy::too_many_lines, reason = "4 sub-studies, each compact")]
#[expect(
    clippy::cast_precision_loss,
    reason = "lattice size ≤ 50, well within f64 mantissa"
)]
pub fn microbiome_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut scenario = scaffold(
        "healthSpring Microbiome Study",
        "Diversity indices, Anderson lattice, C. diff resistance, FMT — 4 experiments",
    );

    let communities: [(&str, Vec<f64>); 4] = [
        ("Healthy", microbiome::communities::HEALTHY_GUT.to_vec()),
        ("Dysbiotic", microbiome::communities::DYSBIOTIC_GUT.to_vec()),
        ("C. diff", microbiome::communities::CDIFF_COLONIZED.to_vec()),
        ("Even", microbiome::communities::PERFECTLY_EVEN.to_vec()),
    ];

    // Diversity indices (exp010)
    let mut shannon_vals = Vec::new();
    let mut simpson_vals = Vec::new();
    let mut pielou_vals = Vec::new();
    let mut cats = Vec::new();
    for (name, ab) in &communities {
        cats.push((*name).to_string());
        shannon_vals.push(microbiome::shannon_index(ab));
        simpson_vals.push(microbiome::simpson_index(ab));
        pielou_vals.push(microbiome::pielou_evenness(ab));
    }
    scenario.ecosystem.primals.push(node(
        "diversity",
        "Diversity Indices",
        "compute",
        100.0,
        100.0,
        &["science.microbiome.diversity"],
        vec![
            bar("shannon", "Shannon H′", cats.clone(), shannon_vals, "nats"),
            bar(
                "simpson",
                "Simpson D",
                cats.clone(),
                simpson_vals,
                "probability",
            ),
            bar(
                "pielou",
                "Pielou J",
                cats.clone(),
                pielou_vals.clone(),
                "evenness",
            ),
        ],
        vec![
            ClinicalRange {
                label: "Healthy Shannon".into(),
                min: 2.5,
                max: 4.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Dysbiotic Shannon".into(),
                min: 0.0,
                max: 1.5,
                status: "critical".into(),
            },
        ],
    ));

    // Anderson lattice (exp011)
    let healthy_j = microbiome::pielou_evenness(&microbiome::communities::HEALTHY_GUT);
    let dysbiotic_j = microbiome::pielou_evenness(&microbiome::communities::DYSBIOTIC_GUT);
    let w_healthy = microbiome::evenness_to_disorder(healthy_j, 5.0);
    let w_dysbiotic = microbiome::evenness_to_disorder(dysbiotic_j, 5.0);
    let lattice_size: usize = 50;
    let half = lattice_size / 2;
    let disorder_h: Vec<f64> = (0..lattice_size)
        .map(|i| {
            let offset = if i >= half {
                (i - half) as f64
            } else {
                -((half - i) as f64)
            };
            w_healthy * (1.0 + 0.1 * offset)
        })
        .collect();
    let ham = microbiome::anderson_hamiltonian_1d(&disorder_h, 1.0);
    let eigvec: Vec<f64> = ham.iter().take(lattice_size).copied().collect();
    let ipr = microbiome::inverse_participation_ratio(&eigvec);
    let xi = microbiome::localization_length_from_ipr(ipr);
    let cr = microbiome::colonization_resistance(xi);
    scenario.ecosystem.primals.push(node(
        "anderson",
        "Anderson Gut Lattice",
        "compute",
        300.0,
        100.0,
        &["science.microbiome.anderson_lattice"],
        vec![
            gauge(
                "ipr",
                "Inverse Participation Ratio",
                ipr,
                0.0,
                1.0,
                "dimensionless",
                [0.0, 0.1],
                [0.1, 0.5],
            ),
            gauge(
                "xi",
                "Localization Length ξ",
                xi,
                0.0,
                100.0,
                "sites",
                [5.0, 50.0],
                [1.0, 5.0],
            ),
            gauge(
                "cr",
                "Colonization Resistance",
                cr,
                0.0,
                1.0,
                "1/ξ",
                [0.02, 0.5],
                [0.5, 0.9],
            ),
            bar(
                "disorder",
                "Anderson Disorder W",
                vec!["Healthy".into(), "Dysbiotic".into()],
                vec![w_healthy, w_dysbiotic],
                "a.u.",
            ),
        ],
        vec![],
    ));

    // C. diff resistance (exp012)
    let mut cr_cats = Vec::new();
    let mut cr_vals = Vec::new();
    for (name, ab) in &communities {
        let j = microbiome::pielou_evenness(ab);
        let w = microbiome::evenness_to_disorder(j, 5.0);
        let disorder_v: Vec<f64> = (0..lattice_size)
            .map(|i| {
                let offset = if i >= half {
                    (i - half) as f64
                } else {
                    -((half - i) as f64)
                };
                w * (1.0 + 0.1 * offset)
            })
            .collect();
        let ham_v = microbiome::anderson_hamiltonian_1d(&disorder_v, 1.0);
        let ev: Vec<f64> = ham_v.iter().take(lattice_size).copied().collect();
        let local_ipr = microbiome::inverse_participation_ratio(&ev);
        let local_xi = microbiome::localization_length_from_ipr(local_ipr);
        cr_cats.push((*name).to_string());
        cr_vals.push(microbiome::colonization_resistance(local_xi));
    }
    scenario.ecosystem.primals.push(node(
        "cdiff",
        "C. diff Colonization Resistance",
        "compute",
        500.0,
        100.0,
        &["science.microbiome.cdiff_resistance"],
        vec![bar(
            "cr_compare",
            "Colonization Resistance by Community",
            cr_cats,
            cr_vals,
            "1/ξ",
        )],
        vec![ClinicalRange {
            label: "Protective CR".into(),
            min: 0.05,
            max: 1.0,
            status: "normal".into(),
        }],
    ));

    // FMT engraftment (exp013)
    let donor = &microbiome::communities::HEALTHY_GUT[..];
    let recipient = &microbiome::communities::CDIFF_COLONIZED[..];
    let engraftments = [0.2, 0.4, 0.6, 0.8, 1.0];
    let mut eng_x = Vec::new();
    let mut shannon_y = Vec::new();
    let mut bc_y = Vec::new();
    for &e in &engraftments {
        let post = microbiome::fmt_blend(donor, recipient, e);
        eng_x.push(e);
        shannon_y.push(microbiome::shannon_index(&post));
        bc_y.push(microbiome::bray_curtis(&post, donor));
    }
    scenario.ecosystem.primals.push(node(
        "fmt",
        "FMT Engraftment",
        "compute",
        300.0,
        300.0,
        &["science.microbiome.fmt"],
        vec![
            timeseries(
                "fmt_shannon",
                "Shannon vs Engraftment",
                "Engraftment",
                "Shannon H′",
                "nats",
                eng_x.clone(),
                shannon_y,
            ),
            timeseries(
                "fmt_bc",
                "Bray-Curtis vs Engraftment",
                "Engraftment",
                "BC Dissimilarity",
                "BC",
                eng_x,
                bc_y,
            ),
        ],
        vec![],
    ));

    let edges = vec![
        edge("diversity", "anderson", "evenness → disorder"),
        edge("anderson", "cdiff", "ξ → resistance"),
        edge("cdiff", "fmt", "FMT intervention"),
    ];
    (scenario, edges)
}

// ---------------------------------------------------------------------------
// Track 3: Biosignal
// ---------------------------------------------------------------------------

/// Build a complete biosignal study scenario with real computed data.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "signal sample counts well within safe range"
)]
#[expect(
    clippy::too_many_lines,
    reason = "4 sub-studies including signal generation"
)]
pub fn biosignal_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold(
        "healthSpring Biosignal Study",
        "Pan-Tompkins QRS, HRV metrics, PPG SpO2, multi-channel fusion — 4 experiments",
    );

    let fs = 360.0;
    let (ecg, true_peaks) = biosignal::generate_synthetic_ecg(fs, 10.0, 72.0, 0.05, 42);
    let result = biosignal::pan_tompkins(&ecg, fs);
    let metrics = biosignal::evaluate_detection(&result.peaks, &true_peaks, (0.1 * fs) as usize);
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    let ecg_t: Vec<f64> = (0..ecg.len()).map(|i| i as f64 / fs).collect();
    let ecg_ds_step = ecg.len() / 500;
    let ecg_ds_step = ecg_ds_step.max(1);
    let ecg_t_ds: Vec<f64> = ecg_t.iter().step_by(ecg_ds_step).copied().collect();
    let ecg_ds: Vec<f64> = ecg.iter().step_by(ecg_ds_step).copied().collect();
    let bp_ds: Vec<f64> = result
        .bandpass
        .iter()
        .step_by(ecg_ds_step)
        .copied()
        .collect();

    s.ecosystem.primals.push(node(
        "qrs",
        "Pan-Tompkins QRS Detection",
        "compute",
        100.0,
        100.0,
        &["science.biosignal.pan_tompkins"],
        vec![
            timeseries(
                "ecg_raw",
                "ECG (Raw)",
                "Time (s)",
                "Amplitude",
                "mV",
                ecg_t_ds.clone(),
                ecg_ds,
            ),
            timeseries(
                "ecg_bandpass",
                "ECG (Bandpass)",
                "Time (s)",
                "Amplitude",
                "mV",
                ecg_t_ds,
                bp_ds,
            ),
            gauge(
                "hr",
                "Heart Rate",
                hr,
                40.0,
                140.0,
                "bpm",
                [60.0, 100.0],
                [40.0, 60.0],
            ),
            gauge(
                "sensitivity",
                "Detection Sensitivity",
                metrics.sensitivity * 100.0,
                0.0,
                100.0,
                "%",
                [90.0, 100.0],
                [80.0, 90.0],
            ),
            gauge(
                "ppv",
                "Detection PPV",
                metrics.ppv * 100.0,
                0.0,
                100.0,
                "%",
                [90.0, 100.0],
                [80.0, 90.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Normal HR".into(),
            min: 60.0,
            max: 100.0,
            status: "normal".into(),
        }],
    ));

    // HRV (exp021)
    let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
    let rmssd = biosignal::rmssd_ms(&result.peaks, fs);
    let pnn50 = biosignal::pnn50(&result.peaks, fs);
    let rr_intervals: Vec<f64> = result
        .peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs * 1000.0)
        .collect();
    let rr_x: Vec<f64> = (0..rr_intervals.len()).map(|i| i as f64).collect();
    s.ecosystem.primals.push(node(
        "hrv",
        "HRV Metrics",
        "compute",
        300.0,
        100.0,
        &["science.biosignal.hrv"],
        vec![
            timeseries(
                "rr_tachogram",
                "RR Tachogram",
                "Beat #",
                "RR Interval",
                "ms",
                rr_x,
                rr_intervals,
            ),
            gauge(
                "sdnn",
                "SDNN",
                sdnn,
                0.0,
                200.0,
                "ms",
                [50.0, 150.0],
                [20.0, 50.0],
            ),
            gauge(
                "rmssd",
                "RMSSD",
                rmssd,
                0.0,
                100.0,
                "ms",
                [20.0, 60.0],
                [10.0, 20.0],
            ),
            gauge(
                "pnn50",
                "pNN50",
                pnn50,
                0.0,
                100.0,
                "%",
                [10.0, 50.0],
                [3.0, 10.0],
            ),
        ],
        vec![],
    ));

    // PPG SpO2 (exp022)
    let ppg = biosignal::generate_synthetic_ppg(fs, 5.0, 72.0, 97.0, 42);
    let (ac_red, dc_red) = biosignal::ppg_extract_ac_dc(&ppg.red);
    let (ac_ir, dc_ir) = biosignal::ppg_extract_ac_dc(&ppg.ir);
    let r_val = biosignal::ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
    let spo2 = biosignal::spo2_from_r(r_val);
    let r_sweep: Vec<f64> = (0..20).map(|i| 0.3 + 0.05 * f64::from(i)).collect();
    let spo2_sweep: Vec<f64> = r_sweep.iter().map(|&r| biosignal::spo2_from_r(r)).collect();
    s.ecosystem.primals.push(node(
        "spo2",
        "PPG SpO2",
        "compute",
        500.0,
        100.0,
        &["science.biosignal.ppg_spo2"],
        vec![
            timeseries(
                "r_calibration",
                "R-value vs SpO2 Calibration",
                "R-value",
                "SpO2",
                "%",
                r_sweep,
                spo2_sweep,
            ),
            gauge(
                "spo2",
                "SpO2",
                spo2,
                70.0,
                100.0,
                "%",
                [95.0, 100.0],
                [90.0, 95.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Normal SpO2".into(),
                min: 95.0,
                max: 100.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Hypoxemia".into(),
                min: 70.0,
                max: 90.0,
                status: "critical".into(),
            },
        ],
    ));

    // Fusion (exp023)
    let eda = biosignal::generate_synthetic_eda(4.0, 30.0, 5.0, &[5.0, 12.0, 20.0, 25.0], 0.8, 42);
    let scl = biosignal::eda_scl(&eda, 16);
    let phasic = biosignal::eda_phasic(&eda, 16);
    let scr_peaks = biosignal::eda_detect_scr(&phasic, 0.1, 8);
    let fused = biosignal::fuse_channels(&result.peaks, fs, spo2, scr_peaks.len(), 30.0);
    let eda_t: Vec<f64> = (0..scl.len()).map(|i| i as f64 / 4.0).collect();
    s.ecosystem.primals.push(node(
        "fusion",
        "Multi-Channel Fusion",
        "compute",
        300.0,
        300.0,
        &["science.biosignal.fusion"],
        vec![
            timeseries(
                "eda_scl",
                "EDA Skin Conductance Level",
                "Time (s)",
                "SCL",
                "µS",
                eda_t.clone(),
                scl,
            ),
            timeseries(
                "eda_phasic",
                "EDA Phasic Component",
                "Time (s)",
                "Phasic",
                "µS",
                eda_t,
                phasic,
            ),
            gauge(
                "stress",
                "Stress Index",
                fused.stress_index,
                0.0,
                1.0,
                "index",
                [0.0, 0.4],
                [0.4, 0.7],
            ),
            gauge(
                "overall",
                "Overall Health Score",
                fused.overall_score,
                0.0,
                100.0,
                "score",
                [70.0, 100.0],
                [50.0, 70.0],
            ),
        ],
        vec![],
    ));

    let edges = vec![
        edge("qrs", "hrv", "R-peaks → HRV"),
        edge("spo2", "fusion", "SpO2 → fusion"),
        edge("hrv", "fusion", "HRV → fusion"),
    ];
    (s, edges)
}

// ---------------------------------------------------------------------------
// Track 4: Endocrinology
// ---------------------------------------------------------------------------

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
        "compute",
        100.0,
        100.0,
        &["science.endocrine.testosterone_im"],
        vec![
            timeseries(
                "im_single",
                "Single IM Dose",
                "Time (days)",
                "C (ng/mL)",
                "ng/mL",
                days.clone(),
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
            status: "normal".into(),
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
        "compute",
        300.0,
        100.0,
        &["science.endocrine.testosterone_pellet"],
        vec![timeseries(
            "pellet_pk",
            "Pellet Concentration",
            "Time (days)",
            "C (ng/mL)",
            "ng/mL",
            pellet_days,
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
        "compute",
        500.0,
        100.0,
        &["science.endocrine.testosterone_decline"],
        vec![
            timeseries(
                "t_low_rate",
                "T Decline (1%/yr)",
                "Age (yr)",
                "T (ng/dL)",
                "ng/dL",
                ages.clone(),
                t_low,
            ),
            timeseries(
                "t_mid_rate",
                "T Decline (1.6%/yr)",
                "Age (yr)",
                "T (ng/dL)",
                "ng/dL",
                ages.clone(),
                t_mid,
            ),
            timeseries(
                "t_high_rate",
                "T Decline (3%/yr)",
                "Age (yr)",
                "T (ng/dL)",
                "ng/dL",
                ages,
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
            status: "critical".into(),
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
        "compute",
        100.0,
        300.0,
        &["science.endocrine.trt_weight"],
        vec![
            timeseries(
                "weight_loss",
                "Weight Loss",
                "Month",
                "ΔWeight (kg)",
                "kg",
                months.clone(),
                weight_curve,
            ),
            timeseries(
                "waist_loss",
                "Waist Loss",
                "Month",
                "ΔWaist (cm)",
                "cm",
                months.clone(),
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
        "compute",
        300.0,
        300.0,
        &["science.endocrine.trt_cardiovascular"],
        vec![
            timeseries(
                "ldl",
                "LDL Cholesterol",
                "Month",
                "LDL (mg/dL)",
                "mg/dL",
                months.clone(),
                ldl_curve,
            ),
            timeseries(
                "sbp",
                "Systolic BP",
                "Month",
                "SBP (mmHg)",
                "mmHg",
                months.clone(),
                sbp_curve,
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
                label: "SBP normal".into(),
                min: 90.0,
                max: 130.0,
                status: "normal".into(),
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
        "compute",
        500.0,
        300.0,
        &["science.endocrine.trt_diabetes"],
        vec![timeseries(
            "hba1c",
            "HbA1c",
            "Month",
            "HbA1c (%)",
            "%",
            dm_months,
            hba1c_curve,
        )],
        vec![ClinicalRange {
            label: "HbA1c target".into(),
            min: 4.0,
            max: 7.0,
            status: "normal".into(),
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
        "compute",
        300.0,
        500.0,
        &["science.endocrine.gut_trt_axis"],
        vec![bar(
            "gut_response",
            "Metabolic Response by Gut Health",
            gut_cats,
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
        "compute",
        500.0,
        500.0,
        &["science.endocrine.hrv_trt"],
        vec![
            timeseries(
                "sdnn_trt",
                "SDNN on TRT",
                "Month",
                "SDNN (ms)",
                "ms",
                hrv_months,
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

// ---------------------------------------------------------------------------
// Full Study (all 4 tracks combined)
// ---------------------------------------------------------------------------

/// Build a combined all-tracks scenario for the complete healthSpring study.
#[must_use]
pub fn full_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let (pkpd, mut pkpd_edges) = pkpd_study();
    let (micro, mut micro_edges) = microbiome_study();
    let (bio, mut bio_edges) = biosignal_study();
    let (endo, mut endo_edges) = endocrine_study();

    let mut s = scaffold(
        "healthSpring Complete Study",
        "All 4 tracks: PK/PD + Microbiome + Biosignal + Endocrinology — 30 experiments",
    );

    // Offset positions so tracks don't overlap
    let offsets: [(f64, f64); 4] = [(0.0, 0.0), (0.0, 700.0), (800.0, 0.0), (800.0, 700.0)];
    for (track, offset) in [pkpd, micro, bio, endo].into_iter().zip(offsets) {
        for mut n in track.ecosystem.primals {
            n.position.x += offset.0;
            n.position.y += offset.1;
            s.ecosystem.primals.push(n);
        }
    }

    let mut all_edges = Vec::new();
    all_edges.append(&mut pkpd_edges);
    all_edges.append(&mut micro_edges);
    all_edges.append(&mut bio_edges);
    all_edges.append(&mut endo_edges);

    // Cross-track links
    all_edges.push(edge(
        "pop_pk",
        "diversity",
        "PK variability × gut diversity",
    ));
    all_edges.push(edge("diversity", "gut_axis", "microbiome → TRT metabolic"));
    all_edges.push(edge("hrv", "hrv_cardiac", "biosignal HRV → TRT cardiac"));
    all_edges.push(edge("one_comp", "t_im", "PK/PD → endocrine PK"));

    (s, all_edges)
}

/// Serialize a scenario + edges to pretty JSON.
///
/// # Panics
/// Cannot panic — all types are `Serialize`.
#[must_use]
pub fn scenario_with_edges_json(scenario: &HealthScenario, edges: &[ScenarioEdge]) -> String {
    #[derive(serde::Serialize)]
    struct WithEdges<'a> {
        #[serde(flatten)]
        scenario: &'a HealthScenario,
        edges: &'a [ScenarioEdge],
    }
    serde_json::to_string_pretty(&WithEdges { scenario, edges }).expect("serialization cannot fail")
}
