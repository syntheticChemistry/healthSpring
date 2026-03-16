// SPDX-License-Identifier: AGPL-3.0-or-later
//! Population Monte Carlo: virtual cohort for percentile context.

use super::assessment::assess_patient_with_config;
use super::{DiagnosticConfig, PatientProfile, PopulationResult};
use crate::endocrine;

fn profile_cv_lognormal(typical: f64, cv: f64, z: f64) -> f64 {
    let (mu, sigma) = endocrine::lognormal_params(typical, cv);
    sigma.mul_add(z, mu).exp()
}

/// Run population Monte Carlo with published-literature defaults.
#[must_use]
pub fn population_montecarlo(
    base_profile: &PatientProfile,
    n_patients: usize,
    seed: u64,
) -> PopulationResult {
    population_montecarlo_with_config(base_profile, n_patients, seed, &DiagnosticConfig::default())
}

/// Run population Monte Carlo with custom configuration.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "patient indices fit f64")]
pub fn population_montecarlo_with_config(
    base_profile: &PatientProfile,
    n_patients: usize,
    seed: u64,
    cfg: &DiagnosticConfig,
) -> PopulationResult {
    let base_assessment = assess_patient_with_config(base_profile, cfg);
    let base_risk = base_assessment.composite_risk;

    let mut rng = seed;
    let mut risks = Vec::with_capacity(n_patients);

    for _ in 0..n_patients {
        let (z, next_rng) = crate::rng::normal_sample(rng);
        rng = next_rng;

        let age_var = (profile_cv_lognormal(base_profile.age_years, cfg.mc_age_cv, z)).max(18.0);
        let weight_var =
            profile_cv_lognormal(base_profile.weight_kg, cfg.mc_weight_cv, z).max(30.0);

        let (z2, next_rng2) = crate::rng::normal_sample(rng);
        rng = next_rng2;

        let t_var = base_profile
            .testosterone_ng_dl
            .map(|t| profile_cv_lognormal(t, cfg.mc_testosterone_cv, z2).max(10.0));

        let virtual_profile = PatientProfile {
            age_years: age_var,
            weight_kg: weight_var,
            testosterone_ng_dl: t_var,
            ..base_profile.clone()
        };

        let assessment = assess_patient_with_config(&virtual_profile, cfg);
        risks.push(assessment.composite_risk);
    }

    let mean = risks.iter().sum::<f64>() / n_patients as f64;
    let variance = risks.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_patients as f64;
    let std = variance.sqrt();

    let below_count = risks.iter().filter(|&&r| r <= base_risk).count();
    let percentile = below_count as f64 / n_patients as f64 * 100.0;

    PopulationResult {
        n_patients,
        composite_risks: risks,
        mean_risk: mean,
        std_risk: std,
        patient_percentile: percentile,
    }
}
