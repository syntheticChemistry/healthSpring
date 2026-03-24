// SPDX-License-Identifier: AGPL-3.0-or-later
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
//! **Tier A — Absorbed upstream** (barraCuda owns canonical ops, rewire ready):
//! - `HillFunctionF64` — `barracuda::ops::HillFunctionF64`
//! - `PopulationPkF64` — `barracuda::ops::PopulationPkF64`
//! - `DiversityFusionGpu` — `barracuda::ops::bio::DiversityFusionGpu`
//! - LCG PRNG — `barracuda::rng::{lcg_step, LCG_MULTIPLIER}`
//! - Eigensolver — `barracuda::special::{tridiagonal_ql, anderson_diagonalize}`
//! - Diversity stats — `barracuda::stats::{shannon, simpson, chao1, pielou, bray_curtis}`
//!
//! **Tier B — Absorbed upstream, rewired** (`barracuda::ops::health`):
//! - `MichaelisMentenBatch` → `barracuda::ops::health::MichaelisMentenBatchGpu`
//! - `ScfaBatch` → `barracuda::ops::health::ScfaBatchGpu`
//! - `BeatClassifyBatch` → `barracuda::ops::health::BeatClassifyGpu`
//!
//! **Pending architectural** (local to healthSpring until next absorption):
//! - `GpuContext` fused pipeline → `barracuda::session::TensorSession`
//! - `strip_f64_enable()` WGSL preprocessor — **legacy path**; sovereign dispatch
//!   uses coralReef's native f64 lowering instead
//! - `shader_for_op()` mapping → barraCuda shader registry

mod cpu_fallback;
mod types;

pub use cpu_fallback::execute_cpu;
pub use types::{GpuError, GpuOp, GpuResult};

/// barraCuda rewire hooks bridging local GPU ops to upstream implementations.
#[cfg(feature = "barracuda-ops")]
pub mod barracuda_rewire;
/// Device, queues, and pipeline state for wgpu-backed execution.
#[cfg(feature = "gpu")]
pub mod context;
/// GPU dispatch entry points and buffer wiring for [`GpuOp`].
#[cfg(feature = "gpu")]
pub mod dispatch;
#[cfg(feature = "gpu")]
mod fused;
/// Sovereign f64 pipeline integration (coralReef-style lowering path).
#[cfg(feature = "gpu")]
pub mod sovereign;

/// ODE system definitions used by codegen shaders (e.g. Michaelis–Menten).
pub mod ode_systems;

/// Cached GPU availability probe — avoids Vulkan init races in parallel tests.
///
/// Absorbed from groundSpring V120 / neuralSpring V120 — `OnceLock` ensures
/// GPU probing happens exactly once, even under `cargo test -j N`.
#[cfg(feature = "gpu")]
static GPU_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Check whether a GPU adapter is available (cached after first probe).
///
/// Safe to call from any thread, any number of times. The first call
/// performs the actual wgpu adapter request; subsequent calls return
/// the cached result without touching the GPU driver.
#[cfg(feature = "gpu")]
#[must_use]
pub fn gpu_available() -> bool {
    *GPU_AVAILABLE.get_or_init(|| {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::default()
        });
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()
            .and_then(|rt| {
                rt.block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }))
                .ok()
            })
            .is_some()
    })
}

/// GPU availability when the `gpu` feature is disabled — always `false`.
#[cfg(not(feature = "gpu"))]
#[must_use]
pub const fn gpu_available() -> bool {
    false
}

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
    /// WGSL for vectorized Hill dose–response (`GpuOp::HillSweep`).
    pub const HILL_DOSE_RESPONSE: &str =
        include_str!("../../shaders/health/hill_dose_response_f64.wgsl");
    /// WGSL for parallel population PK AUC (`GpuOp::PopulationPkBatch`).
    pub const POPULATION_PK: &str = include_str!("../../shaders/health/population_pk_f64.wgsl");
    /// WGSL for batch Shannon/Simpson diversity (`GpuOp::DiversityBatch`).
    pub const DIVERSITY: &str = include_str!("../../shaders/health/diversity_f64.wgsl");
    /// WGSL for batched Michaelis–Menten PK integration (`GpuOp::MichaelisMentenBatch`).
    pub const MICHAELIS_MENTEN_BATCH: &str =
        include_str!("../../shaders/health/michaelis_menten_batch_f64.wgsl");
    /// WGSL for fiber-wise SCFA production (`GpuOp::ScfaBatch`).
    pub const SCFA_BATCH: &str = include_str!("../../shaders/health/scfa_batch_f64.wgsl");
    /// WGSL for template-based ECG beat classification (`GpuOp::BeatClassifyBatch`).
    pub const BEAT_CLASSIFY_BATCH: &str =
        include_str!("../../shaders/health/beat_classify_batch_f64.wgsl");
}

/// Returns the barraCuda codegen'd WGSL shader for ODE-based ops.
///
/// For `MichaelisMentenBatch`, this produces a generic RK4 shader via
/// `BatchedOdeRK4::<MichaelisMentenOde>::generate_shader()` — replacing
/// the handwritten Euler ODE in `michaelis_menten_batch_f64.wgsl`.
///
/// Returns `None` for ops without an `OdeSystem` implementation.
#[must_use]
pub fn codegen_shader_for_op(op: &GpuOp) -> Option<String> {
    use barracuda::numerical::BatchedOdeRK4;

    match op {
        GpuOp::MichaelisMentenBatch { .. } => {
            Some(BatchedOdeRK4::<ode_systems::MichaelisMentenOde>::generate_shader())
        }
        _ => None,
    }
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

/// wgpu-backed session wrapper (pipelines, buffers) when the `gpu` feature is on.
#[cfg(feature = "gpu")]
pub use context::GpuContext;
/// Runs [`GpuOp`] on the GPU with the same semantics as [`execute_cpu`].
#[cfg(feature = "gpu")]
pub use dispatch::execute_gpu;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

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
            assert!(
                results[0].abs() < tolerances::TEST_ASSERTION_TIGHT,
                "E(0) = 0"
            );
            assert!(
                (results[2] - 50.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
                "E(EC50) = Emax/2"
            );
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
    fn mm_batch_codegen_shader_valid() {
        let op = GpuOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 2000,
            n_patients: 64,
            seed: 42,
        };
        let shader = codegen_shader_for_op(&op);
        assert!(shader.is_some(), "MM batch should produce codegen shader");
        let wgsl = shader.unwrap_or_default();
        assert!(
            wgsl.contains("fn deriv"),
            "codegen shader must embed OdeSystem derivative"
        );
        assert!(
            wgsl.contains("vmax") || wgsl.contains("params"),
            "codegen shader should contain Michaelis-Menten parameter references"
        );
    }

    #[test]
    fn codegen_returns_none_for_non_ode_ops() {
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 1.0,
            concentrations: vec![1.0],
        };
        assert!(
            codegen_shader_for_op(&op).is_none(),
            "Hill sweep has no ODE codegen"
        );
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
