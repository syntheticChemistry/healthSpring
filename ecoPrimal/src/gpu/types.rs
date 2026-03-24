// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU operation types: [`GpuOp`], [`GpuResult`], and [`GpuError`].

/// A GPU-dispatchable operation with input/output buffers.
#[derive(Debug, Clone)]
pub enum GpuOp {
    /// Vectorized Hill dose-response: compute E(c) for many concentrations.
    HillSweep {
        /// Maximum effect plateau (same units as the response axis).
        emax: f64,
        /// Half-maximal concentration (Hill EC₅₀).
        ec50: f64,
        /// Hill slope (cooperativity exponent).
        n: f64,
        /// Concentration grid evaluated in parallel.
        concentrations: Vec<f64>,
    },
    /// Batch population PK: simulate N patients in parallel.
    PopulationPkBatch {
        /// Virtual cohort size (parallel AUC outputs).
        n_patients: usize,
        /// Dose per patient (mg) driving AUC scaling.
        dose_mg: f64,
        /// Oral or depot bioavailability fraction (0–1).
        f_bioavail: f64,
        /// PRNG seed for inter-patient variability.
        seed: u64,
    },
    /// Batch diversity indices for multiple communities.
    DiversityBatch {
        /// One abundance vector per community (non-negative, summing to 1 is typical).
        communities: Vec<Vec<f64>>,
    },
    /// Batch Michaelis-Menten PK: parallel Euler ODE per patient.
    MichaelisMentenBatch {
        /// Maximum elimination velocity scale (model-specific units).
        vmax: f64,
        /// Michaelis constant for substrate/concentration scale.
        km: f64,
        /// Volume of distribution (L).
        vd: f64,
        /// Fixed ODE time step (same units as total simulated time).
        dt: f64,
        /// Number of Euler steps per simulation.
        n_steps: u32,
        /// Parallel patient count.
        n_patients: u32,
        /// Base seed for deterministic per-patient parameter jitter.
        seed: u32,
    },
    /// Batch SCFA production: element-wise Michaelis-Menten per fiber input.
    ScfaBatch {
        /// Shared microbiome parameters for all fiber rows.
        params: crate::microbiome::ScfaParams,
        /// Fiber intake values (g/day or model units) per output row.
        fiber_inputs: Vec<f64>,
    },
    /// Batch beat classification: template correlation per beat window.
    BeatClassifyBatch {
        /// One waveform window per beat to classify.
        beats: Vec<Vec<f64>>,
        /// Reference templates (order matches `BeatClass` mapping in CPU path).
        templates: Vec<Vec<f64>>,
    },
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
    /// Michaelis-Menten batch: AUC per patient.
    MichaelisMentenBatch(Vec<f64>),
    /// SCFA batch: (acetate, propionate, butyrate) per fiber input.
    ScfaBatch(Vec<(f64, f64, f64)>),
    /// Beat classify batch: (`template_index`, correlation) per beat.
    BeatClassifyBatch(Vec<(u32, f64)>),
}

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
