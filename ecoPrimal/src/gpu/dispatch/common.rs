// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::gpu::GpuError;

/// Workgroup size used in Hill and `PopPK` shaders. Must match `@workgroup_size(N)`.
pub const WG_SIZE: u32 = 256;

/// Strip `enable f64;` — naga parses f64 types natively when `SHADER_F64`
/// feature is negotiated at device creation.
///
/// ## Legacy path
///
/// This is the **legacy** WGSL preprocessor workaround. The sovereign pipeline
/// ([`crate::gpu::sovereign`]) uses coralReef's native f64 lowering instead —
/// no stripping required. When `sovereign-dispatch` is available and coralReef
/// is discoverable, the sovereign path bypasses this entirely.
pub fn strip_f64_enable(source: &str) -> String {
    source.replace("enable f64;", "")
}

/// Configuration for a single GPU compute dispatch.
pub struct GpuDispatch<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub shader_source: &'a str,
    pub label: &'a str,
    pub bindings: &'a [&'a wgpu::Buffer],
    pub output_buf: &'a wgpu::Buffer,
    pub staging_buf: &'a wgpu::Buffer,
    pub output_bytes: u64,
    pub workgroups: (u32, u32, u32),
}

/// Compile shader, create pipeline, dispatch, readback into `Vec<f64>`.
pub async fn dispatch_and_readback(cfg: &GpuDispatch<'_>) -> Result<Vec<f64>, GpuError> {
    let GpuDispatch {
        device,
        queue,
        shader_source,
        label,
        bindings,
        output_buf,
        staging_buf,
        output_bytes,
        workgroups,
    } = cfg;
    let shader_src = strip_f64_enable(shader_source);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    let entries: Vec<wgpu::BindGroupEntry<'_>> = bindings
        .iter()
        .enumerate()
        .map(|(i, buf)| {
            #[expect(clippy::cast_possible_truncation, reason = "binding indices < 10")]
            let binding = i as u32;
            wgpu::BindGroupEntry {
                binding,
                resource: buf.as_entire_binding(),
            }
        })
        .collect();

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(label),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, Some(&bind_group), &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
    encoder.copy_buffer_to_buffer(output_buf, 0, staging_buf, 0, *output_bytes);
    queue.submit(Some(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = tokio::sync::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    rx.await
        .map_err(|e| GpuError::Readback(format!("{e}")))?
        .map_err(|e| GpuError::Readback(format!("{e}")))?;

    let data = slice.get_mapped_range();
    let results: Vec<f64> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging_buf.unmap();

    Ok(results)
}
