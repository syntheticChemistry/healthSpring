// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU rewire — barraCuda upstream op delegation for Tier A + B.
//!
//! When `barracuda-ops` feature is active, all 6 GPU operations delegate to
//! barraCuda's canonical GPU implementations instead of local WGSL shaders:
//!
//! **Tier A** (absorbed, validated):
//! - `HillSweep` → `barracuda::ops::HillFunctionF64::dose_response()`
//! - `PopulationPkBatch` → `barracuda::ops::PopulationPkF64::new()`
//! - `DiversityBatch` → `barracuda::ops::bio::DiversityFusionGpu::new()`
//!
//! **Tier B** (absorbed upstream, rewired here):
//! - `MichaelisMentenBatch` → `barracuda::ops::health::MichaelisMentenBatchGpu`
//! - `ScfaBatch` → `barracuda::ops::health::ScfaBatchGpu`
//! - `BeatClassifyBatch` → `barracuda::ops::health::BeatClassifyGpu`
//!
//! This module is only compiled when `barracuda-ops` is enabled. CI runs
//! without GPU hardware, so the default feature set excludes this path.

use std::sync::Arc;

use barracuda::device::WgpuDevice;
use barracuda::ops::bio::diversity_fusion::DiversityFusionGpu;
use barracuda::ops::health::beat_classify::BeatClassifyGpu;
use barracuda::ops::health::michaelis_menten_batch::{MichaelisMentenBatchGpu, MmBatchConfig};
use barracuda::ops::health::scfa_batch::ScfaBatchGpu;
use barracuda::ops::hill_f64::HillFunctionF64;
use barracuda::ops::population_pk_f64::{PopulationPkConfig, PopulationPkF64};

use super::{GpuError, GpuResult};

/// Execute Hill dose-response via barraCuda upstream op.
///
/// # Errors
///
/// Returns [`GpuError::Dispatch`] if the GPU op creation or execution fails.
pub fn execute_hill_barracuda(
    device: &Arc<WgpuDevice>,
    emax: f64,
    ec50: f64,
    hill_n: f64,
    concentrations: &[f64],
) -> Result<GpuResult, GpuError> {
    let hill_op = HillFunctionF64::dose_response(Arc::clone(device), ec50, hill_n, emax)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda HillFunctionF64: {e}")))?;
    let results = hill_op
        .apply(concentrations)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda HillFunctionF64::apply: {e}")))?;
    Ok(GpuResult::HillSweep(results))
}

/// Execute population PK Monte Carlo via barraCuda upstream op.
///
/// # Errors
///
/// Returns [`GpuError::Dispatch`] if the GPU op creation or execution fails.
#[expect(
    clippy::cast_possible_truncation,
    reason = "n_patients and seed fit u32 for GPU dispatch"
)]
pub fn execute_pop_pk_barracuda(
    device: &Arc<WgpuDevice>,
    n_patients: usize,
    dose_mg: f64,
    f_bioavail: f64,
    seed: u64,
) -> Result<GpuResult, GpuError> {
    let config = PopulationPkConfig {
        dose_mg,
        f_bioavail,
        base_cl: 10.0,
        cl_low: 0.5,
        cl_high: 1.5,
    };
    let pk_op = PopulationPkF64::new(Arc::clone(device), config)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda PopulationPkF64: {e}")))?;
    let results = pk_op
        .simulate(n_patients as u32, seed as u32)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda PopulationPkF64::simulate: {e}")))?;
    Ok(GpuResult::PopulationPkBatch(results))
}

/// Execute diversity batch via barraCuda upstream op.
///
/// # Errors
///
/// Returns [`GpuError::Dispatch`] if the GPU op creation or execution fails.
pub fn execute_diversity_barracuda(
    device: &Arc<WgpuDevice>,
    communities: &[Vec<f64>],
) -> Result<GpuResult, GpuError> {
    let fusion = DiversityFusionGpu::new(Arc::clone(device))
        .map_err(|e| GpuError::Dispatch(format!("barraCuda DiversityFusionGpu: {e}")))?;

    let n_samples = communities.len();
    let n_species = communities.iter().map(Vec::len).max().unwrap_or(0);

    let mut flat = Vec::with_capacity(n_samples * n_species);
    for c in communities {
        flat.extend_from_slice(c);
        flat.resize(flat.len() + (n_species - c.len()), 0.0);
    }

    let div_results = fusion
        .compute(&flat, n_samples, n_species)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda DiversityFusionGpu::compute: {e}")))?;

    let results: Vec<(f64, f64)> = div_results.iter().map(|r| (r.shannon, r.simpson)).collect();
    Ok(GpuResult::DiversityBatch(results))
}

// ── Tier B rewire ──────────────────────────────────────────────────────

/// Execute Michaelis-Menten batch PK via barraCuda upstream op.
///
/// # Errors
///
/// Returns [`GpuError::Dispatch`] if the GPU op creation or execution fails.
pub fn execute_mm_batch_barracuda(
    device: &Arc<WgpuDevice>,
    vmax: f64,
    km: f64,
    vd: f64,
    dt: f64,
    n_steps: u32,
    n_patients: u32,
    seed: u32,
) -> Result<GpuResult, GpuError> {
    let config = MmBatchConfig {
        vmax,
        km,
        vd,
        dt,
        n_steps,
        n_patients,
        seed,
    };
    let op = MichaelisMentenBatchGpu::new(Arc::clone(device));
    let results = op
        .compute(&config)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda MichaelisMentenBatchGpu: {e}")))?;
    Ok(GpuResult::MichaelisMentenBatch(results))
}

/// Execute SCFA batch production via barraCuda upstream op.
///
/// # Errors
///
/// Returns [`GpuError::Dispatch`] if the GPU op creation or execution fails.
pub fn execute_scfa_batch_barracuda(
    device: &Arc<WgpuDevice>,
    params: &crate::microbiome::ScfaParams,
    fiber_inputs: &[f64],
) -> Result<GpuResult, GpuError> {
    let op = ScfaBatchGpu::new(Arc::clone(device));
    let flat = op
        .compute(fiber_inputs, params)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda ScfaBatchGpu: {e}")))?;
    let results: Vec<(f64, f64, f64)> = flat.chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect();
    Ok(GpuResult::ScfaBatch(results))
}

/// Execute beat classification batch via barraCuda upstream op.
///
/// # Errors
///
/// Returns [`GpuError::Dispatch`] if the GPU op creation or execution fails.
#[expect(
    clippy::cast_possible_truncation,
    reason = "beat/template counts fit u32 for GPU dispatch"
)]
pub fn execute_beat_classify_barracuda(
    device: &Arc<WgpuDevice>,
    beats: &[Vec<f64>],
    templates: &[Vec<f64>],
) -> Result<GpuResult, GpuError> {
    let n_beats = beats.len() as u32;
    let n_templates = templates.len() as u32;
    let window_size = beats.first().map_or(0, Vec::len) as u32;

    let flat_beats: Vec<f64> = beats.iter().flat_map(|b| b.iter().copied()).collect();
    let flat_templates: Vec<f64> = templates.iter().flat_map(|t| t.iter().copied()).collect();

    let op = BeatClassifyGpu::new(Arc::clone(device));
    let gpu_results = op
        .classify(
            &flat_beats,
            &flat_templates,
            n_beats,
            n_templates,
            window_size,
        )
        .map_err(|e| GpuError::Dispatch(format!("barraCuda BeatClassifyGpu: {e}")))?;

    let results: Vec<(u32, f64)> = gpu_results
        .iter()
        .map(|r| (r.template_index, r.correlation))
        .collect();
    Ok(GpuResult::BeatClassifyBatch(results))
}

/// Create a barraCuda `WgpuDevice` for Tier A ops.
///
/// # Errors
///
/// Returns [`GpuError::NoDevice`] if no adapter/device is available.
pub async fn create_barracuda_device() -> Result<Arc<WgpuDevice>, GpuError> {
    let device = WgpuDevice::new()
        .await
        .map_err(|e| GpuError::NoDevice(format!("barraCuda device init: {e}")))?;
    Ok(Arc::new(device))
}
