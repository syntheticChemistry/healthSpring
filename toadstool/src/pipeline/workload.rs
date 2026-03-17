// SPDX-License-Identifier: AGPL-3.0-or-later
//! metalForge workload mapping for substrate routing.
//!
//! Maps toadstool [`Stage`] operations to metalForge [`Workload`] descriptors
//! for capability-based dispatch (CPU/GPU/NPU selection).

use crate::stage::{Stage, StageOp, TransformKind};

/// Map a toadstool [`Stage`] to a metalForge [`Workload`] for substrate routing.
#[expect(clippy::cast_possible_truncation, reason = "element counts fit u32")]
pub fn stage_to_workload(stage: &Stage, input: Option<&[f64]>) -> healthspring_forge::Workload {
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
