// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(
    missing_docs,
    reason = "criterion macros generate undocumented public items"
)]
//! Criterion benchmarks for healthSpring GPU-dispatchable ops (`GpuOp`).
//!
//! Uses [`healthspring_barracuda::gpu::execute_gpu`] when hardware is available
//! ([`healthspring_barracuda::gpu::gpu_available`]), otherwise the CPU
//! reference path ([`healthspring_barracuda::gpu::execute_cpu`]). Mirrors
//! workload sizing from `cpu_parity.rs` where applicable.

use std::sync::OnceLock;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use healthspring_barracuda::gpu::{GpuOp, GpuResult, execute_cpu};
#[cfg(feature = "gpu")]
use healthspring_barracuda::gpu::{execute_gpu, gpu_available};

#[expect(clippy::expect_used, reason = "benchmark Tokio runtime initialization")]
fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime for GPU benchmarks")
    })
}

/// Run `op` on GPU when the `gpu` feature is enabled and an adapter is present;
/// otherwise (or on GPU error) use the CPU fallback.
fn dispatch_gpu_op(op: &GpuOp) -> GpuResult {
    #[cfg(feature = "gpu")]
    {
        if gpu_available() {
            if let Ok(r) = runtime().block_on(execute_gpu(op)) {
                return r;
            }
        }
    }
    execute_cpu(op)
}

fn bench_gpu_hill_batch_10k(c: &mut Criterion) {
    let concentrations: Vec<f64> = (0..10_000)
        .map(|i| 0.1 * 1000.0_f64.powf(f64::from(i) / 9_999.0))
        .collect();
    let op = GpuOp::HillSweep {
        emax: 100.0,
        ec50: 10.0,
        n: 1.5,
        concentrations,
    };
    c.bench_function("gpu_hill_dose_response_batch_10k", |b| {
        b.iter(|| black_box(dispatch_gpu_op(black_box(&op))));
    });
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "benchmark loop bounds guarantee indices fit in u32"
)]
fn bench_gpu_diversity_fusion_batch_1k(c: &mut Criterion) {
    let communities: Vec<Vec<f64>> = (0..1_000)
        .map(|i| {
            let mut a = vec![0.0; 7];
            let mut total = 0.0;
            for (j, val) in a.iter_mut().enumerate() {
                *val = f64::from((i * 7 + j + 1) as u32).sqrt();
                total += *val;
            }
            for val in &mut a {
                *val /= total;
            }
            a
        })
        .collect();
    let op = GpuOp::DiversityBatch { communities };
    c.bench_function("gpu_diversity_fusion_batch_1k", |b| {
        b.iter(|| black_box(dispatch_gpu_op(black_box(&op))));
    });
}

fn bench_gpu_population_pk_mc_batch_5k(c: &mut Criterion) {
    let n_patients = 5_000;
    let op = GpuOp::PopulationPkBatch {
        n_patients,
        dose_mg: 4.0,
        f_bioavail: 0.79,
        seed: 42,
    };
    c.bench_function("gpu_population_pk_montecarlo_batch_5k", |b| {
        b.iter(|| black_box(dispatch_gpu_op(black_box(&op))));
    });
}

fn bench_gpu_michaelis_menten_batch_512(c: &mut Criterion) {
    let op = GpuOp::MichaelisMentenBatch {
        vmax: 500.0,
        km: 5.0,
        vd: 50.0,
        dt: 0.01,
        n_steps: 2000,
        n_patients: 512,
        seed: 42,
    };
    c.bench_function("gpu_michaelis_menten_batch_512", |b| {
        b.iter(|| black_box(dispatch_gpu_op(black_box(&op))));
    });
}

criterion_group!(
    gpu_benches,
    bench_gpu_hill_batch_10k,
    bench_gpu_diversity_fusion_batch_1k,
    bench_gpu_population_pk_mc_batch_5k,
    bench_gpu_michaelis_menten_batch_512,
);
criterion_main!(gpu_benches);
