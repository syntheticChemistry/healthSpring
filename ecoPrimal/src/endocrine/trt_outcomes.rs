// SPDX-License-Identifier: AGPL-3.0-or-later
//! TRT outcome models: weight/waist, cardiovascular, diabetes, gut axis.
//!
//! - [`weight_trajectory`]: Logarithmic weight change under TRT
//! - [`biomarker_trajectory`]: Exponential approach to new setpoint
//! - [`hba1c_trajectory`]: `HbA1c` response to TRT
//! - [`hazard_ratio_model`]: Cardiovascular hazard ratio from T level
//! - [`anderson_localization_length`]: Testosterone-gut axis scaling
//! - [`gut_metabolic_response`]: Metabolic response from gut diversity
//! - [`hrv_trt_response`]: HRV improvement from TRT
//! - [`cardiac_risk_composite`]: Composite cardiac risk from HRV + T

use crate::tolerances;

// ═══════════════════════════════════════════════════════════════════════
// TRT Outcome Trajectories (Exp033–035)
// ═══════════════════════════════════════════════════════════════════════

/// Logarithmic weight trajectory: `ΔW(t) = delta_final * ln(1+t/τ) / ln(1+T/τ)`.
///
/// Models decelerating weight loss under TRT (Saad 2013 registry).
#[must_use]
pub fn weight_trajectory(month: f64, delta_final: f64, tau: f64, total_months: f64) -> f64 {
    let norm = (total_months / tau).ln_1p();
    if norm.abs() < tolerances::DIVISION_GUARD {
        return 0.0;
    }
    delta_final * (month / tau).ln_1p() / norm
}

/// Biomarker trajectory: exponential approach to new setpoint.
///
/// `value(t) = baseline + (endpoint - baseline) * (1 - exp(-t/τ))`
///
/// Used for LDL, HDL, CRP, SBP, DBP, `HOMA-IR`, fasting glucose, `HbA1c`.
#[must_use]
pub fn biomarker_trajectory(month: f64, baseline: f64, endpoint: f64, tau: f64) -> f64 {
    let delta = endpoint - baseline;
    delta.mul_add(1.0 - (-month / tau).exp(), baseline)
}

/// `HbA1c` trajectory (alias for biomarker trajectory with specific semantics).
#[must_use]
pub fn hba1c_trajectory(month: f64, baseline: f64, delta: f64, tau: f64) -> f64 {
    delta.mul_add(1.0 - (-month / tau).exp(), baseline)
}

/// Cardiovascular hazard ratio model (Sharma 2015).
///
/// References: Sharma et al. 2015, JAMA Intern Med; Kapoor et al. 2006 (TRT cardiovascular).
///
/// If T >= threshold: HR = `hr_normalized` (0.44 in Sharma).
/// If T < threshold: linear interpolation from 1.0 to `hr_normalized`.
#[must_use]
pub fn hazard_ratio_model(t_level: f64, threshold: f64, hr_normalized: f64) -> f64 {
    if t_level >= threshold {
        return hr_normalized;
    }
    let ratio = t_level / threshold;
    (1.0 - hr_normalized).mul_add(-ratio, 1.0)
}

/// Saad 2016 cardiovascular parameters.
///
/// LDL, HDL, CRP, SBP, DBP baseline/endpoint from TRT registry data.
/// References: Saad et al. 2011, 2016; Kapoor et al. 2006 (TRT cardiovascular).
pub mod cv_params {
    pub const LDL_BASELINE: f64 = 165.0;
    pub const LDL_ENDPOINT: f64 = 130.0;
    pub const HDL_BASELINE: f64 = 38.0;
    pub const HDL_ENDPOINT: f64 = 55.0;
    pub const CRP_BASELINE: f64 = 1.40;
    pub const CRP_ENDPOINT: f64 = 0.90;
    pub const SBP_BASELINE: f64 = 135.0;
    pub const SBP_ENDPOINT: f64 = 123.0;
    pub const DBP_BASELINE: f64 = 82.0;
    pub const DBP_ENDPOINT: f64 = 76.0;
    pub const TAU_MONTHS: f64 = 12.0;
}

/// Diabetes TRT parameters.
///
/// `HbA1c`, HOMA-IR, fasting glucose from TRT in hypogonadal diabetics.
/// References: Saad et al. 2011, 2016.
pub mod diabetes_params {
    pub const HBA1C_BASELINE: f64 = 7.60;
    pub const HBA1C_DELTA: f64 = -0.37;
    pub const HOMA_BASELINE: f64 = 4.5;
    pub const HOMA_ENDPOINT: f64 = 3.2;
    pub const FG_BASELINE: f64 = 140.0;
    pub const FG_ENDPOINT: f64 = 120.0;
    pub const TAU_MONTHS: f64 = 3.0;
}

/// Weight/waist TRT parameters (Saad 2013).
///
/// 5-year weight, waist, BMI change from TRT registry.
/// References: Saad et al. 2011, 2016 (TRT weight/waist data).
pub mod weight_params {
    pub const WEIGHT_LOSS_5YR_KG: f64 = -16.0;
    pub const WAIST_LOSS_5YR_CM: f64 = -12.0;
    pub const BMI_LOSS_5YR: f64 = -5.6;
    pub const TAU_MONTHS: f64 = 6.0;
    pub const TOTAL_MONTHS: f64 = 60.0;
}

// ═══════════════════════════════════════════════════════════════════════
// Testosterone-Gut Axis (Exp037)
// ═══════════════════════════════════════════════════════════════════════

/// Anderson localization scaling parameters.
///
/// Power-law: `ξ = (L/2) * (W / W_ref)^ν`.
/// ν = 1.5 gives good discrimination across the clinical Pielou range (0.3–0.95).
pub mod anderson_scaling {
    /// Reference disorder strength for normalization.
    pub const W_REF: f64 = 5.0;
    /// Localization length exponent (power-law scaling).
    pub const NU: f64 = 1.5;
}

/// Anderson localization length from disorder strength (power-law scaling).
///
/// `ξ = ξ_0 * (W / W_ref)^ν` where ν = 1.5.
/// Maintains discrimination across the clinical Pielou range.
#[must_use]
pub fn anderson_localization_length(disorder_w: f64, lattice_size: f64) -> f64 {
    if disorder_w <= 0.0 {
        return 1.0;
    }
    let xi_0 = lattice_size * 0.5;
    xi_0 * (disorder_w / anderson_scaling::W_REF).powf(anderson_scaling::NU)
}

/// Pielou evenness → Anderson disorder (linear mapping).
///
/// Delegates to [`crate::microbiome::evenness_to_disorder`] — single
/// source of truth for the `W = J × scale` relationship.
#[must_use]
pub fn evenness_to_disorder(pielou_j: f64, scale: f64) -> f64 {
    crate::microbiome::evenness_to_disorder(pielou_j, scale)
}

/// Metabolic response model: higher ξ → better TRT response.
///
/// Returns weight change (negative = loss).
#[must_use]
pub fn gut_metabolic_response(xi: f64, xi_max: f64, base_response: f64) -> f64 {
    if xi_max <= 0.0 {
        return 0.0;
    }
    base_response * xi / xi_max
}

/// Gut axis parameters.
///
/// Anderson disorder scale, lattice size, base metabolic response.
/// References: Saad et al. 2011, 2016 (weight/waist TRT outcomes).
pub mod gut_axis_params {
    pub const DISORDER_SCALE: f64 = 5.0;
    pub const LATTICE_SIZE: f64 = 100.0;
    pub const BASE_RESPONSE_KG: f64 = -16.0;
}

// ═══════════════════════════════════════════════════════════════════════
// Cross-Track: HRV × TRT Cardiovascular (Exp038 — Mok D3)
// ═══════════════════════════════════════════════════════════════════════

/// Model HRV improvement from TRT-induced cardiovascular benefit.
///
/// As TRT normalizes testosterone → reduced inflammation → improved
/// autonomic balance → increased HRV.
///
/// `SDNN(months) = sdnn_base + delta_sdnn * (1 - exp(-months / tau))`
///
/// Published correlation: SDNN < 50ms associated with ~5× cardiac mortality risk
/// (Kleiger 1987, NEJM). TRT normalization → expected +10-20ms SDNN improvement.
#[must_use]
pub fn hrv_trt_response(
    sdnn_base_ms: f64,
    delta_sdnn_ms: f64,
    tau_months: f64,
    months: f64,
) -> f64 {
    delta_sdnn_ms.mul_add(1.0 - (-months / tau_months).exp(), sdnn_base_ms)
}

/// Cardiac risk thresholds (Kleiger 1987, NEJM; Sharma 2015, JAMA IM).
///
/// SDNN and testosterone cutpoints from mortality/outcome studies.
pub mod cardiac_risk_thresholds {
    /// SDNN (ms) below which HRV doubles cardiac risk.
    pub const SDNN_HIGH_RISK_MS: f64 = 50.0;
    /// SDNN (ms) above which HRV halves cardiac risk.
    pub const SDNN_LOW_RISK_MS: f64 = 100.0;
    /// Testosterone (ng/dL) below which low-T doubles cardiac risk.
    pub const T_HIGH_RISK_NGDL: f64 = 300.0;
    /// Testosterone (ng/dL) above which normalized T halves cardiac risk.
    pub const T_LOW_RISK_NGDL: f64 = 500.0;
}

/// Composite cardiac risk score from HRV + testosterone level.
///
/// Risk = `baseline_risk` * `hrv_factor` * `testosterone_factor`
/// - HRV factor: SDNN < 50ms doubles risk, SDNN > 100ms halves risk
/// - T factor: T < 300 ng/dL doubles risk, T > 500 ng/dL halves risk
#[must_use]
pub fn cardiac_risk_composite(sdnn_ms: f64, testosterone_ng_dl: f64, baseline_risk: f64) -> f64 {
    use cardiac_risk_thresholds as crt;
    let hrv_factor = if sdnn_ms < crt::SDNN_HIGH_RISK_MS {
        2.0 - sdnn_ms / crt::SDNN_HIGH_RISK_MS
    } else if sdnn_ms > crt::SDNN_LOW_RISK_MS {
        0.5
    } else {
        1.0 - 0.5 * (sdnn_ms - crt::SDNN_HIGH_RISK_MS)
            / (crt::SDNN_LOW_RISK_MS - crt::SDNN_HIGH_RISK_MS)
    };

    let t_factor = if testosterone_ng_dl < crt::T_HIGH_RISK_NGDL {
        2.0 - testosterone_ng_dl / crt::T_HIGH_RISK_NGDL
    } else if testosterone_ng_dl > crt::T_LOW_RISK_NGDL {
        0.5
    } else {
        1.0 - 0.5 * (testosterone_ng_dl - crt::T_HIGH_RISK_NGDL)
            / (crt::T_LOW_RISK_NGDL - crt::T_HIGH_RISK_NGDL)
    };

    baseline_risk * hrv_factor * t_factor
}
