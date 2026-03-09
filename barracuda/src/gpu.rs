// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU-dispatchable operations for healthSpring.
//!
//! Defines operations that can execute on either CPU (pure Rust) or GPU
//! (via WGSL shaders through wgpu). CPU fallback is always available.
//! GPU dispatch is activated by the `gpu` feature flag.
//!
//! ## Shader Mapping
//!
//! | healthSpring Op | WGSL Shader | Use Case |
//! |-----------------|-------------|----------|
//! | `HillSweep` | `hill_dose_response_f64.wgsl` | Exp001 vectorized |
//! | `PopulationPkBatch` | `population_pk_f64.wgsl` | Exp005/036 Monte Carlo |
//! | `DiversityBatch` | `diversity_f64.wgsl` | Exp010 batch |
//!
//! ## ABSORPTION CANDIDATES (barraCuda / coralReef)
//!
//! - `GpuContext` (persistent device + queue, `execute_fused`) -> barraCuda compute executor
//! - `execute_cpu()` / `execute_fused()` dual-path pattern -> barraCuda ops
//! - `strip_f64_enable()` WGSL preprocessor -> coralReef naga pass
//! - `shader_for_op()` mapping -> barraCuda shader registry

use crate::microbiome;
use crate::pkpd;

/// Workgroup size used in Hill and `PopPK` shaders. Must match `@workgroup_size(N)`.
#[cfg(feature = "gpu")]
const WG_SIZE: u32 = 256;

/// WGSL shader sources — compiled into the binary.
pub mod shaders {
    pub const HILL_DOSE_RESPONSE: &str =
        include_str!("../shaders/health/hill_dose_response_f64.wgsl");
    pub const POPULATION_PK: &str = include_str!("../shaders/health/population_pk_f64.wgsl");
    pub const DIVERSITY: &str = include_str!("../shaders/health/diversity_f64.wgsl");
}

/// A GPU-dispatchable operation with input/output buffers.
#[derive(Debug, Clone)]
pub enum GpuOp {
    /// Vectorized Hill dose-response: compute E(c) for many concentrations.
    HillSweep {
        emax: f64,
        ec50: f64,
        n: f64,
        concentrations: Vec<f64>,
    },
    /// Batch population PK: simulate N patients in parallel.
    PopulationPkBatch {
        n_patients: usize,
        dose_mg: f64,
        f_bioavail: f64,
        seed: u64,
    },
    /// Batch diversity indices for multiple communities.
    DiversityBatch { communities: Vec<Vec<f64>> },
}

/// Result of a GPU operation.
#[derive(Debug, Clone)]
pub enum GpuResult {
    /// Hill sweep results: one E value per concentration.
    HillSweep(Vec<f64>),
    /// Population PK results: AUC per patient.
    PopulationPkBatch(Vec<f64>),
    /// Diversity results: (shannon, simpson) per community.
    DiversityBatch(Vec<(f64, f64)>),
}

/// Execute a GPU operation using CPU fallback (pure Rust).
///
/// This is the reference implementation. The GPU path (behind `gpu` feature)
/// must produce identical results within f64 tolerance.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "LCG state u64 → f64 for uniform variate; precision sufficient for PK variation"
)]
pub fn execute_cpu(op: &GpuOp) -> GpuResult {
    match op {
        GpuOp::HillSweep {
            emax,
            ec50,
            n,
            concentrations,
        } => {
            let results: Vec<f64> = concentrations
                .iter()
                .map(|&c| pkpd::hill_dose_response(c, *ec50, *n, *emax))
                .collect();
            GpuResult::HillSweep(results)
        }
        GpuOp::PopulationPkBatch {
            n_patients,
            dose_mg,
            f_bioavail,
            seed,
        } => {
            let mut aucs = Vec::with_capacity(*n_patients);
            let mut rng_state = *seed;
            for _ in 0..*n_patients {
                rng_state = rng_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let u = (rng_state >> 33) as f64 / f64::from(u32::MAX);
                let cl_factor = 0.5 + u;
                let cl = 10.0 * cl_factor;
                let auc = f_bioavail * dose_mg / cl;
                aucs.push(auc);
            }
            GpuResult::PopulationPkBatch(aucs)
        }
        GpuOp::DiversityBatch { communities } => {
            let results: Vec<(f64, f64)> = communities
                .iter()
                .map(|c| (microbiome::shannon_index(c), microbiome::simpson_index(c)))
                .collect();
            GpuResult::DiversityBatch(results)
        }
    }
}

/// Shader descriptor: maps a `GpuOp` to its WGSL shader source.
#[must_use]
pub fn shader_for_op(op: &GpuOp) -> &'static str {
    match op {
        GpuOp::HillSweep { .. } => shaders::HILL_DOSE_RESPONSE,
        GpuOp::PopulationPkBatch { .. } => shaders::POPULATION_PK,
        GpuOp::DiversityBatch { .. } => shaders::DIVERSITY,
    }
}

/// Estimate GPU memory requirement for an operation (bytes).
#[must_use]
pub fn gpu_memory_estimate(op: &GpuOp) -> u64 {
    match op {
        GpuOp::HillSweep { concentrations, .. } => (concentrations.len() as u64) * 8 * 2,
        GpuOp::PopulationPkBatch { n_patients, .. } => (*n_patients as u64) * 8 * 5,
        GpuOp::DiversityBatch { communities } => {
            let total_species: usize = communities.iter().map(Vec::len).sum();
            (total_species as u64) * 8 + (communities.len() as u64) * 16
        }
    }
}

// ---------------------------------------------------------------------------
// GPU execution (feature-gated)
// ---------------------------------------------------------------------------

/// Error type for GPU execution.
#[derive(Debug)]
pub enum GpuError {
    /// No GPU device available.
    NoDevice(String),
    /// Shader compilation or dispatch failed.
    Dispatch(String),
    /// Buffer readback failed.
    Readback(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDevice(msg) => write!(f, "GPU: no device: {msg}"),
            Self::Dispatch(msg) => write!(f, "GPU: dispatch failed: {msg}"),
            Self::Readback(msg) => write!(f, "GPU: readback failed: {msg}"),
        }
    }
}

impl std::error::Error for GpuError {}

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
#[cfg(feature = "gpu")]
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
    }
}

/// Strip `enable f64;` — naga parses f64 types natively when `SHADER_F64`
/// feature is negotiated at device creation.
#[cfg(feature = "gpu")]
fn strip_f64_enable(source: &str) -> String {
    source.replace("enable f64;", "")
}

/// Configuration for a single GPU compute dispatch.
#[cfg(feature = "gpu")]
struct GpuDispatch<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    shader_source: &'a str,
    label: &'a str,
    bindings: &'a [&'a wgpu::Buffer],
    output_buf: &'a wgpu::Buffer,
    staging_buf: &'a wgpu::Buffer,
    output_bytes: u64,
    workgroups: (u32, u32, u32),
}

/// Compile shader, create pipeline, dispatch, readback into `Vec<f64>`.
#[cfg(feature = "gpu")]
async fn dispatch_and_readback(cfg: &GpuDispatch<'_>) -> Result<Vec<f64>, GpuError> {
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
#[cfg(feature = "gpu")]
struct HillParams {
    emax: f64,
    ec50: f64,
    hill_n: f64,
    n_concs: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[cfg(feature = "gpu")]
struct PkParams {
    n_patients: u32,
    base_seed: u32,
    dose_mg: f64,
    f_bioavail: f64,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[cfg(feature = "gpu")]
struct DivParams {
    n_communities: u32,
    stride: u32,
    _pad0: u32,
    _pad1: u32,
}

#[cfg(feature = "gpu")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "concentration count fits u32"
)]
async fn execute_hill_gpu(
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

#[cfg(feature = "gpu")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "n_patients and seed truncation documented"
)]
async fn execute_pop_pk_gpu(
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

#[cfg(feature = "gpu")]
#[expect(clippy::cast_possible_truncation, reason = "community sizes fit u32")]
async fn execute_diversity_gpu(
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

// ---------------------------------------------------------------------------
// GpuContext — cached device + fused pipeline dispatch
// ---------------------------------------------------------------------------

/// Persistent GPU context: one device, one queue, all shaders pre-compiled.
///
/// Eliminates per-dispatch device creation overhead. The fused pipeline
/// dispatches all operations in a single command encoder: upload once,
/// N compute passes, readback once — the unidirectional pipeline pattern
/// required for field-deployed devices (e.g., Raspberry Pi + eGPU).
#[cfg(feature = "gpu")]
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_name: String,
}

#[cfg(feature = "gpu")]
impl GpuContext {
    /// Discover GPU and create a persistent context.
    ///
    /// # Errors
    ///
    /// Returns [`GpuError::NoDevice`] if no adapter or device is available.
    pub async fn new() -> Result<Self, GpuError> {
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

        let adapter_name = adapter.get_info().name.clone();

        let mut required_features = wgpu::Features::empty();
        if adapter.features().contains(wgpu::Features::SHADER_F64) {
            required_features |= wgpu::Features::SHADER_F64;
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("healthSpring GPU (persistent)"),
                required_features,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|e| GpuError::NoDevice(format!("{e}")))?;

        Ok(Self {
            device,
            queue,
            adapter_name,
        })
    }

    /// GPU adapter name for diagnostics.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// Execute a single op on the cached device.
    ///
    /// # Errors
    ///
    /// Returns [`GpuError`] on shader compilation, dispatch, or readback failure.
    pub async fn execute(&self, op: &GpuOp) -> Result<GpuResult, GpuError> {
        match op {
            GpuOp::HillSweep {
                emax,
                ec50,
                n,
                concentrations,
            } => {
                execute_hill_gpu(&self.device, &self.queue, *emax, *ec50, *n, concentrations).await
            }
            GpuOp::PopulationPkBatch {
                n_patients,
                dose_mg,
                f_bioavail,
                seed,
            } => {
                execute_pop_pk_gpu(
                    &self.device,
                    &self.queue,
                    *n_patients,
                    *dose_mg,
                    *f_bioavail,
                    *seed,
                )
                .await
            }
            GpuOp::DiversityBatch { communities } => {
                execute_diversity_gpu(&self.device, &self.queue, communities).await
            }
        }
    }

    /// Execute ALL operations in a single command encoder submission.
    ///
    /// Upload all input buffers → N compute passes → one submit → readback all.
    /// No CPU roundtrips between stages. This is the unidirectional pipeline
    /// pattern: data flows to GPU once, stays there through all stages, and
    /// returns once at the end.
    ///
    /// # Errors
    ///
    /// Returns [`GpuError`] on any shader or dispatch failure.
    #[expect(
        clippy::too_many_lines,
        reason = "single-concern fused pipeline: all stages in one encoder"
    )]
    pub async fn execute_fused(&self, ops: &[GpuOp]) -> Result<Vec<GpuResult>, GpuError> {
        use wgpu::util::DeviceExt;

        struct PreparedOp {
            pipeline: wgpu::ComputePipeline,
            bind_group: wgpu::BindGroup,
            workgroups: (u32, u32, u32),
            output_buf: wgpu::Buffer,
            staging_buf: wgpu::Buffer,
            output_bytes: u64,
            kind: OpKind,
        }

        enum OpKind {
            Hill,
            PopPk,
            Diversity { n_communities: usize },
        }

        if ops.is_empty() {
            return Ok(Vec::new());
        }

        let mut prepared: Vec<PreparedOp> = Vec::with_capacity(ops.len());

        for (i, op) in ops.iter().enumerate() {
            let label = format!("fused_{i}");

            match op {
                GpuOp::HillSweep {
                    emax,
                    ec50,
                    n,
                    concentrations,
                } => {
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "concentration count fits u32"
                    )]
                    let n_concs = concentrations.len() as u32;
                    let byte_size = u64::from(n_concs) * 8;
                    let params = HillParams {
                        emax: *emax,
                        ec50: *ec50,
                        hill_n: *n,
                        n_concs,
                        _pad: 0,
                    };
                    let input_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&label),
                                contents: bytemuck::cast_slice(concentrations),
                                usage: wgpu::BufferUsages::STORAGE,
                            });
                    let output_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&label),
                        size: byte_size,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });
                    let params_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&label),
                                contents: bytemuck::bytes_of(&params),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                    let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&label),
                        size: byte_size,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });

                    let src = strip_f64_enable(shaders::HILL_DOSE_RESPONSE);
                    let shader = self
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(&label),
                            source: wgpu::ShaderSource::Wgsl(src.into()),
                        });
                    let pipeline =
                        self.device
                            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: Some(&label),
                                layout: None,
                                module: &shader,
                                entry_point: Some("main"),
                                cache: None,
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            });
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&label),
                        layout: &pipeline.get_bind_group_layout(0),
                        entries: &[
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
                    });

                    prepared.push(PreparedOp {
                        pipeline,
                        bind_group,
                        workgroups: (n_concs.div_ceil(WG_SIZE), 1, 1),
                        output_buf,
                        staging_buf,
                        output_bytes: byte_size,
                        kind: OpKind::Hill,
                    });
                }
                GpuOp::PopulationPkBatch {
                    n_patients,
                    dose_mg,
                    f_bioavail,
                    seed,
                } => {
                    #[expect(clippy::cast_possible_truncation, reason = "n_patients fits u32")]
                    let n = *n_patients as u32;
                    let byte_size = u64::from(n) * 8;
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "seed truncation to u32 documented"
                    )]
                    let params = PkParams {
                        n_patients: n,
                        base_seed: *seed as u32,
                        dose_mg: *dose_mg,
                        f_bioavail: *f_bioavail,
                    };
                    let output_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&label),
                        size: byte_size,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });
                    let params_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&label),
                                contents: bytemuck::bytes_of(&params),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                    let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&label),
                        size: byte_size,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });

                    let src = strip_f64_enable(shaders::POPULATION_PK);
                    let shader = self
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(&label),
                            source: wgpu::ShaderSource::Wgsl(src.into()),
                        });
                    let pipeline =
                        self.device
                            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: Some(&label),
                                layout: None,
                                module: &shader,
                                entry_point: Some("main"),
                                cache: None,
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            });
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&label),
                        layout: &pipeline.get_bind_group_layout(0),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: output_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: params_buf.as_entire_binding(),
                            },
                        ],
                    });

                    prepared.push(PreparedOp {
                        pipeline,
                        bind_group,
                        workgroups: (n.div_ceil(WG_SIZE), 1, 1),
                        output_buf,
                        staging_buf,
                        output_bytes: byte_size,
                        kind: OpKind::PopPk,
                    });
                }
                GpuOp::DiversityBatch { communities } => {
                    #[expect(clippy::cast_possible_truncation, reason = "community count fits u32")]
                    let n_communities = communities.len() as u32;
                    #[expect(clippy::cast_possible_truncation, reason = "stride fits u32")]
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
                    let input_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&label),
                                contents: bytemuck::cast_slice(&flat),
                                usage: wgpu::BufferUsages::STORAGE,
                            });
                    let output_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&label),
                        size: output_bytes,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });
                    let params_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&label),
                                contents: bytemuck::bytes_of(&params),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                    let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&label),
                        size: output_bytes,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });

                    let src = strip_f64_enable(shaders::DIVERSITY);
                    let shader = self
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(&label),
                            source: wgpu::ShaderSource::Wgsl(src.into()),
                        });
                    let pipeline =
                        self.device
                            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: Some(&label),
                                layout: None,
                                module: &shader,
                                entry_point: Some("main"),
                                cache: None,
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            });
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&label),
                        layout: &pipeline.get_bind_group_layout(0),
                        entries: &[
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
                    });

                    prepared.push(PreparedOp {
                        pipeline,
                        bind_group,
                        workgroups: (n_communities, 1, 1),
                        output_buf,
                        staging_buf,
                        output_bytes,
                        kind: OpKind::Diversity {
                            n_communities: communities.len(),
                        },
                    });
                }
            }
        }

        // Phase 2: Single encoder — all compute passes + all copy-to-staging
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fused_pipeline"),
            });
        for prep in &prepared {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fused_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&prep.pipeline);
            pass.set_bind_group(0, Some(&prep.bind_group), &[]);
            pass.dispatch_workgroups(prep.workgroups.0, prep.workgroups.1, prep.workgroups.2);
        }
        for prep in &prepared {
            encoder.copy_buffer_to_buffer(
                &prep.output_buf,
                0,
                &prep.staging_buf,
                0,
                prep.output_bytes,
            );
        }
        self.queue.submit(Some(encoder.finish()));

        // Phase 3: Single poll, then readback all staging buffers
        let mut receivers = Vec::with_capacity(prepared.len());
        for prep in &prepared {
            let slice = prep.staging_buf.slice(..);
            let (tx, rx) = tokio::sync::oneshot::channel();
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
            receivers.push(rx);
        }
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        let mut results = Vec::with_capacity(prepared.len());
        for (prep, rx) in prepared.iter().zip(receivers) {
            rx.await
                .map_err(|e| GpuError::Readback(format!("{e}")))?
                .map_err(|e| GpuError::Readback(format!("{e}")))?;

            let data = prep.staging_buf.slice(..).get_mapped_range();
            let raw: &[f64] = bytemuck::cast_slice(&data);

            let result = match &prep.kind {
                OpKind::Hill => GpuResult::HillSweep(raw.to_vec()),
                OpKind::PopPk => GpuResult::PopulationPkBatch(raw.to_vec()),
                OpKind::Diversity { n_communities } => {
                    let pairs: Vec<(f64, f64)> = (0..*n_communities)
                        .map(|i| (raw[i * 2], raw[i * 2 + 1]))
                        .collect();
                    GpuResult::DiversityBatch(pairs)
                }
            };
            drop(data);
            prep.staging_buf.unmap();
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hill_sweep_cpu() {
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 1.0,
            concentrations: vec![0.0, 5.0, 10.0, 20.0, 100.0],
        };
        if let GpuResult::HillSweep(results) = execute_cpu(&op) {
            assert_eq!(results.len(), 5);
            assert!(results[0].abs() < 1e-10, "E(0) = 0");
            assert!((results[2] - 50.0).abs() < 1e-10, "E(EC50) = Emax/2");
            assert!(results[4] > 90.0, "E(100) → Emax");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn population_pk_batch_cpu() {
        let op = GpuOp::PopulationPkBatch {
            n_patients: 100,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        };
        if let GpuResult::PopulationPkBatch(aucs) = execute_cpu(&op) {
            assert_eq!(aucs.len(), 100);
            assert!(aucs.iter().all(|&a| a > 0.0), "all AUC positive");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn diversity_batch_cpu() {
        let communities = vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]];
        let op = GpuOp::DiversityBatch { communities };
        if let GpuResult::DiversityBatch(results) = execute_cpu(&op) {
            assert_eq!(results.len(), 2);
            assert!(results[0].0 > results[1].0, "even > dominated Shannon");
        } else {
            panic!("wrong result type");
        }
    }

    #[test]
    fn shader_sources_loaded() {
        assert!(shaders::HILL_DOSE_RESPONSE.contains("hill_dose_response"));
        assert!(shaders::POPULATION_PK.contains("population_pk"));
        assert!(shaders::DIVERSITY.contains("diversity"));
    }

    #[test]
    fn memory_estimate_reasonable() {
        let op = GpuOp::PopulationPkBatch {
            n_patients: 10_000,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        };
        let mem = gpu_memory_estimate(&op);
        assert!(mem < 1_000_000, "10K patients < 1MB GPU memory");
    }

    #[test]
    fn hill_sweep_deterministic() {
        let op = GpuOp::HillSweep {
            emax: 100.0,
            ec50: 10.0,
            n: 2.0,
            concentrations: vec![1.0, 5.0, 10.0, 50.0],
        };
        let r1 = execute_cpu(&op);
        let r2 = execute_cpu(&op);
        if let (GpuResult::HillSweep(a), GpuResult::HillSweep(b)) = (&r1, &r2) {
            for (x, y) in a.iter().zip(b.iter()) {
                assert_eq!(
                    x.to_bits(),
                    y.to_bits(),
                    "CPU fallback must be bit-identical"
                );
            }
        }
    }
}
