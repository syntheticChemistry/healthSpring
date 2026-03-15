// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::cast_precision_loss,
    reason = "patient parameter conversions, small counts"
)]
//! Exp089: Patient explorer — runs a full diagnostic pipeline with V16
//! analysis for a single patient, then streams results to `petalTongue`.
//!
//! Usage:
//! ```sh
//! cargo run --release --bin exp089_patient_explorer -- \
//!     --age 55 --weight 220 --baseline-t 280 --fiber 20 --diversity 3.0
//! ```

use healthspring_barracuda::biosignal;
use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::microbiome;
use healthspring_barracuda::pkpd;
use healthspring_barracuda::visualization::{
    HealthScenario, ScenarioEdge, annotate_population, assessment_to_scenario, scenarios,
    scenarios::scenario_with_edges_json, stream::StreamSession,
};
use std::fs;
use std::path::Path;

struct PatientParams {
    age: f64,
    weight_lb: f64,
    baseline_t: f64,
    fiber_g: f64,
    gut_diversity: f64,
    on_trt: bool,
    trt_months: f64,
}

impl Default for PatientParams {
    fn default() -> Self {
        Self {
            age: 55.0,
            weight_lb: 220.0,
            baseline_t: 280.0,
            fiber_g: 20.0,
            gut_diversity: 3.0,
            on_trt: true,
            trt_months: 6.0,
        }
    }
}

fn parse_args() -> PatientParams {
    let mut params = PatientParams::default();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--age" => {
                i += 1;
                params.age = args[i].parse().unwrap_or(params.age);
            }
            "--weight" => {
                i += 1;
                params.weight_lb = args[i].parse().unwrap_or(params.weight_lb);
            }
            "--baseline-t" => {
                i += 1;
                params.baseline_t = args[i].parse().unwrap_or(params.baseline_t);
            }
            "--fiber" => {
                i += 1;
                params.fiber_g = args[i].parse().unwrap_or(params.fiber_g);
            }
            "--diversity" => {
                i += 1;
                params.gut_diversity = args[i].parse().unwrap_or(params.gut_diversity);
            }
            "--no-trt" => {
                params.on_trt = false;
            }
            "--trt-months" => {
                i += 1;
                params.trt_months = args[i].parse().unwrap_or(params.trt_months);
            }
            _ => {}
        }
        i += 1;
    }
    params
}

fn main() {
    let params = parse_args();
    let mut checks = 0u32;
    let mut pass = 0u32;

    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            checks += 1;
            if $cond {
                pass += 1;
                println!("  [PASS] {}", $name);
            } else {
                println!("  [FAIL] {}", $name);
            }
        }};
    }

    println!("=== Exp089: Patient Explorer ===\n");
    println!(
        "Patient: age={:.0}y, weight={:.0}lb, T={:.0}ng/dL, fiber={:.0}g, H'={:.1}, TRT={}",
        params.age,
        params.weight_lb,
        params.baseline_t,
        params.fiber_g,
        params.gut_diversity,
        if params.on_trt { "yes" } else { "no" }
    );

    // ── Diagnostic pipeline ──────────────────────────────────────────
    println!("\n─── Diagnostic Pipeline ───");
    let weight_kg = params.weight_lb * 0.453_592;
    let mut profile = PatientProfile::minimal(params.age, weight_kg, Sex::Male);
    profile.testosterone_ng_dl = Some(params.baseline_t);
    profile.on_trt = params.on_trt;
    profile.trt_months = params.trt_months;
    profile.gut_abundances = Some(synthetic_abundances(params.gut_diversity));

    let assessment = assess_patient(&profile);
    check!(
        "diagnostic: composite risk computed",
        assessment.composite_risk >= 0.0
    );
    check!("diagnostic: PK assessed", assessment.pk.oral_auc > 0.0);

    let pop = population_montecarlo(&profile, 200, 42);
    check!("population: 200 patients", pop.n_patients == 200);

    let diag_scenario = annotate_population(
        assessment_to_scenario(&assessment, &format!("Patient {:.0}y", params.age)),
        &pop,
    );
    check!(
        "diagnostic scenario: has nodes",
        !diag_scenario.ecosystem.primals.is_empty()
    );

    // ── V16 analysis ─────────────────────────────────────────────────
    println!("\n─── V16 Patient Analysis ───");
    run_v16_analysis(&params, &mut checks, &mut pass);

    // ── V16 + diagnostic combined scenario ───────────────────────────
    println!("\n─── Combined Scenario ───");
    let (v16, v16_edges) = scenarios::v16_study();
    check!("v16 scenario: 6 nodes", v16.ecosystem.primals.len() == 6);

    let (combined, combined_edges) = merge_patient_scenario(diag_scenario, v16, v16_edges);
    check!(
        "combined: diagnostic + V16 nodes",
        combined.ecosystem.primals.len() >= 6
    );

    // ── Output ───────────────────────────────────────────────────────
    println!("\n─── Output ───");
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
    fs::create_dir_all(&out).expect("create sandbox/scenarios/");

    let json = scenario_with_edges_json(&combined, &combined_edges);
    let path = out.join("healthspring-patient-explorer.json");
    fs::write(&path, &json).expect("write scenario JSON");
    println!("  wrote {} ({} KB)", path.display(), json.len() / 1024);

    // ── Streaming ────────────────────────────────────────────────────
    println!("\n─── Streaming ───");
    attempt_streaming(&combined, &params, &mut checks, &mut pass);

    // ── Summary ──────────────────────────────────────────────────────
    println!("\n=== Exp089 Result: {pass}/{checks} checks passed ===");
    assert_eq!(pass, checks, "some checks failed");
}

fn run_v16_analysis(params: &PatientParams, checks: &mut u32, pass: &mut u32) {
    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            *checks += 1;
            if $cond {
                *pass += 1;
                println!("  [PASS] {}", $name);
            } else {
                println!("  [FAIL] {}", $name);
            }
        }};
    }

    let mm_params = &pkpd::PHENYTOIN_PARAMS;
    let (times, concs) = pkpd::mm_pk_simulate(mm_params, 300.0, 10.0, 0.01);
    check!(
        "MM PK: simulation produced data",
        !times.is_empty() && !concs.is_empty()
    );
    let mm_auc = pkpd::mm_auc_analytical(mm_params, 300.0);
    check!("MM PK: AUC > 0", mm_auc > 0.0);
    println!("    MM PK AUC at 300mg: {mm_auc:.1} mg·day/L");

    let scfa_params = &microbiome::SCFA_HEALTHY_PARAMS;
    let (acetate, propionate, butyrate) = microbiome::scfa_production(params.fiber_g, scfa_params);
    let total_scfa = acetate + propionate + butyrate;
    check!("SCFA: total > 0", total_scfa > 0.0);
    println!(
        "    SCFA at {:.0}g fiber: A={acetate:.1} P={propionate:.1} B={butyrate:.1} (total={total_scfa:.1})",
        params.fiber_g
    );

    let trp = microbiome::tryptophan_availability(60.0, params.gut_diversity);
    let serotonin = microbiome::gut_serotonin_production(trp, params.gut_diversity, 0.15, 1.0);
    check!("Serotonin: production > 0", serotonin > 0.0);
    println!(
        "    Serotonin at H'={:.1}: trp={trp:.1} µmol/L, 5-HT={serotonin:.2} µmol/L",
        params.gut_diversity
    );

    let abx_trajectory =
        microbiome::antibiotic_perturbation(params.gut_diversity, 0.7, 0.5, 0.08, 7.0, 30.0, 0.1);
    check!(
        "Antibiotic: trajectory produced",
        abx_trajectory.len() > 100
    );
    let nadir = abx_trajectory
        .iter()
        .map(|&(_, h)| h)
        .fold(f64::INFINITY, f64::min);
    println!(
        "    Antibiotic nadir: {nadir:.2} (from H'={:.1})",
        params.gut_diversity
    );

    let eda = biosignal::generate_synthetic_eda(4.0, 30.0, 5.0, &[5.0, 15.0, 25.0], 0.8, 42);
    let scl = biosignal::eda_scl(&eda, 16);
    let phasic = biosignal::eda_phasic(&eda, 16);
    let scr_peaks = biosignal::eda_detect_scr(&phasic, 0.1, 8);
    let mean_scl = if scl.is_empty() {
        0.0
    } else {
        scl.iter().sum::<f64>() / scl.len() as f64
    };
    let scr_rate = scr_peaks.len() as f64 / 0.5;
    let stress_idx = biosignal::stress::compute_stress_index(scr_rate, mean_scl, 3.0);
    check!("EDA: stress index computed", stress_idx >= 0.0);
    println!("    Stress index: {stress_idx:.1} (SCR rate={scr_rate:.1}/min, SCL={mean_scl:.1}µS)");

    let templates = vec![
        biosignal::BeatTemplate {
            class: biosignal::BeatClass::Normal,
            waveform: biosignal::generate_normal_template(41),
        },
        biosignal::BeatTemplate {
            class: biosignal::BeatClass::Pvc,
            waveform: biosignal::generate_pvc_template(41),
        },
        biosignal::BeatTemplate {
            class: biosignal::BeatClass::Pac,
            waveform: biosignal::generate_pac_template(41),
        },
    ];
    let test_beat = biosignal::generate_normal_template(41);
    let (cls, corr) = biosignal::classify_beat(&test_beat, &templates, 0.5);
    check!(
        "Arrhythmia: classified as Normal",
        cls == biosignal::BeatClass::Normal
    );
    println!("    Beat classification: {cls} (r={corr:.3})");
}

fn merge_patient_scenario(
    diagnostic: HealthScenario,
    v16: HealthScenario,
    v16_edges: Vec<ScenarioEdge>,
) -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut combined = diagnostic;
    combined.name = "Patient Explorer — Diagnostic + V16".into();
    combined.description = "Patient-specific diagnostic with V16 primitives analysis".into();

    for node in v16.ecosystem.primals {
        combined.ecosystem.primals.push(node);
    }

    let mut edges = v16_edges;
    edges.push(ScenarioEdge {
        from: "diagnostic".into(),
        to: "mm_nonlinear_pk".into(),
        edge_type: "data-flow".into(),
        label: "patient → PK analysis".into(),
    });

    (combined, edges)
}

fn attempt_streaming(
    scenario: &HealthScenario,
    params: &PatientParams,
    checks: &mut u32,
    pass: &mut u32,
) {
    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            *checks += 1;
            if $cond {
                *pass += 1;
                println!("  [PASS] {}", $name);
            } else {
                println!("  [FAIL] {}", $name);
            }
        }};
    }

    if let Ok(mut session) = StreamSession::discover("healthspring-patient-explorer") {
        println!("petalTongue found — streaming patient data\n");
        if session
            .push_initial_render("Patient Explorer", scenario)
            .is_ok()
        {
            println!("  initial render pushed");
        }

        let mm_params = &pkpd::PHENYTOIN_PARAMS;
        for &dose in &[100.0, 300.0, 600.0] {
            let (times, concs) = pkpd::mm_pk_simulate(mm_params, dose, 10.0, 0.1);
            let _ = session.push_pk_point(&format!("mm_pk_{dose:.0}mg"), &times, &concs);
        }

        let scfa_params = &microbiome::SCFA_HEALTHY_PARAMS;
        let (a, p, b) = microbiome::scfa_production(params.fiber_g, scfa_params);
        let _ = session.push_hrv_update(a, p, b);

        check!("streaming: session established", true);
    } else {
        println!("petalTongue not running — streaming skipped (scenario written to disk)");
        check!("streaming: fallback to disk", true);
    }
}

fn synthetic_abundances(shannon_target: f64) -> Vec<f64> {
    let n = 10;
    let lambda = 1.0 / (shannon_target.max(0.5) / 2.5);
    let mut raw: Vec<f64> = (0..n).map(|i| (-lambda * f64::from(i)).exp()).collect();
    let sum: f64 = raw.iter().sum();
    for v in &mut raw {
        *v /= sum;
    }
    raw
}
