// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod batch_ops;
pub(crate) mod common;
pub(crate) mod diversity;
pub(crate) mod hill;
pub(crate) mod pop_pk;

// Re-exports for gpu/context.rs, gpu/fused.rs, and gpu/mod.rs
pub(crate) use batch_ops::{
    BeatClassifyParams, MmParams, ScfaGpuParams, execute_beat_classify_gpu, execute_mm_gpu,
    execute_scfa_gpu,
};
pub(crate) use common::{WG_SIZE, strip_f64_enable};
pub(crate) use diversity::{DivParams, execute_diversity_gpu};
pub(crate) use hill::{HillParams, execute_hill_gpu};
pub(crate) use pop_pk::{PkParams, execute_pop_pk_gpu};

use super::{GpuError, GpuOp, GpuResult};

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
#[expect(clippy::too_many_lines, reason = "dispatch match over GpuOp variants")]
pub async fn execute_gpu(op: &GpuOp) -> Result<GpuResult, GpuError> {
    // ── Tier A: barraCuda upstream ops (when feature-gated) ─────────
    #[cfg(feature = "barracuda-ops")]
    {
        use super::barracuda_rewire;
        let bc_device = barracuda_rewire::create_barracuda_device().await?;
        match op {
            GpuOp::HillSweep {
                emax,
                ec50,
                n,
                concentrations,
            } => {
                return barracuda_rewire::execute_hill_barracuda(
                    &bc_device,
                    *emax,
                    *ec50,
                    *n,
                    concentrations,
                );
            }
            GpuOp::PopulationPkBatch {
                n_patients,
                dose_mg,
                f_bioavail,
                seed,
            } => {
                return barracuda_rewire::execute_pop_pk_barracuda(
                    &bc_device,
                    *n_patients,
                    *dose_mg,
                    *f_bioavail,
                    *seed,
                );
            }
            GpuOp::DiversityBatch { communities } => {
                return barracuda_rewire::execute_diversity_barracuda(&bc_device, communities);
            }
            _ => {} // Tier B ops fall through to local WGSL path below
        }
    }

    // ── Local WGSL path (Tier A fallback + all Tier B ops) ──────────
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
