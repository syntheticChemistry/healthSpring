// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Lint policy: workspace-level [lints] in root Cargo.toml.
// forbid(unsafe_code), deny(clippy::{all,pedantic,nursery,unwrap_used,expect_used}).

//! Runtime capability discovery (`probe_gpu`, `probe_npu`).

use crate::types::{GpuInfo, NpuInfo};

/// Discovered compute capabilities at runtime.
#[derive(Debug, Default)]
pub struct Capabilities {
    /// CPU execution is always treated as available for fallback paths.
    pub cpu: bool,
    /// GPU probe result when the `gpu` feature is enabled and an adapter exists.
    pub gpu: Option<GpuInfo>,
    /// NPU probe result when a neuromorphic device is integrated (`npu` path).
    pub npu: Option<NpuInfo>,
}

impl Capabilities {
    /// Discover available compute substrates at runtime.
    ///
    /// CPU is always available. GPU discovery attempts wgpu adapter
    /// enumeration when the `gpu` feature is enabled; otherwise returns
    /// None. NPU discovery is feature-gated behind `npu`.
    #[must_use]
    pub fn discover() -> Self {
        Self {
            cpu: true,
            gpu: Self::probe_gpu(),
            npu: Self::probe_npu(),
        }
    }

    /// Construct with explicitly injected capabilities (for testing or
    /// when biomeOS provides hardware topology).
    #[must_use]
    pub const fn with_known(gpu: Option<GpuInfo>, npu: Option<NpuInfo>) -> Self {
        Self {
            cpu: true,
            gpu,
            npu,
        }
    }

    #[cfg_attr(
        not(feature = "gpu"),
        expect(
            clippy::missing_const_for_fn,
            reason = "non-const when gpu feature is enabled"
        )
    )]
    fn probe_gpu() -> Option<GpuInfo> {
        #[cfg(feature = "gpu")]
        {
            let instance = wgpu::Instance::default();
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    ..Default::default()
                }));
            adapter.ok().map(|a| {
                let info = a.get_info();
                let fp64_native = a.features().contains(wgpu::Features::SHADER_F64);
                let f64_shared_mem_reliable = fp64_native;
                let precision = if !fp64_native {
                    crate::types::PrecisionRouting::Df64Only
                } else if f64_shared_mem_reliable {
                    crate::types::PrecisionRouting::F64Native
                } else {
                    crate::types::PrecisionRouting::F64NativeNoSharedMem
                };
                GpuInfo {
                    name: info.name,
                    fp64_native,
                    f64_shared_mem_reliable,
                    max_workgroups: a.limits().max_compute_workgroups_per_dimension,
                    precision,
                }
            })
        }
        #[cfg(not(feature = "gpu"))]
        {
            None
        }
    }

    /// Probe for neuromorphic accelerator.
    ///
    /// Returns `None` unconditionally — Akida driver integration is
    /// feature-gated behind `npu` and requires the `akida-driver` crate
    /// (which binds to the `BrainChip` `MetaTF` runtime). When the driver is
    /// available, this will query device topology and inference capacity.
    const fn probe_npu() -> Option<NpuInfo> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_always_available() {
        let caps = Capabilities::discover();
        assert!(caps.cpu, "CPU must always be available");
    }

    #[test]
    fn capability_discovery_returns_cpu_true() {
        let caps = Capabilities::discover();
        assert!(caps.cpu);
    }
}
