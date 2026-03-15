// SPDX-License-Identifier: AGPL-3.0-only
//! GPU-dispatchable operations for healthSpring.
//!
//! Defines operations that can execute on either CPU (pure Rust) or GPU
//! (via WGSL shaders through wgpu). CPU fallback is always available.
//! GPU dispatch is activated by the `gpu` feature flag.
//!
//! ## Shader Mapping
//!
//! | healthSpring Op | WGSL Shader | Use Case |
//! |-----------------|-------------|----------|
//! | `HillSweep` | `hill_dose_response_f64.wgsl` | Exp001 vectorized |
//! | `PopulationPkBatch` | `population_pk_f64.wgsl` | Exp005/036 Monte Carlo |
//! | `DiversityBatch` | `diversity_f64.wgsl` | Exp010 batch |
//! | `MichaelisMentenBatch` | `michaelis_menten_batch_f64.wgsl` | Exp077 batch PK ODE |
//! | `ScfaBatch` | `scfa_batch_f64.wgsl` | Exp079 batch metabolic |
//! | `BeatClassifyBatch` | `beat_classify_batch_f64.wgsl` | Exp082 batch ECG |
//!
//! ## ABSORPTION STATUS (barraCuda S-latest / coralReef Phase 10)
//!
//! **Absorbed upstream** (barraCuda owns canonical versions):
//! - `HillFunctionF64` — `barracuda::ops::HillFunctionF64`
//! - `PopulationPkF64` — `barracuda::ops::PopulationPkF64`
//! - `DiversityFusionGpu` — `barracuda::ops::bio::DiversityFusionGpu`
//! - LCG PRNG — `barracuda::rng::{lcg_step, LCG_MULTIPLIER}`
//! - Eigensolver — `barracuda::special::{tridiagonal_ql, anderson_diagonalize}`
//! - Diversity stats — `barracuda::stats::{shannon, simpson, chao1, pielou, bray_curtis}`
//!
//! **Pending** (local to healthSpring until next absorption):
//! - `GpuContext` fused pipeline pattern → barraCuda compute executor
//! - `strip_f64_enable()` WGSL preprocessor → coralReef naga pass
//! - `shader_for_op()` mapping → barraCuda shader registry
//!
//! **Precision evolution**: metalForge now has `PrecisionRouting` (mirroring
//! toadStool S128 `PrecisionRoutingAdvice`). GPU context should use this to
//! select f64/DF64/f32 shader variants. coralReef Phase 10 provides full
//! f64 transcendental support via DFMA polynomial lowering.

use crate::microbiome;
use crate::pkpd;

#[cfg(feature = "gpu")]
pub mod context;
#[cfg(feature = "gpu")]
pub mod dispatch;

/// WGSL shader sources — compiled into the binary.
///
/// ## Provenance
///
/// Canonical upstream versions now live in barraCuda:
/// - `barracuda::shaders::math::hill_f64.wgsl`
/// - `barracuda::shaders::science::population_pk_f64.wgsl`
/// - `barracuda::shaders::bio::diversity_fusion_f64.wgsl`
///
/// These local copies are the Spring validation targets — bit-identical
/// to the versions absorbed by barraCuda. When healthSpring evolves to
/// consume `barracuda::ops::{HillFunctionF64, PopulationPkF64}` and
/// `barracuda::ops::bio::DiversityFusionGpu`, these local copies will
/// be removed.
///
/// ## Precision
///
/// All shaders use `f64` with f32 transcendental workarounds:
/// - `pow(f64,f64)` → `exp(f32(n * log(c)))` (~7 decimal digits)
/// - `log_f64(x)` → `f64(log(f32(x)))` for driver portability
///
/// barraCuda's `Fp64Strategy` and coralReef's f64 lowering will replace
/// these workarounds when the sovereign pipeline is complete.
pub mod shaders {
    pub const HILL_DOSE_RESPONSE: &str =
        include_str!("../../shaders/health/hill_dose_response_f64.wgsl");
    pub const POPULATION_PK: &str = include_str!("../../shaders/health/population_pk_f64.wgsl");
    pub const DIVERSITY: &str = include_str!("../../shaders/health/diversity_f64.wgsl");
    pub const MICHAELIS_MENTEN_BATCH: &str =
        include_str!("../../shaders/health/michaelis_menten_batch_f64.wgsl");
    pub const SCFA_BATCH: &str = include_str!("../../shaders/health/scfa_batch_f64.wgsl");
    pub const BEAT_CLASSIFY_BATCH: &str =
        include_str!("../../shaders/health/beat_classify_batch_f64.wgsl");
}

/// A GPU-dispatchable operation with input/output buffers.
#[derive(Debug, Clone)]
pub enum GpuOp {
    /// Vectorized Hill dose-response: compute E(c) for many concentrations.
    HillSweep {
        emax: f64,
        ec50: f64,
        n: f64,
        concentrations: Vec<f64>,
    },
    /// Batch population PK: simulate N patients in parallel.
    PopulationPkBatch {
        n_patients: usize,
        dose_mg: f64,
        f_bioavail: f64,
        seed: u64,
    },
    /// Batch diversity indices for multiple communities.
    DiversityBatch { communities: Vec<Vec<f64>> },
    /// Batch Michaelis-Menten PK: parallel Euler ODE per patient.
    MichaelisMentenBatch {
        vmax: f64,
        km: f64,
        vd: f64,
        dt: f64,
        n_steps: u32,
        n_patients: u32,
        seed: u32,
    },
    /// Batch SCFA production: element-wise Michaelis-Menten per fiber input.
    ScfaBatch {
        params: crate::microbiome::ScfaParams,
        fiber_inputs: Vec<f64>,
    },
    /// Batch beat classification: template correlation per beat window.
    BeatClassifyBatch {
        beats: Vec<Vec<f64>>,
        templates: Vec<Vec<f64>>,
    },
}

/// Result of a GPU operation.
#[derive(Debug, Clone)]
pub enum GpuResult {
    /// Hill sweep results: one E value per concentration.
    HillSweep(Vec<f64>),
    /// Population PK results: AUC per patient.
    PopulationPkBatch(Vec<f64>),
    /// Diversity results: (shannon, simpson) per community.
    DiversityBatch(Vec<(f64, f64)>),
    /// Michaelis-Menten batch: AUC per patient.
    MichaelisMentenBatch(Vec<f64>),
    /// SCFA batch: (acetate, propionate, butyrate) per fiber input.
    ScfaBatch(Vec<(f64, f64, f64)>),
    /// Beat classify batch: (`template_index`, correlation) per beat.
    BeatClassifyBatch(Vec<(u32, f64)>),
}

/// Execute a GPU operation using CPU fallback (pure Rust).
///
/// This is the reference implementation. The GPU path (behind `gpu` feature)
/// must produce identical results within f64 tolerance.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "LCG state u64 → f64 for uniform variate; precision sufficient for PK variation"
)]
#[expect(clippy::too_many_lines, reason = "flat match over all GpuOp variants")]
pub fn execute_cpu(op: &GpuOp) -> GpuResult {
    match op {
        GpuOp::HillSweep {
            emax,
            ec50,
            n,
            concentrations,
        } => {
            let results: Vec<f64> = concentrations
                .iter()
                .map(|&c| pkpd::hill_dose_response(c, *ec50, *n, *emax))
                .collect();
            GpuResult::HillSweep(results)
        }
        GpuOp::PopulationPkBatch {
            n_patients,
            dose_mg,
            f_bioavail,
            seed,
        } => {
            let mut aucs = Vec::with_capacity(*n_patients);
            let mut rng_state = *seed;
            for _ in 0..*n_patients {
                rng_state = crate::rng::lcg_step(rng_state);
                let u = (rng_state >> 33) as f64 / f64::from(u32::MAX);
                let cl_factor = 0.5 + u;
                let cl = 10.0 * cl_factor;
                let auc = f_bioavail * dose_mg / cl;
                aucs.push(auc);
            }
            GpuResult::PopulationPkBatch(aucs)
        }
        GpuOp::DiversityBatch { communities } => {
            let results: Vec<(f64, f64)> = communities
                .iter()
                .map(|c| (microbiome::shannon_index(c), microbiome::simpson_index(c)))
                .collect();
            GpuResult::DiversityBatch(results)
        }
        GpuOp::MichaelisMentenBatch {
            vmax,
            km,
            vd,
            dt,
            n_steps,
            n_patients,
            seed,
        } => {
            let params = pkpd::MichaelisMentenParams {
                vmax: *vmax,
                km: *km,
                vd: *vd,
            };
            let dose_mg = vd * 6.0;
            let t_end = f64::from(*n_steps) * dt;
            let mut aucs = Vec::with_capacity(*n_patients as usize);
            for i in 0..*n_patients {
                let u = wang_hash_uniform(seed.wrapping_add(i));
                let factor = u.mul_add(0.6, 0.7);
                let patient_params = pkpd::MichaelisMentenParams {
                    vmax: params.vmax * factor,
                    ..params.clone()
                };
                let (_, concs) = pkpd::mm_pk_simulate(&patient_params, dose_mg, t_end, *dt);
                let auc = pkpd::mm_auc(&concs, *dt);
                aucs.push(auc);
            }
            GpuResult::MichaelisMentenBatch(aucs)
        }
        GpuOp::ScfaBatch {
            params,
            fiber_inputs,
        } => {
            let results: Vec<(f64, f64, f64)> = fiber_inputs
                .iter()
                .map(|&f| microbiome::scfa_production(f, params))
                .collect();
            GpuResult::ScfaBatch(results)
        }
        GpuOp::BeatClassifyBatch { beats, templates } => {
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
    }
}

/// Wang hash for deterministic per-patient PRNG (mirrors WGSL shader).
fn wang_hash_uniform(seed: u32) -> f64 {
    let mut s = seed;
    s = (s ^ 61) ^ (s >> 16);
    s = s.wrapping_mul(9);
    s = s ^ (s >> 4);
    s = s.wrapping_mul(0x27d4_eb2d);
    s = s ^ (s >> 15);
    // xorshift32
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    f64::from(s) / f64::from(u32::MAX)
}

/// Shader descriptor: maps a `GpuOp` to its WGSL shader source.
#[must_use]
pub const fn shader_for_op(op: &GpuOp) -> &'static str {
    match op {
        GpuOp::HillSweep { .. } => shaders::HILL_DOSE_RESPONSE,
        GpuOp::PopulationPkBatch { .. } => shaders::POPULATION_PK,
        GpuOp::DiversityBatch { .. } => shaders::DIVERSITY,
        GpuOp::MichaelisMentenBatch { .. } => shaders::MICHAELIS_MENTEN_BATCH,
        GpuOp::ScfaBatch { .. } => shaders::SCFA_BATCH,
        GpuOp::BeatClassifyBatch { .. } => shaders::BEAT_CLASSIFY_BATCH,
    }
}

/// Estimate GPU memory requirement for an operation (bytes).
#[must_use]
pub fn gpu_memory_estimate(op: &GpuOp) -> u64 {
    match op {
        GpuOp::HillSweep { concentrations, .. } => (concentrations.len() as u64) * 8 * 2,
        GpuOp::PopulationPkBatch { n_patients, .. } => (*n_patients as u64) * 8 * 5,
        GpuOp::DiversityBatch { communities } => {
            let total_species: usize = communities.iter().map(Vec::len).sum();
            (total_species as u64) * 8 + (communities.len() as u64) * 16
        }
        GpuOp::MichaelisMentenBatch { n_patients, .. } => u64::from(*n_patients) * 8,
        GpuOp::ScfaBatch { fiber_inputs, .. } => (fiber_inputs.len() as u64) * 8 * 4,
        GpuOp::BeatClassifyBatch {
            beats, templates, ..
        } => {
            let ws = beats.first().map_or(0, Vec::len) as u64;
            (beats.len() as u64) * ws * 8
                + (templates.len() as u64) * ws * 8
                + (beats.len() as u64) * 16
        }
    }
}

// ---------------------------------------------------------------------------
// GPU execution (feature-gated)
// ---------------------------------------------------------------------------

/// Error type for GPU execution.
#[derive(Debug)]
pub enum GpuError {
    /// No GPU device available.
    NoDevice(String),
    /// Shader compilation or dispatch failed.
    Dispatch(String),
    /// Buffer readback failed.
    Readback(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDevice(msg) => write!(f, "GPU: no device: {msg}"),
            Self::Dispatch(msg) => write!(f, "GPU: dispatch failed: {msg}"),
            Self::Readback(msg) => write!(f, "GPU: readback failed: {msg}"),
        }
    }
}

impl std::error::Error for GpuError {}

#[cfg(feature = "gpu")]
pub use context::GpuContext;
#[cfg(feature = "gpu")]
pub use dispatch::execute_gpu;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hill_sweep_cpu() {
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 1.0,
            concentrations: vec![0.0, 5.0, 10.0, 20.0, 100.0],
        };
        if let GpuResult::HillSweep(results) = execute_cpu(&op) {
            assert_eq!(results.len(), 5);
            assert!(results[0].abs() < 1e-10, "E(0) = 0");
            assert!((results[2] - 50.0).abs() < 1e-10, "E(EC50) = Emax/2");
            assert!(results[4] > 90.0, "E(100) → Emax");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn population_pk_batch_cpu() {
        let op = GpuOp::PopulationPkBatch {
            n_patients: 100,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        };
        if let GpuResult::PopulationPkBatch(aucs) = execute_cpu(&op) {
            assert_eq!(aucs.len(), 100);
            assert!(aucs.iter().all(|&a| a > 0.0), "all AUC positive");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn diversity_batch_cpu() {
        let communities = vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]];
        let op = GpuOp::DiversityBatch { communities };
        if let GpuResult::DiversityBatch(results) = execute_cpu(&op) {
            assert_eq!(results.len(), 2);
            assert!(results[0].0 > results[1].0, "even > dominated Shannon");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn shader_sources_loaded() {
        assert!(shaders::HILL_DOSE_RESPONSE.contains("hill_dose_response"));
        assert!(shaders::POPULATION_PK.contains("population_pk"));
        assert!(shaders::DIVERSITY.contains("diversity"));
    }

    #[test]
    fn memory_estimate_reasonable() {
        let op = GpuOp::PopulationPkBatch {
            n_patients: 10_000,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        };
        let mem = gpu_memory_estimate(&op);
        assert!(mem < 1_000_000, "10K patients < 1MB GPU memory");
    }

    #[test]
    fn michaelis_menten_batch_cpu() {
        let op = GpuOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 2000,
            n_patients: 64,
            seed: 42,
        };
        if let GpuResult::MichaelisMentenBatch(aucs) = execute_cpu(&op) {
            assert_eq!(aucs.len(), 64);
            assert!(aucs.iter().all(|&a| a > 0.0), "all AUC positive");
            #[expect(clippy::cast_precision_loss, reason = "patient count ≪ 2^52")]
            let mean: f64 = aucs.iter().sum::<f64>() / aucs.len() as f64;
            assert!(mean > 1.0, "mean AUC should be physiological: {mean}");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn scfa_batch_cpu() {
        let op = GpuOp::ScfaBatch {
            params: crate::microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: vec![5.0, 10.0, 20.0, 30.0],
        };
        if let GpuResult::ScfaBatch(results) = execute_cpu(&op) {
            assert_eq!(results.len(), 4);
            for &(a, p, b) in &results {
                assert!(a > 0.0 && p > 0.0 && b > 0.0, "all SCFA > 0");
                assert!(a > p && a > b, "acetate dominant");
            }
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn beat_classify_batch_cpu() {
        use crate::biosignal::classification;
        let templates = vec![
            classification::generate_normal_template(41),
            classification::generate_pvc_template(41),
            classification::generate_pac_template(41),
        ];
        let beats = vec![
            classification::generate_normal_template(41),
            classification::generate_pvc_template(41),
        ];
        let op = GpuOp::BeatClassifyBatch { beats, templates };
        if let GpuResult::BeatClassifyBatch(results) = execute_cpu(&op) {
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].0, 0, "first beat → Normal (template 0)");
            assert_eq!(results[1].0, 1, "second beat → PVC (template 1)");
            assert!(results[0].1 > 0.99, "self-correlation ~ 1.0");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn new_shader_sources_loaded() {
        assert!(shaders::MICHAELIS_MENTEN_BATCH.contains("michaelis_menten"));
        assert!(shaders::SCFA_BATCH.contains("scfa"));
        assert!(shaders::BEAT_CLASSIFY_BATCH.contains("beat_classify"));
    }

    #[test]
    fn hill_sweep_deterministic() {
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 2.0,
            concentrations: vec![1.0, 5.0, 10.0, 50.0],
        };
        let r1 = execute_cpu(&op);
        let r2 = execute_cpu(&op);
        if let (GpuResult::HillSweep(a), GpuResult::HillSweep(b)) = (&r1, &r2) {
            for (x, y) in a.iter().zip(b.iter()) {
                assert_eq!(
                    x.to_bits(),
                    y.to_bits(),
                    "CPU fallback must be bit-identical"
                );
            }
        }
    }
}
