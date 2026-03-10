// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lognormal params, population PK CPU, `PatientExposure`.

use super::compartment::pk_oral_one_compartment;
use super::util::{auc_trapezoidal, find_cmax_tmax};

// ═══════════════════════════════════════════════════════════════════════
// Population PK (Exp005)
// ═══════════════════════════════════════════════════════════════════════

/// Lognormal sampling parameters.
#[derive(Debug, Clone, Copy)]
pub struct LognormalParam {
    pub typical: f64,
    pub cv: f64,
}

impl LognormalParam {
    /// Compute underlying normal μ and σ for lognormal with given typical (median) and CV.
    #[must_use]
    pub fn to_normal_params(self) -> (f64, f64) {
        let omega_sq = (1.0 + self.cv * self.cv).ln();
        let mu = self.typical.ln() - omega_sq / 2.0;
        let sigma = omega_sq.sqrt();
        (mu, sigma)
    }
}

/// Population PK parameters for baricitinib-like oral dosing.
pub mod pop_baricitinib {
    use super::LognormalParam;
    pub const CL: LognormalParam = LognormalParam {
        typical: 10.0,
        cv: 0.30,
    };
    pub const VD: LognormalParam = LognormalParam {
        typical: 80.0,
        cv: 0.25,
    };
    pub const KA: LognormalParam = LognormalParam {
        typical: 1.5,
        cv: 0.40,
    };
    pub const F_BIOAVAIL: f64 = 0.79;
    pub const DOSE_MG: f64 = 4.0;
}

/// Per-patient PK exposure metrics.
#[derive(Debug, Clone, Copy)]
pub struct PatientExposure {
    pub cmax: f64,
    pub tmax: f64,
    pub auc: f64,
}

/// Compute population PK for a cohort of patients (CPU, sequential).
///
/// For each patient, uses provided PK parameters and computes the oral
/// Bateman equation concentration-time curve.
///
/// # Panics
///
/// Panics if `cl_params`, `vd_params`, or `ka_params` length differs
/// from `n_patients`.
#[must_use]
pub fn population_pk_cpu(
    n_patients: usize,
    cl_params: &[f64],
    vd_params: &[f64],
    ka_params: &[f64],
    dose_mg: f64,
    f_bioavail: f64,
    times: &[f64],
) -> Vec<PatientExposure> {
    assert_eq!(cl_params.len(), n_patients);
    assert_eq!(vd_params.len(), n_patients);
    assert_eq!(ka_params.len(), n_patients);

    (0..n_patients)
        .map(|i| {
            let cl = cl_params[i];
            let vd = vd_params[i];
            let ka = ka_params[i];
            let ke = cl / vd;

            let concs: Vec<f64> = times
                .iter()
                .map(|&t| pk_oral_one_compartment(dose_mg, f_bioavail, vd, ka, ke, t))
                .collect();

            let (cmax, tmax) = find_cmax_tmax(times, &concs);
            let auc = auc_trapezoidal(times, &concs);

            PatientExposure { cmax, tmax, auc }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-10;

    #[test]
    fn lognormal_params_roundtrip() {
        let p = LognormalParam {
            typical: 10.0,
            cv: 0.30,
        };
        let (mu, sigma) = p.to_normal_params();
        let recovered_median = mu.exp();
        assert!((recovered_median - 10.0).abs() < 0.5, "median ~ typical");
        assert!(sigma > 0.0);
    }

    #[test]
    fn population_pk_cpu_basic() {
        let n = 5;
        let cl = vec![10.0, 12.0, 8.0, 11.0, 9.0];
        let vd = vec![80.0, 85.0, 75.0, 82.0, 78.0];
        let ka = vec![1.5, 1.8, 1.2, 1.6, 1.4];
        let times: Vec<f64> = (0..200).map(|i| 24.0 * f64::from(i) / 199.0).collect();
        let results = population_pk_cpu(n, &cl, &vd, &ka, 4.0, 0.79, &times);
        assert_eq!(results.len(), n);
        for r in &results {
            assert!(r.auc > 0.0, "AUC > 0");
            assert!(r.cmax > 0.0, "Cmax > 0");
            assert!(r.tmax >= 0.0, "Tmax ≥ 0");
        }
    }

    #[test]
    fn population_pk_higher_cl_lower_auc() {
        let times: Vec<f64> = (0..500).map(|i| 24.0 * f64::from(i) / 499.0).collect();
        let r_low = population_pk_cpu(1, &[5.0], &[80.0], &[1.5], 4.0, 0.79, &times);
        let r_high = population_pk_cpu(1, &[20.0], &[80.0], &[1.5], 4.0, 0.79, &times);
        assert!(r_low[0].auc > r_high[0].auc, "lower CL → higher AUC");
    }

    #[test]
    fn population_pk_c_zero_at_t0() {
        let times = vec![0.0, 1.0, 2.0];
        let results = population_pk_cpu(1, &[10.0], &[80.0], &[1.5], 4.0, 0.79, &times);
        let c0 = pk_oral_one_compartment(4.0, 0.79, 80.0, 1.5, 10.0 / 80.0, 0.0);
        assert!(c0.abs() < TOL, "C(0) = 0 for oral");
        assert!(results[0].cmax > 0.0);
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "loop indices small — safe for f64"
    )]
    fn population_pk_deterministic() {
        // Run the same population PK computation twice, verify identical results
        let n = 10;
        let cl: Vec<f64> = (0..n).map(|i| 8.0 + 0.3 * (i as f64)).collect();
        let vd: Vec<f64> = (0..n).map(|i| 70.0 + 2.0 * (i as f64)).collect();
        let ka: Vec<f64> = (0..n).map(|i| 1.0 + 0.1 * (i as f64)).collect();
        let times: Vec<f64> = (0..100).map(|i| 24.0 * f64::from(i) / 99.0).collect();

        let run1 = population_pk_cpu(n, &cl, &vd, &ka, 4.0, 0.79, &times);
        let run2 = population_pk_cpu(n, &cl, &vd, &ka, 4.0, 0.79, &times);

        for (r1, r2) in run1.iter().zip(run2.iter()) {
            assert_eq!(r1.auc.to_bits(), r2.auc.to_bits(), "AUC must be deterministic");
            assert_eq!(r1.cmax.to_bits(), r2.cmax.to_bits(), "Cmax must be deterministic");
            assert_eq!(r1.tmax.to_bits(), r2.tmax.to_bits(), "Tmax must be deterministic");
        }
    }
}
