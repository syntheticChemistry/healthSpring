// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Lint policy: workspace-level [lints] in root Cargo.toml.
// forbid(unsafe_code), deny(clippy::{all,pedantic,nursery,unwrap_used,expect_used}).

//! Core dispatch and hardware capability types.

/// Available compute substrates for healthSpring workloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Substrate {
    /// Pure Rust CPU — always available.
    Cpu,
    /// barraCuda GPU via WGSL shaders — requires `gpu` feature.
    Gpu,
    /// Neuromorphic NPU (Akida AKD1000) — requires `npu` feature.
    Npu,
}

/// Workload descriptor for capability-based dispatch.
///
/// Primals describe their workload semantics; metalForge maps them to
/// the best available substrate without the caller knowing what hardware
/// exists. The `element_count` field drives the CPU/GPU threshold decision.
#[derive(Debug, Clone, Copy)]
pub enum Workload {
    /// Embarrassingly parallel — no inter-element dependencies (GPU ideal).
    PopulationPk {
        /// Virtual cohort size used for GPU offload thresholding.
        n_patients: u32,
    },
    /// Element-wise sweep — independent per element (GPU ideal).
    DoseResponse {
        /// Number of concentration points in the Hill or dose–response sweep.
        n_concentrations: u32,
    },
    /// Fused map-reduce over a collection (GPU possible above threshold).
    DiversityIndex {
        /// Sample count feeding diversity or reduce-style GPU routing.
        n_samples: u32,
    },
    /// Streaming time-series pipeline (NPU ideal, latency-critical).
    BiosignalDetect {
        /// Input sampling rate (Hz) for streaming/NPU affinity.
        sample_rate_hz: u32,
    },
    /// Multi-channel biosignal fusion (CPU or NPU).
    BiosignalFusion {
        /// Parallel physiological channels (e.g. ECG, PPG) to fuse.
        channels: u32,
    },
    /// Endocrine PK computation (CPU-only, analytical).
    EndocrinePk {
        /// Time grid resolution for analytical PK integration.
        n_timepoints: u32,
    },
    /// Batch Michaelis-Menten PK ODE per patient (GPU ideal).
    MichaelisMentenBatch {
        /// Parallel ODE trajectories (patients) in the batch.
        n_patients: u32,
    },
    /// Batch SCFA metabolic production per fiber input (GPU ideal).
    ScfaBatch {
        /// Fiber inputs or production sites in the microbiome batch.
        n_elements: u32,
    },
    /// Batch beat template-matching classification (GPU ideal).
    BeatClassifyBatch {
        /// Beats classified per batch for GPU dispatch sizing.
        n_beats: u32,
    },
    /// Small analytical computation — CPU always.
    Analytical,
}

impl Workload {
    /// The element count that drives the GPU-offload threshold decision.
    #[must_use]
    pub const fn element_count(&self) -> u32 {
        match *self {
            Self::PopulationPk { n_patients } | Self::MichaelisMentenBatch { n_patients } => {
                n_patients
            }
            Self::DoseResponse { n_concentrations } => n_concentrations,
            Self::DiversityIndex { n_samples } => n_samples,
            Self::BiosignalDetect { sample_rate_hz } => sample_rate_hz,
            Self::BiosignalFusion { channels } => channels,
            Self::EndocrinePk { n_timepoints } => n_timepoints,
            Self::ScfaBatch { n_elements } => n_elements,
            Self::BeatClassifyBatch { n_beats } => n_beats,
            Self::Analytical => 0,
        }
    }

    /// Whether this workload benefits from streaming/low-latency NPU.
    #[must_use]
    pub const fn prefers_npu(&self) -> bool {
        matches!(
            self,
            Self::BiosignalDetect { .. } | Self::BiosignalFusion { .. }
        )
    }
}

/// Precision routing advice for f64 GPU workloads.
///
/// Mirrors toadStool's `PrecisionRoutingAdvice` (S128) and barraCuda's
/// `Fp64Strategy`. Determines how shaders should handle f64 arithmetic
/// based on discovered hardware capabilities.
///
/// See also: `GPU_F64_NUMERICAL_STABILITY` in wateringHole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionRouting {
    /// Hardware has native f64 ALUs and reliable shared memory.
    /// Use `enable f64;` WGSL shaders directly.
    F64Native,
    /// Hardware has native f64 compute but shared-memory reduction
    /// returns zeros (naga/SPIR-V bug on some drivers).
    /// Avoid f64 workgroup shared memory; use per-thread accumulators.
    F64NativeNoSharedMem,
    /// No native f64; use double-float emulation (DF64).
    /// coralReef lowers f64 ops to paired f32 arithmetic.
    Df64Only,
    /// f64 compute returns zeros (e.g., NVK Volta).
    /// Fall back to f32 shaders with documented precision loss.
    F32Only,
}

/// Discovered GPU capabilities.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// Reported adapter or device name from the graphics runtime.
    pub name: String,
    /// Whether `SHADER_F64` (native double) is available for WGSL.
    pub fp64_native: bool,
    /// Whether f64 shared-memory reductions are reliable on this GPU.
    pub f64_shared_mem_reliable: bool,
    /// `wgpu` limit: max compute workgroups per dimension for dispatch sizing.
    pub max_workgroups: u32,
    /// Routing advice for f64 workloads on this GPU.
    pub precision: PrecisionRouting,
}

/// Discovered NPU capabilities.
#[derive(Debug, Clone)]
pub struct NpuInfo {
    /// Enumerated neuromorphic device label (e.g. Akida SKU).
    pub name: String,
    /// Peak sustained inference rate for latency/capacity estimates.
    pub max_inference_rate_hz: u32,
}

/// Configurable thresholds for GPU offload decisions.
///
/// Below these thresholds, CPU is faster due to dispatch overhead.
/// Callers (or biomeOS) can tune these based on profiled hardware.
#[derive(Debug, Clone)]
pub struct DispatchThresholds {
    /// Minimum cohort/parallel element count before `PopulationPk`-style work uses GPU.
    pub parallel_gpu_min: u32,
    /// Minimum sweep length before dose–response / element-wise work uses GPU.
    pub sweep_gpu_min: u32,
    /// Minimum collection size before map–reduce / diversity-style work uses GPU.
    pub reduce_gpu_min: u32,
}

impl Default for DispatchThresholds {
    fn default() -> Self {
        Self {
            parallel_gpu_min: 100,
            sweep_gpu_min: 1000,
            reduce_gpu_min: 500,
        }
    }
}
