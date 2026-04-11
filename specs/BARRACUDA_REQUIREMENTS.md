<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring BarraCUDA Requirements

**Last Updated**: April 10, 2026
**Status**: V49 — Composition validation audit. barraCuda v0.3.11 (CI-gated pin, rev `7f6649f`). All six GPU ops LIVE. `uncertainty::std_dev` delegated to `barracuda::stats::correlation::std_dev`. Local WGSL shaders retained for validation but all absorbed upstream. TensorSession adoption pending upstream API. Sovereign dispatch pilot (Hill-only via coralReef). Prior: V44 Cross-Spring Absorption (handoff in archive).

---

## Overview

healthSpring consumes primitives from the standalone `barraCuda` library (vendor-agnostic GPU math, f64-canonical WGSL shaders). This document tracks which primitives are available, which need to be written locally (Write phase), which have been validated locally and are ready for absorption (Absorb phase), and which have been absorbed upstream (Lean phase).

**V42 GPU surface:** The six LIVE ops above are validated by Exp053 (Tier A trio), Exp083 (V16 trio parity), and dispatch/scaling experiments Exp054–Exp060, Exp085–Exp087.

---

## Available from barraCuda (consume directly)

| Category | Primitives | barraCuda version |
|----------|-----------|-------------------|
| Core math | exp, log, pow, sqrt, abs, floor, ceil, clamp | v0.3.x |
| Linear algebra | matmul, dot, transpose, norm, solve_triangular | v0.3.x |
| Reduction | sum, mean, variance, min, max | v0.3.x |
| Statistics | histogram, percentile, median | v0.3.x |
| Activation | relu, sigmoid, tanh, gelu, softmax | v0.3.x |
| Attention | scaled_dot_product, multi_head, causal, sparse, rotary, cross, alibi | v0.3.x |
| CNN | conv2d, batch_norm, pooling, elementwise | v0.3.x |
| Loss | focal, contrastive, huber, bce, mse, mae | v0.3.x |
| Optimizer | sgd, adam, adagrad, rmsprop | v0.3.x |

## Written Locally — Ready for Absorption

These primitives have been validated in healthSpring and are ready for the barraCuda team to absorb.

| Category | Primitive | Purpose | WGSL Shader | Status |
|----------|----------|---------|-------------|--------|
| PK/PD | `hill_dose_response` | Vectorized Hill dose-response | `hill_dose_response_f64.wgsl` | **GPU LIVE** — Exp053, crossover at 100K |
| PK/PD | `population_pk_cpu` | Monte Carlo virtual patient generation | `population_pk_f64.wgsl` | **GPU LIVE** — Exp053, crossover at 5M |
| Microbiome | `shannon_index` + `simpson_index` | Parallel diversity computation | `diversity_f64.wgsl` | **GPU LIVE** — Exp053, workgroup reduction |
| PK/PD | `GpuOp::MichaelisMentenBatch` / MM PK batch | Batch nonlinear PK (MM) | `michaelis_menten_batch_f64.wgsl` | **GPU LIVE** — Exp083/085/086/087; `GpuContext` + `dispatch::execute_gpu` |
| Microbiome | `GpuOp::ScfaBatch` | Batch SCFA production (MM kinetics) | `scfa_batch_f64.wgsl` | **GPU LIVE** — same as above |
| Biosignal | `GpuOp::BeatClassifyBatch` | Template correlation beat classification | `beat_classify_batch_f64.wgsl` | **GPU LIVE** — same as above |
| PK/PD | `pk_iv_bolus`, `pk_oral_one_compartment`, `pk_two_compartment_iv` | Compartment PK models | CPU only | **VALIDATED** — Exp001-003, 39 lib tests |
| PK/PD | `pbpk_iv_simulate`, `pbpk_iv_tissue_profiles` | PBPK multi-compartment ODE with tissue profiles | CPU only | **VALIDATED** — Exp006, 13 checks + lib tests |
| PK/PD | `allometric_scale`, `mab_pk_sc` | mAb cross-species scaling | CPU only | **VALIDATED** — Exp004, 7 checks |
| PK/PD | `auc_trapezoidal` | AUC computation (trapezoidal rule) | CPU only | **VALIDATED** — ready for parallel prefix |
| Microbiome | `anderson_hamiltonian_1d`, `ipr`, `localization_length_from_ipr` | Anderson gut lattice | CPU only | **VALIDATED** — Exp011, Anderson spectra exposed |
| Microbiome | `bray_curtis` | Community dissimilarity | CPU only | **VALIDATED** — ready for pairwise GPU |
| Microbiome | `fmt_blend` | FMT transplant model | CPU only | **VALIDATED** — Exp013, 12 checks |
| Biosignal | `pan_tompkins_qrs` | QRS detection (5-stage pipeline) | CPU only | **VALIDATED** — Exp020, intermediates exposed |
| Biosignal | `heart_rate_from_peaks`, `sdnn_ms`, `rmssd_ms`, `pnn50` | HRV metrics | CPU only | **VALIDATED** — Exp021, 10 checks |
| Biosignal | `ppg_r_value`, `spo2_from_r` | PPG SpO2 calibration | CPU only | **VALIDATED** — Exp022, 11 checks |
| Biosignal | `fuse_channels` | Multi-channel biosignal fusion | CPU only | **VALIDATED** — Exp023, pipeline stage candidate |
| Endocrine | `testosterone_decline`, `im_injection_pk`, `pellet_pk` | TRT pharmacokinetics | CPU only | **VALIDATED** — Exp030-032 |
| Endocrine | `hazard_ratio_model`, `cardiac_risk_composite` | Cardiovascular risk | CPU only | **VALIDATED** — Exp034-038 |
| Visualization | `PetalTonguePushClient`, `StreamSession` | petalTongue IPC (render, append, replace, gauge, caps, subscribe) | N/A | **VALIDATED** — Exp064, Exp073, Exp074 |
| NLME | `foce_estimate`, `saem_estimate` | FOCE + SAEM population PK estimation (sovereign NONMEM/Monolix) | CPU only | **VALIDATED** — Exp075, 30 subjects, theta/omega/sigma recovery |
| NCA | `nca_analysis` | Non-compartmental analysis: λz, AUC∞, MRT, CL, Vss (sovereign WinNonlin) | CPU only | **VALIDATED** — Exp075, lambda_z 5%, AUC_inf 5% |
| Diagnostics | `cwres_compute`, `vpc_simulate`, `gof_compute` | CWRES, VPC (50 sims), GOF | CPU only | **VALIDATED** — Exp075, CWRES mean <2.0, GOF R²≥0 |
| WFDB | `decode_format_212`, `decode_format_16` | PhysioNet streaming parser + beat annotations | CPU only | **VALIDATED** — Exp076, format round-trip |

### GPU Promotion Candidates (V14)

| Primitive | GPU Pattern | Priority |
|-----------|------------|----------|
| `foce_estimate` | Per-subject gradient is independent → batch parallel | High — FOCE is the NONMEM bottleneck |
| `vpc_simulate` | Each simulation is independent → embarrassingly parallel Monte Carlo | High — VPC with 1000+ sims needs GPU |
| `saem_estimate` | E-step sampling is parallelizable → batched Monte Carlo | Medium — SAEM E-step maps to existing PopPK pattern |
| `nca_analysis` | Per-subject NCA is independent → batch element-wise | Low — NCA is already fast on CPU |

Kokkos-equivalent benchmarks (`ecoPrimal/benches/kokkos_parity.rs`) validate these patterns ahead of GPU shader promotion: reduction, scatter, Monte Carlo, ODE batch, NLME iteration.

## V27 ODE→WGSL Codegen (barraCuda `OdeSystem` trait implementations)

healthSpring now implements `barracuda::numerical::OdeSystem` for 3 PK models in `gpu/ode_systems.rs`. Each provides `wgsl_derivative()` + `cpu_derivative()` for use with `BatchedOdeRK4::generate_shader()`.

| ODE System | States | Params | WGSL Status |
|-----------|--------|--------|-------------|
| `MichaelisMentenOde` | 1 (C) | 3 (Vmax, Km, Vd) | **Codegen ready** — `generate_shader()` produces valid WGSL |
| `OralOneCompartmentOde` | 2 (A_gut, C_plasma) | 5 (dose, F, Vd, Ka, Ke) | **Codegen ready** |
| `TwoCompartmentOde` | 2 (C1, C2) | 4 (k10, k12, k21, V1) | **Codegen ready** |

This resolves the `compartment_ode_rk4` "Write Phase" item below — ODE integration for GPU is now handled by the `OdeSystem` trait + `BatchedOdeRK4` generic solver.

## Still Needed: Write Phase (local WGSL)

| Category | Primitive | Purpose | Priority |
|----------|----------|---------|----------|
| PK/PD | ~~`compartment_ode_rk4`~~ | ~~Higher-order ODE integration for GPU~~ | ~~High~~ — **Resolved** by `OdeSystem` trait (V27) |
| Microbiome | `anderson_xi_1d_gut` | 1D localization length GPU kernel | Medium |
| Biosignal | `bandpass_iir_gpu` | IIR bandpass filter (ECG conditioning) | Medium |
| Biosignal | `qrs_detect_gpu` | Parallel QRS detection across channels | Medium — NPU path preferred |
| Biosignal | `fft_radix2_f64` | Radix-2 FFT for HRV power spectrum | Medium |

## Absorbed Upstream (Lean Phase)

| Primitive | absorbed into | Version |
|----------|---------------|---------|
| `DataChannel` enum | petalTongue `petal-tongue-core` | V6 |
| `ClinicalRange` struct | petalTongue `petal-tongue-core` | V6 |
| Chart renderers | petalTongue `petal-tongue-graph` | V6 |
| Clinical theme | petalTongue `petal-tongue-graph` | V6 |

---

## Absorption Priority for barraCuda Team

### P0 — Core Math (absorb into barraCuda v0.4.x)

1. `hill_dose_response` + WGSL shader — element-wise pharmacology primitive
2. `population_pk_cpu` + WGSL shader — embarrassingly parallel Monte Carlo ODE
3. `shannon_index` + `simpson_index` + WGSL shader — workgroup reduction ecology primitive
4. `push_replace`, `push_render_with_config`, `query_capabilities`, `subscribe_interactions` — petalTongue IPC client

### P1 — Health-Specific (absorb into barraCuda health module)

5. `pbpk_iv_simulate` + `pbpk_iv_tissue_profiles` — multi-compartment PBPK ODE
6. `PatientTrtProfile`, `trt_clinical_scenario()` — clinical parameterization
7. `auc_trapezoidal` — parallel prefix candidate

### P2 — Signal Processing (absorb into barraCuda signal module)

8. `pan_tompkins_qrs` — streaming detection pipeline (NPU path)
9. `fuse_channels` — multi-modal biosignal fusion
10. `bray_curtis` — pairwise dissimilarity matrix

---

## Track 6+7 Absorption Targets (V21 — NEW)

### Comparative Medicine (Track 6)

| Category | Primitive | Purpose | GPU Pattern | Priority |
|----------|----------|---------|-------------|----------|
| Cross-species | `species_params_registry` | Species parameter lookup (canine, human, feline, equine) | CPU lookup | P1 |
| Cross-species | `allometric_bridge` | Cross-species PK scaling (CL, Vd, t½ by body weight) | Element-wise | P1 |
| Cross-species | `species_pk_batch` | Species-parameterized compartment PK (batch) | Embarrassingly parallel | P1 — extends existing PopPK shader |
| Microbiome | `cross_species_gut_anderson` | Comparative gut Anderson (dog/human/mouse Pielou → W) | Workgroup reduction | P2 — extends diversity shader |
| Tissue | `species_tissue_lattice` | Species-parameterized tissue Anderson lattice | GPU eigensolve | P2 — hotSpring BatchedEighGpu |
| Immune | `species_immune_lattice` | Cross-species cytokine receptor density lattice | GPU eigensolve | P3 |

### Drug Discovery (Track 7)

| Category | Primitive | Purpose | GPU Pattern | Priority |
|----------|----------|---------|-------------|----------|
| Scoring | `matrix_score` | Fajgenbaum MATRIX drug repurposing framework | Batch element-wise | **P0 — FRONT** |
| Scoring | `anderson_matrix_score` | Anderson geometry augmented MATRIX | Workgroup reduction | **P0 — FRONT** |
| HTS | `hts_plate_analysis` | HTS plate reader data: Z'-factor, SSMD, hit scoring | Element-wise | **P0 — FRONT** |
| HTS | `compound_ic50_sweep` | Batch IC50/EC50 for compound library (8K × N concentrations) | Embarrassingly parallel (Hill sweep) | **P0 — FRONT** |
| QS | `qs_drug_target` | QS gene profiling → microbial drug target identification | Matrix ops | P1 |
| iPSC | `ipsc_readout_analysis` | iPSC viability/cytokine readout → computational validation | CPU structured | P2 |
| ChEMBL | `chembl_bioactivity_fetch` | ChEMBL REST API compound data extraction + normalization | CPU I/O | P2 |

### GPU Promotion for Track 7

| Primitive | GPU Pattern | Why GPU |
|-----------|------------|---------|
| `compound_ic50_sweep` | 8K compounds × 10 concentrations × 6 targets = 480K Hill evaluations | Existing `hill_dose_response_f64.wgsl` handles directly |
| `anderson_matrix_score` | Per-compound Anderson eigensolve + MATRIX score | Extends `diversity_f64.wgsl` + eigensolve |
| `matrix_score` | Per-compound scoring across drug-disease pairs | Element-wise, trivially parallel |

---

## ODE Solver Note

**V27 Update**: barraCuda now provides `BatchedOdeRK4` with the `OdeSystem` trait and `generate_shader()` for automatic WGSL kernel generation. healthSpring implements 3 `OdeSystem` variants (see above). The PBPK model still uses simple Euler integration (dt=0.01 hr) — migrating to `OdeSystem` trait is a future target. The `generate_shader()` path replaces the need for handwritten compartment ODE shaders.

---

## GPU Learnings for barraCuda Team

1. `enable f64;` in WGSL must be stripped — wgpu/naga handles f64 via device features, not shader directives
2. `pow(f64, f64)` is unsupported on NVIDIA via NVVM — use `exp(n * log(c))` cast through f32
3. u64 PRNG not portable — use u32-only xorshift32 + Wang hash for GPU Monte Carlo
4. Fused pipeline (single encoder) eliminates ~30x overhead at small sizes vs individual dispatches
5. At 10M+ elements, memory bandwidth dominates — buffer streaming needed for next tier
6. IPC response buffer should be 64KB minimum for capability responses
