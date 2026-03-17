// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sovereign GPU dispatch via `barraCuda` `CoralReefDevice`.
//!
//! Routes GPU compute through the sovereign pipeline:
//! `barraCuda` WGSL → `coralReef` compile → `toadStool` dispatch → native binary
//!
//! This replaces the `strip_f64_enable` WGSL preprocessor workaround
//! with `coralReef`'s native f64 lowering
//! (DFMA on NVIDIA, hardware on AMD).
//!
//! ## Dispatch order
//!
//! 1. `try_sovereign_dispatch()` attempts the sovereign path when coralReef
//!    and toadStool are discoverable.
//! 2. Falls back to the existing wgpu path (which uses `strip_f64_enable`)
//!    if sovereign is unavailable or dispatch fails.
//!
//! ## Discovery
//!
//! Uses [`discover_shader_compiler`](crate::ipc::socket::discover_shader_compiler)
//! from `ipc/socket.rs` as a fast-path hint: if no shader primal is visible
//! in the socket directory, sovereign is skipped without probing barraCuda.
//!
//! ## Cache requirement
//!
//! `CoralReefDevice` requires pre-compiled native binaries in the coral cache.
//! The first wgpu run can spawn `spawn_coral_compile` in the background;
//! subsequent runs may hit the cache and use sovereign dispatch.

use super::{GpuOp, GpuResult};

/// Attempt sovereign GPU dispatch via CoralReefDevice.
///
/// Returns `None` if the sovereign path should not be tried (feature disabled,
/// coralReef not discoverable, or CoralReefDevice unavailable). Returns
/// `Some(Ok(result))` on success, `Some(Err(e))` when sovereign was attempted
/// but failed (e.g. cache miss, toadStool unreachable).
///
/// Callers should fall back to the wgpu path when this returns `None` or
/// `Some(Err(_))`.
#[cfg(feature = "gpu")]
#[cfg(feature = "sovereign-dispatch")]
pub fn try_sovereign_dispatch(op: &GpuOp) -> Option<Result<GpuResult, super::GpuError>> {
    use crate::ipc::socket::discover_shader_compiler;

    if discover_shader_compiler().is_none() {
        return None;
    }

    let device = match barracuda::device::CoralReefDevice::with_auto_device() {
        Ok(d) => d,
        Err(_) => return None,
    };

    if !device.has_compiler() || !device.has_dispatch() {
        return None;
    }

    Some(dispatch_via_sovereign(&device, op))
}

/// Attempt sovereign GPU dispatch (sovereign-dispatch feature disabled).
///
/// Always returns `None` when the feature is off.
#[cfg(feature = "gpu")]
#[cfg(not(feature = "sovereign-dispatch"))]
#[must_use]
pub const fn try_sovereign_dispatch(_op: &GpuOp) -> Option<Result<GpuResult, super::GpuError>> {
    None
}

/// Attempt sovereign GPU dispatch (gpu feature disabled).
#[cfg(not(feature = "gpu"))]
#[must_use]
pub const fn try_sovereign_dispatch(_op: &GpuOp) -> Option<Result<GpuResult, super::GpuError>> {
    None
}

#[cfg(all(feature = "gpu", feature = "sovereign-dispatch"))]
fn dispatch_via_sovereign(
    device: &barracuda::device::CoralReefDevice,
    op: &GpuOp,
) -> Result<GpuResult, super::GpuError> {
    use barracuda::device::backend::{BufferBinding, DispatchDescriptor, GpuBackend};

    use super::GpuError;
    use super::dispatch::WG_SIZE;
    use super::shaders;

    match op {
        GpuOp::HillSweep {
            emax,
            ec50,
            n,
            concentrations,
        } => {
            let n_concs = concentrations.len() as u32;
            let byte_size = u64::from(n_concs) * 8;

            #[repr(C)]
            #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
            struct HillParams {
                emax: f64,
                ec50: f64,
                hill_n: f64,
                n_concs: u32,
                _pad: u32,
            }
            let params = HillParams {
                emax: *emax,
                ec50: *ec50,
                hill_n: *n,
                n_concs,
                _pad: 0,
            };

            let input_buf = device
                .alloc_buffer_init("hill_input", bytemuck::cast_slice(concentrations))
                .map_err(|e| GpuError::Dispatch(format!("{e}")))?;
            let output_buf = device
                .alloc_buffer("hill_output", byte_size)
                .map_err(|e| GpuError::Dispatch(format!("{e}")))?;
            let params_buf = device
                .alloc_uniform("hill_params", bytemuck::bytes_of(&params))
                .map_err(|e| GpuError::Dispatch(format!("{e}")))?;

            let workgroups = (n_concs.div_ceil(WG_SIZE), 1, 1);
            let desc = DispatchDescriptor {
                label: "hill_dose_response",
                shader_source: shaders::HILL_DOSE_RESPONSE,
                entry_point: "main",
                bindings: vec![
                    BufferBinding {
                        index: 0,
                        buffer: &input_buf,
                        read_only: true,
                        is_uniform: false,
                    },
                    BufferBinding {
                        index: 1,
                        buffer: &output_buf,
                        read_only: false,
                        is_uniform: false,
                    },
                    BufferBinding {
                        index: 2,
                        buffer: &params_buf,
                        read_only: true,
                        is_uniform: true,
                    },
                ],
                workgroups,
                f64_shader: true,
                df64_shader: false,
            };

            device
                .dispatch_compute(desc)
                .map_err(|e| GpuError::Dispatch(format!("{e}")))?;

            let data = device
                .download(&output_buf, byte_size)
                .map_err(|e| GpuError::Readback(format!("{e}")))?;
            let results: Vec<f64> = bytemuck::cast_slice(&data).to_vec();
            Ok(GpuResult::HillSweep(results))
        }
        _ => Err(GpuError::Dispatch(
            "sovereign dispatch: only HillSweep supported; other ops use wgpu path".into(),
        )),
    }
}
