// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tier A GPU rewire — barraCuda upstream op delegation.
//!
//! When `barracuda-ops` feature is active, the 3 Tier A operations delegate to
//! barraCuda's canonical GPU implementations instead of local WGSL shaders:
//!
//! - `HillSweep` → `barracuda::ops::HillFunctionF64::dose_response()`
//! - `PopulationPkBatch` → `barracuda::ops::PopulationPkF64::new()`
//! - `DiversityBatch` → `barracuda::ops::bio::DiversityFusionGpu::new()`
//!
//! Tier B ops (MM, SCFA, BeatClassify) remain on local shaders until barraCuda
//! absorbs them.
//!
//! This module is only compiled when `barracuda-ops` is enabled. CI runs
//! without GPU hardware, so the default feature set excludes this path.

use std::sync::Arc;

use barracuda::device::WgpuDevice;
use barracuda::ops::hill_f64::HillFunctionF64;
use barracuda::ops::population_pk_f64::{PopulationPkConfig, PopulationPkF64};
use barracuda::ops::bio::diversity_fusion::DiversityFusionGpu;

use super::{GpuError, GpuResult};

/// Execute Hill dose-response via barraCuda upstream op.
pub async fn execute_hill_barracuda(
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
pub async fn execute_pop_pk_barracuda(
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
        .simulate(n_patients, seed)
        .map_err(|e| GpuError::Dispatch(format!("barraCuda PopulationPkF64::simulate: {e}")))?;
    Ok(GpuResult::PopulationPkBatch(results))
}

/// Execute diversity batch via barraCuda upstream op.
#[expect(clippy::cast_possible_truncation, reason = "community sizes fit u32")]
pub async fn execute_diversity_barracuda(
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

    let results: Vec<(f64, f64)> = div_results
        .iter()
        .map(|r| (r.shannon, r.simpson))
        .collect();
    Ok(GpuResult::DiversityBatch(results))
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
