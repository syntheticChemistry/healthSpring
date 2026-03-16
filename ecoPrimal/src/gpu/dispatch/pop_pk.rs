// SPDX-License-Identifier: AGPL-3.0-or-later

use super::common::{dispatch_and_readback, GpuDispatch, WG_SIZE};
use crate::gpu::{shaders, GpuError, GpuResult};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PkParams {
    pub n_patients: u32,
    pub base_seed: u32,
    pub dose_mg: f64,
    pub f_bioavail: f64,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "n_patients and seed truncation documented"
)]
pub async fn execute_pop_pk_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    n_patients: usize,
    dose_mg: f64,
    f_bioavail: f64,
    seed: u64,
) -> Result<GpuResult, GpuError> {
    use wgpu::util::DeviceExt;

    let n = n_patients as u32;
    let byte_size = u64::from(n) * 8;
    let params = PkParams {
        n_patients: n,
        base_seed: seed as u32,
        dose_mg,
        f_bioavail,
    };

    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pk output"),
        size: byte_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pk params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pk staging"),
        size: byte_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let results = dispatch_and_readback(&GpuDispatch {
        device,
        queue,
        shader_source: shaders::POPULATION_PK,
        label: "population_pk",
        bindings: &[&output_buf, &params_buf],
        output_buf: &output_buf,
        staging_buf: &staging_buf,
        output_bytes: byte_size,
        workgroups: (n.div_ceil(WG_SIZE), 1, 1),
    })
    .await?;

    Ok(GpuResult::PopulationPkBatch(results))
}
