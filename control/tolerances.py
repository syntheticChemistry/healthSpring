# SPDX-License-Identifier: AGPL-3.0-or-later
"""Centralized tolerance constants — Python subset of ecoPrimal/src/tolerances.rs.

This file contains the tolerances used by Python cross-validation baselines.
It is an intentional *subset* of the Rust source of truth (tolerances.rs).
Constants present here MUST match their Rust counterparts exactly.

Rust-only constants (NLME algorithm params, IPC buffers, classification
thresholds, upstream contract tolerances, test helpers) are deliberately
omitted — they have no Python consumer.  The authoritative registry is
ecoPrimal/src/tolerances.rs + specs/TOLERANCE_REGISTRY.md.
"""

# ── Machine Epsilon Class (1e-10 to 1e-15) ──────────────────────────
MACHINE_EPSILON = 1e-10
MACHINE_EPSILON_TIGHT = 1e-12
MACHINE_EPSILON_STRICT = 1e-15
ANDERSON_IDENTITY = 1e-14
TWO_COMPARTMENT_IDENTITY = 1e-8
DIVERSITY_CROSS_VALIDATE = 1e-8

# ── Numerical Method Class (1e-6 to 0.01) ───────────────────────────
AUC_TRAPEZOIDAL = 0.01
TMAX_NUMERICAL = 0.1
HALF_LIFE_POINT = 1e-6
ALLOMETRIC_CL_RATIO = 1e-6
ABUNDANCE_NORMALIZATION = 1e-6
LEVEL_SPACING_RATIO = 0.02
W_CROSS_VALIDATE = 0.01
PIELOU_BOUNDARY = 0.001
EXPONENTIAL_RESIDUAL = 0.01
AUC_TRUNCATED = 0.10
TERMINAL_SLOPE = 0.01
TMAX_CROSS_SPECIES = 0.5

# ── Population / Statistical Class (0.05 to 0.15) ───────────────────
LOGNORMAL_RECOVERY = 0.05
POP_VD_MEDIAN = 0.05
POP_KA_MEDIAN = 0.10
POP_AUC_CV = 0.15
ACCUMULATION_FACTOR = 0.25
WASHOUT_HALF_LIFE = 0.15

# ── Clinical Plausibility Class (0.25 to 0.60) ──────────────────────
FRONT_LOADED_WEIGHT = 0.60
FRONT_LOADED_LDL = 0.55
FRONT_LOADED_HBA1C = 0.80
QRS_SENSITIVITY = 0.80
HR_DETECTION_BPM = 10.0
SDNN_UPPER_MS = 200.0
QRS_PEAK_MATCH_MS = 75.0

# ── GPU Parity Class (1e-4 to 0.25) ─────────────────────────────────
GPU_F32_TRANSCENDENTAL = 1e-4
GPU_STATISTICAL_PARITY = 0.25
GPU_FUSED_PARITY = 1e-4
GPU_SCALING_LINEARITY = 0.01

# ── CPU Parity Class (1e-10) ────────────────────────────────────────
CPU_PARITY = 1e-10
PIPELINE_STAGE = 1e-10

# ── Diagnostic Pipeline Class ───────────────────────────────────────
HILL_AT_EC50 = 1.0
DETERMINISM = 1e-12

# ── PCIe / Hardware Class ───────────────────────────────────────────
PCIE_BANDWIDTH = 0.1
PCIE_GEN3_16X_GBPS = 15.76
PCIE_GEN4_16X_GBPS = 31.504
PCIE_GEN5_16X_GBPS = 63.008

# ── Gut-Brain Serotonin Class ───────────────────────────────────────
TRP_RANGE_LOW = 80.0
TRP_RANGE_HIGH = 180.0
SEROTONIN_MIDPOINT_LOW = 40.0
SEROTONIN_MIDPOINT_HIGH = 60.0

# ── Hill Saturation Class ───────────────────────────────────────────
HILL_SATURATION_100X = 0.99

# ── NLME / Pipeline Class ───────────────────────────────────────────
FOCE_THETA_RECOVERY = 0.30
SAEM_THETA_RECOVERY = 0.50
NCA_LAMBDA_Z = 0.05
NCA_AUC_INF = 0.05
CWRES_MEAN = 2.0
PBPK_MASS_CONSERVATION = 0.25
BIOMARKER_ENDPOINT = 0.5
LOGNORMAL_MEAN = 0.01

# ── Track 7: Drug Discovery Class ───────────────────────────────────
MATRIX_PATHWAY = 1e-10
TISSUE_GEOMETRY = 1e-10
DISORDER_IMPACT = 1e-10
MATRIX_COMBINED = 1e-10
HTS_Z_PRIME = 1e-10
HTS_SSMD = 1e-10
HTS_PERCENT_INHIBITION = 1e-10
IC50_ESTIMATION = 0.10
SELECTIVITY_INDEX = 1e-10

# ── Track 6: Comparative Medicine Class ──────────────────────────────
IL31_INITIAL = 1e-10
IL31_STEADY_STATE = 0.01
PRURITUS_AT_EC50 = 1e-10
LOKIVETMAB_DECAY = 1e-10
JAK1_SELECTIVITY = 1e-10
ALLOMETRIC_ROUNDTRIP = 1e-6
CROSS_SPECIES_PK = 0.05
PRURITUS_TIME_COURSE = 0.01
LOKIVETMAB_DURATION = 1.0
LOKIVETMAB_ONSET = 1.0
FIBROTIC_GEOMETRY = 1e-10
ANTI_FIBROTIC_SCORE = 1e-10
FRACTIONAL_AT_IC50 = 1e-10
FELINE_MM_PK = 0.01
FELINE_T4_RESPONSE = 0.01
CANINE_GUT_ANDERSON = 0.01

# ── Numerical Guard Constants ───────────────────────────────────────
DECOMPOSITION_GUARD = 1e-30
