// SPDX-License-Identifier: AGPL-3.0-or-later
//! Nonlinear (Michaelis-Menten) pharmacokinetics.
//!
//! Capacity-limited elimination: `dC/dt = -Vmax·C/(Km + C)`.
//!
//! At low concentrations (C << Km) elimination is approximately first-order
//! with apparent rate constant `k = Vmax/(Km·Vd)`. At high concentrations
//! (C >> Km) elimination approaches zero-order at rate `Vmax/Vd`.
//!
//! Reference: Rowland & Tozer, *Clinical Pharmacokinetics and
//! Pharmacodynamics*, 5th ed., Ch. 20.
//! Example drug: phenytoin (Ludden et al. 1977).

/// Michaelis-Menten PK parameters.
#[derive(Debug, Clone)]
pub struct MichaelisMentenParams {
    /// Maximum elimination rate (mg/day)
    pub vmax: f64,
    /// Concentration at half-maximal elimination rate (mg/L)
    pub km: f64,
    /// Volume of distribution (L)
    pub vd: f64,
}

/// Phenytoin-like reference parameters (Ludden 1977).
///
/// Vmax ≈ 500 mg/day, Km ≈ 5 mg/L, Vd ≈ 50 L. Classical Michaelis-Menten PK.
/// References: Winter 2009, Basic Clinical Pharmacokinetics 5th ed; Bauer 2008,
/// Applied Clinical Pharmacokinetics 2nd ed; Richens & Dunlop 1975, Lancet.
pub const PHENYTOIN_PARAMS: MichaelisMentenParams = MichaelisMentenParams {
    vmax: 500.0,
    km: 5.0,
    vd: 50.0,
};

/// Simulate Michaelis-Menten PK via Euler integration.
///
/// Returns concentration time course at each step from t=0 to `t_end`.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "t_end / dt is small positive"
)]
pub fn mm_pk_simulate(
    params: &MichaelisMentenParams,
    dose_mg: f64,
    t_end: f64,
    dt: f64,
) -> (Vec<f64>, Vec<f64>) {
    let n_steps = (t_end / dt) as usize;
    let mut times = Vec::with_capacity(n_steps + 1);
    let mut concs = Vec::with_capacity(n_steps + 1);

    let mut c = dose_mg / params.vd;
    let mut t = 0.0;
    times.push(t);
    concs.push(c);

    for _ in 0..n_steps {
        let elim_rate = params.vmax * c / (params.km + c);
        c -= (elim_rate / params.vd) * dt;
        c = c.max(0.0);
        t += dt;
        times.push(t);
        concs.push(c);
    }
    (times, concs)
}

/// Steady-state concentration for constant-rate IV infusion.
///
/// At steady state: `R_inf = Vmax·Css/(Km + Css)` → `Css = R_inf·Km/(Vmax - R_inf)`.
/// Returns `None` if `rate_mg_per_day >= Vmax` (no steady state — accumulation).
#[must_use]
pub fn mm_css_infusion(params: &MichaelisMentenParams, rate_mg_per_day: f64) -> Option<f64> {
    if rate_mg_per_day >= params.vmax {
        return None;
    }
    Some(rate_mg_per_day * params.km / (params.vmax - rate_mg_per_day))
}

/// Apparent first-order half-life at a given concentration.
///
/// `t½_app = 0.693·(Km + C)·Vd / Vmax`
///
/// This increases with C — the hallmark of nonlinear PK.
#[must_use]
pub fn mm_apparent_half_life(params: &MichaelisMentenParams, concentration: f64) -> f64 {
    0.693 * (params.km + concentration) * params.vd / params.vmax
}

/// AUC for Michaelis-Menten elimination (numerical trapezoidal).
#[must_use]
pub fn mm_auc(concs: &[f64], dt: f64) -> f64 {
    if concs.len() < 2 {
        return 0.0;
    }
    concs.windows(2).map(|w| w[0].midpoint(w[1]) * dt).sum()
}

/// Analytical AUC for Michaelis-Menten IV bolus (exact, not approximation).
///
/// For `dC/dt = -Vmax·C/(Vd·(Km + C))`:
/// `AUC = (Km·Vd/Vmax)·ln(C0/C_final) + Vd·(C0 - C_final)/(2·Vmax)`...
/// Actually, the exact integral gives:
/// `AUC(0→∞) = Km·D/Vmax + D²/(2·Vmax·Vd)`
///
/// where D = dose.
#[must_use]
pub fn mm_auc_analytical(params: &MichaelisMentenParams, dose_mg: f64) -> f64 {
    let c0 = dose_mg / params.vd;
    params.km * c0 * params.vd / params.vmax + c0 * c0 * params.vd / (2.0 * params.vmax)
}

/// Check if a drug exhibits nonlinear PK: compare AUC ratio to dose ratio.
///
/// For linear PK, `AUC(2D)/AUC(D) = 2.0`.
/// For Michaelis-Menten, `AUC(2D)/AUC(D) > 2.0` (supralinear).
#[must_use]
pub fn mm_nonlinearity_ratio(params: &MichaelisMentenParams, dose1: f64, dose2: f64) -> f64 {
    let auc1 = mm_auc_analytical(params, dose1);
    let auc2 = mm_auc_analytical(params, dose2);
    (auc2 / auc1) / (dose2 / dose1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mm_simulation_monotone_decline() {
        let (_, concs) = mm_pk_simulate(&PHENYTOIN_PARAMS, 300.0, 5.0, 0.001);
        for w in concs.windows(2) {
            assert!(w[1] <= w[0] + 1e-14, "concentration must decline");
        }
    }

    #[test]
    fn mm_simulation_starts_at_c0() {
        let (_, concs) = mm_pk_simulate(&PHENYTOIN_PARAMS, 300.0, 1.0, 0.001);
        let c0 = 300.0 / PHENYTOIN_PARAMS.vd;
        assert!(
            (concs[0] - c0).abs() < 1e-10,
            "initial concentration should be dose/Vd"
        );
    }

    #[test]
    fn mm_css_below_vmax() {
        let Some(css) = mm_css_infusion(&PHENYTOIN_PARAMS, 250.0) else {
            panic!("250 mg/day < vmax 500 should yield Some");
        };
        assert!(css > 0.0 && css.is_finite());
    }

    #[test]
    fn mm_css_at_vmax_returns_none() {
        assert!(mm_css_infusion(&PHENYTOIN_PARAMS, 500.0).is_none());
        assert!(mm_css_infusion(&PHENYTOIN_PARAMS, 600.0).is_none());
    }

    #[test]
    fn mm_half_life_increases_with_concentration() {
        let t_low = mm_apparent_half_life(&PHENYTOIN_PARAMS, 1.0);
        let t_high = mm_apparent_half_life(&PHENYTOIN_PARAMS, 20.0);
        assert!(
            t_high > t_low,
            "half-life should increase with concentration"
        );
    }

    #[test]
    fn mm_auc_supralinear() {
        let ratio = mm_nonlinearity_ratio(&PHENYTOIN_PARAMS, 200.0, 400.0);
        assert!(
            ratio > 1.0,
            "AUC should increase supralinearly: ratio={ratio}"
        );
    }

    #[test]
    fn mm_numerical_auc_close_to_analytical() {
        let (_, concs) = mm_pk_simulate(&PHENYTOIN_PARAMS, 300.0, 20.0, 0.0001);
        let numerical = mm_auc(&concs, 0.0001);
        let analytical = mm_auc_analytical(&PHENYTOIN_PARAMS, 300.0);
        let rel_err = (numerical - analytical).abs() / analytical;
        assert!(rel_err < 0.02, "numerical vs analytical AUC: {rel_err:.4}");
    }

    #[test]
    fn mm_low_dose_approaches_linear() {
        let ratio = mm_nonlinearity_ratio(&PHENYTOIN_PARAMS, 10.0, 20.0);
        assert!(
            (ratio - 1.0).abs() < 0.15,
            "at low doses, should approach linear: ratio={ratio}"
        );
    }
}
