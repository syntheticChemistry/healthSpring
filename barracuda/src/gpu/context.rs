// SPDX-License-Identifier: AGPL-3.0-or-later

use super::dispatch::{
    DivParams, HillParams, PkParams, WG_SIZE, execute_diversity_gpu, execute_hill_gpu,
    execute_pop_pk_gpu, strip_f64_enable,
};
use super::{GpuError, GpuOp, GpuResult, shaders};

/// Persistent GPU context: one device, one queue, all shaders pre-compiled.
///
/// Eliminates per-dispatch device creation overhead. The fused pipeline
/// dispatches all operations in a single command encoder: upload once,
/// N compute passes, readback once — the unidirectional pipeline pattern
/// required for field-deployed devices (e.g., Raspberry Pi + eGPU).
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_name: String,
}

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
