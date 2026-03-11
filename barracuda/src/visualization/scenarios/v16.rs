// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{ClinicalRange, HealthScenario, ScenarioEdge};
use super::{bar, edge, gauge, node, scaffold, timeseries};
use crate::biosignal::classification;
use crate::biosignal::stress;
use crate::{biosignal, microbiome, pkpd};

/// Build a V16 primitives study scenario with real computed data.
///
/// Six nodes: Michaelis-Menten nonlinear PK, antibiotic perturbation,
/// SCFA production, gut-brain serotonin, EDA stress detection,
/// arrhythmia beat classification.
#[must_use]
pub fn v16_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold(
        "healthSpring V16 Primitives",
        "Michaelis-Menten PK, antibiotic perturbation, SCFA, serotonin, EDA stress, arrhythmia — 6 experiments",
    );

    build_mm_nonlinear_pk(&mut s);
    build_antibiotic_perturbation(&mut s);
    build_scfa_production(&mut s);
    build_gut_brain_serotonin(&mut s);
    build_eda_stress(&mut s);
    build_arrhythmia_classify(&mut s);

    let edges = vec![
        edge("mm_nonlinear_pk", "scfa_prod", "PK saturation kinetics"),
        edge("abx_perturbation", "scfa_prod", "post-antibiotic recovery"),
        edge("scfa_prod", "gut_serotonin", "gut metabolites → neuro"),
        edge("eda_stress", "arrhythmia_classify", "stress → cardiac"),
        edge("abx_perturbation", "gut_serotonin", "diversity → serotonin"),
    ];
    (s, edges)
}

fn build_mm_nonlinear_pk(s: &mut HealthScenario) {
    let params = &pkpd::PHENYTOIN_PARAMS;

    let doses = [100.0, 300.0, 600.0];
    let mut channels = Vec::new();

    for &dose in &doses {
        let (raw_times, raw_concs) = pkpd::mm_pk_simulate(params, dose, 10.0, 0.01);
        let n = raw_times.len();
        let step = (n / 500).max(1);
        let times: Vec<f64> = raw_times.iter().step_by(step).copied().collect();
        let concs: Vec<f64> = raw_concs.iter().step_by(step).copied().collect();
        channels.push(timeseries(
            &format!("mm_pk_{dose}mg"),
            &format!("MM PK {dose:.0}mg"),
            "Time (days)",
            "C (mg/L)",
            "mg/L",
            times,
            concs,
        ));
    }

    let auc_cats: Vec<String> = doses.iter().map(|d| format!("{d:.0}mg")).collect();
    let auc_vals: Vec<f64> = doses
        .iter()
        .map(|&d| pkpd::mm_auc_analytical(params, d))
        .collect();
    channels.push(bar(
        "mm_auc_compare",
        "AUC vs Dose (nonlinear increase)",
        auc_cats,
        auc_vals,
        "mg·day/L",
    ));

    let t_half = pkpd::mm_apparent_half_life(params, 15.0);
    channels.push(gauge(
        "mm_t_half",
        "Apparent t½ at 15 mg/L",
        t_half,
        0.0,
        2.0,
        "days",
        [0.1, 0.8],
        [0.8, 1.5],
    ));

    s.ecosystem.primals.push(node(
        "mm_nonlinear_pk",
        "Michaelis-Menten Nonlinear PK",
        "compute",
        &["science.pkpd.michaelis_menten_nonlinear"],
        channels,
        vec![
            ClinicalRange {
                label: "Therapeutic (phenytoin)".into(),
                min: 10.0,
                max: 20.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Toxic".into(),
                min: 20.0,
                max: 40.0,
                status: "critical".into(),
            },
        ],
    ));
}

fn build_antibiotic_perturbation(s: &mut HealthScenario) {
    let h0 = 3.2;
    let depth = 0.7;
    let k_decline = 0.5;
    let k_recovery = 0.08;
    let treatment_days = 7.0;
    let total_days = 30.0;
    let dt = 0.1;

    let trajectory = microbiome::antibiotic_perturbation(
        h0,
        depth,
        k_decline,
        k_recovery,
        treatment_days,
        total_days,
        dt,
    );
    let times: Vec<f64> = trajectory.iter().map(|&(t, _)| t).collect();
    let diversity: Vec<f64> = trajectory.iter().map(|&(_, h)| h).collect();
    let nadir = diversity.iter().copied().fold(f64::INFINITY, f64::min);

    s.ecosystem.primals.push(node(
        "abx_perturbation",
        "Antibiotic Perturbation",
        "compute",
        &["science.microbiome.antibiotic_perturbation"],
        vec![
            timeseries(
                "abx_diversity",
                "Shannon Diversity (7d ciprofloxacin)",
                "Time (days)",
                "Shannon H′",
                "nats",
                times,
                diversity,
            ),
            gauge(
                "abx_nadir",
                "Diversity Nadir",
                nadir,
                0.0,
                4.0,
                "nats",
                [2.5, 4.0],
                [1.0, 2.5],
            ),
            gauge(
                "abx_baseline",
                "Pre-Antibiotic Diversity",
                h0,
                0.0,
                4.0,
                "nats",
                [2.5, 4.0],
                [1.5, 2.5],
            ),
        ],
        vec![ClinicalRange {
            label: "Healthy Shannon".into(),
            min: 2.5,
            max: 4.0,
            status: "normal".into(),
        }],
    ));
}

fn build_scfa_production(s: &mut HealthScenario) {
    let params = &microbiome::SCFA_HEALTHY_PARAMS;

    let fiber_levels = [5.0, 20.0, 50.0];
    let level_labels: Vec<String> = fiber_levels.iter().map(|f| format!("{f:.0}g")).collect();

    let mut acetate_vals = Vec::new();
    let mut propionate_vals = Vec::new();
    let mut butyrate_vals = Vec::new();
    for &fiber in &fiber_levels {
        let (a, p, b) = microbiome::scfa_production(fiber, params);
        acetate_vals.push(a);
        propionate_vals.push(p);
        butyrate_vals.push(b);
    }

    let sweep_fibers: Vec<f64> = (0..61).map(f64::from).collect();
    let sweep_total: Vec<f64> = sweep_fibers
        .iter()
        .map(|&f| {
            let (a, p, b) = microbiome::scfa_production(f, params);
            a + p + b
        })
        .collect();

    let (a_ref, p_ref, b_ref) = microbiome::scfa_production(20.0, params);
    let total_ref = a_ref + p_ref + b_ref;

    s.ecosystem.primals.push(node(
        "scfa_prod",
        "SCFA Production",
        "compute",
        &["science.microbiome.scfa_production"],
        vec![
            bar(
                "scfa_acetate",
                "Acetate by Fiber",
                level_labels.clone(),
                acetate_vals,
                "mmol/L",
            ),
            bar(
                "scfa_propionate",
                "Propionate by Fiber",
                level_labels.clone(),
                propionate_vals,
                "mmol/L",
            ),
            bar(
                "scfa_butyrate",
                "Butyrate by Fiber",
                level_labels,
                butyrate_vals,
                "mmol/L",
            ),
            timeseries(
                "scfa_saturation",
                "Total SCFA vs Fiber (Michaelis-Menten)",
                "Fiber (g/L)",
                "Total SCFA",
                "mmol/L",
                sweep_fibers,
                sweep_total,
            ),
            gauge(
                "scfa_ratio",
                "A:P:B Ratio at 20g",
                a_ref / total_ref * 100.0,
                0.0,
                100.0,
                "% acetate",
                [50.0, 70.0],
                [30.0, 50.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Healthy SCFA total".into(),
            min: 50.0,
            max: 150.0,
            status: "normal".into(),
        }],
    ));
}

fn build_gut_brain_serotonin(s: &mut HealthScenario) {
    let diversity_levels = [
        ("Healthy (H=3.2)", 3.2),
        ("Moderate (H=2.0)", 2.0),
        ("Dysbiotic (H=0.8)", 0.8),
    ];
    let dietary_trp = 60.0;
    let k_synth = 0.15;
    let scale = 1.0;

    let mut cats = Vec::new();
    let mut serotonin_vals = Vec::new();
    let mut trp_vals = Vec::new();
    for (label, h) in &diversity_levels {
        let trp = microbiome::tryptophan_availability(dietary_trp, *h);
        let serotonin = microbiome::gut_serotonin_production(trp, *h, k_synth, scale);
        cats.push((*label).into());
        serotonin_vals.push(serotonin);
        trp_vals.push(trp);
    }

    s.ecosystem.primals.push(node(
        "gut_serotonin",
        "Gut-Brain Serotonin Axis",
        "compute",
        &["science.microbiome.gut_brain_serotonin"],
        vec![
            bar(
                "serotonin_by_diversity",
                "Serotonin Production",
                cats.clone(),
                serotonin_vals,
                "µmol/L",
            ),
            bar(
                "trp_by_diversity",
                "Tryptophan Availability",
                cats,
                trp_vals,
                "µmol/L",
            ),
            gauge(
                "trp_healthy",
                "Tryptophan (Healthy Gut)",
                microbiome::tryptophan_availability(dietary_trp, 3.2),
                0.0,
                80.0,
                "µmol/L",
                [30.0, 60.0],
                [15.0, 30.0],
            ),
        ],
        vec![],
    ));
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "signal sample counts bounded by generation parameters"
)]
fn build_eda_stress(s: &mut HealthScenario) {
    let fs = 4.0;
    let duration = 60.0;
    let eda =
        biosignal::generate_synthetic_eda(fs, duration, 5.0, &[10.0, 25.0, 40.0, 52.0], 0.8, 7);
    let scl = biosignal::eda_scl(&eda, 16);
    let phasic = biosignal::eda_phasic(&eda, 16);
    let scr_peaks = biosignal::eda_detect_scr(&phasic, 0.1, 8);

    let eda_t: Vec<f64> = (0..eda.len()).map(|i| i as f64 / fs).collect();
    let scl_t: Vec<f64> = (0..scl.len()).map(|i| i as f64 / fs).collect();
    let phasic_t: Vec<f64> = (0..phasic.len()).map(|i| i as f64 / fs).collect();

    let mean_scl = if scl.is_empty() {
        0.0
    } else {
        scl.iter().sum::<f64>() / scl.len() as f64
    };
    let scr_rate = scr_peaks.len() as f64 / (duration / 60.0);
    let stress_idx = stress::compute_stress_index(scr_rate, mean_scl, 3.0);

    let epoch_dur = 15.0;
    let n_epochs = (duration / epoch_dur) as usize;
    let mut epoch_labels = Vec::with_capacity(n_epochs);
    let mut epoch_scr = Vec::with_capacity(n_epochs);
    for e in 0..n_epochs {
        let t_start = e as f64 * epoch_dur;
        let t_end = t_start + epoch_dur;
        epoch_labels.push(format!("{t_start:.0}-{t_end:.0}s"));
        let count = scr_peaks
            .iter()
            .filter(|&&pk| {
                let t = pk as f64 / fs;
                t >= t_start && t < t_end
            })
            .count();
        epoch_scr.push(count as f64);
    }

    s.ecosystem.primals.push(node(
        "eda_stress",
        "EDA Stress Detection",
        "compute",
        &[
            "science.biosignal.eda_stress_detection",
            "science.biosignal.eda_analysis",
        ],
        vec![
            timeseries(
                "eda_raw",
                "Raw EDA Signal",
                "Time (s)",
                "Conductance",
                "µS",
                eda_t,
                eda,
            ),
            timeseries(
                "eda_tonic",
                "Tonic SCL",
                "Time (s)",
                "SCL",
                "µS",
                scl_t,
                scl,
            ),
            timeseries(
                "eda_phasic_v16",
                "Phasic Component",
                "Time (s)",
                "Phasic",
                "µS",
                phasic_t,
                phasic,
            ),
            gauge(
                "stress_index",
                "Composite Stress Index",
                stress_idx,
                0.0,
                100.0,
                "score",
                [0.0, 40.0],
                [40.0, 70.0],
            ),
            bar(
                "scr_per_epoch",
                "SCR Count per 15s Epoch",
                epoch_labels,
                epoch_scr,
                "count",
            ),
        ],
        vec![ClinicalRange {
            label: "Low Stress".into(),
            min: 0.0,
            max: 40.0,
            status: "normal".into(),
        }],
    ));
}

#[expect(clippy::cast_precision_loss, reason = "sample index fits f64")]
#[expect(
    clippy::similar_names,
    reason = "pvc/pac template names are domain terms"
)]
fn build_arrhythmia_classify(s: &mut HealthScenario) {
    let n_samples = 41;
    let normal_tmpl = classification::generate_normal_template(n_samples);
    let pvc_template = classification::generate_pvc_template(n_samples);
    let pac_template = classification::generate_pac_template(n_samples);

    let sample_x: Vec<f64> = (0..n_samples).map(|i| i as f64).collect();

    let templates = vec![
        classification::BeatTemplate {
            class: classification::BeatClass::Normal,
            waveform: normal_tmpl.clone(),
        },
        classification::BeatTemplate {
            class: classification::BeatClass::Pvc,
            waveform: pvc_template.clone(),
        },
        classification::BeatTemplate {
            class: classification::BeatClass::Pac,
            waveform: pac_template.clone(),
        },
    ];

    let n_beats = 100u32;
    let mut counts = [0u32; 3]; // [normal, pvc, pac]
    let mut total_corr = 0.0;
    for i in 0..n_beats {
        let beat = match i % 10 {
            7 => classification::generate_pvc_template(n_samples),
            9 => classification::generate_pac_template(n_samples),
            _ => classification::generate_normal_template(n_samples),
        };
        let (cls, corr) = classification::classify_beat(&beat, &templates, 0.5);
        total_corr += corr;
        match cls {
            classification::BeatClass::Normal => counts[0] += 1,
            classification::BeatClass::Pvc => counts[1] += 1,
            classification::BeatClass::Pac => counts[2] += 1,
            classification::BeatClass::Unknown => {}
        }
    }

    s.ecosystem.primals.push(node(
        "arrhythmia_classify",
        "Arrhythmia Beat Classification",
        "compute",
        &["science.biosignal.arrhythmia_classification"],
        vec![
            timeseries(
                "tmpl_normal",
                "Normal Template",
                "Sample",
                "Amplitude",
                "mV",
                sample_x.clone(),
                normal_tmpl,
            ),
            timeseries(
                "tmpl_pvc",
                "PVC Template",
                "Sample",
                "Amplitude",
                "mV",
                sample_x.clone(),
                pvc_template,
            ),
            timeseries(
                "tmpl_pac",
                "PAC Template",
                "Sample",
                "Amplitude",
                "mV",
                sample_x,
                pac_template,
            ),
            bar(
                "beat_distribution",
                "Beat Classification (100 beats)",
                vec!["Normal".into(), "PVC".into(), "PAC".into()],
                vec![
                    f64::from(counts[0]),
                    f64::from(counts[1]),
                    f64::from(counts[2]),
                ],
                "count",
            ),
            gauge(
                "avg_correlation",
                "Average Correlation",
                total_corr / f64::from(n_beats),
                0.0,
                1.0,
                "r",
                [0.9, 1.0],
                [0.7, 0.9],
            ),
        ],
        vec![],
    ));
}
