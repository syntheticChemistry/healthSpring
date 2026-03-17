// SPDX-License-Identifier: AGPL-3.0-or-later
//! CPU execution helpers for pipeline stages.
//!
//! Provides elemental operations: generate, transform, reduce, biosignal fusion,
//! AUC trapezoidal, and Bray-Curtis dissimilarity.

use super::{ReduceKind, Stage, StageResult, TransformKind};

#[expect(
    clippy::cast_precision_loss,
    reason = "elapsed microseconds fits f64 for timing"
)]
pub(super) fn failed_stage_result(stage: &Stage, start: &std::time::Instant) -> StageResult {
    StageResult {
        stage_name: stage.name.clone(),
        substrate: stage.substrate,
        output_data: vec![],
        elapsed_us: start.elapsed().as_micros() as f64,
        success: false,
    }
}

#[expect(clippy::cast_precision_loss, reason = "counter/PRNG fits f64")]
pub(super) fn generate_data(n: usize, seed: u64) -> Vec<f64> {
    let mut data = Vec::with_capacity(n);
    let mut state = seed;
    for _ in 0..n {
        state = healthspring_barracuda::rng::lcg_step(state);
        let val = (state >> 33) as f64 / f64::from(u32::MAX);
        data.push(val);
    }
    data
}

pub(super) fn apply_transform(data: &[f64], kind: TransformKind) -> Vec<f64> {
    match kind {
        TransformKind::Hill { emax, ec50, n } => {
            healthspring_barracuda::pkpd::hill_sweep(ec50, n, emax, data)
        }
        TransformKind::Square => data.iter().map(|&x| x * x).collect(),
        TransformKind::ExpDecay { k, t } => data.iter().map(|&x| x * (-k * t).exp()).collect(),
    }
}

#[expect(clippy::cast_precision_loss, reason = "element count fits f64")]
pub(super) fn apply_reduce(data: &[f64], kind: ReduceKind) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    match kind {
        ReduceKind::Sum => data.iter().sum(),
        ReduceKind::Mean => data.iter().sum::<f64>() / data.len() as f64,
        ReduceKind::Max => data.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ReduceKind::Min => data.iter().copied().fold(f64::INFINITY, f64::min),
        ReduceKind::Variance => {
            let mean = data.iter().sum::<f64>() / data.len() as f64;
            data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
        }
    }
}

/// Multi-channel biosignal fusion: averages interleaved channel data into
/// a single fused signal, then appends per-channel energy ratios.
#[expect(clippy::cast_precision_loss, reason = "element count fits f64")]
pub(super) fn fuse_biosignal_channels(data: &[f64], n_channels: usize) -> Vec<f64> {
    if n_channels == 0 || data.is_empty() {
        return vec![];
    }
    let samples_per_ch = data.len() / n_channels;
    if samples_per_ch == 0 {
        return vec![];
    }
    let mut fused = vec![0.0; samples_per_ch];
    for (i, val) in data.iter().enumerate().take(samples_per_ch * n_channels) {
        fused[i % samples_per_ch] += val / n_channels as f64;
    }
    let total_energy: f64 = fused.iter().map(|&x| x * x).sum();
    for ch in 0..n_channels {
        let ch_energy: f64 = (0..samples_per_ch)
            .map(|s| {
                let idx = ch * samples_per_ch + s;
                if idx < data.len() {
                    data[idx] * data[idx]
                } else {
                    0.0
                }
            })
            .sum();
        let ratio = if total_energy > 0.0 {
            ch_energy / total_energy
        } else {
            0.0
        };
        fused.push(ratio);
    }
    fused
}

/// AUC trapezoidal: treats input as concentration values equally spaced
/// over `[0, t_max]` and returns the area under the curve.
#[expect(clippy::cast_precision_loss, reason = "time point count fits f64")]
pub(super) fn compute_auc_trapezoidal(concs: &[f64], t_max: f64) -> f64 {
    if concs.len() < 2 {
        return 0.0;
    }
    let dt = t_max / (concs.len() - 1) as f64;
    let times: Vec<f64> = (0..concs.len()).map(|i| dt * i as f64).collect();
    healthspring_barracuda::pkpd::auc_trapezoidal(&times, concs)
}

/// Bray-Curtis pairwise dissimilarity: returns the upper triangle of the
/// dissimilarity matrix as a flat vector.
pub(super) fn compute_bray_curtis_matrix(communities: &[Vec<f64>]) -> Vec<f64> {
    let n = communities.len();
    let mut result = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            result.push(healthspring_barracuda::microbiome::bray_curtis(
                &communities[i],
                &communities[j],
            ));
        }
    }
    result
}
