// SPDX-License-Identifier: AGPL-3.0-or-later

use super::common::{GpuDispatch, WG_SIZE, dispatch_and_readback};
use crate::gpu::{GpuError, GpuResult, shaders};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HillParams {
    pub emax: f64,
    pub ec50: f64,
    pub hill_n: f64,
    pub n_concs: u32,
    pub _pad: u32,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "concentration count fits u32"
)]
pub async fn execute_hill_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    emax: f64,
    ec50: f64,
    hill_n: f64,
    concentrations: &[f64],
) -> Result<GpuResult, GpuError> {
    use wgpu::util::DeviceExt;

    let n_concs = concentrations.len() as u32;
    let byte_size = u64::from(n_concs) * 8;
    let params = HillParams {
        emax,
        ec50,
        hill_n,
        n_concs,
        _pad: 0,
    };

    let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hill input"),
        contents: bytemuck::cast_slice(concentrations),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("hill output"),
        size: byte_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hill params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("hill staging"),
        size: byte_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let results = dispatch_and_readback(&GpuDispatch {
        device,
        queue,
        shader_source: shaders::HILL_DOSE_RESPONSE,
        label: "hill_dose_response",
        bindings: &[&input_buf, &output_buf, &params_buf],
        output_buf: &output_buf,
        staging_buf: &staging_buf,
        output_bytes: byte_size,
        workgroups: (n_concs.div_ceil(WG_SIZE), 1, 1),
    })
    .await?;

    Ok(GpuResult::HillSweep(results))
}
