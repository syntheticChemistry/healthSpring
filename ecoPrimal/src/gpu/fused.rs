// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fused pipeline preparation: per-op buffer layout, shader compilation, and
//! bind group construction for single-encoder multi-op dispatch.
//!
//! Each `prepare_*` function takes a `wgpu::Device`, an op's parameters, and
//! a label, and returns a [`PreparedOp`] ready for compute-pass recording.
//!
//! ## Design: Local WGSL for All Ops
//!
//! The fused pipeline intentionally uses local WGSL shaders for ALL ops
//! (including Tier A ops that are rewired to barraCuda in single-op dispatch)
//! because:
//!
//! 1. The fused pipeline's value proposition is single-encoder multi-op
//!    dispatch (upload once → N compute passes → readback once).
//! 2. Mixing barraCuda ops (which create their own encoders) with local WGSL
//!    in a single encoder would break the unidirectional pattern.
//!
//! ## `TensorSession` relationship (evaluated 2026-03-24)
//!
//! `barracuda::session::TensorSession` provides fused execution for
//! **dependent** operation chains (matmul → relu → softmax) via a graph
//! of `SessionTensor` objects. healthSpring's fused pipeline dispatches
//! **independent** parallel ops (Hill, `PopPK`, Diversity in one encoder).
//!
//! These patterns are complementary:
//! - Independent parallel batch → local fused pipeline (this module)
//! - Dependent operation chain → `TensorSession` (future integration)
//!
//! When healthSpring builds dependent multi-op pipelines (e.g., a
//! PK → concentration → diversity chain with data flowing between ops),
//! `TensorSession` becomes the correct abstraction.

use super::dispatch::{
    BeatClassifyParams, DivParams, HillParams, MmParams, PkParams, ScfaGpuParams, WG_SIZE,
    strip_f64_enable,
};
use super::shaders;

/// A fully prepared GPU operation: pipeline, bind group, buffers, and
/// readback metadata. One per op in the fused encoder.
pub(super) struct PreparedOp {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub workgroups: (u32, u32, u32),
    pub output_buf: wgpu::Buffer,
    pub staging_buf: wgpu::Buffer,
    pub output_bytes: u64,
    pub kind: OpKind,
}

/// Discriminant for readback decoding after the fused submission completes.
pub(super) enum OpKind {
    Hill,
    PopPk,
    Diversity { n_communities: usize },
    MichaelisMenten,
    Scfa { n_elements: usize },
    BeatClassify { n_beats: usize },
}

fn compile_pipeline(device: &wgpu::Device, label: &str, wgsl_src: &str) -> wgpu::ComputePipeline {
    let src = strip_f64_enable(wgsl_src);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    })
}

fn staging_pair(
    device: &wgpu::Device,
    label: &str,
    byte_size: u64,
) -> (wgpu::Buffer, wgpu::Buffer) {
    use wgpu::BufferUsages;
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: byte_size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: byte_size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    (output, staging)
}

fn uniform_buf(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

fn storage_buf(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn bind(
    device: &wgpu::Device,
    label: &str,
    pipeline: &wgpu::ComputePipeline,
    entries: &[wgpu::BindGroupEntry<'_>],
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &pipeline.get_bind_group_layout(0),
        entries,
    })
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "concentration count fits u32"
)]
pub(super) fn prepare_hill(
    device: &wgpu::Device,
    label: &str,
    emax: f64,
    ec50: f64,
    n: f64,
    concentrations: &[f64],
) -> PreparedOp {
    let n_concs = concentrations.len() as u32;
    let byte_size = u64::from(n_concs) * 8;
    let params = HillParams {
        emax,
        ec50,
        hill_n: n,
        n_concs,
        _pad: 0,
    };

    let input_buf = storage_buf(device, label, bytemuck::cast_slice(concentrations));
    let (output_buf, staging_buf) = staging_pair(device, label, byte_size);
    let params_buf = uniform_buf(device, label, bytemuck::bytes_of(&params));

    let pipeline = compile_pipeline(device, label, shaders::HILL_DOSE_RESPONSE);
    let bind_group = bind(
        device,
        label,
        &pipeline,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buf.as_entire_binding(),
            },
        ],
    );

    PreparedOp {
        pipeline,
        bind_group,
        workgroups: (n_concs.div_ceil(WG_SIZE), 1, 1),
        output_buf,
        staging_buf,
        output_bytes: byte_size,
        kind: OpKind::Hill,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "n_patients and seed fit u32"
)]
pub(super) fn prepare_pop_pk(
    device: &wgpu::Device,
    label: &str,
    n_patients: usize,
    dose_mg: f64,
    f_bioavail: f64,
    seed: u64,
) -> PreparedOp {
    let n = n_patients as u32;
    let byte_size = u64::from(n) * 8;
    let params = PkParams {
        n_patients: n,
        base_seed: seed as u32,
        dose_mg,
        f_bioavail,
    };

    let (output_buf, staging_buf) = staging_pair(device, label, byte_size);
    let params_buf = uniform_buf(device, label, bytemuck::bytes_of(&params));

    let pipeline = compile_pipeline(device, label, shaders::POPULATION_PK);
    let bind_group = bind(
        device,
        label,
        &pipeline,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buf.as_entire_binding(),
            },
        ],
    );

    PreparedOp {
        pipeline,
        bind_group,
        workgroups: (n.div_ceil(WG_SIZE), 1, 1),
        output_buf,
        staging_buf,
        output_bytes: byte_size,
        kind: OpKind::PopPk,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "community/stride counts fit u32"
)]
pub(super) fn prepare_diversity(
    device: &wgpu::Device,
    label: &str,
    communities: &[Vec<f64>],
) -> PreparedOp {
    let n_communities = communities.len() as u32;
    let stride = communities.iter().map(Vec::len).max().unwrap_or(0) as u32;
    let output_bytes = u64::from(n_communities) * 2 * 8;

    let mut flat = Vec::with_capacity((n_communities * stride) as usize);
    for community in communities {
        flat.extend_from_slice(community);
        flat.resize(flat.len() + (stride as usize - community.len()), 0.0);
    }

    let params = DivParams {
        n_communities,
        stride,
        _pad0: 0,
        _pad1: 0,
    };
    let input_buf = storage_buf(device, label, bytemuck::cast_slice(&flat));
    let (output_buf, staging_buf) = staging_pair(device, label, output_bytes);
    let params_buf = uniform_buf(device, label, bytemuck::bytes_of(&params));

    let pipeline = compile_pipeline(device, label, shaders::DIVERSITY);
    let bind_group = bind(
        device,
        label,
        &pipeline,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buf.as_entire_binding(),
            },
        ],
    );

    PreparedOp {
        pipeline,
        bind_group,
        workgroups: (n_communities, 1, 1),
        output_buf,
        staging_buf,
        output_bytes,
        kind: OpKind::Diversity {
            n_communities: communities.len(),
        },
    }
}

pub(super) fn prepare_michaelis_menten(
    device: &wgpu::Device,
    label: &str,
    params: &MmParams,
) -> PreparedOp {
    let byte_size = u64::from(params.n_patients) * 8;
    let (output_buf, staging_buf) = staging_pair(device, label, byte_size);
    let params_buf = uniform_buf(device, label, bytemuck::bytes_of(params));

    let pipeline = compile_pipeline(device, label, shaders::MICHAELIS_MENTEN_BATCH);
    let bind_group = bind(
        device,
        label,
        &pipeline,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buf.as_entire_binding(),
            },
        ],
    );

    PreparedOp {
        pipeline,
        bind_group,
        workgroups: (params.n_patients.div_ceil(WG_SIZE), 1, 1),
        output_buf,
        staging_buf,
        output_bytes: byte_size,
        kind: OpKind::MichaelisMenten,
    }
}

#[expect(clippy::cast_possible_truncation, reason = "element count fits u32")]
pub(super) fn prepare_scfa(
    device: &wgpu::Device,
    label: &str,
    scfa_params: &crate::microbiome::ScfaParams,
    fiber_inputs: &[f64],
) -> PreparedOp {
    let n_elements = fiber_inputs.len() as u32;
    let output_bytes = u64::from(n_elements) * 3 * 8;
    let params = ScfaGpuParams {
        vmax_acetate: scfa_params.vmax_acetate,
        km_acetate: scfa_params.km_acetate,
        vmax_propionate: scfa_params.vmax_propionate,
        km_propionate: scfa_params.km_propionate,
        vmax_butyrate: scfa_params.vmax_butyrate,
        km_butyrate: scfa_params.km_butyrate,
        n_elements,
        _pad: 0,
    };

    let input_buf = storage_buf(device, label, bytemuck::cast_slice(fiber_inputs));
    let (output_buf, staging_buf) = staging_pair(device, label, output_bytes);
    let params_buf = uniform_buf(device, label, bytemuck::bytes_of(&params));

    let pipeline = compile_pipeline(device, label, shaders::SCFA_BATCH);
    let bind_group = bind(
        device,
        label,
        &pipeline,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buf.as_entire_binding(),
            },
        ],
    );

    PreparedOp {
        pipeline,
        bind_group,
        workgroups: (n_elements.div_ceil(WG_SIZE), 1, 1),
        output_buf,
        staging_buf,
        output_bytes,
        kind: OpKind::Scfa {
            n_elements: fiber_inputs.len(),
        },
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "beat/template/window counts fit u32"
)]
pub(super) fn prepare_beat_classify(
    device: &wgpu::Device,
    label: &str,
    beats: &[Vec<f64>],
    templates: &[Vec<f64>],
) -> PreparedOp {
    let n_beats = beats.len() as u32;
    let n_templates = templates.len() as u32;
    let window_size = beats.first().map_or(0, Vec::len) as u32;
    let output_bytes = u64::from(n_beats) * 2 * 8;
    let params = BeatClassifyParams {
        n_beats,
        n_templates,
        window_size,
        _pad: 0,
    };

    let mut flat_beats = Vec::with_capacity(n_beats as usize * window_size as usize);
    for b in beats {
        flat_beats.extend_from_slice(b);
        flat_beats.resize(flat_beats.len() + (window_size as usize - b.len()), 0.0);
    }
    let mut flat_templates = Vec::with_capacity(n_templates as usize * window_size as usize);
    for t in templates {
        flat_templates.extend_from_slice(t);
        flat_templates.resize(flat_templates.len() + (window_size as usize - t.len()), 0.0);
    }

    let beats_buf = storage_buf(device, label, bytemuck::cast_slice(&flat_beats));
    let templates_buf = storage_buf(device, label, bytemuck::cast_slice(&flat_templates));
    let (output_buf, staging_buf) = staging_pair(device, label, output_bytes);
    let params_buf = uniform_buf(device, label, bytemuck::bytes_of(&params));

    let pipeline = compile_pipeline(device, label, shaders::BEAT_CLASSIFY_BATCH);
    let bind_group = bind(
        device,
        label,
        &pipeline,
        &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: beats_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: templates_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: params_buf.as_entire_binding(),
            },
        ],
    );

    PreparedOp {
        pipeline,
        bind_group,
        workgroups: (n_beats.div_ceil(WG_SIZE), 1, 1),
        output_buf,
        staging_buf,
        output_bytes,
        kind: OpKind::BeatClassify {
            n_beats: beats.len(),
        },
    }
}

/// Decode raw f64 readback into the appropriate `GpuResult` variant.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "template index fits u32"
)]
pub(super) fn decode_readback(kind: &OpKind, raw: &[f64]) -> super::GpuResult {
    use super::GpuResult;
    match kind {
        OpKind::Hill => GpuResult::HillSweep(raw.to_vec()),
        OpKind::PopPk => GpuResult::PopulationPkBatch(raw.to_vec()),
        OpKind::Diversity { n_communities } => {
            let pairs: Vec<(f64, f64)> = (0..*n_communities)
                .map(|i| (raw[i * 2], raw[i * 2 + 1]))
                .collect();
            GpuResult::DiversityBatch(pairs)
        }
        OpKind::MichaelisMenten => GpuResult::MichaelisMentenBatch(raw.to_vec()),
        OpKind::Scfa { n_elements } => {
            let triples: Vec<(f64, f64, f64)> = (0..*n_elements)
                .map(|i| (raw[i * 3], raw[i * 3 + 1], raw[i * 3 + 2]))
                .collect();
            GpuResult::ScfaBatch(triples)
        }
        OpKind::BeatClassify { n_beats } => {
            let pairs: Vec<(u32, f64)> = (0..*n_beats)
                .map(|i| (raw[i * 2] as u32, raw[i * 2 + 1]))
                .collect();
            GpuResult::BeatClassifyBatch(pairs)
        }
    }
}
