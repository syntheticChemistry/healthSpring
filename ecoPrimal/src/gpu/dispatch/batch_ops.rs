// SPDX-License-Identifier: AGPL-3.0-or-later

use super::common::{GpuDispatch, WG_SIZE, dispatch_and_readback};
use crate::gpu::{GpuError, GpuResult, shaders};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MmParams {
    pub vmax: f64,
    pub km: f64,
    pub vd: f64,
    pub dt: f64,
    pub n_steps: u32,
    pub n_patients: u32,
    pub base_seed: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScfaGpuParams {
    pub vmax_acetate: f64,
    pub km_acetate: f64,
    pub vmax_propionate: f64,
    pub km_propionate: f64,
    pub vmax_butyrate: f64,
    pub km_butyrate: f64,
    pub n_elements: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BeatClassifyParams {
    pub n_beats: u32,
    pub n_templates: u32,
    pub window_size: u32,
    pub _pad: u32,
}

pub async fn execute_mm_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    params: &MmParams,
) -> Result<GpuResult, GpuError> {
    use wgpu::util::DeviceExt;

    let n_patients = params.n_patients;
    let byte_size = u64::from(n_patients) * 8;

    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mm output"),
        size: byte_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mm params"),
        contents: bytemuck::bytes_of(params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mm staging"),
        size: byte_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let results = dispatch_and_readback(&GpuDispatch {
        device,
        queue,
        shader_source: shaders::MICHAELIS_MENTEN_BATCH,
        label: "michaelis_menten_batch",
        bindings: &[&output_buf, &params_buf],
        output_buf: &output_buf,
        staging_buf: &staging_buf,
        output_bytes: byte_size,
        workgroups: (n_patients.div_ceil(WG_SIZE), 1, 1),
    })
    .await?;

    Ok(GpuResult::MichaelisMentenBatch(results))
}

#[expect(clippy::cast_possible_truncation, reason = "element count fits u32")]
pub async fn execute_scfa_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scfa_params: &crate::microbiome::ScfaParams,
    fiber_inputs: &[f64],
) -> Result<GpuResult, GpuError> {
    use wgpu::util::DeviceExt;

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

    let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("scfa input"),
        contents: bytemuck::cast_slice(fiber_inputs),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scfa output"),
        size: output_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("scfa params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scfa staging"),
        size: output_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let raw = dispatch_and_readback(&GpuDispatch {
        device,
        queue,
        shader_source: shaders::SCFA_BATCH,
        label: "scfa_batch",
        bindings: &[&input_buf, &output_buf, &params_buf],
        output_buf: &output_buf,
        staging_buf: &staging_buf,
        output_bytes,
        workgroups: (n_elements.div_ceil(WG_SIZE), 1, 1),
    })
    .await?;

    let results: Vec<(f64, f64, f64)> = (0..n_elements as usize)
        .map(|i| (raw[i * 3], raw[i * 3 + 1], raw[i * 3 + 2]))
        .collect();

    Ok(GpuResult::ScfaBatch(results))
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "beat/template counts fit u32"
)]
pub async fn execute_beat_classify_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    beats: &[Vec<f64>],
    templates: &[Vec<f64>],
) -> Result<GpuResult, GpuError> {
    use wgpu::util::DeviceExt;

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

    let mut flat_beats: Vec<f64> = Vec::with_capacity(n_beats as usize * window_size as usize);
    for b in beats {
        flat_beats.extend_from_slice(b);
        flat_beats.resize(flat_beats.len() + (window_size as usize - b.len()), 0.0);
    }

    let mut flat_templates: Vec<f64> =
        Vec::with_capacity(n_templates as usize * window_size as usize);
    for t in templates {
        flat_templates.extend_from_slice(t);
        flat_templates.resize(flat_templates.len() + (window_size as usize - t.len()), 0.0);
    }

    let beats_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("beats input"),
        contents: bytemuck::cast_slice(&flat_beats),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let templates_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("templates input"),
        contents: bytemuck::cast_slice(&flat_templates),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("classify output"),
        size: output_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("classify params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("classify staging"),
        size: output_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let raw = dispatch_and_readback(&GpuDispatch {
        device,
        queue,
        shader_source: shaders::BEAT_CLASSIFY_BATCH,
        label: "beat_classify_batch",
        bindings: &[&beats_buf, &templates_buf, &output_buf, &params_buf],
        output_buf: &output_buf,
        staging_buf: &staging_buf,
        output_bytes,
        workgroups: (n_beats.div_ceil(WG_SIZE), 1, 1),
    })
    .await?;

    let results: Vec<(u32, f64)> = (0..n_beats as usize)
        .map(|i| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "template index fits u32"
            )]
            let idx = raw[i * 2] as u32;
            (idx, raw[i * 2 + 1])
        })
        .collect();

    Ok(GpuResult::BeatClassifyBatch(results))
}
