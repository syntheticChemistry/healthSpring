// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pipeline stage: a single compute operation within a pipeline.

mod exec;
#[cfg(test)]
mod tests;

use healthspring_barracuda::gpu::GpuOp;
use healthspring_forge::Substrate;

/// A single compute stage within a pipeline.
#[derive(Debug)]
pub struct Stage {
    /// Label surfaced in `StageResult` and progress callbacks.
    pub name: String,
    /// Intended substrate hint; GPU paths may override via metalForge routing.
    pub substrate: Substrate,
    /// Domain operation executed on CPU and/or mapped to barraCuda GPU kernels.
    pub operation: StageOp,
}

/// The operation a stage performs.
#[derive(Debug, Clone)]
pub enum StageOp {
    /// Generate input data (source stage).
    Generate {
        /// Number of synthetic samples to emit in `[0, 1]`.
        n_elements: usize,
        /// LCG seed for reproducible pipelines.
        seed: u64,
    },
    /// Population PK Monte Carlo: generate AUC per patient (GPU-native).
    PopulationPk {
        /// Parallel PK draws (virtual patients).
        n_patients: usize,
        /// Oral dose used in the one-compartment simulation.
        dose_mg: f64,
        /// Fractional bioavailability applied to the dose.
        f_bioavail: f64,
        /// RNG seed for the population batch.
        seed: u64,
    },
    /// Element-wise transform: apply f(x) to each element.
    ElementwiseTransform {
        /// Hill, square, or exponential decay mapping on upstream samples.
        kind: TransformKind,
    },
    /// Reduce: aggregate elements to a scalar or smaller array.
    Reduce {
        /// Sum, mean, variance, or extrema over the input vector.
        kind: ReduceKind,
    },
    /// Batch diversity indices over communities (GPU-native via `DiversityBatch`).
    DiversityReduce {
        /// Abundance vectors per community (simplex rows) for Shannon/Simpson.
        communities: Vec<Vec<f64>>,
    },
    /// Filter: keep elements matching a predicate.
    Filter {
        /// Strict lower bound; values above this are kept.
        threshold: f64,
    },
    /// Multi-channel biosignal fusion (ECG+PPG+EDA). CPU path, NPU-ready.
    BiosignalFusion {
        /// Count of interleaved channels in the flat input buffer.
        n_channels: usize,
    },
    /// AUC trapezoidal: integrate concentration-time curve to a scalar.
    AucTrapezoidal {
        /// End time for equally spaced samples in `[0, t_max]`.
        t_max: f64,
    },
    /// Bray-Curtis pairwise dissimilarity matrix over communities.
    BrayCurtis {
        /// Abundance profiles compared pairwise (upper triangle flattened).
        communities: Vec<Vec<f64>>,
    },
    /// Batch Michaelis-Menten PK: parallel ODE per patient (GPU-native).
    MichaelisMentenBatch {
        /// Michaelis–Menten `Vmax` (amount per time).
        vmax: f64,
        /// Michaelis constant `Km`.
        km: f64,
        /// Apparent volume of distribution scaling concentrations.
        vd: f64,
        /// Fixed integration step for the explicit ODE solver.
        dt: f64,
        /// Steps per trajectory after the initial condition.
        n_steps: u32,
        /// Independent patient trajectories in the batch.
        n_patients: u32,
        /// RNG seed for initial conditions or stochastic terms.
        seed: u32,
    },
    /// Batch SCFA production: element-wise Michaelis-Menten (GPU-native).
    ScfaBatch {
        /// Shared microbiome parameters for all fiber rows.
        params: healthspring_barracuda::microbiome::ScfaParams,
        /// Fiber substrate levels driving production rates.
        fiber_inputs: Vec<f64>,
    },
    /// Batch beat classification: template correlation (GPU-native).
    BeatClassifyBatch {
        /// Windowed beats aligned to template length.
        beats: Vec<Vec<f64>>,
        /// Reference morphologies scored against each beat.
        templates: Vec<Vec<f64>>,
    },
}

/// Kind of element-wise transform.
#[derive(Debug, Clone, Copy)]
pub enum TransformKind {
    /// Hill dose-response: E = Emax * c^n / (EC50^n + c^n)
    Hill {
        /// Asymptotic effect at high concentration.
        emax: f64,
        /// Half-maximal concentration (potency).
        ec50: f64,
        /// Hill slope (sigmoid steepness).
        n: f64,
    },
    /// Squaring: y = x^2
    Square,
    /// Exponential decay: y = x * exp(-k * t)
    ExpDecay {
        /// Decay rate constant multiplying time.
        k: f64,
        /// Time horizon for the decay factor.
        t: f64,
    },
}

/// Kind of reduction.
#[derive(Debug, Clone, Copy)]
pub enum ReduceKind {
    /// Total mass or count across samples.
    Sum,
    /// First moment of the input vector.
    Mean,
    /// Largest sample (or `-∞` on empty input in current CPU path).
    Max,
    /// Smallest sample (or `∞` on empty input in current CPU path).
    Min,
    /// Population variance about the sample mean.
    Variance,
}

/// Result of executing a single stage.
#[derive(Debug, Clone)]
pub struct StageResult {
    /// Copy of `Stage::name` for correlation without holding `Stage`.
    pub stage_name: String,
    /// Substrate that actually ran (CPU vs GPU in GPU pipeline paths).
    pub substrate: Substrate,
    /// Flattened output buffer passed to the next stage or sink.
    pub output_data: Vec<f64>,
    /// CPU timing or GPU batch share in microseconds.
    pub elapsed_us: f64,
    /// Whether execution completed without GPU/CPU mapping failure.
    pub success: bool,
}

impl Stage {
    /// Convert this stage into a [`GpuOp`] if the operation is GPU-mappable.
    ///
    /// Stages that require upstream data (transforms, reductions) need `input`
    /// to construct the `GpuOp`. Generator stages are self-contained.
    /// Returns `None` for operations that have no GPU kernel (e.g. filter).
    #[must_use]
    pub fn to_gpu_op(&self, input: Option<&[f64]>) -> Option<GpuOp> {
        match &self.operation {
            StageOp::ElementwiseTransform {
                kind: TransformKind::Hill { emax, ec50, n },
            } => {
                let concentrations = input.unwrap_or(&[]).to_vec();
                Some(GpuOp::HillSweep {
                    emax: *emax,
                    ec50: *ec50,
                    n: *n,
                    concentrations,
                })
            }
            StageOp::PopulationPk {
                n_patients,
                dose_mg,
                f_bioavail,
                seed,
            } => Some(GpuOp::PopulationPkBatch {
                n_patients: *n_patients,
                dose_mg: *dose_mg,
                f_bioavail: *f_bioavail,
                seed: *seed,
            }),
            StageOp::DiversityReduce { communities } => Some(GpuOp::DiversityBatch {
                communities: communities.clone(),
            }),
            StageOp::MichaelisMentenBatch {
                vmax,
                km,
                vd,
                dt,
                n_steps,
                n_patients,
                seed,
            } => Some(GpuOp::MichaelisMentenBatch {
                vmax: *vmax,
                km: *km,
                vd: *vd,
                dt: *dt,
                n_steps: *n_steps,
                n_patients: *n_patients,
                seed: *seed,
            }),
            StageOp::ScfaBatch {
                params,
                fiber_inputs,
            } => Some(GpuOp::ScfaBatch {
                params: params.clone(),
                fiber_inputs: fiber_inputs.clone(),
            }),
            StageOp::BeatClassifyBatch { beats, templates } => Some(GpuOp::BeatClassifyBatch {
                beats: beats.clone(),
                templates: templates.clone(),
            }),
            StageOp::Generate { .. }
            | StageOp::ElementwiseTransform { .. }
            | StageOp::Reduce { .. }
            | StageOp::Filter { .. }
            | StageOp::BiosignalFusion { .. }
            | StageOp::AucTrapezoidal { .. }
            | StageOp::BrayCurtis { .. } => None,
        }
    }

    /// Execute this stage on CPU.
    ///
    /// Returns a failed [`StageResult`] (empty output, `success: false`) if a
    /// GPU-native stage cannot be mapped to a `GpuOp`; this cannot happen for
    /// valid `StageOp` variants.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "elapsed microseconds fits f64")]
    #[expect(
        clippy::tuple_array_conversions,
        reason = "destructured (shannon, simpson) to [f64; 2] is clearer than From"
    )]
    pub fn execute(&self, input: Option<&[f64]>) -> StageResult {
        use exec::{
            apply_reduce, apply_transform, compute_auc_trapezoidal, compute_bray_curtis_matrix,
            failed_stage_result, fuse_biosignal_channels, generate_data,
        };
        use healthspring_barracuda::gpu::{GpuResult, execute_cpu as gpu_cpu};

        let start = std::time::Instant::now();
        let output = match &self.operation {
            StageOp::Generate { n_elements, seed } => generate_data(*n_elements, *seed),
            StageOp::PopulationPk {
                n_patients,
                dose_mg,
                f_bioavail,
                seed,
            } => {
                let op = GpuOp::PopulationPkBatch {
                    n_patients: *n_patients,
                    dose_mg: *dose_mg,
                    f_bioavail: *f_bioavail,
                    seed: *seed,
                };
                match gpu_cpu(&op) {
                    GpuResult::PopulationPkBatch(v) => v,
                    _ => return failed_stage_result(self, &start),
                }
            }
            StageOp::ElementwiseTransform { kind } => {
                let data = input.unwrap_or(&[]);
                apply_transform(data, *kind)
            }
            StageOp::Reduce { kind } => {
                let data = input.unwrap_or(&[]);
                vec![apply_reduce(data, *kind)]
            }
            StageOp::DiversityReduce { communities } => {
                let op = GpuOp::DiversityBatch {
                    communities: communities.clone(),
                };
                match gpu_cpu(&op) {
                    GpuResult::DiversityBatch(pairs) => {
                        pairs.iter().flat_map(|&(s, d)| [s, d]).collect()
                    }
                    _ => return failed_stage_result(self, &start),
                }
            }
            StageOp::Filter { threshold } => {
                let data = input.unwrap_or(&[]);
                data.iter().copied().filter(|&x| x > *threshold).collect()
            }
            StageOp::BiosignalFusion { n_channels } => {
                let data = input.unwrap_or(&[]);
                fuse_biosignal_channels(data, *n_channels)
            }
            StageOp::AucTrapezoidal { t_max } => {
                let data = input.unwrap_or(&[]);
                vec![compute_auc_trapezoidal(data, *t_max)]
            }
            StageOp::BrayCurtis { communities } => compute_bray_curtis_matrix(communities),
            StageOp::MichaelisMentenBatch { .. } => {
                let Some(op) = self.to_gpu_op(input) else {
                    return failed_stage_result(self, &start);
                };
                match gpu_cpu(&op) {
                    GpuResult::MichaelisMentenBatch(v) => v,
                    _ => return failed_stage_result(self, &start),
                }
            }
            StageOp::ScfaBatch { .. } => {
                let Some(op) = self.to_gpu_op(input) else {
                    return failed_stage_result(self, &start);
                };
                match gpu_cpu(&op) {
                    GpuResult::ScfaBatch(triples) => {
                        triples.iter().flat_map(|&(a, p, b)| [a, p, b]).collect()
                    }
                    _ => return failed_stage_result(self, &start),
                }
            }
            StageOp::BeatClassifyBatch { .. } => {
                let Some(op) = self.to_gpu_op(input) else {
                    return failed_stage_result(self, &start);
                };
                match gpu_cpu(&op) {
                    GpuResult::BeatClassifyBatch(pairs) => pairs
                        .iter()
                        .flat_map(|&(idx, corr)| [f64::from(idx), corr])
                        .collect(),
                    _ => return failed_stage_result(self, &start),
                }
            }
        };
        let elapsed = start.elapsed().as_micros() as f64;

        StageResult {
            stage_name: self.name.clone(),
            substrate: self.substrate,
            output_data: output,
            elapsed_us: elapsed,
            success: true,
        }
    }
}
