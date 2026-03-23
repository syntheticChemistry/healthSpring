// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(
    missing_docs,
    reason = "criterion macros generate undocumented public items"
)]
//! Kokkos-equivalent benchmarks for healthSpring workloads.
//!
//! These benchmarks measure CPU performance of patterns commonly benchmarked
//! in the Kokkos performance-portable framework (reduction, scatter, Monte Carlo,
//! ODE batch, stencil). healthSpring-domain workloads are used so results directly
//! compare Tier 1 (Rust CPU) against future Tier 2 (WGSL GPU) implementations.
//!
//! Kokkos reference: <https://github.com/kokkos/kokkos-benchmarks>

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use healthspring_barracuda::microbiome::{shannon_index, simpson_index};
use healthspring_barracuda::pkpd::{
    NlmeConfig, SyntheticPopConfig, auc_trapezoidal, foce, generate_synthetic_population, nca_iv,
    oral_one_compartment_model, population_pk_cpu,
};

// ═══════════════════════════════════════════════════════════════════════
// Benchmark 1: Parallel Reduction (Kokkos parallel_reduce)
//
// Domain: AUC trapezoidal integration over large concentration arrays.
// Kokkos equivalent: sum reduction over contiguous memory.
// ═══════════════════════════════════════════════════════════════════════

fn bench_reduction_auc_10k(c: &mut Criterion) {
    let n = 10_000_i32;
    let times: Vec<f64> = (0..n).map(|i| f64::from(i) * 0.01).collect();
    let concs: Vec<f64> = times.iter().map(|&t| 10.0 * (-0.1 * t).exp()).collect();
    c.bench_function("kokkos_reduce_auc_10K", |b| {
        b.iter(|| auc_trapezoidal(black_box(&times), black_box(&concs)));
    });
}

fn bench_reduction_auc_100k(c: &mut Criterion) {
    let n = 100_000_i32;
    let times: Vec<f64> = (0..n).map(|i| f64::from(i) * 0.001).collect();
    let concs: Vec<f64> = times.iter().map(|&t| 10.0 * (-0.1 * t).exp()).collect();
    c.bench_function("kokkos_reduce_auc_100K", |b| {
        b.iter(|| auc_trapezoidal(black_box(&times), black_box(&concs)));
    });
}

// ═══════════════════════════════════════════════════════════════════════
// Benchmark 2: Scatter-Gather (Kokkos parallel_for with scatter)
//
// Domain: Diversity index batch computation — read communities, write indices.
// Kokkos equivalent: scatter pattern over independent work items.
// ═══════════════════════════════════════════════════════════════════════

#[expect(
    clippy::cast_precision_loss,
    reason = "index arithmetic product fits f64 mantissa"
)]
fn bench_scatter_diversity_10k(c: &mut Criterion) {
    let communities: Vec<Vec<f64>> = (0..10_000_usize)
        .map(|i| {
            let mut abundances = vec![0.0; 20];
            let mut total = 0.0;
            for (j, val) in abundances.iter_mut().enumerate() {
                *val = ((i * 20 + j + 1) as f64).sqrt();
                total += *val;
            }
            for val in &mut abundances {
                *val /= total;
            }
            abundances
        })
        .collect();
    c.bench_function("kokkos_scatter_diversity_10K", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(communities.len());
            for comm in &communities {
                results.push((
                    shannon_index(black_box(comm)),
                    simpson_index(black_box(comm)),
                ));
            }
            results
        });
    });
}

// ═══════════════════════════════════════════════════════════════════════
// Benchmark 3: Monte Carlo (Kokkos parallel_reduce with RNG)
//
// Domain: Population PK Monte Carlo — N independent PK simulations.
// Kokkos equivalent: embarrassingly parallel stochastic simulations.
// ═══════════════════════════════════════════════════════════════════════

#[expect(
    clippy::cast_precision_loss,
    reason = "1000 patient indices fit f64 mantissa"
)]
fn bench_montecarlo_pop_pk_1k(c: &mut Criterion) {
    let n = 1_000_usize;
    let cl_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.005, 8.0)).collect();
    let vd_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.02, 70.0)).collect();
    let ka_params: Vec<f64> = (0..n).map(|i| (i as f64).mul_add(0.0005, 1.2)).collect();
    let times: Vec<f64> = (0..=200).map(|i| f64::from(i) * 24.0 / 200.0).collect();
    c.bench_function("kokkos_montecarlo_popPK_1K", |b| {
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

// ═══════════════════════════════════════════════════════════════════════
// Benchmark 4: ODE Batch (Kokkos parallel_for with per-thread ODE solve)
//
// Domain: NCA analysis over batch of PK profiles — each requires full
// trapezoidal integration, lambda-z regression, and derived metrics.
// Kokkos equivalent: batch of independent ODE/integration workloads.
// ═══════════════════════════════════════════════════════════════════════

#[expect(
    clippy::cast_precision_loss,
    reason = "profile/point indices fit f64 mantissa"
)]
fn bench_ode_batch_nca_100(c: &mut Criterion) {
    let n_profiles = 100_usize;
    let n_points = 500_usize;
    let profiles: Vec<(Vec<f64>, Vec<f64>)> = (0..n_profiles)
        .map(|i| {
            let ke = 0.002f64.mul_add(i as f64, 0.05);
            let vd = 0.5f64.mul_add(i as f64, 10.0);
            let dose = 100.0;
            let c0 = dose / vd;
            let times: Vec<f64> = (0..n_points)
                .map(|j| 48.0 * j as f64 / (n_points - 1) as f64)
                .collect();
            let concs: Vec<f64> = times.iter().map(|&t| c0 * (-ke * t).exp()).collect();
            (times, concs)
        })
        .collect();
    c.bench_function("kokkos_ode_batch_nca_100", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(n_profiles);
            for (times, concs) in &profiles {
                results.push(nca_iv(black_box(times), black_box(concs), 100.0, 3));
            }
            results
        });
    });
}

// ═══════════════════════════════════════════════════════════════════════
// Benchmark 5: NLME Iteration (Kokkos team-parallel nested pattern)
//
// Domain: Single FOCE iteration — inner loop optimizes per-subject etas,
// outer loop updates population parameters. Tests nested parallelism.
// Kokkos equivalent: team-level parallelism with thread-level inner loops.
// ═══════════════════════════════════════════════════════════════════════

fn bench_nlme_foce_20subj_10iter(c: &mut Criterion) {
    let theta = vec![2.3, 4.4, 0.4];
    let omega = vec![0.04, 0.04, 0.09];
    let sigma = 0.01;
    let times: Vec<f64> = (0..12).map(|i| f64::from(i) * 2.0).collect();
    let subjects = generate_synthetic_population(&SyntheticPopConfig {
        model: oral_one_compartment_model,
        theta: &theta,
        omega: &omega,
        sigma,
        n_subjects: 20,
        times: &times,
        dose: 4.0,
        seed: 42,
    });
    let config = NlmeConfig {
        n_theta: 3,
        n_eta: 3,
        max_iter: 10,
        tol: 1e-10,
        seed: 12_345,
    };
    c.bench_function("kokkos_team_nlme_foce_20subj", |b| {
        b.iter(|| {
            foce(
                oral_one_compartment_model,
                black_box(&subjects),
                &theta,
                &omega,
                sigma,
                &config,
            )
        });
    });
}

criterion_group!(
    kokkos_benches,
    bench_reduction_auc_10k,
    bench_reduction_auc_100k,
    bench_scatter_diversity_10k,
    bench_montecarlo_pop_pk_1k,
    bench_ode_batch_nca_100,
    bench_nlme_foce_20subj_10iter,
);
criterion_main!(kokkos_benches);
