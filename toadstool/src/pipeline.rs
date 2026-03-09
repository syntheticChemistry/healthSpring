// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compute pipeline: a sequence of stages that execute on heterogeneous hardware.
//!
//! ## ABSORPTION CANDIDATES (toadStool)
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
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Whether the pipeline has no stages.
    #[must_use]
    pub fn is_empty(&self) -> bool {
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

    /// Execute the pipeline on GPU via a fused single-submission dispatch.
    ///
    /// Stages that map to GPU ops are batched and dispatched in a single
    /// command encoder through [`GpuContext::execute_fused`]. Stages without
    /// a GPU kernel fall back to CPU automatically.
    ///
    /// This is the unidirectional pipeline: data flows CPU → GPU → CPU with
    /// no round-trips between stages.
    #[cfg(feature = "gpu")]
    #[expect(clippy::cast_precision_loss)]
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
                        #[expect(clippy::cast_precision_loss)]
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
    #[expect(clippy::cast_precision_loss)]
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
        let mut cpu_stages = Vec::new();

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
            cpu_stages.push(i);

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
                    #[expect(clippy::cast_precision_loss)]
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
fn gpu_result_to_vec(result: &GpuResult) -> Vec<f64> {
    match result {
        GpuResult::HillSweep(v) | GpuResult::PopulationPkBatch(v) => v.clone(),
        GpuResult::DiversityBatch(pairs) => pairs.iter().flat_map(|&(s, d)| [s, d]).collect(),
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
        StageOp::DiversityReduce { communities } => {
            healthspring_forge::Workload::DiversityIndex {
                n_samples: communities.len() as u32,
            }
        }
        StageOp::Reduce { .. } => healthspring_forge::Workload::DiversityIndex { n_samples: n },
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
        assert_eq!(result.total_time_us, 0.0);
    }
}
