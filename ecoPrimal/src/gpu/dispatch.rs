// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{GpuError, GpuOp, GpuResult, shaders};

/// Workgroup size used in Hill and `PopPK` shaders. Must match `@workgroup_size(N)`.
pub(crate) const WG_SIZE: u32 = 256;

/// Strip `enable f64;` — naga parses f64 types natively when `SHADER_F64`
/// feature is negotiated at device creation.
pub(crate) fn strip_f64_enable(source: &str) -> String {
    source.replace("enable f64;", "")
}

/// Configuration for a single GPU compute dispatch.
pub(crate) struct GpuDispatch<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) shader_source: &'a str,
    pub(crate) label: &'a str,
    pub(crate) bindings: &'a [&'a wgpu::Buffer],
    pub(crate) output_buf: &'a wgpu::Buffer,
    pub(crate) staging_buf: &'a wgpu::Buffer,
    pub(crate) output_bytes: u64,
    pub(crate) workgroups: (u32, u32, u32),
}

/// Compile shader, create pipeline, dispatch, readback into `Vec<f64>`.
pub(crate) async fn dispatch_and_readback(cfg: &GpuDispatch<'_>) -> Result<Vec<f64>, GpuError> {
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct HillParams {
    pub(crate) emax: f64,
    pub(crate) ec50: f64,
    pub(crate) hill_n: f64,
    pub(crate) n_concs: u32,
    pub(crate) _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PkParams {
    pub(crate) n_patients: u32,
    pub(crate) base_seed: u32,
    pub(crate) dose_mg: f64,
    pub(crate) f_bioavail: f64,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct DivParams {
    pub(crate) n_communities: u32,
    pub(crate) stride: u32,
    pub(crate) _pad0: u32,
    pub(crate) _pad1: u32,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "concentration count fits u32"
)]
pub(crate) async fn execute_hill_gpu(
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

#[expect(
    clippy::cast_possible_truncation,
    reason = "n_patients and seed truncation documented"
)]
pub(crate) async fn execute_pop_pk_gpu(
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

#[expect(clippy::cast_possible_truncation, reason = "community sizes fit u32")]
pub(crate) async fn execute_diversity_gpu(
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MmParams {
    pub(crate) vmax: f64,
    pub(crate) km: f64,
    pub(crate) vd: f64,
    pub(crate) dt: f64,
    pub(crate) n_steps: u32,
    pub(crate) n_patients: u32,
    pub(crate) base_seed: u32,
    pub(crate) _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ScfaGpuParams {
    pub(crate) vmax_acetate: f64,
    pub(crate) km_acetate: f64,
    pub(crate) vmax_propionate: f64,
    pub(crate) km_propionate: f64,
    pub(crate) vmax_butyrate: f64,
    pub(crate) km_butyrate: f64,
    pub(crate) n_elements: u32,
    pub(crate) _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BeatClassifyParams {
    pub(crate) n_beats: u32,
    pub(crate) n_templates: u32,
    pub(crate) window_size: u32,
    pub(crate) _pad: u32,
}

pub(crate) async fn execute_mm_gpu(
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
pub(crate) async fn execute_scfa_gpu(
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
pub(crate) async fn execute_beat_classify_gpu(
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

/// Execute a GPU operation on live hardware.
///
/// Acquires a wgpu device, compiles the appropriate WGSL shader, dispatches
/// compute, and reads back results.
///
/// # Errors
///
/// Returns [`GpuError::NoDevice`] if no adapter/device is available,
/// [`GpuError::Dispatch`] on shader compilation failure, or
/// [`GpuError::Readback`] on buffer map failure.
pub async fn execute_gpu(op: &GpuOp) -> Result<GpuResult, GpuError> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..wgpu::InstanceDescriptor::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .map_err(|e| GpuError::NoDevice(format!("{e}")))?;

    let mut required_features = wgpu::Features::empty();
    if adapter.features().contains(wgpu::Features::SHADER_F64) {
        required_features |= wgpu::Features::SHADER_F64;
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("healthSpring GPU"),
            required_features,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::default(),
            trace: wgpu::Trace::default(),
        })
        .await
        .map_err(|e| GpuError::NoDevice(format!("{e}")))?;

    match op {
        GpuOp::HillSweep {
            emax,
            ec50,
            n,
            concentrations,
        } => execute_hill_gpu(&device, &queue, *emax, *ec50, *n, concentrations).await,
        GpuOp::PopulationPkBatch {
            n_patients,
            dose_mg,
            f_bioavail,
            seed,
        } => execute_pop_pk_gpu(&device, &queue, *n_patients, *dose_mg, *f_bioavail, *seed).await,
        GpuOp::DiversityBatch { communities } => {
            execute_diversity_gpu(&device, &queue, communities).await
        }
        GpuOp::MichaelisMentenBatch {
            vmax,
            km,
            vd,
            dt,
            n_steps,
            n_patients,
            seed,
        } => {
            let mm_params = MmParams {
                vmax: *vmax,
                km: *km,
                vd: *vd,
                dt: *dt,
                n_steps: *n_steps,
                n_patients: *n_patients,
                base_seed: *seed,
                _pad: 0,
            };
            execute_mm_gpu(&device, &queue, &mm_params).await
        }
        GpuOp::ScfaBatch {
            params,
            fiber_inputs,
        } => execute_scfa_gpu(&device, &queue, params, fiber_inputs).await,
        GpuOp::BeatClassifyBatch { beats, templates } => {
            execute_beat_classify_gpu(&device, &queue, beats, templates).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{WG_SIZE, strip_f64_enable};

    #[test]
    fn strip_f64_enable_removes_directive() {
        let src = "enable f64;\n@compute @workgroup_size(256)\nfn main() {}";
        let stripped = strip_f64_enable(src);
        assert!(!stripped.contains("enable f64"));
        assert!(stripped.contains("@compute"));
    }

    #[test]
    fn strip_f64_enable_preserves_rest() {
        let src = "enable f64;\nlet x = 1.0;";
        let stripped = strip_f64_enable(src);
        assert_eq!(stripped, "\nlet x = 1.0;");
    }

    #[test]
    fn strip_f64_enable_no_match() {
        let src = "fn main() { }";
        assert_eq!(strip_f64_enable(src), src);
    }

    #[test]
    fn wg_size_constant() {
        assert_eq!(WG_SIZE, 256, "WG_SIZE must match shader @workgroup_size");
    }

    #[test]
    fn workgroup_calculation_hill() {
        let n_concs: u32 = 500;
        let workgroups = n_concs.div_ceil(WG_SIZE);
        assert_eq!(workgroups, 2);
    }

    #[test]
    fn workgroup_calculation_pop_pk() {
        let n_patients: u32 = 1000;
        let workgroups = n_patients.div_ceil(WG_SIZE);
        assert_eq!(workgroups, 4);
    }

    #[test]
    fn workgroup_calculation_diversity() {
        let n_communities: u32 = 10;
        let workgroups = (n_communities, 1, 1);
        assert_eq!(workgroups.0, 10, "diversity uses 1 workgroup per community");
    }

    #[test]
    fn diversity_flat_buffer_stride() {
        let communities: Vec<Vec<f64>> =
            vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]];
        let stride = communities.iter().map(Vec::len).max().unwrap_or(0);
        assert_eq!(stride, 4);
        let flat_len: usize = communities.len() * stride;
        assert_eq!(flat_len, 8);
    }

    #[test]
    fn beat_classify_output_bytes() {
        let n_beats: u32 = 50;
        let output_bytes = u64::from(n_beats) * 2 * 8;
        assert_eq!(output_bytes, 800, "50 beats × (idx + corr) × 8 bytes");
    }
}
