// SPDX-License-Identifier: AGPL-3.0-only
//! Centralized tolerance constants sourced from `specs/TOLERANCE_REGISTRY.md`.
//!
//! Every constant here maps to a documented, justified entry in the registry.
//! Inline magic numbers in experiment binaries should migrate to these named
//! constants so that a single audit covers all validation thresholds.

// ── Machine Epsilon Class (1e-10 to 1e-15) ────────────────────────────

/// IEEE 754 f64 analytical identity — `|observed - expected| ≤ 1e-10`.
pub const MACHINE_EPSILON: f64 = 1e-10;

/// Tighter machine epsilon for comparison guards (≤ vs ≥ with rounding).
pub const MACHINE_EPSILON_TIGHT: f64 = 1e-12;

/// Strictest machine epsilon for monotonicity guards.
pub const MACHINE_EPSILON_STRICT: f64 = 1e-15;

/// Anderson Hamiltonian symmetry / eigenvalue identity checks.
pub const ANDERSON_IDENTITY: f64 = 1e-14;

/// Two-compartment macro/micro rate identities (α+β, α·β).
pub const TWO_COMPARTMENT_IDENTITY: f64 = 1e-8;

/// Diversity cross-validation to Python baselines (Shannon, Simpson, Chao1).
pub const DIVERSITY_CROSS_VALIDATE: f64 = 1e-8;

// ── Numerical Method Class (1e-6 to 0.01) ─────────────────────────────

/// Trapezoidal AUC with N ≥ 1000 steps.
pub const AUC_TRAPEZOIDAL: f64 = 0.01;

/// Numerical Tmax on discrete grid (~6 min at 1000 pts over 48 hr).
pub const TMAX_NUMERICAL: f64 = 0.1;

/// Half-life point: `C(t½) = C0/2` via `exp(-ln2)`.
pub const HALF_LIFE_POINT: f64 = 1e-6;

/// Allometric CL ratio involving `powf` and division.
pub const ALLOMETRIC_CL_RATIO: f64 = 1e-6;

/// Abundance normalization accumulation.
pub const ABUNDANCE_NORMALIZATION: f64 = 1e-6;

/// Level spacing ratio finite-sample variance.
pub const LEVEL_SPACING_RATIO: f64 = 0.02;

/// Pielou → W cross-validation to Python.
pub const W_CROSS_VALIDATE: f64 = 0.01;

/// Pielou boundary slack.
pub const PIELOU_BOUNDARY: f64 = 0.001;

/// Exponential residual at long time.
pub const EXPONENTIAL_RESIDUAL: f64 = 0.01;

/// Truncated AUC error (~10% for finite integration window).
pub const AUC_TRUNCATED: f64 = 0.10;

/// Terminal slope regression.
pub const TERMINAL_SLOPE: f64 = 0.01;

/// Tmax cross-species resolution.
pub const TMAX_CROSS_SPECIES: f64 = 0.5;

// ── Population / Statistical Class (0.05 to 0.15) ─────────────────────

/// Lognormal parameter recovery.
pub const LOGNORMAL_RECOVERY: f64 = 0.05;

/// Population Vd median.
pub const POP_VD_MEDIAN: f64 = 0.05;

/// Population Ka median (higher CV → wider tolerance).
pub const POP_KA_MEDIAN: f64 = 0.10;

/// Population AUC CV lower bound.
pub const POP_AUC_CV: f64 = 0.15;

/// Multi-dose accumulation factor.
pub const ACCUMULATION_FACTOR: f64 = 0.25;

/// Washout half-life numerical search.
pub const WASHOUT_HALF_LIFE: f64 = 0.15;

// ── Clinical Plausibility Class (0.25 to 0.60) ────────────────────────

/// Front-loaded weight loss fraction (>60% by 24 mo).
pub const FRONT_LOADED_WEIGHT: f64 = 0.60;

/// Front-loaded LDL improvement (>55% by 12 mo).
pub const FRONT_LOADED_LDL: f64 = 0.55;

/// Front-loaded `HbA1c` drop (>80% by 6 mo).
pub const FRONT_LOADED_HBA1C: f64 = 0.80;

/// Pan-Tompkins sensitivity / PPV threshold.
pub const QRS_SENSITIVITY: f64 = 0.80;

/// Heart rate detection tolerance (±10 bpm).
pub const HR_DETECTION_BPM: f64 = 10.0;

/// HRV SDNN upper bound (200 ms for synthetic).
pub const SDNN_UPPER_MS: f64 = 200.0;

/// QRS peak match tolerance (75 ms ≈ 27 samples at 360 Hz).
pub const QRS_PEAK_MATCH_MS: f64 = 75.0;

// ── GPU Parity Class (1e-4 to 0.25) ───────────────────────────────────

/// Hill GPU vs CPU — f32 transcendental precision.
pub const GPU_F32_TRANSCENDENTAL: f64 = 1e-4;

/// `PopPK` GPU vs CPU — different PRNG → statistical parity.
pub const GPU_STATISTICAL_PARITY: f64 = 0.25;

/// Fused pipeline vs individual dispatch.
pub const GPU_FUSED_PARITY: f64 = 1e-4;

/// GPU scaling linearity (timing noise).
pub const GPU_SCALING_LINEARITY: f64 = 0.01;

// ── CPU Parity Class (1e-10) ──────────────────────────────────────────

/// CPU-only validation (Hill, `PopPK`, Diversity) — pure f64.
pub const CPU_PARITY: f64 = 1e-10;

/// Pipeline stage transform (exp decay, dispatch).
pub const PIPELINE_STAGE: f64 = 1e-10;

// ── NLME / Pipeline Class ─────────────────────────────────────────────

/// FOCE theta recovery (`CL`, `Vd`, `Ka`) — 30% relative.
pub const FOCE_THETA_RECOVERY: f64 = 0.30;

/// `SAEM` theta recovery — 50% relative (Monte Carlo noise).
pub const SAEM_THETA_RECOVERY: f64 = 0.50;

/// NCA `lambda_z` terminal slope — 5% relative.
pub const NCA_LAMBDA_Z: f64 = 0.05;

/// NCA `AUC_inf` — 5% relative.
pub const NCA_AUC_INF: f64 = 0.05;

/// CWRES mean absolute bound.
pub const CWRES_MEAN: f64 = 2.0;

/// `PBPK` mass conservation — Euler discretization error.
pub const PBPK_MASS_CONSERVATION: f64 = 0.25;

/// Biomarker at 10τ (trajectory approaches endpoint).
pub const BIOMARKER_ENDPOINT: f64 = 0.5;

/// Lognormal mean recovery.
pub const LOGNORMAL_MEAN: f64 = 0.01;

#[cfg(test)]
mod tests {
    use super::*;

    fn lt(a: f64, b: f64) -> bool {
        a < b
    }
    fn gt(a: f64, b: f64) -> bool {
        a > b
    }

    #[test]
    fn tolerance_ordering() {
        assert!(lt(MACHINE_EPSILON_STRICT, MACHINE_EPSILON_TIGHT));
        assert!(lt(MACHINE_EPSILON_TIGHT, MACHINE_EPSILON));
        assert!(lt(MACHINE_EPSILON, HALF_LIFE_POINT));
        assert!(lt(HALF_LIFE_POINT, AUC_TRAPEZOIDAL));
        assert!(lt(AUC_TRAPEZOIDAL, LOGNORMAL_RECOVERY));
        assert!(lt(LOGNORMAL_RECOVERY, ACCUMULATION_FACTOR));
        assert!(lt(ACCUMULATION_FACTOR, FRONT_LOADED_WEIGHT));
    }

    #[test]
    fn gpu_tolerances_wider_than_cpu() {
        assert!(gt(GPU_F32_TRANSCENDENTAL, CPU_PARITY));
        assert!(gt(GPU_STATISTICAL_PARITY, GPU_F32_TRANSCENDENTAL));
    }
}
