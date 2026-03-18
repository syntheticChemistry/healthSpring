// SPDX-License-Identifier: AGPL-3.0-or-later

use super::dispatch::{
    MmParams, execute_beat_classify_gpu, execute_diversity_gpu, execute_hill_gpu, execute_mm_gpu,
    execute_pop_pk_gpu, execute_scfa_gpu,
};
use super::fused::{self, PreparedOp};
use super::{GpuError, GpuOp, GpuResult};

/// Persistent GPU context: one device, one queue, all shaders pre-compiled.
///
/// Eliminates per-dispatch device creation overhead. The fused pipeline
/// dispatches all operations in a single command encoder: upload once,
/// N compute passes, readback once — the unidirectional pipeline pattern
/// required for field-deployed devices (e.g., Raspberry Pi + eGPU).
///
/// When `barracuda-ops` is enabled, Tier A ops (Hill, `PopPK`, Diversity)
/// are delegated to barraCuda's canonical GPU implementations via `execute()`.
/// The fused pipeline still uses local WGSL until `TensorSession` is available.
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_name: String,
    #[cfg(feature = "barracuda-ops")]
    barracuda_device: Option<std::sync::Arc<barracuda::device::WgpuDevice>>,
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

        #[cfg(feature = "barracuda-ops")]
        let barracuda_device = super::barracuda_rewire::create_barracuda_device()
            .await
            .ok();

        Ok(Self {
            device,
            queue,
            adapter_name,
            #[cfg(feature = "barracuda-ops")]
            barracuda_device,
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
        #[cfg(feature = "barracuda-ops")]
        if let Some(bc) = &self.barracuda_device {
            return Self::execute_via_barracuda(bc, op);
        }

        self.execute_local_wgsl(op).await
    }

    /// Delegate all ops to barraCuda canonical GPU implementations.
    ///
    /// Tier A (Hill, PopPK, Diversity) — absorbed and validated upstream.
    /// Tier B (MM, SCFA, BeatClassify) — absorbed upstream, rewired here.
    #[cfg(feature = "barracuda-ops")]
    fn execute_via_barracuda(
        bc: &std::sync::Arc<barracuda::device::WgpuDevice>,
        op: &GpuOp,
    ) -> Result<GpuResult, GpuError> {
        use super::barracuda_rewire;
        match op {
            GpuOp::HillSweep {
                emax,
                ec50,
                n,
                concentrations,
            } => barracuda_rewire::execute_hill_barracuda(bc, *emax, *ec50, *n, concentrations),
            GpuOp::PopulationPkBatch {
                n_patients,
                dose_mg,
                f_bioavail,
                seed,
            } => barracuda_rewire::execute_pop_pk_barracuda(
                bc,
                *n_patients,
                *dose_mg,
                *f_bioavail,
                *seed,
            ),
            GpuOp::DiversityBatch { communities } => {
                barracuda_rewire::execute_diversity_barracuda(bc, communities)
            }
            GpuOp::MichaelisMentenBatch {
                vmax,
                km,
                vd,
                dt,
                n_steps,
                n_patients,
                seed,
            } => barracuda_rewire::execute_mm_batch_barracuda(
                bc,
                *vmax,
                *km,
                *vd,
                *dt,
                *n_steps,
                *n_patients,
                *seed,
            ),
            GpuOp::ScfaBatch {
                params,
                fiber_inputs,
            } => barracuda_rewire::execute_scfa_batch_barracuda(bc, params, fiber_inputs),
            GpuOp::BeatClassifyBatch { beats, templates } => {
                barracuda_rewire::execute_beat_classify_barracuda(bc, beats, templates)
            }
        }
    }

    /// Execute via local WGSL shaders (fallback path without barraCuda).
    async fn execute_local_wgsl(&self, op: &GpuOp) -> Result<GpuResult, GpuError> {
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
                execute_mm_gpu(&self.device, &self.queue, &mm_params).await
            }
            GpuOp::ScfaBatch {
                params,
                fiber_inputs,
            } => execute_scfa_gpu(&self.device, &self.queue, params, fiber_inputs).await,
            GpuOp::BeatClassifyBatch { beats, templates } => {
                execute_beat_classify_gpu(&self.device, &self.queue, beats, templates).await
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
    pub async fn execute_fused(&self, ops: &[GpuOp]) -> Result<Vec<GpuResult>, GpuError> {
        if ops.is_empty() {
            return Ok(Vec::new());
        }

        let prepared = self.prepare_all_ops(ops);

        self.submit_compute_passes(&prepared);

        self.readback_all(&prepared).await
    }

    fn prepare_all_ops(&self, ops: &[GpuOp]) -> Vec<PreparedOp> {
        let mut prepared: Vec<PreparedOp> = Vec::with_capacity(ops.len());

        for (i, op) in ops.iter().enumerate() {
            let label = format!("fused_{i}");
            let prep = match op {
                GpuOp::HillSweep {
                    emax,
                    ec50,
                    n,
                    concentrations,
                } => fused::prepare_hill(&self.device, &label, *emax, *ec50, *n, concentrations),
                GpuOp::PopulationPkBatch {
                    n_patients,
                    dose_mg,
                    f_bioavail,
                    seed,
                } => fused::prepare_pop_pk(
                    &self.device,
                    &label,
                    *n_patients,
                    *dose_mg,
                    *f_bioavail,
                    *seed,
                ),
                GpuOp::DiversityBatch { communities } => {
                    fused::prepare_diversity(&self.device, &label, communities)
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
                    let params = MmParams {
                        vmax: *vmax,
                        km: *km,
                        vd: *vd,
                        dt: *dt,
                        n_steps: *n_steps,
                        n_patients: *n_patients,
                        base_seed: *seed,
                        _pad: 0,
                    };
                    fused::prepare_michaelis_menten(&self.device, &label, &params)
                }
                GpuOp::ScfaBatch {
                    params,
                    fiber_inputs,
                } => fused::prepare_scfa(&self.device, &label, params, fiber_inputs),
                GpuOp::BeatClassifyBatch { beats, templates } => {
                    fused::prepare_beat_classify(&self.device, &label, beats, templates)
                }
            };
            prepared.push(prep);
        }

        prepared
    }

    fn submit_compute_passes(&self, prepared: &[PreparedOp]) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fused_pipeline"),
            });

        for prep in prepared {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("fused_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&prep.pipeline);
            pass.set_bind_group(0, Some(&prep.bind_group), &[]);
            pass.dispatch_workgroups(prep.workgroups.0, prep.workgroups.1, prep.workgroups.2);
        }

        for prep in prepared {
            encoder.copy_buffer_to_buffer(
                &prep.output_buf,
                0,
                &prep.staging_buf,
                0,
                prep.output_bytes,
            );
        }

        self.queue.submit(Some(encoder.finish()));
    }

    async fn readback_all(&self, prepared: &[PreparedOp]) -> Result<Vec<GpuResult>, GpuError> {
        let mut receivers = Vec::with_capacity(prepared.len());
        for prep in prepared {
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
            let result = fused::decode_readback(&prep.kind, raw);
            drop(data);
            prep.staging_buf.unmap();
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::gpu::dispatch::{
        BeatClassifyParams, DivParams, HillParams, MmParams, PkParams, ScfaGpuParams, WG_SIZE,
    };

    fn hill_buffer_bytes(n_concs: u32) -> u64 {
        u64::from(n_concs) * 8
    }

    fn workgroup_count_1d(n: u32) -> u32 {
        n.div_ceil(WG_SIZE)
    }

    #[test]
    fn hill_params_layout() {
        let params = HillParams {
            emax: 100.0,
            ec50: 10.0,
            hill_n: 1.5,
            n_concs: 50,
            _pad: 0,
        };
        let bytes = bytemuck::bytes_of(&params);
        assert!(
            bytes.len() >= 32,
            "HillParams must fit 4×f64 + 2×u32 for WGSL uniform"
        );
        assert_eq!(params.n_concs, 50);
    }

    #[test]
    fn pk_params_layout() {
        let params = PkParams {
            n_patients: 1000,
            base_seed: 42,
            dose_mg: 4.0,
            f_bioavail: 0.79,
        };
        let bytes = bytemuck::bytes_of(&params);
        assert!(
            bytes.len() >= 16,
            "PkParams must fit 2×u32 + 2×f64 for WGSL uniform"
        );
    }

    #[test]
    fn div_params_layout() {
        let params = DivParams {
            n_communities: 10,
            stride: 20,
            _pad0: 0,
            _pad1: 0,
        };
        let bytes = bytemuck::bytes_of(&params);
        assert_eq!(bytes.len(), 16, "DivParams: 4×u32 = 16 bytes");
    }

    #[test]
    fn mm_params_layout() {
        let params = MmParams {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 2000,
            n_patients: 64,
            base_seed: 42,
            _pad: 0,
        };
        let bytes = bytemuck::bytes_of(&params);
        assert!(
            bytes.len() >= 40,
            "MmParams must fit 4×f64 + 3×u32 for WGSL uniform"
        );
    }

    #[test]
    fn scfa_gpu_params_layout() {
        let params = ScfaGpuParams {
            vmax_acetate: 1.0,
            km_acetate: 2.0,
            vmax_propionate: 0.5,
            km_propionate: 1.0,
            vmax_butyrate: 0.3,
            km_butyrate: 0.8,
            n_elements: 100,
            _pad: 0,
        };
        let bytes = bytemuck::bytes_of(&params);
        assert!(
            bytes.len() >= 48,
            "ScfaGpuParams must fit 6×f64 + 2×u32 for WGSL uniform"
        );
    }

    #[test]
    fn beat_classify_params_layout() {
        let params = BeatClassifyParams {
            n_beats: 100,
            n_templates: 3,
            window_size: 41,
            _pad: 0,
        };
        let bytes = bytemuck::bytes_of(&params);
        assert_eq!(bytes.len(), 16, "BeatClassifyParams: 4×u32 = 16 bytes");
    }

    #[test]
    fn hill_buffer_bytes_known_values() {
        assert_eq!(hill_buffer_bytes(0), 0);
        assert_eq!(hill_buffer_bytes(1), 8);
        assert_eq!(hill_buffer_bytes(256), 2048);
        assert_eq!(hill_buffer_bytes(1000), 8000);
    }

    #[test]
    fn workgroup_count_1d_known_values() {
        assert_eq!(workgroup_count_1d(0), 0);
        assert_eq!(workgroup_count_1d(1), 1);
        assert_eq!(workgroup_count_1d(256), 1);
        assert_eq!(workgroup_count_1d(257), 2);
        assert_eq!(workgroup_count_1d(512), 2);
        assert_eq!(workgroup_count_1d(513), 3);
    }

    #[test]
    fn diversity_output_bytes() {
        let n_communities: u32 = 5;
        let output_bytes = u64::from(n_communities) * 2 * 8;
        assert_eq!(output_bytes, 80, "5 communities × 2 f64s × 8 bytes");
    }

    #[test]
    fn scfa_output_bytes() {
        let n_elements: u32 = 10;
        let output_bytes = u64::from(n_elements) * 3 * 8;
        assert_eq!(output_bytes, 240, "10 elements × 3 f64s × 8 bytes");
    }
}
