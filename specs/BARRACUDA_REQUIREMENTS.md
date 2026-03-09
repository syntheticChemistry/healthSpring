# healthSpring BarraCUDA Requirements

**Last Updated**: March 9, 2026
**Status**: V7 — Tier 2 GPU live (3 WGSL shaders, GpuContext, fused pipeline). Full visualization.

---

## Overview

healthSpring consumes primitives from the standalone `barraCuda` library (vendor-agnostic GPU math, f64-canonical WGSL shaders). This document tracks which primitives are available, which need to be written locally (Write phase), and which have been absorbed upstream (Lean phase).

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

## Needed: Write Phase (local WGSL)

| Category | Primitive | Purpose | Priority |
|----------|----------|---------|----------|
| PK/PD | `compartment_ode_euler` | 1/2/3-compartment ODE integration | High |
| PK/PD | `compartment_ode_rk4` | Higher-order ODE integration | High |
| PK/PD | `hill_equation_gpu` | Vectorized Hill dose-response | High |
| PK/PD | `population_pk_sample` | Monte Carlo virtual patient generation | High |
| Microbiome | `shannon_diversity_gpu` | Parallel Shannon H' computation | Medium |
| Microbiome | `anderson_xi_1d_gut` | 1D localization length for gut lattice | Medium |
| Biosignal | `bandpass_iir_gpu` | IIR bandpass filter (ECG conditioning) | Medium |
| Biosignal | `qrs_detect_gpu` | Parallel QRS detection across channels | Medium |
| Biosignal | `ppg_spo2_ratio` | R-value to SpO2 lookup | Medium |

## Absorption Targets (upstream to barraCuda)

Once validated locally, health-specific primitives that generalize (ODE solvers, diversity metrics, bandpass filters) are candidates for absorption into the standalone `barraCuda` library, following the Write → Absorb → Lean cycle.

---

## ODE Solver Note

wetSpring already has ODE solvers (Euler, RK4) in its Rust CPU tier. These may already be in the absorption pipeline to barraCuda. Check `wetSpring/metalForge/ABSORPTION_STRATEGY.md` and `barraCuda/CHANGELOG.md` before writing local copies.
