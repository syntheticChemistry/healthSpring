// SPDX-License-Identifier: AGPL-3.0-or-later
//! Benchmarks comparing local healthSpring implementations vs upstream
//! barraCuda canonical versions.
//!
//! ## Cross-spring provenance
//!
//! These benchmarks verify that rewiring to upstream is performance-neutral.
//! The upstream implementations were absorbed from healthSpring and should
//! be algorithmically identical — any delta is due to inlining, LTO, or
//! subtle instruction ordering differences.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use healthspring_barracuda::microbiome::{
    anderson_diagonalize, bray_curtis, pielou_evenness, shannon_index, simpson_index,
};
use healthspring_barracuda::rng::{lcg_step, state_to_f64};

fn bench_shannon_local_vs_upstream(c: &mut Criterion) {
    let abundances = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02];
    let mut group = c.benchmark_group("shannon");
    group.bench_function("local", |b| {
        b.iter(|| shannon_index(black_box(&abundances)));
    });
    group.bench_function("upstream", |b| {
        b.iter(|| barracuda::stats::shannon_from_frequencies(black_box(&abundances)));
    });
    group.finish();
}

fn bench_simpson_local_vs_upstream(c: &mut Criterion) {
    let abundances = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02];
    let mut group = c.benchmark_group("simpson");
    group.bench_function("local", |b| {
        b.iter(|| simpson_index(black_box(&abundances)));
    });
    group.bench_function("upstream", |b| {
        b.iter(|| barracuda::stats::simpson(black_box(&abundances)));
    });
    group.finish();
}

fn bench_pielou_local_vs_upstream(c: &mut Criterion) {
    let abundances = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02];
    let mut group = c.benchmark_group("pielou");
    group.bench_function("local", |b| {
        b.iter(|| pielou_evenness(black_box(&abundances)));
    });
    group.bench_function("upstream", |b| {
        b.iter(|| barracuda::stats::pielou_evenness(black_box(&abundances)));
    });
    group.finish();
}

fn bench_bray_curtis_local_vs_upstream(c: &mut Criterion) {
    let a = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02];
    let b_arr = [0.10, 0.10, 0.10, 0.30, 0.20, 0.10, 0.10];
    let mut group = c.benchmark_group("bray_curtis");
    group.bench_function("local", |b| {
        b.iter(|| bray_curtis(black_box(&a), black_box(&b_arr)));
    });
    group.bench_function("upstream", |b| {
        b.iter(|| barracuda::stats::bray_curtis(black_box(&a), black_box(&b_arr)));
    });
    group.finish();
}

fn bench_hill_local_vs_upstream(c: &mut Criterion) {
    let concs: Vec<f64> = (0..1000)
        .map(|i| 0.1 * 1000.0_f64.powf(f64::from(i) / 999.0))
        .collect();
    let mut group = c.benchmark_group("hill_1k");
    group.bench_function("local", |b| {
        b.iter(|| {
            concs
                .iter()
                .map(|&c| {
                    healthspring_barracuda::pkpd::hill_dose_response(black_box(c), 10.0, 1.5, 100.0)
                })
                .collect::<Vec<_>>()
        });
    });
    group.bench_function("upstream", |b| {
        b.iter(|| {
            concs
                .iter()
                .map(|&c| barracuda::stats::hill(black_box(c), 10.0, 1.5) * 100.0)
                .collect::<Vec<_>>()
        });
    });
    group.finish();
}

fn bench_lcg_local_vs_upstream(c: &mut Criterion) {
    let mut group = c.benchmark_group("lcg_1m_steps");
    group.bench_function("local_delegated", |b| {
        b.iter(|| {
            let mut s = black_box(42_u64);
            for _ in 0..1_000_000 {
                s = lcg_step(s);
            }
            s
        });
    });
    group.bench_function("upstream_direct", |b| {
        b.iter(|| {
            let mut s = black_box(42_u64);
            for _ in 0..1_000_000 {
                s = barracuda::rng::lcg_step(s);
            }
            s
        });
    });
    group.finish();
}

fn bench_state_to_f64_local_vs_upstream(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_to_f64");
    group.bench_function("local_delegated", |b| {
        b.iter(|| state_to_f64(black_box(123_456_789_u64)));
    });
    group.bench_function("upstream_direct", |b| {
        b.iter(|| barracuda::rng::state_to_f64(black_box(123_456_789_u64)));
    });
    group.finish();
}

fn bench_anderson_upstream(c: &mut Criterion) {
    let disorder: Vec<f64> = (0..50).map(|i| f64::from(i) * 0.1 - 2.5).collect();
    let mut group = c.benchmark_group("anderson_50");
    group.bench_function("upstream_delegate", |b| {
        b.iter(|| anderson_diagonalize(black_box(&disorder), 1.0));
    });
    group.bench_function("upstream_direct", |b| {
        b.iter(|| barracuda::special::anderson_diagonalize(black_box(&disorder), 1.0));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_shannon_local_vs_upstream,
    bench_simpson_local_vs_upstream,
    bench_pielou_local_vs_upstream,
    bench_bray_curtis_local_vs_upstream,
    bench_hill_local_vs_upstream,
    bench_lcg_local_vs_upstream,
    bench_state_to_f64_local_vs_upstream,
    bench_anderson_upstream,
);
criterion_main!(benches);
