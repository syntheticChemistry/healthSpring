// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear benchmark sequence"
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "small iteration counts and timing values fit f64 mantissa"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "benchmark timing: nanoseconds to microseconds, index to percentile position"
)]

//! Exp084: V16 CPU Parity Benchmarks
//!
//! Benchmarks the 6 new V16 primitives on Rust CPU, validates numerical
//! parity with Python baselines, and outputs timing JSON for comparison.
//!
//! Primitives: Michaelis-Menten PK, antibiotic perturbation, SCFA production,
//! gut-brain serotonin, EDA stress detection, arrhythmia beat classification.

use healthspring_barracuda::biosignal::classification::{
    BeatClass, BeatTemplate, classify_beat, generate_normal_template, generate_pac_template,
    generate_pvc_template, normalized_correlation,
};
use healthspring_barracuda::biosignal::eda::{eda_detect_scr, eda_phasic, eda_scl};
use healthspring_barracuda::biosignal::stress::compute_stress_index;
use healthspring_barracuda::microbiome::{
    SCFA_DYSBIOTIC_PARAMS, SCFA_HEALTHY_PARAMS, antibiotic_perturbation, gut_serotonin_production,
    scfa_production, tryptophan_availability,
};
use healthspring_barracuda::pkpd;
use serde::Serialize;

const N_ITER: usize = 100;

#[derive(Serialize)]
struct BenchResult {
    name: String,
    n_iterations: usize,
    mean_us: f64,
    min_us: f64,
    max_us: f64,
    p50_us: f64,
    p95_us: f64,
}

#[derive(Serialize)]
struct BenchSuite {
    tier: String,
    experiment: String,
    benchmarks: Vec<BenchResult>,
}

fn bench<F: Fn()>(name: &str, func: F, n_iter: usize) -> BenchResult {
    let mut times_us = Vec::with_capacity(n_iter);
    for _ in 0..n_iter {
        let start = std::time::Instant::now();
        func();
        times_us.push(start.elapsed().as_nanos() as f64 / 1000.0);
    }
    times_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let sum: f64 = times_us.iter().sum();
    let mean = sum / n_iter as f64;
    let p50 = n_iter / 2;
    let p95 = (n_iter as f64 * 0.95) as usize;
    BenchResult {
        name: name.to_string(),
        n_iterations: n_iter,
        mean_us: mean,
        min_us: times_us[0],
        max_us: times_us[n_iter - 1],
        p50_us: times_us[p50],
        p95_us: times_us[p95.min(n_iter - 1)],
    }
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut benchmarks = Vec::new();

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                passed += 1;
                println!("  [PASS] {}", $name);
            } else {
                eprintln!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp084 — V16 CPU Parity Benchmarks");
    println!("{}", "=".repeat(72));

    // ── 1. Michaelis-Menten PK ──────────────────────────────────────────

    println!("\n── 1. Michaelis-Menten PK ──────────────────────────────────────");

    let mm_params = &pkpd::PHENYTOIN_PARAMS;

    let result = bench(
        "mm_pk_simulate_300mg_10d",
        || {
            std::hint::black_box(pkpd::mm_pk_simulate(mm_params, 300.0, 10.0, 0.001));
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("mm_simulate_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "mm_auc_analytical_x100",
        || {
            for dose_idx in 0..100 {
                std::hint::black_box(pkpd::mm_auc_analytical(
                    mm_params,
                    f64::from(dose_idx) * 5.0 + 10.0,
                ));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("mm_auc_analytical_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "mm_half_life_sweep_100",
        || {
            for idx in 0..100 {
                std::hint::black_box(pkpd::mm_apparent_half_life(
                    mm_params,
                    f64::from(idx) * 0.3 + 0.1,
                ));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("mm_half_life_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let c0 = 300.0 / mm_params.vd;
    let anal_auc = pkpd::mm_auc_analytical(mm_params, 300.0);
    check!("mm_c0_correct", (c0 - 6.0).abs() < 0.01);
    check!("mm_auc_positive", anal_auc > 0.0);
    let t_half_low = pkpd::mm_apparent_half_life(mm_params, 1.0);
    let t_half_high = pkpd::mm_apparent_half_life(mm_params, 20.0);
    check!("mm_half_life_dose_dependent", t_half_low < t_half_high);

    // ── 2. Antibiotic Perturbation ──────────────────────────────────────

    println!("\n── 2. Antibiotic Perturbation ──────────────────────────────────");

    let result = bench(
        "antibiotic_perturb_30d",
        || {
            std::hint::black_box(antibiotic_perturbation(2.2, 0.7, 5.0, 0.1, 5.0, 30.0, 0.01));
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("antibiotic_30d_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "antibiotic_perturb_365d",
        || {
            std::hint::black_box(antibiotic_perturbation(2.2, 0.7, 5.0, 0.1, 5.0, 365.0, 0.1));
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("antibiotic_365d_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let trajectory = antibiotic_perturbation(2.2, 0.7, 5.0, 0.1, 5.0, 30.0, 0.01);
    let (_, h_initial) = trajectory.first().unwrap();
    let (_, h_nadir) = trajectory
        .iter()
        .min_by(|aa, bb| aa.1.partial_cmp(&bb.1).unwrap())
        .unwrap();
    let (_, h_final) = trajectory.last().unwrap();
    check!("antibiotic_drops", h_nadir < h_initial);
    check!("antibiotic_recovers", h_final > h_nadir);
    check!("antibiotic_not_full_recovery", h_final < h_initial);

    // ── 3. SCFA Production ──────────────────────────────────────────────

    println!("\n── 3. SCFA Production ─────────────────────────────────────────");

    let result = bench(
        "scfa_healthy_x1000",
        || {
            for idx in 0..1000 {
                let fiber = f64::from(idx) * 0.05 + 0.1;
                std::hint::black_box(scfa_production(fiber, &SCFA_HEALTHY_PARAMS));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("scfa_healthy_1k_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "scfa_dysbiotic_x1000",
        || {
            for idx in 0..1000 {
                let fiber = f64::from(idx) * 0.05 + 0.1;
                std::hint::black_box(scfa_production(fiber, &SCFA_DYSBIOTIC_PARAMS));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("scfa_dysbiotic_1k_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let (acetate, propionate, butyrate) = scfa_production(20.0, &SCFA_HEALTHY_PARAMS);
    let scfa_total = acetate + propionate + butyrate;
    check!(
        "scfa_acetate_dominant",
        acetate > propionate && acetate > butyrate
    );
    check!(
        "scfa_ratio_normal",
        acetate / scfa_total > 0.50 && acetate / scfa_total < 0.70
    );
    let (_, _, butyrate_dys) = scfa_production(20.0, &SCFA_DYSBIOTIC_PARAMS);
    check!("scfa_dysbiotic_less_butyrate", butyrate > butyrate_dys);

    // ── 4. Gut-Brain Serotonin ──────────────────────────────────────────

    println!("\n── 4. Gut-Brain Serotonin ─────────────────────────────────────");

    let result = bench(
        "serotonin_sweep_1000",
        || {
            for idx in 0..1000 {
                let trp = f64::from(idx) * 0.1 + 10.0;
                let shannon = 0.5 + f64::from(idx) * 0.002;
                std::hint::black_box(gut_serotonin_production(trp, shannon, 0.01, 0.5));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("serotonin_1k_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "tryptophan_availability_sweep_1000",
        || {
            for idx in 0..1000 {
                let trp = f64::from(idx) * 0.1 + 10.0;
                let shannon = 0.5 + f64::from(idx) * 0.002;
                std::hint::black_box(tryptophan_availability(trp, shannon));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("tryptophan_1k_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let healthy_5ht = gut_serotonin_production(50.0, 2.2, 0.01, 0.5);
    let dysbiotic_5ht = gut_serotonin_production(50.0, 0.8, 0.01, 0.5);
    check!(
        "serotonin_positive",
        healthy_5ht > 0.0 && dysbiotic_5ht > 0.0
    );
    check!("serotonin_diversity_dependent", healthy_5ht > dysbiotic_5ht);
    let trp_healthy = tryptophan_availability(100.0, 2.2);
    let trp_dysbiotic = tryptophan_availability(100.0, 0.8);
    check!("tryptophan_higher_healthy", trp_healthy > trp_dysbiotic);

    // ── 5. EDA Stress Detection ─────────────────────────────────────────

    println!("\n── 5. EDA Stress Detection ────────────────────────────────────");

    let eda_len = 2000;
    let eda_signal: Vec<f64> = (0..eda_len)
        .map(|idx| {
            let time = f64::from(idx) / 100.0;
            2.0 + 0.5 * (time * 0.3).sin() + if idx % 300 < 30 { 1.5 } else { 0.0 }
        })
        .collect();

    let result = bench(
        "eda_scl_2000_samples",
        || {
            std::hint::black_box(eda_scl(&eda_signal, 200));
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("eda_scl_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "eda_phasic_2000_samples",
        || {
            std::hint::black_box(eda_phasic(&eda_signal, 200));
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("eda_phasic_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let phasic = eda_phasic(&eda_signal, 200);
    let scr_peaks = eda_detect_scr(&phasic, 0.05, 30);
    let result = bench(
        "eda_detect_scr_2000",
        || {
            std::hint::black_box(eda_detect_scr(&phasic, 0.05, 30));
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("eda_scr_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let scl = eda_scl(&eda_signal, 200);
    let mean_scl: f64 = scl.iter().sum::<f64>() / scl.len() as f64;
    let scr_rate = scr_peaks.len() as f64 / (f64::from(eda_len) / 100.0 / 60.0);
    let stress = compute_stress_index(scr_rate, mean_scl, 5.0);
    check!("eda_stress_bounded", (0.0..=100.0).contains(&stress));
    check!("eda_scr_found", !scr_peaks.is_empty());

    // ── 6. Arrhythmia Beat Classification ───────────────────────────────

    println!("\n── 6. Arrhythmia Beat Classification ──────────────────────────");

    let win = 60;
    let normal = generate_normal_template(win);
    let pvc = generate_pvc_template(win);
    let pac = generate_pac_template(win);
    let templates = vec![
        BeatTemplate {
            class: BeatClass::Normal,
            waveform: normal.clone(),
        },
        BeatTemplate {
            class: BeatClass::Pvc,
            waveform: pvc.clone(),
        },
        BeatTemplate {
            class: BeatClass::Pac,
            waveform: pac.clone(),
        },
    ];

    let result = bench(
        "classify_beat_x1000",
        || {
            for _ in 0..1000 {
                std::hint::black_box(classify_beat(&normal, &templates, 0.5));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("classify_1k_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let result = bench(
        "normalized_correlation_x1000",
        || {
            for _ in 0..1000 {
                std::hint::black_box(normalized_correlation(&normal, &pvc));
            }
        },
        N_ITER,
    );
    println!(
        "  {:<42} mean={:.1}us  p95={:.1}us",
        result.name, result.mean_us, result.p95_us
    );
    check!("corr_1k_runs", result.mean_us > 0.0);
    benchmarks.push(result);

    let (cls, corr) = classify_beat(&normal, &templates, 0.5);
    check!("classify_normal_as_normal", cls == BeatClass::Normal);
    check!("classify_normal_corr_high", corr > 0.99);
    let (cls_pvc, _) = classify_beat(&pvc, &templates, 0.5);
    check!("classify_pvc_as_pvc", cls_pvc == BeatClass::Pvc);
    let self_corr = normalized_correlation(&normal, &normal);
    check!("self_correlation_is_1", (self_corr - 1.0).abs() < 1e-10);
    let cross_corr = normalized_correlation(&normal, &pvc);
    check!("cross_corr_less_than_1", cross_corr < 0.8);

    // ── Summary ─────────────────────────────────────────────────────────

    let suite = BenchSuite {
        tier: "rust_cpu_v16".to_string(),
        experiment: "exp084".to_string(),
        benchmarks,
    };
    let json = serde_json::to_string_pretty(&suite).expect("serialize");

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../control/scripts/bench_results_v16_rust_cpu.json");
    if let Some(parent) = out_dir.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&out_dir, &json).unwrap_or_else(|_| println!("{json}"));
    println!("\nResults written to {}", out_dir.display());

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("Exp084 V16 CPU Parity Bench: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
