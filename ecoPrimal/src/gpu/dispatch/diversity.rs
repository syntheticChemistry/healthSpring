// SPDX-License-Identifier: AGPL-3.0-or-later

use super::common::{dispatch_and_readback, GpuDispatch};
use crate::gpu::{shaders, GpuError, GpuResult};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DivParams {
    pub n_communities: u32,
    pub stride: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

#[expect(clippy::cast_possible_truncation, reason = "community sizes fit u32")]
pub async fn execute_diversity_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    communities: &[Vec<f64>],
) -> Result<GpuResult, GpuError> {
    use wgpu::util::DeviceExt;

    let n_communities = communities.len() as u32;
    let stride = communities.iter().map(Vec::len).max().unwrap_or(0) as u32;

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
    let output_bytes = u64::from(n_communities) * 2 * 8;

    let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("div input"),
        contents: bytemuck::cast_slice(&flat),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("div output"),
        size: output_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("div params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("div staging"),
        size: output_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let raw = dispatch_and_readback(&GpuDispatch {
        device,
        queue,
        shader_source: shaders::DIVERSITY,
        label: "diversity",
        bindings: &[&input_buf, &output_buf, &params_buf],
        output_buf: &output_buf,
        staging_buf: &staging_buf,
        output_bytes,
        workgroups: (n_communities, 1, 1),
    })
    .await?;

    let results: Vec<(f64, f64)> = (0..n_communities as usize)
        .map(|i| (raw[i * 2], raw[i * 2 + 1]))
        .collect();

    Ok(GpuResult::DiversityBatch(results))
}
