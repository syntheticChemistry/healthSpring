// SPDX-License-Identifier: AGPL-3.0-or-later
//! CPU reference implementations for each [`GpuOp`].
//!
//! Every GPU operation has a pure-Rust CPU path here. The GPU dispatch
//! must produce identical results within f64 tolerance.

use crate::{microbiome, pkpd};

use super::types::{GpuOp, GpuResult};

/// Execute a GPU operation using CPU fallback (pure Rust).
///
/// This is the reference implementation. The GPU path (behind `gpu` feature)
/// must produce identical results within f64 tolerance.
#[must_use]
pub fn execute_cpu(op: &GpuOp) -> GpuResult {
    match op {
        GpuOp::HillSweep {
            emax,
            ec50,
            n,
            concentrations,
        } => execute_cpu_hill_sweep(*emax, *ec50, *n, concentrations),
        GpuOp::PopulationPkBatch {
            n_patients,
            dose_mg,
            f_bioavail,
            seed,
        } => execute_cpu_population_pk_batch(*n_patients, *dose_mg, *f_bioavail, *seed),
        GpuOp::DiversityBatch { communities } => execute_cpu_diversity_batch(communities),
        GpuOp::MichaelisMentenBatch {
            vmax,
            km,
            vd,
            dt,
            n_steps,
            n_patients,
            seed,
        } => execute_cpu_mm_batch(*vmax, *km, *vd, *dt, *n_steps, *n_patients, *seed),
        GpuOp::ScfaBatch {
            params,
            fiber_inputs,
        } => execute_cpu_scfa_batch(params, fiber_inputs),
        GpuOp::BeatClassifyBatch { beats, templates } => {
            execute_cpu_beat_classify_batch(beats, templates)
        }
    }
}

fn execute_cpu_hill_sweep(emax: f64, ec50: f64, n: f64, concentrations: &[f64]) -> GpuResult {
    let results: Vec<f64> = concentrations
        .iter()
        .map(|&c| pkpd::hill_dose_response(c, ec50, n, emax))
        .collect();
    GpuResult::HillSweep(results)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "LCG state u64 → f64 for uniform variate; precision sufficient for PK variation"
)]
fn execute_cpu_population_pk_batch(
    n_patients: usize,
    dose_mg: f64,
    f_bioavail: f64,
    seed: u64,
) -> GpuResult {
    let mut aucs = Vec::with_capacity(n_patients);
    let mut rng_state = seed;
    for _ in 0..n_patients {
        rng_state = crate::rng::lcg_step(rng_state);
        let u = (rng_state >> 33) as f64 / f64::from(u32::MAX);
        let cl_factor = 0.5 + u;
        let cl = 10.0 * cl_factor;
        let auc = f_bioavail * dose_mg / cl;
        aucs.push(auc);
    }
    GpuResult::PopulationPkBatch(aucs)
}

fn execute_cpu_diversity_batch(communities: &[Vec<f64>]) -> GpuResult {
    let results: Vec<(f64, f64)> = communities
        .iter()
        .map(|c| (microbiome::shannon_index(c), microbiome::simpson_index(c)))
        .collect();
    GpuResult::DiversityBatch(results)
}

fn execute_cpu_mm_batch(
    vmax: f64,
    km: f64,
    vd: f64,
    dt: f64,
    n_steps: u32,
    n_patients: u32,
    seed: u32,
) -> GpuResult {
    let params = pkpd::MichaelisMentenParams { vmax, km, vd };
    let dose_mg = vd * 6.0;
    let t_end = f64::from(n_steps) * dt;
    let mut aucs = Vec::with_capacity(n_patients as usize);
    for i in 0..n_patients {
        let u = wang_hash_uniform(seed.wrapping_add(i));
        let factor = u.mul_add(0.6, 0.7);
        let patient_params = pkpd::MichaelisMentenParams {
            vmax: params.vmax * factor,
            ..params.clone()
        };
        let (_, concs) = pkpd::mm_pk_simulate(&patient_params, dose_mg, t_end, dt);
        let auc = pkpd::mm_auc(&concs, dt);
        aucs.push(auc);
    }
    GpuResult::MichaelisMentenBatch(aucs)
}

fn execute_cpu_scfa_batch(params: &microbiome::ScfaParams, fiber_inputs: &[f64]) -> GpuResult {
    let results: Vec<(f64, f64, f64)> = fiber_inputs
        .iter()
        .map(|&f| microbiome::scfa_production(f, params))
        .collect();
    GpuResult::ScfaBatch(results)
}

fn execute_cpu_beat_classify_batch(beats: &[Vec<f64>], templates: &[Vec<f64>]) -> GpuResult {
    use crate::biosignal::classification;
    let tmpl_structs: Vec<classification::BeatTemplate> = templates
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let class = match i {
                0 => classification::BeatClass::Normal,
                1 => classification::BeatClass::Pvc,
                2 => classification::BeatClass::Pac,
                _ => classification::BeatClass::Unknown,
            };
            classification::BeatTemplate {
                class,
                waveform: w.clone(),
            }
        })
        .collect();
    let results: Vec<(u32, f64)> = beats
        .iter()
        .map(|beat| {
            let (class, corr) = classification::classify_beat(beat, &tmpl_structs, 0.0);
            let idx = match class {
                classification::BeatClass::Normal => 0,
                classification::BeatClass::Pvc => 1,
                classification::BeatClass::Pac => 2,
                classification::BeatClass::Unknown => u32::MAX,
            };
            (idx, corr)
        })
        .collect();
    GpuResult::BeatClassifyBatch(results)
}

/// Wang hash for deterministic per-patient PRNG (mirrors WGSL shader).
///
/// This is a u32 hash producing a uniform `[0, 1)` variate — used for GPU
/// parity with WGSL shaders that lack u64. The canonical LCG PRNG is in
/// [`crate::rng`] via `barracuda::rng`; this hash exists solely for
/// Michaelis-Menten batch parity with the WGSL kernel.
fn wang_hash_uniform(seed: u32) -> f64 {
    let mut s = seed;
    s = (s ^ 0x3d) ^ (s >> 16);
    s = s.wrapping_mul(9);
    s = s ^ (s >> 4);
    s = s.wrapping_mul(0x27d4_eb2d);
    s = s ^ (s >> 15);
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    f64::from(s) / f64::from(u32::MAX)
}
