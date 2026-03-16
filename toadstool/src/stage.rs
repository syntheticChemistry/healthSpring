// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pipeline stage: a single compute operation within a pipeline.

use healthspring_barracuda::gpu::GpuOp;
use healthspring_forge::Substrate;

/// A single compute stage within a pipeline.
#[derive(Debug)]
pub struct Stage {
    pub name: String,
    pub substrate: Substrate,
    pub operation: StageOp,
}

/// The operation a stage performs.
#[derive(Debug, Clone)]
pub enum StageOp {
    /// Generate input data (source stage).
    Generate { n_elements: usize, seed: u64 },
    /// Population PK Monte Carlo: generate AUC per patient (GPU-native).
    PopulationPk {
        n_patients: usize,
        dose_mg: f64,
        f_bioavail: f64,
        seed: u64,
    },
    /// Element-wise transform: apply f(x) to each element.
    ElementwiseTransform { kind: TransformKind },
    /// Reduce: aggregate elements to a scalar or smaller array.
    Reduce { kind: ReduceKind },
    /// Batch diversity indices over communities (GPU-native via `DiversityBatch`).
    DiversityReduce { communities: Vec<Vec<f64>> },
    /// Filter: keep elements matching a predicate.
    Filter { threshold: f64 },
    /// Multi-channel biosignal fusion (ECG+PPG+EDA). CPU path, NPU-ready.
    BiosignalFusion { n_channels: usize },
    /// AUC trapezoidal: integrate concentration-time curve to a scalar.
    AucTrapezoidal { t_max: f64 },
    /// Bray-Curtis pairwise dissimilarity matrix over communities.
    BrayCurtis { communities: Vec<Vec<f64>> },
    /// Batch Michaelis-Menten PK: parallel ODE per patient (GPU-native).
    MichaelisMentenBatch {
        vmax: f64,
        km: f64,
        vd: f64,
        dt: f64,
        n_steps: u32,
        n_patients: u32,
        seed: u32,
    },
    /// Batch SCFA production: element-wise Michaelis-Menten (GPU-native).
    ScfaBatch {
        params: healthspring_barracuda::microbiome::ScfaParams,
        fiber_inputs: Vec<f64>,
    },
    /// Batch beat classification: template correlation (GPU-native).
    BeatClassifyBatch {
        beats: Vec<Vec<f64>>,
        templates: Vec<Vec<f64>>,
    },
}

/// Kind of element-wise transform.
#[derive(Debug, Clone, Copy)]
pub enum TransformKind {
    /// Hill dose-response: E = Emax * c^n / (EC50^n + c^n)
    Hill { emax: f64, ec50: f64, n: f64 },
    /// Squaring: y = x^2
    Square,
    /// Exponential decay: y = x * exp(-k * t)
    ExpDecay { k: f64, t: f64 },
}

/// Kind of reduction.
#[derive(Debug, Clone, Copy)]
pub enum ReduceKind {
    Sum,
    Mean,
    Max,
    Min,
    Variance,
}

/// Result of executing a single stage.
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_name: String,
    pub substrate: Substrate,
    pub output_data: Vec<f64>,
    pub elapsed_us: f64,
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
    #[expect(clippy::cast_precision_loss)]
    #[expect(
        clippy::tuple_array_conversions,
        reason = "destructured (shannon, simpson) to [f64; 2] is clearer than From"
    )]
    pub fn execute(&self, input: Option<&[f64]>) -> StageResult {
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

#[expect(
    clippy::cast_precision_loss,
    reason = "elapsed microseconds fits f64 for timing"
)]
fn failed_stage_result(stage: &Stage, start: &std::time::Instant) -> StageResult {
    StageResult {
        stage_name: stage.name.clone(),
        substrate: stage.substrate,
        output_data: vec![],
        elapsed_us: start.elapsed().as_micros() as f64,
        success: false,
    }
}

#[expect(clippy::cast_precision_loss)]
fn generate_data(n: usize, seed: u64) -> Vec<f64> {
    let mut data = Vec::with_capacity(n);
    let mut state = seed;
    for _ in 0..n {
        state = healthspring_barracuda::rng::lcg_step(state);
        let val = (state >> 33) as f64 / f64::from(u32::MAX);
        data.push(val);
    }
    data
}

fn apply_transform(data: &[f64], kind: TransformKind) -> Vec<f64> {
    match kind {
        TransformKind::Hill { emax, ec50, n } => {
            healthspring_barracuda::pkpd::hill_sweep(ec50, n, emax, data)
        }
        TransformKind::Square => data.iter().map(|&x| x * x).collect(),
        TransformKind::ExpDecay { k, t } => data.iter().map(|&x| x * (-k * t).exp()).collect(),
    }
}

#[expect(clippy::cast_precision_loss)]
fn apply_reduce(data: &[f64], kind: ReduceKind) -> f64 {
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
#[expect(clippy::cast_precision_loss)]
fn fuse_biosignal_channels(data: &[f64], n_channels: usize) -> Vec<f64> {
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
fn compute_auc_trapezoidal(concs: &[f64], t_max: f64) -> f64 {
    if concs.len() < 2 {
        return 0.0;
    }
    let dt = t_max / (concs.len() - 1) as f64;
    let times: Vec<f64> = (0..concs.len()).map(|i| dt * i as f64).collect();
    healthspring_barracuda::pkpd::auc_trapezoidal(&times, concs)
}

/// Bray-Curtis pairwise dissimilarity: returns the upper triangle of the
/// dissimilarity matrix as a flat vector.
fn compute_bray_curtis_matrix(communities: &[Vec<f64>]) -> Vec<f64> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use healthspring_forge::Substrate;

    #[test]
    fn generate_produces_deterministic_data() {
        let stage = Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 5,
                seed: 123,
            },
        };
        let r1 = stage.execute(None);
        let r2 = stage.execute(None);
        assert_eq!(r1.output_data, r2.output_data);
        assert_eq!(r1.output_data.len(), 5);
    }

    #[test]
    fn generate_values_in_unit_interval() {
        let stage = Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 100,
                seed: 0,
            },
        };
        let r = stage.execute(None);
        for &v in &r.output_data {
            assert!((0.0..=1.0).contains(&v), "value {v} outside [0,1]");
        }
    }

    #[test]
    fn transform_square() {
        let stage = Stage {
            name: "sq".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Square,
            },
        };
        let input = [1.0, 2.0, 3.0];
        let r = stage.execute(Some(&input));
        assert_eq!(r.output_data, [1.0, 4.0, 9.0]);
    }

    #[test]
    fn transform_hill_at_zero_is_zero() {
        let stage = Stage {
            name: "hill".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Hill {
                    emax: 1.0,
                    ec50: 0.5,
                    n: 2.0,
                },
            },
        };
        let input = [0.0];
        let r = stage.execute(Some(&input));
        assert!(r.output_data[0].abs() < f64::EPSILON);
    }

    #[test]
    fn reduce_sum() {
        let stage = Stage {
            name: "sum".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Sum,
            },
        };
        let input = [1.0, 2.0, 3.0];
        let r = stage.execute(Some(&input));
        assert_eq!(r.output_data.len(), 1);
        assert!((r.output_data[0] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn reduce_mean() {
        let stage = Stage {
            name: "mean".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Mean,
            },
        };
        let input = [2.0, 4.0, 6.0];
        let r = stage.execute(Some(&input));
        assert_eq!(r.output_data.len(), 1);
        assert!((r.output_data[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn reduce_empty_returns_zero() {
        let stage = Stage {
            name: "sum".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Sum,
            },
        };
        let r = stage.execute(Some(&[]));
        assert_eq!(r.output_data, [0.0]);
    }

    #[test]
    fn filter_keeps_above_threshold() {
        let stage = Stage {
            name: "filter".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Filter { threshold: 0.5 },
        };
        let input = [0.3, 0.6, 0.4, 0.8];
        let r = stage.execute(Some(&input));
        assert_eq!(r.output_data, [0.6, 0.8]);
    }

    #[test]
    fn population_pk_produces_positive_aucs() {
        let stage = Stage {
            name: "pop_pk".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::PopulationPk {
                n_patients: 50,
                dose_mg: 4.0,
                f_bioavail: 0.79,
                seed: 42,
            },
        };
        let r = stage.execute(None);
        assert_eq!(r.output_data.len(), 50);
        assert!(r.output_data.iter().all(|&a| a > 0.0));
    }

    #[test]
    fn population_pk_maps_to_gpu_op() {
        let stage = Stage {
            name: "pop_pk".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::PopulationPk {
                n_patients: 100,
                dose_mg: 4.0,
                f_bioavail: 0.79,
                seed: 7,
            },
        };
        let op = stage.to_gpu_op(None);
        let Some(op) = op else {
            panic!("to_gpu_op returns Some for PopulationPk");
        };
        assert!(matches!(op, GpuOp::PopulationPkBatch { .. }));
    }

    #[test]
    fn diversity_reduce_computes_shannon_simpson() {
        let stage = Stage {
            name: "diversity".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::DiversityReduce {
                communities: vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]],
            },
        };
        let r = stage.execute(None);
        assert_eq!(r.output_data.len(), 4);
        let (even_shannon, even_simpson) = (r.output_data[0], r.output_data[1]);
        let (dom_shannon, _dom_simpson) = (r.output_data[2], r.output_data[3]);
        assert!(even_shannon > dom_shannon, "even > dominated");
        assert!(even_simpson > 0.0);
    }

    #[test]
    fn diversity_reduce_maps_to_gpu_op() {
        let stage = Stage {
            name: "diversity".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::DiversityReduce {
                communities: vec![vec![0.5, 0.5]],
            },
        };
        let op = stage.to_gpu_op(None);
        let Some(op) = op else {
            panic!("to_gpu_op returns Some for DiversityReduce");
        };
        assert!(matches!(op, GpuOp::DiversityBatch { .. }));
    }
}
