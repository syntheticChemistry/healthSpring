// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compute pipeline: a sequence of stages that execute on heterogeneous hardware.
//!
//! ## ABSORPTION STATUS (toadStool)
//!
//! - `Pipeline::execute_auto()` with metalForge routing -> toadStool core pipeline
//! - `stage_to_workload()` mapping -> toadStool workload classification
//! - `Pipeline::execute_gpu()` fused batch dispatch -> toadStool GPU executor

use crate::stage::{Stage, StageResult};
#[cfg(feature = "gpu")]
use healthspring_barracuda::gpu::{GpuContext, GpuResult};
#[cfg(feature = "gpu")]
use healthspring_forge::Substrate;

/// A compute pipeline is an ordered sequence of stages.
///
/// Each stage executes on a substrate (CPU/GPU/NPU) determined by metalForge.
/// Data flows forward through stages; no stage reads output of a later stage.
#[derive(Debug)]
pub struct Pipeline {
    pub name: String,
    stages: Vec<Stage>,
}

/// Result of executing a pipeline.
#[derive(Debug)]
pub struct PipelineResult {
    pub stage_results: Vec<StageResult>,
    pub total_time_us: f64,
    pub success: bool,
}

impl Pipeline {
    /// Create a new empty pipeline.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stages: Vec::new(),
        }
    }

    /// Add a stage to the pipeline.
    pub fn add_stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }

    /// Number of stages in the pipeline.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.stages.len()
    }

    /// Whether the pipeline has no stages.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Execute the pipeline on CPU (reference implementation).
    ///
    /// Each stage runs sequentially. Output of stage N is input to stage N+1.
    #[must_use]
    pub fn execute_cpu(&self) -> PipelineResult {
        let mut results = Vec::with_capacity(self.stages.len());
        let mut total_time = 0.0;
        let mut all_success = true;
        let mut input_data: Option<Vec<f64>> = None;

        for stage in &self.stages {
            let result = stage.execute(input_data.as_deref());
            total_time += result.elapsed_us;
            all_success &= result.success;
            input_data = Some(result.output_data.clone());
            results.push(result);
        }

        PipelineResult {
            stage_results: results,
            total_time_us: total_time,
            success: all_success,
        }
    }

    /// Execute the pipeline on CPU with per-stage streaming callbacks.
    ///
    /// After each stage completes, the `on_stage` callback is invoked with
    /// the stage index, total stage count, and the `StageResult`. This
    /// enables integration with `petalTongue` `StreamSession` for live
    /// progress reporting.
    #[must_use]
    pub fn execute_streaming<F>(&self, mut on_stage: F) -> PipelineResult
    where
        F: FnMut(usize, usize, &StageResult),
    {
        let n_stages = self.stages.len();
        let mut results = Vec::with_capacity(n_stages);
        let mut total_time = 0.0;
        let mut all_success = true;
        let mut input_data: Option<Vec<f64>> = None;

        for (i, stage) in self.stages.iter().enumerate() {
            let result = stage.execute(input_data.as_deref());
            total_time += result.elapsed_us;
            all_success &= result.success;
            on_stage(i, n_stages, &result);
            input_data = Some(result.output_data.clone());
            results.push(result);
        }

        PipelineResult {
            stage_results: results,
            total_time_us: total_time,
            success: all_success,
        }
    }

    /// Execute the pipeline on GPU via a fused single-submission dispatch.
    ///
    /// Stages that map to GPU ops are batched and dispatched in a single
    /// command encoder through [`GpuContext::execute_fused`]. Stages without
    /// a GPU kernel fall back to CPU automatically.
    ///
    /// This is the unidirectional pipeline: data flows CPU → GPU → CPU with
    /// no round-trips between stages.
    #[cfg(feature = "gpu")]
    #[expect(clippy::cast_precision_loss, reason = "nanosecond timing fits f64")]
    pub async fn execute_gpu(&self, ctx: &GpuContext) -> PipelineResult {
        let mut results = Vec::with_capacity(self.stages.len());
        let mut total_time = 0.0;
        let mut all_success = true;
        let mut input_data: Option<Vec<f64>> = None;

        // Scan forward: collect contiguous GPU-mappable stages
        let mut i = 0;
        while i < self.stages.len() {
            let stage = &self.stages[i];
            if let Some(gpu_op) = stage.to_gpu_op(input_data.as_deref()) {
                // Collect a batch of consecutive GPU-mappable stages
                let mut batch_ops = vec![gpu_op];
                let mut batch_indices = vec![i];
                let mut j = i + 1;
                // Look ahead: further stages that can also be GPU-dispatched
                // with independent inputs are batched together.
                while j < self.stages.len() {
                    // Each GPU op is self-contained (no inter-stage dependency
                    // within the GPU batch); dependent stages go CPU.
                    if let Some(op_j) = self.stages[j].to_gpu_op(None) {
                        batch_ops.push(op_j);
                        batch_indices.push(j);
                        j += 1;
                    } else {
                        break;
                    }
                }

                let start = std::time::Instant::now();
                match ctx.execute_fused(&batch_ops).await {
                    Ok(gpu_results) => {
                        let elapsed = start.elapsed();
                        #[expect(
                            clippy::cast_precision_loss,
                            reason = "nanosecond timing fits f64"
                        )]
                        let elapsed_us = elapsed.as_nanos() as f64 / 1000.0;
                        let per_stage = elapsed_us / batch_indices.len().max(1) as f64;

                        for (idx, gpu_result) in batch_indices.iter().zip(gpu_results.into_iter()) {
                            let output = gpu_result_to_vec(&gpu_result);
                            total_time += per_stage;
                            input_data = Some(output.clone());
                            results.push(StageResult {
                                stage_name: self.stages[*idx].name.clone(),
                                substrate: Substrate::Gpu,
                                output_data: output,
                                elapsed_us: per_stage,
                                success: true,
                            });
                        }
                    }
                    Err(_) => {
                        // GPU dispatch failed — fall back to CPU for this batch
                        for &idx in &batch_indices {
                            let result = self.stages[idx].execute(input_data.as_deref());
                            total_time += result.elapsed_us;
                            all_success &= result.success;
                            input_data = Some(result.output_data.clone());
                            results.push(result);
                        }
                    }
                }
                i = j;
            } else {
                // CPU fallback for non-GPU-mappable stages
                let result = stage.execute(input_data.as_deref());
                total_time += result.elapsed_us;
                all_success &= result.success;
                input_data = Some(result.output_data.clone());
                results.push(result);
                i += 1;
            }
        }

        PipelineResult {
            stage_results: results,
            total_time_us: total_time,
            success: all_success,
        }
    }

    /// Execute with automatic substrate selection per stage.
    ///
    /// Uses metalForge routing: GPU-capable stages with sufficient element
    /// counts go to GPU; everything else runs on CPU.
    #[cfg(feature = "gpu")]
    #[expect(clippy::cast_precision_loss, reason = "nanosecond timing fits f64")]
    pub async fn execute_auto(
        &self,
        ctx: &GpuContext,
        caps: &healthspring_forge::Capabilities,
    ) -> PipelineResult {
        let mut results = Vec::with_capacity(self.stages.len());
        let mut total_time = 0.0;
        let mut all_success = true;
        let mut input_data: Option<Vec<f64>> = None;

        // Collect GPU-eligible stages based on metalForge routing
        let mut gpu_ops = Vec::new();
        let mut gpu_stage_indices = Vec::new();

        for (i, stage) in self.stages.iter().enumerate() {
            let workload = stage_to_workload(stage, input_data.as_deref());
            let substrate = healthspring_forge::select_substrate(&workload, caps);

            if substrate == Substrate::Gpu {
                if let Some(op) = stage.to_gpu_op(input_data.as_deref()) {
                    gpu_ops.push(op);
                    gpu_stage_indices.push(i);
                    continue;
                }
            }
            // Run CPU stages that precede GPU batch immediately
            let result = stage.execute(input_data.as_deref());
            total_time += result.elapsed_us;
            all_success &= result.success;
            input_data = Some(result.output_data.clone());
            results.push(result);
        }

        // Dispatch GPU batch
        if !gpu_ops.is_empty() {
            let start = std::time::Instant::now();
            match ctx.execute_fused(&gpu_ops).await {
                Ok(gpu_results) => {
                    let elapsed = start.elapsed();
                    #[expect(clippy::cast_precision_loss, reason = "nanosecond timing fits f64")]
                    let elapsed_us = elapsed.as_nanos() as f64 / 1000.0;
                    let per_stage = elapsed_us / gpu_stage_indices.len().max(1) as f64;

                    for (idx, gpu_result) in gpu_stage_indices.iter().zip(gpu_results.into_iter()) {
                        let output = gpu_result_to_vec(&gpu_result);
                        total_time += per_stage;
                        let _ = input_data.insert(output.clone());
                        results.push(StageResult {
                            stage_name: self.stages[*idx].name.clone(),
                            substrate: Substrate::Gpu,
                            output_data: output,
                            elapsed_us: per_stage,
                            success: true,
                        });
                    }
                }
                Err(_) => {
                    for &idx in &gpu_stage_indices {
                        let result = self.stages[idx].execute(input_data.as_deref());
                        total_time += result.elapsed_us;
                        all_success &= result.success;
                        input_data = Some(result.output_data.clone());
                        results.push(result);
                    }
                }
            }
        }

        PipelineResult {
            stage_results: results,
            total_time_us: total_time,
            success: all_success,
        }
    }

    /// Get stage names for display.
    #[must_use]
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name.as_str()).collect()
    }
}

/// Convert a [`GpuResult`] to a flat `Vec<f64>` for pipeline data flow.
#[cfg(feature = "gpu")]
#[expect(
    clippy::tuple_array_conversions,
    reason = "destructured (shannon, simpson) to [f64; 2] is clearer than From"
)]
fn gpu_result_to_vec(result: &GpuResult) -> Vec<f64> {
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

/// Map a toadstool [`Stage`] to a metalForge [`Workload`] for substrate routing.
#[cfg(feature = "gpu")]
#[expect(clippy::cast_possible_truncation, reason = "element counts fit u32")]
fn stage_to_workload(stage: &Stage, input: Option<&[f64]>) -> healthspring_forge::Workload {
    use crate::stage::{StageOp, TransformKind};

    let n = input.map_or(0, <[f64]>::len) as u32;
    match &stage.operation {
        StageOp::ElementwiseTransform {
            kind: TransformKind::Hill { .. },
        } => healthspring_forge::Workload::DoseResponse {
            n_concentrations: n,
        },
        StageOp::PopulationPk { n_patients, .. } => healthspring_forge::Workload::PopulationPk {
            n_patients: *n_patients as u32,
        },
        StageOp::Generate { n_elements, .. } => healthspring_forge::Workload::PopulationPk {
            n_patients: *n_elements as u32,
        },
        StageOp::DiversityReduce { communities } | StageOp::BrayCurtis { communities } => {
            healthspring_forge::Workload::DiversityIndex {
                n_samples: communities.len() as u32,
            }
        }
        StageOp::Reduce { .. } => healthspring_forge::Workload::DiversityIndex { n_samples: n },
        StageOp::BiosignalFusion { n_channels } => healthspring_forge::Workload::BiosignalFusion {
            channels: *n_channels as u32,
        },
        StageOp::MichaelisMentenBatch { n_patients, .. } => {
            healthspring_forge::Workload::MichaelisMentenBatch {
                n_patients: *n_patients,
            }
        }
        StageOp::ScfaBatch { fiber_inputs, .. } => healthspring_forge::Workload::ScfaBatch {
            n_elements: fiber_inputs.len() as u32,
        },
        StageOp::BeatClassifyBatch { beats, .. } => {
            healthspring_forge::Workload::BeatClassifyBatch {
                n_beats: beats.len() as u32,
            }
        }
        _ => healthspring_forge::Workload::Analytical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::{ReduceKind, StageOp, TransformKind};
    use healthspring_forge::Substrate;

    fn make_generate_stage() -> Stage {
        Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 10,
                seed: 42,
            },
        }
    }

    fn make_transform_stage() -> Stage {
        Stage {
            name: "hill".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Hill {
                    emax: 1.0,
                    ec50: 0.5,
                    n: 2.0,
                },
            },
        }
    }

    fn make_reduce_stage() -> Stage {
        Stage {
            name: "mean".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Mean,
            },
        }
    }

    fn make_diversity_stage() -> Stage {
        Stage {
            name: "diversity".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::DiversityReduce {
                communities: vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]],
            },
        }
    }

    fn make_population_pk_stage() -> Stage {
        Stage {
            name: "pop_pk".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::PopulationPk {
                n_patients: 20,
                dose_mg: 4.0,
                f_bioavail: 0.79,
                seed: 123,
            },
        }
    }

    fn make_filter_stage(threshold: f64) -> Stage {
        Stage {
            name: "filter".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Filter { threshold },
        }
    }

    #[test]
    fn pipeline_new_is_empty() {
        let p = Pipeline::new("test");
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
        assert_eq!(p.name, "test");
    }

    #[test]
    fn pipeline_add_stage_increases_len() {
        let mut p = Pipeline::new("test");
        p.add_stage(make_generate_stage());
        assert_eq!(p.len(), 1);
        p.add_stage(make_transform_stage());
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn pipeline_stage_names() {
        let mut p = Pipeline::new("test");
        p.add_stage(make_generate_stage());
        p.add_stage(make_transform_stage());
        p.add_stage(make_reduce_stage());
        assert_eq!(p.stage_names(), ["gen", "hill", "mean"]);
    }

    #[test]
    fn pipeline_execute_cpu_generate_transform_reduce() {
        let mut p = Pipeline::new("dose_response");
        p.add_stage(make_generate_stage());
        p.add_stage(make_transform_stage());
        p.add_stage(make_reduce_stage());

        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results.len(), 3);
        assert!(result.total_time_us >= 0.0);

        // First stage: 10 elements generated
        assert_eq!(result.stage_results[0].output_data.len(), 10);

        // Second stage: 10 elements transformed
        assert_eq!(result.stage_results[1].output_data.len(), 10);

        // Third stage: single scalar (mean)
        assert_eq!(result.stage_results[2].output_data.len(), 1);
    }

    #[test]
    fn pipeline_execute_empty_returns_empty_result() {
        let p = Pipeline::new("empty");
        let result = p.execute_cpu();
        assert!(result.success);
        assert!(result.stage_results.is_empty());
        assert!(result.total_time_us.abs() < f64::EPSILON);
    }

    #[test]
    fn pipeline_execute_cpu_with_diversity_reduce() {
        let mut p = Pipeline::new("diversity");
        p.add_stage(make_diversity_stage());

        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results.len(), 1);
        // DiversityReduce outputs [shannon1, simpson1, shannon2, simpson2] per community
        assert_eq!(result.stage_results[0].output_data.len(), 4);
        assert!(result.total_time_us > 0.0);
    }

    #[test]
    fn pipeline_execute_cpu_with_population_pk() {
        let mut p = Pipeline::new("pop_pk");
        p.add_stage(make_population_pk_stage());

        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results.len(), 1);
        assert_eq!(result.stage_results[0].output_data.len(), 20);
        assert!(result.stage_results[0].output_data.iter().all(|&v| v > 0.0));
    }

    #[test]
    fn pipeline_execute_cpu_with_filter() {
        let mut p = Pipeline::new("filtered");
        p.add_stage(make_generate_stage());
        p.add_stage(make_filter_stage(0.5));

        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results.len(), 2);
        let filtered = &result.stage_results[1].output_data;
        assert!(filtered.iter().all(|&x| x > 0.5));
        assert!(filtered.len() <= 10);
    }

    #[test]
    fn pipeline_success_tracking_generate_transform_filter_reduce() {
        let mut p = Pipeline::new("full");
        p.add_stage(make_generate_stage());
        p.add_stage(make_transform_stage());
        p.add_stage(make_filter_stage(0.1));
        p.add_stage(Stage {
            name: "sum".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Sum,
            },
        });

        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results.len(), 4);
        assert_eq!(result.stage_results[3].output_data.len(), 1);
    }

    #[test]
    fn pipeline_data_flow_correctness() {
        let mut p = Pipeline::new("flow");
        p.add_stage(Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 5,
                seed: 999,
            },
        });
        p.add_stage(Stage {
            name: "square".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Square,
            },
        });
        p.add_stage(Stage {
            name: "sum".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Sum,
            },
        });

        let result = p.execute_cpu();
        assert!(result.success);
        let gen_out = &result.stage_results[0].output_data;
        let square_out = &result.stage_results[1].output_data;
        let sum_out = &result.stage_results[2].output_data;

        assert_eq!(gen_out.len(), 5);
        assert_eq!(square_out.len(), 5);
        for i in 0..5 {
            let expected = gen_out[i] * gen_out[i];
            assert!(
                (square_out[i] - expected).abs() < 1e-10,
                "square[{i}] = {} expected {}",
                square_out[i],
                expected
            );
        }
        let expected_sum: f64 = square_out.iter().sum();
        assert!(
            (sum_out[0] - expected_sum).abs() < 1e-10,
            "sum = {} expected {}",
            sum_out[0],
            expected_sum
        );
    }

    #[test]
    fn pipeline_result_total_time_positive() {
        let mut p = Pipeline::new("timed");
        p.add_stage(make_generate_stage());
        p.add_stage(make_reduce_stage());

        let result = p.execute_cpu();
        assert!(result.success);
        assert!(result.total_time_us >= 0.0);
    }

    #[test]
    fn pipeline_reduce_sum() {
        let mut p = Pipeline::new("sum");
        p.add_stage(Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 4,
                seed: 1,
            },
        });
        p.add_stage(Stage {
            name: "sum".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Sum,
            },
        });
        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results[1].output_data.len(), 1);
    }

    #[test]
    fn pipeline_reduce_max() {
        let mut p = Pipeline::new("max");
        p.add_stage(make_generate_stage());
        p.add_stage(Stage {
            name: "max".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Max,
            },
        });
        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results[1].output_data.len(), 1);
        assert!(result.stage_results[1].output_data[0] <= 1.0);
    }

    #[test]
    fn pipeline_reduce_min() {
        let mut p = Pipeline::new("min");
        p.add_stage(make_generate_stage());
        p.add_stage(Stage {
            name: "min".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Reduce {
                kind: ReduceKind::Min,
            },
        });
        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results[1].output_data.len(), 1);
        assert!(result.stage_results[1].output_data[0] >= 0.0);
    }

    #[test]
    fn pipeline_transform_square() {
        let mut p = Pipeline::new("square");
        p.add_stage(Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 3,
                seed: 0,
            },
        });
        p.add_stage(Stage {
            name: "sq".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::Square,
            },
        });
        let result = p.execute_cpu();
        assert!(result.success);
        let out = &result.stage_results[1].output_data;
        assert_eq!(out.len(), 3);
        for (i, &out_val) in out.iter().enumerate() {
            let inp = result.stage_results[0].output_data[i];
            assert!(inp.mul_add(-inp, out_val).abs() < 1e-10);
        }
    }

    #[test]
    fn pipeline_transform_exp_decay() {
        let mut p = Pipeline::new("exp_decay");
        p.add_stage(Stage {
            name: "gen".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::Generate {
                n_elements: 5,
                seed: 42,
            },
        });
        p.add_stage(Stage {
            name: "decay".into(),
            substrate: Substrate::Cpu,
            operation: StageOp::ElementwiseTransform {
                kind: TransformKind::ExpDecay { k: 0.1, t: 1.0 },
            },
        });
        let result = p.execute_cpu();
        assert!(result.success);
        assert_eq!(result.stage_results[1].output_data.len(), 5);
    }

    #[test]
    fn pipeline_execute_streaming_invokes_callback() {
        let mut p = Pipeline::new("streaming_test");
        p.add_stage(make_generate_stage());
        p.add_stage(make_transform_stage());
        p.add_stage(make_reduce_stage());

        let mut callback_calls = Vec::new();
        let result = p.execute_streaming(|idx, total, stage_result| {
            callback_calls.push((idx, total, stage_result.stage_name.clone()));
        });

        assert!(result.success);
        assert_eq!(result.stage_results.len(), 3);
        assert_eq!(callback_calls.len(), 3);
        assert_eq!(callback_calls[0], (0, 3, "gen".to_string()));
        assert_eq!(callback_calls[1], (1, 3, "hill".to_string()));
        assert_eq!(callback_calls[2], (2, 3, "mean".to_string()));
    }

    #[test]
    fn pipeline_execute_streaming_matches_cpu() {
        let mut p = Pipeline::new("streaming_cpu_match");
        p.add_stage(make_generate_stage());
        p.add_stage(make_transform_stage());
        p.add_stage(make_reduce_stage());

        let cpu_result = p.execute_cpu();
        let streaming_result = p.execute_streaming(|_, _, _| {});

        assert_eq!(
            cpu_result.stage_results.len(),
            streaming_result.stage_results.len()
        );
        for (cpu, stream) in cpu_result
            .stage_results
            .iter()
            .zip(streaming_result.stage_results.iter())
        {
            assert_eq!(cpu.output_data, stream.output_data);
            assert_eq!(cpu.stage_name, stream.stage_name);
        }
    }
}
