<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring BarraCUDA Requirements

**Last Updated**: March 10, 2026
**Status**: V14 — Tier 2+3 GPU live. NLME population PK (FOCE + SAEM), NCA, diagnostics (CWRES, VPC, GOF), WFDB parser. Kokkos-equivalent benchmarks validate GPU-portable patterns. Full petalTongue pipeline: 28 nodes, 29 edges, 121 channels. Sovereign NONMEM/Monolix/WinNonlin replacements validated.

---

## Overview

healthSpring consumes primitives from the standalone `barraCuda` library (vendor-agnostic GPU math, f64-canonical WGSL shaders). This document tracks which primitives are available, which need to be written locally (Write phase), which have been validated locally and are ready for absorption (Absorb phase), and which have been absorbed upstream (Lean phase).

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

Kokkos-equivalent benchmarks (`barracuda/benches/kokkos_parity.rs`) validate these patterns ahead of GPU shader promotion: reduction, scatter, Monte Carlo, ODE batch, NLME iteration.

## Still Needed: Write Phase (local WGSL)

| Category | Primitive | Purpose | Priority |
|----------|----------|---------|----------|
| PK/PD | `compartment_ode_rk4` | Higher-order ODE integration for GPU | High |
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

## ODE Solver Note

wetSpring has ODE solvers (Euler, RK4) in its Rust CPU tier. These may already be in the absorption pipeline to barraCuda. Check `wetSpring/metalForge/ABSORPTION_STRATEGY.md` and `barraCuda/CHANGELOG.md` before writing local copies. The healthSpring PBPK model uses simple Euler integration (dt=0.01 hr) — an RK4 GPU kernel would improve both accuracy and throughput.

---

## GPU Learnings for barraCuda Team

1. `enable f64;` in WGSL must be stripped — wgpu/naga handles f64 via device features, not shader directives
2. `pow(f64, f64)` is unsupported on NVIDIA via NVVM — use `exp(n * log(c))` cast through f32
3. u64 PRNG not portable — use u32-only xorshift32 + Wang hash for GPU Monte Carlo
4. Fused pipeline (single encoder) eliminates ~30x overhead at small sizes vs individual dispatches
5. At 10M+ elements, memory bandwidth dominates — buffer streaming needed for next tier
6. IPC response buffer should be 64KB minimum for capability responses
