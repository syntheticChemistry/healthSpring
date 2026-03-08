# healthSpring V4 → barraCuda + toadStool + metalForge Evolution Handoff
#
# SPDX-License-Identifier: AGPL-3.0-or-later

**Date**: March 8, 2026
**From**: healthSpring (human health applications)
**To**: barraCuda (GPU math), toadStool (heterogeneous dispatch), metalForge (substrate routing)
**License**: AGPL-3.0-or-later
**healthSpring Version**: V4 (Tier 0+1 complete — all 4 tracks + validation)
**barraCuda Version**: v0.3.3 (`a898dee`)
**toadStool Version**: S130+ (`bfe7977b`)
**Supersedes**: V3 handoff (archived)

---

## Executive Summary

- healthSpring V4: **24 experiments** across 4 tracks + 1 validation
- **280 binary checks, 185 unit tests (145 barracuda + 27 metalForge + 13 toadStool), 104 cross-validation**
- **96.84% code coverage** (llvm-cov)
- New: GPU dispatch layer (`gpu.rs`), metalForge NUCLEUS atomics, toadStool pipeline skeleton
- Zero unsafe, zero clippy warnings, zero TODOs in source
- All queued papers complete. Ready for GPU Tier 2 absorption.

---

## Part 1: What healthSpring Built (V3 → V4)

### 1.1 Track Summary

| Track | Domain | Experiments | Rust Binary | Lib Tests |
|-------|--------|:-----------:|:-----------:|:---------:|
| 1 — PK/PD | Pharmacokinetics, dose-response, population, PBPK | 6 (001-006) | 52 | 45 |
| 2 — Microbiome | Gut diversity, Anderson lattice, C. diff, FMT | 4 (010-013) | 36 | 18 |
| 3 — Biosignal | QRS, HRV, PPG SpO2, fusion | 4 (020-023) | 40 | 28 |
| 4 — Endocrinology | Testosterone PK, TRT outcomes, gut axis, HRV×TRT | 9 (030-038) | 117 | 55 |
| Validation | barraCuda CPU parity | 1 (040) | 15 | — |
| **Total** | | **24** | **280** | **185** |

### 1.2 New Experiments (7 added since V3)

- **Exp006**: PBPK 5-tissue physiological compartments (Gabrielsson & Weiner)
- **Exp013**: FMT microbiota transplant for rCDI (engraftment → diversity restoration)
- **Exp021**: HRV metrics (SDNN, RMSSD, pNN50) from R-peak intervals
- **Exp022**: PPG SpO2 R-value calibration (Beer-Lambert)
- **Exp023**: Multi-channel biosignal fusion (ECG + PPG + EDA → stress index)
- **Exp038**: HRV × TRT cardiovascular cross-track (Mok hypothesis D3)
- **Exp040**: barraCuda CPU parity (15 analytical contracts)

### 1.3 New Library Functions

**pkpd** (`pbpk.rs` module):
- `TissueCompartment`, `PbpkState`, `pbpk_iv_simulate`, `cardiac_output`

**biosignal**:
- `rmssd_ms`, `pnn50`, `ppg_r_value`, `spo2_from_r`, `generate_synthetic_ppg`, `ppg_extract_ac_dc`
- `eda_scl`, `eda_phasic`, `eda_detect_scr`, `fuse_channels`, `FusedHealthAssessment`

**microbiome**:
- `fmt_blend`, `bray_curtis`

**endocrine**:
- `hrv_trt_response`, `cardiac_risk_composite`

**gpu** (`gpu.rs`):
- `GpuOp` (HillSweep, PopulationPkBatch, DiversityBatch), `execute_cpu`, `shader_for_op`, `gpu_memory_estimate`

### 1.4 New Infrastructure

- **metalForge** (27 tests): Substrate/Workload dispatch, NUCLEUS atomics (Tower→Node→Nest), PCIe P2P transfer planning, GPU/NPU/CPU capability discovery
- **toadStool** (13 tests): Pipeline/Stage/StageOp with Generate/Transform(Hill,Square,ExpDecay)/Reduce(Sum,Mean,Max,Min,Variance)/Filter, CPU execution reference

---

## Part 2: GPU Workload Candidates (updated)

Same priority as V3 but now with concrete `GpuOp` enum and `shader_for_op()` mapping:

| healthSpring Op | barraCuda Shader | Use Case |
|-----------------|------------------|----------|
| HillSweep | `batched_elementwise_f64.wgsl` | Exp001 vectorized |
| PopulationPkBatch | custom `population_pk_f64.wgsl` | Exp005/036 Monte Carlo |
| DiversityBatch | `mean_variance_f64.wgsl` | Exp010 batch |
| FftBiosignal | `fft_radix2_f64.wgsl` | Exp020 real-time |
| AndersonEigensolve | `anderson_lyapunov_f64.wgsl` | Exp011/037 |

**Exp005/036 (population PK Monte Carlo) remains the recommended first GPU experiment.**

---

## Part 3: Absorption Candidates (updated with new functions)

Add to V3 list:

| Function | Current Location | Absorption Target |
|----------|-----------------|-------------------|
| `pbpk_iv_simulate` | `pkpd/pbpk.rs` | `barraCuda::bio::pbpk` (GPU ODE integration for tissue models) |
| `bray_curtis` | `microbiome.rs` | `barraCuda::bio::ecology` (community dissimilarity) |
| `fmt_blend` | `microbiome.rs` | `barraCuda::bio::ecology` (weighted community blend) |
| `fuse_channels` | `biosignal.rs` | toadStool pipeline stage (multi-modal sensor fusion) |
| `cardiac_risk_composite` | `endocrine.rs` | `barraCuda::epi::risk` (composite risk scoring) |
| NUCLEUS atomics | `metalForge/forge` | toadStool absorption (Tower/Node/Nest topology, P2P transfer) |
| Pipeline/Stage/StageOp | `toadstool` | toadStool core (compute DAG execution) |

**Existing V3 absorption candidates** (unchanged): `pk_multiple_dose`, `hill_dose_response`, `allometric_scale`, `shannon_index`, `anderson_localization_length`, `lognormal_params`, `population_pk_cpu`, `hazard_ratio_model`, `pan_tompkins_qrs`.

### Absorption Priority

1. `population_pk_cpu` → GPU template (highest impact)
2. `pbpk_iv_simulate` → GPU ODE (PBPK tissue models)
3. `hill_dose_response` → GPU element-wise (universal in pharmacology)
4. `lognormal_params` → utility (needed by any population model)
5. `bray_curtis`, `fmt_blend` → ecology module
6. `fuse_channels` → toadStool pipeline stage
7. NUCLEUS atomics → toadStool topology

---

## Part 4: What We Learned (updated)

### 4.1 For barraCuda

1. **PBPK models need GPU ODE integration** (Euler minimum, RK4 preferred) — PBPK is NOT embarrassingly parallel across patients; it's embarrassingly parallel across time steps within each patient.

2. **PPG SpO2 is element-wise** (R-value → calibration curve) — natural `batched_elementwise_f64.wgsl` target.

3. **EDA processing reuses moving-window integration** — same as biosignal MWI.

4. **Analytical contracts (exp040) define the mathematical parity spec** between CPU and GPU — 15 contracts cover Hill, Bateman, diversity, Anderson, lognormal, biomarker trajectories.

5. **Bateman equation does not need an ODE solver** — analytical IM depot model. ODE solvers are for multi-compartment PBPK.

6. **Lognormal IIV is standard** — `lognormal_params(typical, cv)` utility benefits all springs doing Monte Carlo.

7. **Anderson localization generalizes** — parameterized `ξ(W)` unifies soil/gut/skin substrates.

### 4.2 For toadStool

1. **Multi-channel fusion (ECG+PPG+EDA) is a natural pipeline** — 3 source stages → 3 processing stages → 1 fusion stage.

2. **NUCLEUS atomics define the hardware topology** for dispatch — Tower→Node→Nest hierarchy.

3. **PCIe P2P DMA estimates enable scheduling decisions** — GPU↔NPU bypass vs host-staged.

4. **Pipeline stages map directly to WGSL shader dispatch units** — StageOp (Generate/Transform/Reduce/Filter) → workgroup launch.

5. **Population Monte Carlo remains the first metalForge workload** — 10K-workgroup dispatch, streaming params in → stats out.

6. **NPU path for biosignal** — Pan-Tompkins, HRV, PPG SpO2; Akida AKD1000 target.

### 4.3 For coralReef

1. **All healthSpring WGSL targets use f64 precision** — coralReef must emit fp64 instructions.

2. **The `df64_core.wgsl` preamble pattern from barraCuda is required** for consumer GPUs without native fp64.

### 4.4 Cross-Track Discovery: Testosterone-Gut Axis (Exp037)

Unchanged from V3: gut microbiome diversity (Pielou evenness) correlates with TRT metabolic response via Anderson localization. **Exp038** extends this: HRV (SDNN) × TRT cardiovascular risk — Mok hypothesis D3 validated at Tier 0+1.

---

## Part 5: Status and Pins

| Component | Version | Pin |
|-----------|---------|-----|
| healthSpring | V4 | — |
| barraCuda | v0.3.3 | `a898dee` |
| toadStool | S130+ | `bfe7977b` |
| metalForge (forge) | 0.1.0 | healthSpring workspace |
| coralReef | Iteration 10 | `d29a734` |
| wetSpring | V99 | — |
| neuralSpring | V90 | — |
| groundSpring | V100 | — |
| hotSpring | v0.6.17+ | — |
| airSpring | v0.7.5 | — |

| Metric | Value |
|--------|-------|
| Experiments | 24 |
| Rust binary checks | 280 |
| Unit tests | 185 (145 barracuda + 27 metalForge + 13 toadStool) |
| Cross-validation checks | 104 |
| Code coverage (llvm-cov) | 96.84% |
| Rust edition | 2024 |
| rust-version | 1.87 |

---

## Part 6: Recommended Next Steps

### For barraCuda team:

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 1 | Write `population_pk_f64.wgsl` (first healthSpring GPU experiment) | **P0** | Template: 10K workgroups × 1 patient/wg, Bateman equation, lognormal IIV |
| 2 | Confirm ODE solver absorption status (Euler, RK4 from wetSpring) | P1 | Needed for PBPK Exp006, not for analytical PK |
| 3 | Add `lognormal_params(typical, cv) → (μ, σ)` utility | P1 | Used by all population models |
| 4 | Parameterize Anderson `ξ(W)` as reusable module | P2 | Unifies soil/gut/skin substrates |
| 5 | Assess fused-op chain for Shannon/Simpson (log→mul→sum) | P2 | May eliminate need for custom diversity shaders |
| 6 | Consider `allometric_scale` in core math utilities | P3 | Universal in pharmacology |
| 7 | Add `bray_curtis`, `fmt_blend` to ecology module | P3 | FMT/rCDI, community dissimilarity |

### For toadStool team:

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 1 | Validate 10K-workgroup dispatch for population PK Monte Carlo | **P0** | Streaming: params in → stats out |
| 2 | Absorb NUCLEUS atomics (Tower/Node/Nest topology) | P1 | From metalForge/forge |
| 3 | Absorb Pipeline/Stage/StageOp as core compute DAG | P1 | From healthspring-toadstool |
| 4 | Add `fuse_channels` as multi-modal pipeline stage | P2 | ECG+PPG+EDA → stress index |
| 5 | NPU dispatch path for biosignal inference (Pan-Tompkins, HRV) | P2 | Akida AKD1000 target |
| 6 | metalForge routing: GPU (server) → CPU (desktop) → NPU (wearable) | P2 | Clinical PK cross-substrate |

### For coralReef:

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 1 | Emit fp64 instructions for all healthSpring WGSL targets | **P0** | Hill, population PK, diversity, FFT, Anderson |
| 2 | Support `df64_core.wgsl` preamble for non-native fp64 GPUs | P1 | Consumer GPU compatibility |

---

**License:** AGPL-3.0-or-later
