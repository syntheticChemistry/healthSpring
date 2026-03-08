// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compute pipeline: a sequence of stages that execute on heterogeneous hardware.

use crate::stage::{Stage, StageResult};

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

    /// Get stage names for display.
    #[must_use]
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name.as_str()).collect()
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
