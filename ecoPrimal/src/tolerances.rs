// SPDX-License-Identifier: AGPL-3.0-or-later
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

// ── Diagnostic Pipeline Class ────────────────────────────────────────

/// Hill response at `EC50`: exactly `E_max/2` (analytical identity).
pub const HILL_AT_EC50: f64 = 1.0;

/// Endocrine testosterone passthrough — input value echoed ±0.1 ng/dL.
pub const ENDOCRINE_TESTOSTERONE_PASSTHROUGH: f64 = 0.1;

/// Deterministic rerun parity — bit-identical assessment.
pub const DETERMINISM: f64 = 1e-12;

// ── PCIe / Hardware Class ────────────────────────────────────────────

/// `PCIe` bandwidth estimation tolerance (lane math rounding).
pub const PCIE_BANDWIDTH: f64 = 0.1;

/// `PCIe` Gen3 ×16 expected bandwidth (GB/s).
pub const PCIE_GEN3_16X_GBPS: f64 = 15.76;

/// `PCIe` Gen4 ×16 expected bandwidth (GB/s).
pub const PCIE_GEN4_16X_GBPS: f64 = 31.504;

/// `PCIe` Gen5 ×16 expected bandwidth (GB/s).
pub const PCIE_GEN5_16X_GBPS: f64 = 63.008;

// ── Gut-Brain Serotonin Class ────────────────────────────────────────

/// Tryptophan physiological range lower bound (µmol/L).
pub const TRP_RANGE_LOW: f64 = 80.0;

/// Tryptophan physiological range upper bound (µmol/L).
pub const TRP_RANGE_HIGH: f64 = 180.0;

/// Serotonin sigmoid midpoint range lower bound.
pub const SEROTONIN_MIDPOINT_LOW: f64 = 40.0;

/// Serotonin sigmoid midpoint range upper bound.
pub const SEROTONIN_MIDPOINT_HIGH: f64 = 60.0;

// ── Hill Saturation Class ────────────────────────────────────────────

/// Hill saturation threshold at `100×IC50`.
pub const HILL_SATURATION_100X: f64 = 0.99;

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

// ── Track 7: Drug Discovery Class ───────────────────────────────────

/// MATRIX pathway selectivity score — closed-form from IC50 ratios.
pub const MATRIX_PATHWAY: f64 = 1e-10;

/// Anderson tissue geometry factor — closed-form exponential.
pub const TISSUE_GEOMETRY: f64 = 1e-10;

/// Disorder impact factor — ratio of W values.
pub const DISORDER_IMPACT: f64 = 1e-10;

/// Combined MATRIX-Anderson score — product of three factors.
pub const MATRIX_COMBINED: f64 = 1e-10;

/// Z'-factor — closed-form from control statistics.
pub const HTS_Z_PRIME: f64 = 1e-10;

/// SSMD — closed-form standardized difference.
pub const HTS_SSMD: f64 = 1e-10;

/// Percent inhibition — linear normalization.
pub const HTS_PERCENT_INHIBITION: f64 = 1e-10;

/// IC50 estimation from Hill fit — iterative bisection.
pub const IC50_ESTIMATION: f64 = 0.10;

/// Selectivity index — ratio of IC50 values.
pub const SELECTIVITY_INDEX: f64 = 1e-10;

// ── Track 6: Comparative Medicine Class ─────────────────────────────

/// IL-31 kinetics at t=0 — initial condition identity.
pub const IL31_INITIAL: f64 = 1e-10;

/// IL-31 steady-state recovery — first-order kinetics.
pub const IL31_STEADY_STATE: f64 = 0.01;

/// Pruritus VAS at EC50 — Hill identity (`VAS_max` / 2).
pub const PRURITUS_AT_EC50: f64 = 1e-10;

/// Lokivetmab PK decay — exponential elimination.
pub const LOKIVETMAB_DECAY: f64 = 1e-10;

/// JAK1 selectivity ratio — ratio of geometric means.
pub const JAK1_SELECTIVITY: f64 = 1e-10;

/// Cross-species allometric roundtrip — same-weight identity.
pub const ALLOMETRIC_ROUNDTRIP: f64 = 1e-6;

/// Cross-species PK scaling — allometric exponent precision.
pub const CROSS_SPECIES_PK: f64 = 0.05;

/// Pruritus time-course VAS monotonicity check.
pub const PRURITUS_TIME_COURSE: f64 = 0.01;

/// Lokivetmab effective duration (days above threshold).
pub const LOKIVETMAB_DURATION: f64 = 1.0;

/// Lokivetmab onset time (hours to therapeutic level).
pub const LOKIVETMAB_ONSET: f64 = 1.0;

/// Fibrotic geometry factor — inverse of standard tissue geometry.
pub const FIBROTIC_GEOMETRY: f64 = 1e-10;

/// Anti-fibrotic pathway score — weighted sum.
pub const ANTI_FIBROTIC_SCORE: f64 = 1e-10;

/// Fractional inhibition at IC50 — Hill identity.
pub const FRACTIONAL_AT_IC50: f64 = 1e-10;

/// Feline methimazole PK simulation — Euler integration.
pub const FELINE_MM_PK: f64 = 0.01;

/// Feline T4 response — first-order normalization.
pub const FELINE_T4_RESPONSE: f64 = 0.01;

/// Canine gut Anderson disorder — cross-species diversity.
pub const CANINE_GUT_ANDERSON: f64 = 0.01;

// ── NLME Algorithm Parameters ────────────────────────────────────────

/// Central finite-difference step for gradient computation in NLME inner loop.
pub const NLME_FINITE_DIFF_STEP: f64 = 1e-6;

/// Theta gradient perturbation step (outer loop central differences).
pub const NLME_THETA_PERTURBATION: f64 = 1e-5;

/// Floor for omega (inter-individual variance) — prevents degenerate zero variance.
pub const NLME_OMEGA_FLOOR: f64 = 1e-8;

/// Floor for sigma (residual variance) — prevents degenerate zero residuals.
pub const NLME_SIGMA_FLOOR: f64 = 1e-10;

/// Gauss-Newton convergence threshold — stop when max step magnitude < this.
pub const NLME_CONVERGENCE_STEP: f64 = 1e-8;

/// Default convergence tolerance for NLME iteration relative change.
pub const NLME_DEFAULT_TOL: f64 = 1e-6;

/// FOCE learning rate base — `lr = base / (decay * iter + 1)`.
pub const FOCE_LR_BASE: f64 = 0.0001;

/// FOCE learning rate decay per iteration.
pub const FOCE_LR_DECAY: f64 = 0.01;

/// SAEM Metropolis-Hastings proposal scale — fraction of omega standard deviation.
pub const SAEM_MH_PROPOSAL_SCALE: f64 = 0.3;

/// SAEM minimum proposal standard deviation.
pub const SAEM_MH_MIN_SD: f64 = 0.01;

/// SAEM Robbins-Monro burn-in fraction — iterations before step-size decay begins.
pub const SAEM_BURNIN_FRACTION: f64 = 0.5;

/// SAEM initial sigma estimate.
pub const SAEM_INITIAL_SIGMA: f64 = 0.01;

/// SAEM initial omega diagonal.
pub const SAEM_INITIAL_OMEGA: f64 = 0.1;

// ── Simulation Default Parameters ────────────────────────────────────

/// Default tissue damage excess cap (prevents > 50% organism penalty).
pub const TISSUE_EXCESS_CAP: f64 = 0.5;

/// Default ecosystem simulation competition coefficient.
pub const DEFAULT_COMPETITION_COEFF: f64 = 0.7;

/// Default ecosystem simulation time step.
pub const DEFAULT_ECOSYSTEM_DT: f64 = 0.1;

// ── RPC Handler Default Parameters ───────────────────────────────────

/// Default VPC sigma when caller omits it.
pub const VPC_DEFAULT_SIGMA: f64 = 0.01;

/// Default VPC time grid step.
pub const VPC_DEFAULT_DT: f64 = 0.1;

// ── Toxicology Class ────────────────────────────────────────────────

/// Clearance utilization threshold for linear regime safety (20%).
pub const CLEARANCE_LINEAR_THRESHOLD: f64 = 0.20;

/// Hormetic zone lower bound: toxic threshold / this = top of hormetic range.
pub const HORMETIC_LOW_DIVISOR: f64 = 10.0;

/// Hormetic zone upper bound: toxic threshold / this = bottom of hormetic range.
pub const HORMETIC_HIGH_DIVISOR: f64 = 100.0;

/// Default tissue repair capacity (fraction of binding load absorbable).
pub const TISSUE_REPAIR_CAPACITY: f64 = 0.05;

/// Toxicity IPR threshold: below this, toxicity is delocalized (manageable).
pub const TOX_IPR_DELOCALIZED: f64 = 0.15;

/// Toxicity IPR threshold: above this, toxicity is localized (dangerous).
pub const TOX_IPR_LOCALIZED: f64 = 0.50;

// ── Classification Bound Constants ────────────────────────────────────

/// Tissue geometry near-saturation lower bound — large ξ → geometry → 1.0.
pub const TISSUE_GEOMETRY_SATURATION: f64 = 0.99;

/// Tissue geometry near-zero upper bound — small ξ → geometry → 0.0.
pub const TISSUE_GEOMETRY_ZERO: f64 = 0.02;

/// Normalized cross-correlation discrimination threshold — PVC vs BBB.
pub const NCC_DISCRIMINATION: f64 = 0.95;

/// Minimum energy threshold for nonzero beat template validity.
pub const BEAT_ENERGY_FLOOR: f64 = 0.1;

/// Heart rate physiological lower bound (bpm).
pub const HR_PHYSIO_LOW_BPM: f64 = 40.0;

/// Heart rate physiological upper bound (bpm).
pub const HR_PHYSIO_HIGH_BPM: f64 = 200.0;

// ── Numerical Guard Constants ────────────────────────────────────────

/// RMSE decomposition near-zero guard — prevents `0/0` in bias fraction.
pub const DECOMPOSITION_GUARD: f64 = 1e-30;

/// Box-Muller `u1` clamp — prevents `ln(0)` in normal sampling.
pub const BOX_MULLER_CLAMP: f64 = 1e-30;

// ── Upstream Contract Tolerances ─────────────────────────────────────
//
// Values agreed with other ecosystem components via wateringHole handoffs.
// Changes here must be coordinated with the upstream primal/spring.

/// barraCuda GPU f32 parity for Hill: cross-spring agreed tolerance.
/// Source: `HEALTHSPRING_V42_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR24_2026.md`
pub const UPSTREAM_GPU_HILL_PARITY: f64 = 1e-4;

/// barraCuda GPU diversity dispatch: cross-spring agreed tolerance.
pub const UPSTREAM_GPU_DIVERSITY_PARITY: f64 = 1e-4;

/// barraCuda fused pipeline vs sequential dispatch: agreed tolerance.
pub const UPSTREAM_GPU_FUSED_PARITY: f64 = 1e-4;

/// PRNG seed determinism: cross-spring agreed — LCG with identical seed
/// must produce identical sequences across springs.
pub const UPSTREAM_PRNG_DETERMINISM: f64 = 0.0;

/// Shannon/Simpson cross-validation to Python: agreed with groundSpring.
pub const UPSTREAM_DIVERSITY_CROSS_VALIDATE: f64 = 1e-8;

// ── IPC Configuration Constants ──────────────────────────────────────

/// JSON-RPC response buffer size (bytes) for capability probes.
pub const IPC_PROBE_BUF: usize = 8192;

/// JSON-RPC response buffer size (bytes) for Songbird / petalTongue.
pub const IPC_RESPONSE_BUF: usize = 4096;

/// IPC socket read/write timeout (milliseconds).
pub const IPC_TIMEOUT_MS: u64 = 500;

// ── Test and guard constants ────────────────────────────────────────────

/// Unit test assertions — tight tolerance for analytical identities (qs, gpu, uncertainty, etc.).
pub const TEST_ASSERTION_TIGHT: f64 = 1e-10;

/// Unit test assertions — loose tolerance for pkpd diagnostics.
pub const TEST_ASSERTION_LOOSE: f64 = 0.01;

/// Unit test assertions — medium tolerance for diagnostics, NCA, compound tests.
pub const TEST_ASSERTION_MEDIUM: f64 = 1e-6;

/// Denominator guard — prevents division by zero in ppg, nca, diagnostics, discovery.
pub const DIVISION_GUARD: f64 = 1e-15;

/// `SpO2` clinical tolerance (percent) in ppg tests.
pub const SPO2_CLINICAL_TOLERANCE: f64 = 5.0;

/// NCA local tolerance — iterative bisection and numerical identity checks.
pub const NCA_TOLERANCE: f64 = 1e-6;

/// Unit test assertions — 2% relative error for AUC, NCA, compartment tests.
pub const TEST_ASSERTION_2_PERCENT: f64 = 0.02;

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
