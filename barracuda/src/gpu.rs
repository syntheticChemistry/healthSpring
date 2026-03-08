// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU-dispatchable operations for healthSpring.
//!
//! Defines operations that can execute on either CPU (pure Rust) or GPU
//! (via barraCuda WGSL shaders). CPU fallback is always available.
//! GPU dispatch is activated by the `gpu` feature flag.
//!
//! ## Shader Mapping
//!
//! | healthSpring Op | barraCuda Shader | Use Case |
//! |-----------------|------------------|----------|
//! | `HillSweep` | `batched_elementwise_f64.wgsl` | Exp001 vectorized |
//! | `PopulationPkBatch` | custom `population_pk_f64.wgsl` | Exp005/036 Monte Carlo |
//! | `DiversityBatch` | `mean_variance_f64.wgsl` + map | Exp010 batch |
//! | `FftBiosignal` | `fft_radix2_f64.wgsl` | Exp020 real-time |
//! | `AndersonEigensolve` | `anderson_lyapunov_f64.wgsl` | Exp011/037 |

use crate::microbiome;
use crate::pkpd;

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
            // Simplified: use lognormal variation on CL and Vd
            let mut aucs = Vec::with_capacity(*n_patients);
            let mut rng_state = *seed;
            for _ in 0..*n_patients {
                // Simple LCG for patient variation
                rng_state = rng_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let u = (rng_state >> 33) as f64 / f64::from(u32::MAX);
                let cl_factor = 0.5 + u; // CL varies 0.5x to 1.5x
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

/// Shader descriptor: maps a `GpuOp` to its barraCuda WGSL shader path.
#[must_use]
pub fn shader_for_op(op: &GpuOp) -> &'static str {
    match op {
        GpuOp::HillSweep { .. } => "shaders/science/batched_elementwise_f64.wgsl",
        GpuOp::PopulationPkBatch { .. } => "shaders/health/population_pk_f64.wgsl",
        GpuOp::DiversityBatch { .. } => "shaders/reduce/mean_variance_f64.wgsl",
    }
}

/// Estimate GPU memory requirement for an operation (bytes).
#[must_use]
pub fn gpu_memory_estimate(op: &GpuOp) -> u64 {
    match op {
        GpuOp::HillSweep { concentrations, .. } => {
            // Input buffer + output buffer, each f64
            (concentrations.len() as u64) * 8 * 2
        }
        GpuOp::PopulationPkBatch { n_patients, .. } => {
            // Parameters per patient + output AUC
            (*n_patients as u64) * 8 * 5
        }
        GpuOp::DiversityBatch { communities } => {
            let total_species: usize = communities.iter().map(Vec::len).sum();
            (total_species as u64) * 8 + (communities.len() as u64) * 16
        }
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
    fn shader_mapping() {
        let op = GpuOp::HillSweep {
            emax: 1.0,
            ec50: 1.0,
            n: 1.0,
            concentrations: vec![],
        };
        assert!(shader_for_op(&op).contains("elementwise"));
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
