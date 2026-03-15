// SPDX-License-Identifier: AGPL-3.0-only
//! Criterion benchmarks for healthSpring Rust CPU operations.
//!
//! Mirrors the Python benchmark suite in `control/scripts/bench_barracuda_cpu_vs_python.py`
//! to enable Tier 0 (Python) vs Tier 1 (Rust CPU) timing comparison.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use healthspring_barracuda::microbiome::{pielou_evenness, shannon_index, simpson_index};
use healthspring_barracuda::pkpd::{
    auc_trapezoidal, hill_dose_response, pk_oral_one_compartment, population_pk_cpu,
};

fn hill_sweep(concs: &[f64], ic50: f64, hill_n: f64, e_max: f64) -> Vec<f64> {
    concs
        .iter()
        .map(|&c| hill_dose_response(c, ic50, hill_n, e_max))
        .collect()
}

#[expect(
    clippy::cast_precision_loss,
    reason = "benchmark indices fit f64 mantissa"
)]
fn pk_curve(dose: f64, f_bio: f64, vd: f64, ka: f64, ke: f64, n_points: usize) -> Vec<f64> {
    let dt = 24.0 / n_points as f64;
    (0..=n_points)
        .map(|i| pk_oral_one_compartment(dose, f_bio, vd, ka, ke, i as f64 * dt))
        .collect()
}

fn bench_hill_sweep_50(c: &mut Criterion) {
    let concs: Vec<f64> = (0..50)
        .map(|i| 0.1 * 1000.0_f64.powf(f64::from(i) / 49.0))
        .collect();
    c.bench_function("hill_sweep_50", |b| {
        b.iter(|| hill_sweep(black_box(&concs), 10.0, 1.5, 100.0));
    });
}

fn bench_hill_sweep_10k(c: &mut Criterion) {
    let concs: Vec<f64> = (0..10_000)
        .map(|i| 0.1 * 1000.0_f64.powf(f64::from(i) / 9_999.0))
        .collect();
    c.bench_function("hill_sweep_10K", |b| {
        b.iter(|| hill_sweep(black_box(&concs), 10.0, 1.5, 100.0));
    });
}

fn bench_pk_curve_101(c: &mut Criterion) {
    let cl = 0.15 * (85.0_f64 / 70.0).powf(0.75);
    let vd = 15.0 * (85.0 / 70.0);
    let ke = cl / vd;
    c.bench_function("pk_curve_101_points", |b| {
        b.iter(|| pk_curve(black_box(4.0), 0.79, vd, 1.5, ke, 100));
    });
}

fn bench_diversity_indices_7(c: &mut Criterion) {
    let abundances = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02];
    c.bench_function("diversity_indices_7_genera", |b| {
        b.iter(|| {
            let s = shannon_index(black_box(&abundances));
            let d = simpson_index(black_box(&abundances));
            let p = pielou_evenness(black_box(&abundances));
            (s, d, p)
        });
    });
}

#[expect(
    clippy::cast_precision_loss,
    reason = "benchmark indices fit f64 mantissa"
)]
fn bench_diversity_batch_1k(c: &mut Criterion) {
    let communities: Vec<Vec<f64>> = (0..1_000)
        .map(|i| {
            let mut a = vec![0.0; 7];
            let mut total = 0.0;
            for (j, val) in a.iter_mut().enumerate() {
                *val = ((i * 7 + j + 1) as f64).sqrt();
                total += *val;
            }
            for val in &mut a {
                *val /= total;
            }
            a
        })
        .collect();
    c.bench_function("diversity_batch_1K", |b| {
        b.iter(|| {
            for community in &communities {
                let _ = black_box(shannon_index(community));
            }
        });
    });
}

fn bench_auc_trapezoidal_101(c: &mut Criterion) {
    let cl = 0.15 * (85.0_f64 / 70.0).powf(0.75);
    let vd = 15.0 * (85.0 / 70.0);
    let ke = cl / vd;
    let times: Vec<f64> = (0..=100).map(|i| f64::from(i) * 24.0 / 100.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(4.0, 0.79, vd, 1.5, ke, t))
        .collect();
    c.bench_function("auc_trapezoidal_101_points", |b| {
        b.iter(|| auc_trapezoidal(black_box(&times), black_box(&concs)));
    });
}

#[expect(
    clippy::cast_precision_loss,
    reason = "benchmark indices fit f64 mantissa"
)]
fn bench_population_montecarlo_500(c: &mut Criterion) {
    let n = 500;
    let cl_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.01, 8.0)).collect();
    let vd_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.05, 70.0)).collect();
    let ka_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.001, 1.2)).collect();
    let times: Vec<f64> = (0..=100).map(|i| f64::from(i) * 24.0 / 100.0).collect();
    c.bench_function("population_montecarlo_500", |b| {
        b.iter(|| {
            population_pk_cpu(
                black_box(n),
                black_box(&cl_params),
                black_box(&vd_params),
                black_box(&ka_params),
                4.0,
                0.79,
                &times,
            )
        });
    });
}

#[expect(
    clippy::cast_precision_loss,
    reason = "benchmark indices fit f64 mantissa"
)]
fn bench_population_montecarlo_5000(c: &mut Criterion) {
    let n = 5_000;
    let cl_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.001, 8.0)).collect();
    let vd_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.005, 70.0)).collect();
    let ka_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.0001, 1.2)).collect();
    let times: Vec<f64> = (0..=100).map(|i| f64::from(i) * 24.0 / 100.0).collect();
    c.bench_function("population_montecarlo_5000", |b| {
        b.iter(|| {
            population_pk_cpu(
                black_box(n),
                black_box(&cl_params),
                black_box(&vd_params),
                black_box(&ka_params),
                4.0,
                0.79,
                &times,
            )
        });
    });
}

fn bench_michaelis_menten_pk(c: &mut Criterion) {
    use healthspring_barracuda::pkpd;
    c.bench_function("mm_pk_simulate_10day", |b| {
        b.iter(|| pkpd::mm_pk_simulate(black_box(&pkpd::PHENYTOIN_PARAMS), 300.0, 10.0, 0.001));
    });
}

fn bench_scfa_production(c: &mut Criterion) {
    use healthspring_barracuda::microbiome;
    c.bench_function("scfa_production", |b| {
        b.iter(|| microbiome::scfa_production(black_box(20.0), &microbiome::SCFA_HEALTHY_PARAMS));
    });
}

fn bench_gut_serotonin(c: &mut Criterion) {
    use healthspring_barracuda::microbiome;
    c.bench_function("gut_serotonin_production", |b| {
        b.iter(|| microbiome::gut_serotonin_production(black_box(150.0), black_box(2.2), 0.8, 0.1));
    });
}

fn bench_antibiotic_perturbation(c: &mut Criterion) {
    use healthspring_barracuda::microbiome;
    c.bench_function("antibiotic_perturbation_42d", |b| {
        b.iter(|| {
            microbiome::antibiotic_perturbation(black_box(2.2), 0.5, 0.3, 0.1, 7.0, 42.0, 0.1)
        });
    });
}

fn bench_stress_index(c: &mut Criterion) {
    use healthspring_barracuda::biosignal;
    c.bench_function("stress_index", |b| {
        b.iter(|| biosignal::compute_stress_index(black_box(5.0), black_box(4.0), black_box(2.5)));
    });
}

fn bench_beat_classify(c: &mut Criterion) {
    use healthspring_barracuda::biosignal;
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
    let beat = biosignal::generate_normal_template(41);
    c.bench_function("beat_classify_3templates", |b| {
        b.iter(|| biosignal::classify_beat(black_box(&beat), &templates, 0.7));
    });
}

criterion_group!(
    benches,
    bench_hill_sweep_50,
    bench_hill_sweep_10k,
    bench_pk_curve_101,
    bench_diversity_indices_7,
    bench_diversity_batch_1k,
    bench_auc_trapezoidal_101,
    bench_population_montecarlo_500,
    bench_population_montecarlo_5000,
    bench_michaelis_menten_pk,
    bench_scfa_production,
    bench_gut_serotonin,
    bench_antibiotic_perturbation,
    bench_stress_index,
    bench_beat_classify,
);
criterion_main!(benches);
