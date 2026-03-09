// SPDX-License-Identifier: AGPL-3.0-or-later
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

fn pk_curve(dose: f64, f_bio: f64, vd: f64, ka: f64, ke: f64, n_points: usize) -> Vec<f64> {
    let dt = 24.0 / n_points as f64;
    (0..=n_points)
        .map(|i| pk_oral_one_compartment(dose, f_bio, vd, ka, ke, i as f64 * dt))
        .collect()
}

fn bench_hill_sweep_50(c: &mut Criterion) {
    let concs: Vec<f64> = (0..50)
        .map(|i| 0.1 * 1000.0_f64.powf(i as f64 / 49.0))
        .collect();
    c.bench_function("hill_sweep_50", |b| {
        b.iter(|| hill_sweep(black_box(&concs), 10.0, 1.5, 100.0));
    });
}

fn bench_hill_sweep_10k(c: &mut Criterion) {
    let concs: Vec<f64> = (0..10_000)
        .map(|i| 0.1 * 1000.0_f64.powf(i as f64 / 9_999.0))
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
    let times: Vec<f64> = (0..=100).map(|i| i as f64 * 24.0 / 100.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(4.0, 0.79, vd, 1.5, ke, t))
        .collect();
    c.bench_function("auc_trapezoidal_101_points", |b| {
        b.iter(|| auc_trapezoidal(black_box(&times), black_box(&concs)));
    });
}

fn bench_population_montecarlo_500(c: &mut Criterion) {
    let n = 500;
    let cl_params: Vec<f64> = (0..n).map(|i| 8.0 + (i as f64 * 0.01)).collect();
    let vd_params: Vec<f64> = (0..n).map(|i| 70.0 + (i as f64 * 0.05)).collect();
    let ka_params: Vec<f64> = (0..n).map(|i| 1.2 + (i as f64 * 0.001)).collect();
    let times: Vec<f64> = (0..=100).map(|i| i as f64 * 24.0 / 100.0).collect();
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

fn bench_population_montecarlo_5000(c: &mut Criterion) {
    let n = 5_000;
    let cl_params: Vec<f64> = (0..n).map(|i| 8.0 + (i as f64 * 0.001)).collect();
    let vd_params: Vec<f64> = (0..n).map(|i| 70.0 + (i as f64 * 0.005)).collect();
    let ka_params: Vec<f64> = (0..n).map(|i| 1.2 + (i as f64 * 0.0001)).collect();
    let times: Vec<f64> = (0..=100).map(|i| i as f64 * 24.0 / 100.0).collect();
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
);
criterion_main!(benches);
