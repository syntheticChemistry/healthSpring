// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lognormal params, population PK CPU, `PatientExposure`.

use super::compartment::pk_oral_one_compartment;
use super::util::{auc_trapezoidal, find_cmax_tmax};
use crate::rng::normal_sample;

// ═══════════════════════════════════════════════════════════════════════
// Population PK (Exp005)
// ═══════════════════════════════════════════════════════════════════════

/// Lognormal sampling parameters.
#[derive(Debug, Clone, Copy)]
pub struct LognormalParam {
    /// Population median (typical value) on the original scale.
    pub typical: f64,
    /// Coefficient of variation of the underlying lognormal distribution.
    pub cv: f64,
}

impl LognormalParam {
    /// Compute underlying normal μ and σ for lognormal with given typical (median) and CV.
    #[must_use]
    pub fn to_normal_params(self) -> (f64, f64) {
        let omega_sq = self.cv.mul_add(self.cv, 1.0).ln();
        let mu = self.typical.ln() - omega_sq / 2.0;
        let sigma = omega_sq.sqrt();
        (mu, sigma)
    }
}

/// Population PK parameters for baricitinib-like oral dosing.
pub mod pop_baricitinib {
    use super::LognormalParam;
    /// Lognormal prior for clearance (typical + IIV).
    pub const CL: LognormalParam = LognormalParam {
        typical: 10.0,
        cv: 0.30,
    };
    /// Lognormal prior for volume of distribution.
    pub const VD: LognormalParam = LognormalParam {
        typical: 80.0,
        cv: 0.25,
    };
    /// Lognormal prior for absorption rate constant Ka.
    pub const KA: LognormalParam = LognormalParam {
        typical: 1.5,
        cv: 0.40,
    };
    /// Oral bioavailability fraction F.
    pub const F_BIOAVAIL: f64 = 0.79;
    /// Reference oral dose in mg.
    pub const DOSE_MG: f64 = 4.0;
}

/// Per-patient PK exposure metrics.
#[derive(Debug, Clone, Copy)]
pub struct PatientExposure {
    /// Maximum concentration over the simulated schedule.
    pub cmax: f64,
    /// Time of Cmax.
    pub tmax: f64,
    /// AUC by trapezoidal rule over `times`.
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

/// Population PK Monte Carlo: sample CL, Vd, Ka from lognormal IIV, then compute exposure.
///
/// Uses the LCG PRNG with given `seed` for reproducible simulations.
/// Same seed produces bit-identical results across runs.
#[must_use]
#[expect(
    clippy::too_many_arguments,
    reason = "PK/PD simulation API requires all pharmacokinetic params"
)]
pub fn population_pk_monte_carlo(
    n_patients: usize,
    seed: u64,
    cl_param: LognormalParam,
    vd_param: LognormalParam,
    ka_param: LognormalParam,
    dose_mg: f64,
    f_bioavail: f64,
    times: &[f64],
) -> Vec<PatientExposure> {
    let (mu_cl, sigma_cl) = cl_param.to_normal_params();
    let (mu_vd, sigma_vd) = vd_param.to_normal_params();
    let (mu_ka, sigma_ka) = ka_param.to_normal_params();

    let mut rng = seed;
    let mut cl_params = Vec::with_capacity(n_patients);
    let mut vd_params = Vec::with_capacity(n_patients);
    let mut ka_params = Vec::with_capacity(n_patients);

    for _ in 0..n_patients {
        let (z_cl, next) = normal_sample(rng);
        rng = next;
        let (z_vd, next) = normal_sample(rng);
        rng = next;
        let (z_ka, next) = normal_sample(rng);
        rng = next;

        cl_params.push(sigma_cl.mul_add(z_cl, mu_cl).exp().max(0.1));
        vd_params.push(sigma_vd.mul_add(z_vd, mu_vd).exp().max(1.0));
        ka_params.push(sigma_ka.mul_add(z_ka, mu_ka).exp().max(0.01));
    }

    population_pk_cpu(
        n_patients, &cl_params, &vd_params, &ka_params, dose_mg, f_bioavail, times,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn lognormal_params_roundtrip() {
        let p = LognormalParam {
            typical: 10.0,
            cv: 0.30,
        };
        let (mu, sigma) = p.to_normal_params();
        let recovered_median = mu.exp();
        assert!(
            (recovered_median - 10.0).abs() < 10.0 * tolerances::LOGNORMAL_RECOVERY,
            "median ~ typical"
        );
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
        assert!(
            c0.abs() < tolerances::TEST_ASSERTION_TIGHT,
            "C(0) = 0 for oral"
        );
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
        let cl: Vec<f64> = (0..n).map(|i| 0.3f64.mul_add(i as f64, 8.0)).collect();
        let vd: Vec<f64> = (0..n).map(|i| 2.0f64.mul_add(i as f64, 70.0)).collect();
        let ka: Vec<f64> = (0..n).map(|i| 0.1f64.mul_add(i as f64, 1.0)).collect();
        let times: Vec<f64> = (0..100).map(|i| 24.0 * f64::from(i) / 99.0).collect();

        let run1 = population_pk_cpu(n, &cl, &vd, &ka, 4.0, 0.79, &times);
        let run2 = population_pk_cpu(n, &cl, &vd, &ka, 4.0, 0.79, &times);

        for (r1, r2) in run1.iter().zip(run2.iter()) {
            assert_eq!(
                r1.auc.to_bits(),
                r2.auc.to_bits(),
                "AUC must be deterministic"
            );
            assert_eq!(
                r1.cmax.to_bits(),
                r2.cmax.to_bits(),
                "Cmax must be deterministic"
            );
            assert_eq!(
                r1.tmax.to_bits(),
                r2.tmax.to_bits(),
                "Tmax must be deterministic"
            );
        }
    }

    #[test]
    fn population_pk_monte_carlo_deterministic() {
        // Same seed must produce bit-identical results across runs
        let seed = 42_u64;
        let times: Vec<f64> = (0..100).map(|i| 24.0 * f64::from(i) / 99.0).collect();

        let run1 = population_pk_monte_carlo(
            50,
            seed,
            pop_baricitinib::CL,
            pop_baricitinib::VD,
            pop_baricitinib::KA,
            pop_baricitinib::DOSE_MG,
            pop_baricitinib::F_BIOAVAIL,
            &times,
        );
        let run2 = population_pk_monte_carlo(
            50,
            seed,
            pop_baricitinib::CL,
            pop_baricitinib::VD,
            pop_baricitinib::KA,
            pop_baricitinib::DOSE_MG,
            pop_baricitinib::F_BIOAVAIL,
            &times,
        );

        assert_eq!(run1.len(), run2.len());
        for (r1, r2) in run1.iter().zip(run2.iter()) {
            assert_eq!(r1.auc.to_bits(), r2.auc.to_bits(), "AUC bit-identical");
            assert_eq!(r1.cmax.to_bits(), r2.cmax.to_bits(), "Cmax bit-identical");
            assert_eq!(r1.tmax.to_bits(), r2.tmax.to_bits(), "Tmax bit-identical");
        }
    }
}
