// SPDX-License-Identifier: AGPL-3.0-or-later
//! Testosterone pharmacokinetics: IM depot, pellet depot, age-related decline.
//!
//! - [`pk_im_depot`]: First-order absorption from IM injection site
//! - [`pellet_concentration`]: Zero-order release depot (pellet implant)
//! - [`testosterone_decline`]: Exponential age-related decline
//! - [`age_at_threshold`]: Age when T crosses a clinical threshold

use crate::pkpd;

// ═══════════════════════════════════════════════════════════════════════
// Testosterone IM Injection PK (Exp030)
// ═══════════════════════════════════════════════════════════════════════

/// Testosterone cypionate published PK parameters.
///
/// T½, Ka, Vd from IM depot bioavailability studies.
/// References: Harman et al. 2001, JCEM (doi:10.1210/jcem.86.2.7219).
pub mod testosterone_cypionate {
    use core::f64::consts::LN_2;
    /// Elimination half-life used for IM cypionate PK (days).
    pub const T_HALF_DAYS: f64 = 8.0;
    /// First-order elimination rate constant (1/day) from `T_HALF_DAYS`.
    pub const K_E: f64 = LN_2 / T_HALF_DAYS;
    /// First-order absorption rate from the IM depot (1/day).
    pub const K_A_IM: f64 = LN_2 / 1.5;
    /// Volume of distribution (L) for central compartment scaling.
    pub const VD_L: f64 = 70.0;
    /// Bioavailability fraction for the IM route (0–1).
    pub const F_IM: f64 = 1.0;
    /// Typical weekly maintenance dose (mg) for reference regimens.
    pub const DOSE_WEEKLY_MG: f64 = 100.0;
    /// Typical every-two-weeks dose (mg) for reference regimens.
    pub const DOSE_BIWEEKLY_MG: f64 = 200.0;
    /// Dosing interval for weekly schedule (days).
    pub const INTERVAL_WEEKLY: f64 = 7.0;
    /// Dosing interval for biweekly schedule (days).
    pub const INTERVAL_BIWEEKLY: f64 = 14.0;
}

/// First-order absorption from IM depot (Bateman equation).
///
/// Same math as [`pkpd::pk_oral_one_compartment`] — IM depot absorption
/// follows the same first-order kinetics as oral absorption.
#[must_use]
pub fn pk_im_depot(dose_mg: f64, f: f64, vd: f64, ka: f64, ke: f64, t: f64) -> f64 {
    pkpd::pk_oral_one_compartment(dose_mg, f, vd, ka, ke, t)
}

/// Compute IM steady-state metrics for a repeated dosing regimen.
///
/// Returns `(ss_cmax, ss_trough)` estimated from the last interval of
/// `n_doses` repeated injections.
/// IM dosing regimen for [`im_steady_state_metrics`].
pub struct ImRegimen {
    /// Mass per injection (mg).
    pub dose_mg: f64,
    /// Bioavailability fraction applied to each dose.
    pub f: f64,
    /// Volume of distribution (L).
    pub vd: f64,
    /// First-order absorption rate from depot (1/day).
    pub ka: f64,
    /// First-order elimination rate constant (1/day).
    pub ke: f64,
    /// Time between injections (same units as simulation time).
    pub interval: f64,
    /// Number of doses accumulated before evaluating steady-state window.
    pub n_doses: usize,
}

/// Estimates steady-state peak and trough from the last dosing interval after dose stacking.
///
/// Scans `times` after the start of the final interval and returns `(cmax, min)` there.
#[must_use]
pub fn im_steady_state_metrics(regimen: &ImRegimen, times: &[f64]) -> (f64, f64) {
    let concs = pkpd::pk_multiple_dose(
        |t| {
            pk_im_depot(
                regimen.dose_mg,
                regimen.f,
                regimen.vd,
                regimen.ka,
                regimen.ke,
                t,
            )
        },
        regimen.interval,
        regimen.n_doses,
        times,
    );
    #[expect(clippy::cast_precision_loss, reason = "n_doses always small")]
    let last_start = (regimen.n_doses - 1) as f64 * regimen.interval;
    let mut cmax = 0.0_f64;
    let mut trough = f64::INFINITY;
    for (&t, &c) in times.iter().zip(concs.iter()) {
        if t >= last_start {
            cmax = cmax.max(c);
            trough = trough.min(c);
        }
    }
    if trough == f64::INFINITY {
        trough = 0.0;
    }
    (cmax, trough)
}

// ═══════════════════════════════════════════════════════════════════════
// Testosterone Pellet PK (Exp031)
// ═══════════════════════════════════════════════════════════════════════

/// Pellet PK parameters.
///
/// Dose scaling (10 mg/lb), duration ~150 days from implant PK.
/// References: Saad et al. 2011, 2016 (TRT pellet regimens).
pub mod pellet_params {
    /// Reference body weight (lb) for published dose scaling.
    pub const BODY_WEIGHT_LB: f64 = 200.0;
    /// Total implant dose (mg) from mg/lb scaling.
    pub const DOSE_MG: f64 = 10.0 * BODY_WEIGHT_LB;
    /// Nominal release window before washout-dominated phase (days).
    pub const DURATION_DAYS: f64 = 150.0;
    /// Zero-order release rate (mg/day) over `DURATION_DAYS`.
    pub const RELEASE_RATE: f64 = DOSE_MG / DURATION_DAYS;
}

/// Pellet concentration: zero-order input for `duration` days, then washout.
///
/// During infusion (t <= duration):
///   C(t) = R0/(Vd*ke) * (1 - exp(-ke*t))
///
/// After infusion (t > duration):
///   `C(t) = C_plateau * exp(-ke*(t - duration))`
#[must_use]
pub fn pellet_concentration(t: f64, release_rate: f64, ke: f64, vd: f64, duration: f64) -> f64 {
    let c_ss = release_rate / (vd * ke);
    if t <= duration {
        c_ss * (1.0 - (-ke * t).exp())
    } else {
        c_ss * (1.0 - (-ke * duration).exp()) * (-ke * (t - duration)).exp()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Age-Related Testosterone Decline (Exp032)
// ═══════════════════════════════════════════════════════════════════════

/// Published decline parameters.
///
/// T0 and rate from longitudinal aging studies.
/// References: Harman et al. 2001, JCEM (doi:10.1210/jcem.86.2.7219).
pub mod decline_params {
    /// Population mean baseline total T (ng/dL) before age decline.
    pub const T0_MEAN_NGDL: f64 = 600.0;
    /// Coefficient of variation on baseline T for inter-individual spread.
    pub const T0_CV: f64 = 0.25;
    /// Lower bound annual fractional decline rate (1/year) for sensitivity.
    pub const RATE_LOW: f64 = 0.01;
    /// Mid estimate annual fractional decline rate (1/year).
    pub const RATE_MID: f64 = 0.017;
    /// Upper bound annual fractional decline rate (1/year) for sensitivity.
    pub const RATE_HIGH: f64 = 0.03;
    /// Clinical hypogonadal threshold (ng/dL) for age-at-threshold queries.
    pub const THRESHOLD_CLINICAL: f64 = 300.0;
}

/// Exponential testosterone decline: T(age) = T0 * exp(-rate * (age - onset)).
#[must_use]
pub fn testosterone_decline(t0: f64, rate: f64, age: f64, onset: f64) -> f64 {
    t0 * (-rate * (age - onset)).exp()
}

/// Age when T crosses below a threshold.
///
/// Returns `onset` if T0 is already below threshold.
#[must_use]
pub fn age_at_threshold(t0: f64, rate: f64, threshold: f64, onset: f64) -> f64 {
    if t0 <= threshold {
        return onset;
    }
    onset + (t0 / threshold).ln() / rate
}

// ═══════════════════════════════════════════════════════════════════════
// Population TRT Monte Carlo (Exp036)
// ═══════════════════════════════════════════════════════════════════════

/// Population PK parameters for testosterone cypionate IM.
///
/// Vd, Ka, Ke typical values and CV from published IM depot studies.
/// References: Harman et al. 2001, JCEM (doi:10.1210/jcem.86.2.7219).
pub mod pop_trt {
    /// Typical volume of distribution (L) for Monte Carlo sampling.
    pub const VD_TYPICAL: f64 = 70.0;
    /// Inter-individual CV on `VD_TYPICAL`.
    pub const VD_CV: f64 = 0.25;
    /// Typical IM absorption rate constant (1/day).
    pub const KA_TYPICAL: f64 = 0.462;
    /// Inter-individual CV on `KA_TYPICAL`.
    pub const KA_CV: f64 = 0.30;
    /// Typical elimination rate constant (1/day).
    pub const KE_TYPICAL: f64 = 0.0866;
    /// Inter-individual CV on `KE_TYPICAL`.
    pub const KE_CV: f64 = 0.20;
    /// Typical baseline total T (ng/dL) before age adjustment.
    pub const T0_TYPICAL: f64 = 600.0;
    /// Inter-individual CV on `T0_TYPICAL`.
    pub const T0_CV: f64 = 0.25;
    /// Annual fractional decline applied in age-adjusted baseline T.
    pub const DECLINE_RATE: f64 = 0.017;
}

/// Lognormal underlying parameters from typical value and CV.
///
/// Returns (mu, sigma) for the normal distribution underlying the lognormal.
#[must_use]
pub fn lognormal_params(typical: f64, cv: f64) -> (f64, f64) {
    crate::pkpd::LognormalParam { typical, cv }.to_normal_params()
}

/// Compute age-adjusted baseline testosterone.
#[must_use]
pub fn age_adjusted_t0(t0: f64, age: f64, decline_rate: f64) -> f64 {
    t0 * (-decline_rate * (age - 30.0)).exp()
}
