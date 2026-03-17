// SPDX-License-Identifier: AGPL-3.0-or-later
//! Feline pharmacology models for comparative medicine.
//!
//! Hyperthyroidism is the most common endocrine disease in cats.
//! Treatment with methimazole exhibits capacity-limited (Michaelis-Menten)
//! PK — the same nonlinear kinetics validated for phenytoin in Exp077.
//! The math is identical; parameters change for the feline species.
//!
//! References:
//! - Trepanier LA (2006) *JVIM* 20:18 — Methimazole pharmacokinetics in cats
//! - Peterson ME (2012) *JFMS* 14:804 — Feline hyperthyroidism management

use serde::{Deserialize, Serialize};

/// Feline methimazole PK parameters (Michaelis-Menten).
///
/// Methimazole is a thionamide antithyroid drug. In cats, elimination is
/// capacity-limited at therapeutic doses, exhibiting classic nonlinear PK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FelineMethimazoleParams {
    /// Maximum elimination rate (mg/day).
    pub vmax: f64,
    /// Michaelis constant (mg/L).
    pub km: f64,
    /// Volume of distribution (L).
    pub vd: f64,
    /// Typical cat body weight (kg).
    pub body_weight_kg: f64,
}

/// Published feline methimazole PK parameters.
///
/// Trepanier 2006: Vmax ~3.6 mg/day, Km ~1.5 mg/L in a 4.5 kg cat.
/// Vd ~1.2 L (~0.27 L/kg). Oral bioavailability ~0.93.
pub const FELINE_METHIMAZOLE: FelineMethimazoleParams = FelineMethimazoleParams {
    vmax: 3.6,
    km: 1.5,
    vd: 1.2,
    body_weight_kg: 4.5,
};

/// Human methimazole PK parameters for cross-species comparison.
///
/// Standard human: Vmax ~30 mg/day, Km ~2.0 mg/L, Vd ~40 L.
pub const HUMAN_METHIMAZOLE: FelineMethimazoleParams = FelineMethimazoleParams {
    vmax: 30.0,
    km: 2.0,
    vd: 40.0,
    body_weight_kg: 70.0,
};

/// Simulate feline methimazole PK via Euler integration.
///
/// Identical to `pkpd::nonlinear::mm_pk_simulate` but parameterized for
/// the feline case. Returns `(times, concentrations)`.
///
/// `dC/dt = -Vmax × C / (Vd × (Km + C))`
#[must_use]
pub fn methimazole_simulate(
    params: &FelineMethimazoleParams,
    dose_mg: f64,
    t_end_hr: f64,
    dt_hr: f64,
) -> (Vec<f64>, Vec<f64>) {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t_end_hr / dt_hr is small positive"
    )]
    let n_steps = (t_end_hr / dt_hr).ceil() as usize;
    let mut times = Vec::with_capacity(n_steps + 1);
    let mut concs = Vec::with_capacity(n_steps + 1);

    let mut c = dose_mg / params.vd;
    let mut t = 0.0;
    times.push(t);
    concs.push(c);

    let dt_day = dt_hr / 24.0;
    for _ in 0..n_steps {
        let denom = params.vd * (params.km + c);
        let dc = if denom > 1e-15 {
            -params.vmax * c / denom * dt_day
        } else {
            0.0
        };
        c = (c + dc).max(0.0);
        t += dt_hr;
        times.push(t);
        concs.push(c);
    }

    (times, concs)
}

/// Feline methimazole apparent half-life (concentration-dependent).
///
/// `t½ = 0.693 × (Km + C) × Vd / Vmax`
///
/// At low C: t½ ≈ 0.693 × Km × Vd / Vmax (first-order approximation)
/// At high C: t½ increases with C (capacity saturation)
#[must_use]
pub fn methimazole_apparent_half_life(params: &FelineMethimazoleParams, concentration: f64) -> f64 {
    0.693 * (params.km + concentration) * params.vd / params.vmax
}

/// Steady-state concentration for continuous infusion.
///
/// `Css = Km × R / (Vmax - R)` where R = infusion rate (mg/day).
/// Returns `None` if `rate >= Vmax` (no steady state — drug accumulates).
#[must_use]
pub fn methimazole_css(params: &FelineMethimazoleParams, rate_mg_per_day: f64) -> Option<f64> {
    if rate_mg_per_day >= params.vmax {
        return None;
    }
    Some(params.km * rate_mg_per_day / (params.vmax - rate_mg_per_day))
}

/// Feline T4 (thyroxine) response to methimazole.
///
/// Hyperthyroid cats have elevated T4 (>4.0 µg/dL, normal 1.0–4.0).
/// Methimazole inhibits thyroid peroxidase, reducing T4 production.
///
/// Model: `T4(t) = T4_target + (T4_baseline - T4_target) × exp(-k × t)`
///
/// where `k` depends on methimazole concentration (higher C → faster normalization).
///
/// Returns T4 level in µg/dL at `t_days` after treatment start.
#[must_use]
pub fn t4_response(t4_baseline: f64, methimazole_conc: f64, t_days: f64) -> f64 {
    let t4_target = 2.5;
    let k = 0.1 * methimazole_conc / (1.0 + methimazole_conc);
    let delta = t4_baseline - t4_target;
    (-k * t_days).exp().mul_add(delta, t4_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    #[expect(
        clippy::assertions_on_constants,
        reason = "validating const parameter sanity"
    )]
    fn feline_params_positive() {
        assert!(FELINE_METHIMAZOLE.vmax > 0.0);
        assert!(FELINE_METHIMAZOLE.km > 0.0);
        assert!(FELINE_METHIMAZOLE.vd > 0.0);
    }

    #[test]
    fn simulate_decays() {
        let (_, concs) = methimazole_simulate(&FELINE_METHIMAZOLE, 2.5, 24.0, 0.5);
        assert!(concs.last().copied().unwrap_or(0.0) < concs[0]);
        assert!(concs.last().copied().unwrap_or(-1.0) >= 0.0);
    }

    #[test]
    fn half_life_increases_with_concentration() {
        let t_low = methimazole_apparent_half_life(&FELINE_METHIMAZOLE, 1.0);
        let t_high = methimazole_apparent_half_life(&FELINE_METHIMAZOLE, 10.0);
        assert!(t_high > t_low);
    }

    #[test]
    fn css_exists_below_vmax() {
        let css = methimazole_css(&FELINE_METHIMAZOLE, 1.0);
        assert!(css.is_some());
        assert!(css.unwrap_or(0.0) > 0.0);
    }

    #[test]
    fn css_none_above_vmax() {
        assert!(methimazole_css(&FELINE_METHIMAZOLE, FELINE_METHIMAZOLE.vmax + 1.0).is_none());
    }

    #[test]
    fn t4_response_normalizes() {
        let t4_hyper = 8.0;
        let t4_treated = t4_response(t4_hyper, 2.0, 30.0);
        assert!(t4_treated < t4_hyper);
        assert!(t4_treated > 1.0);
    }

    #[test]
    fn t4_at_time_zero() {
        let t4 = t4_response(8.0, 2.0, 0.0);
        assert!((t4 - 8.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    #[expect(
        clippy::assertions_on_constants,
        reason = "validating const parameter sanity"
    )]
    fn cross_species_vmax_scaling() {
        assert!(HUMAN_METHIMAZOLE.vmax > FELINE_METHIMAZOLE.vmax);
        assert!(HUMAN_METHIMAZOLE.vd > FELINE_METHIMAZOLE.vd);
    }
}
