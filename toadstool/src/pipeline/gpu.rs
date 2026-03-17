// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU execution helpers for the pipeline.
//!
//! Converts GPU results to pipeline data format and provides fused batch dispatch support.

#[cfg(feature = "gpu")]
use healthspring_barracuda::gpu::GpuResult;

/// Convert a [`GpuResult`] to a flat `Vec<f64>` for pipeline data flow.
#[cfg(feature = "gpu")]
#[expect(
    clippy::tuple_array_conversions,
    reason = "destructured (shannon, simpson) to [f64; 2] is clearer than From"
)]
pub fn gpu_result_to_vec(result: &GpuResult) -> Vec<f64> {
    match result {
        GpuResult::HillSweep(v)
        | GpuResult::PopulationPkBatch(v)
        | GpuResult::MichaelisMentenBatch(v) => v.clone(),
        GpuResult::DiversityBatch(pairs) => pairs.iter().flat_map(|&(s, d)| [s, d]).collect(),
        GpuResult::ScfaBatch(triples) => triples.iter().flat_map(|&(a, p, b)| [a, p, b]).collect(),
        GpuResult::BeatClassifyBatch(pairs) => pairs
            .iter()
            .flat_map(|&(idx, corr)| [f64::from(idx), corr])
            .collect(),
    }
}
