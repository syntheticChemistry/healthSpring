<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring V27 Deep Evolution — barraCuda / toadStool Handoff

**Date**: March 15, 2026
**From**: healthSpring V27 (Deep Evolution Sprint)
**To**: barraCuda team, toadStool team, coralReef, biomeOS
**Supersedes**: V25 Track 6+7 Handoff

---

## Summary

V27 completes a deep evolution sprint: cross-spring absorptions, modern Rust idioms, IPC safety, ODE→WGSL codegen integration, and uncertainty quantification. 601 tests, zero clippy warnings, zero unsafe.

---

## V27 Changes Relevant to Upstream

### 1. ODE→WGSL Codegen (barraCuda `OdeSystem` integration)

healthSpring now implements barraCuda's `OdeSystem` trait for 3 PK models:

| ODE System | States | Params | File |
|-----------|--------|--------|------|
| `MichaelisMentenOde` | 1 (C) | 3 (Vmax, Km, Vd) | `gpu/ode_systems.rs` |
| `OralOneCompartmentOde` | 2 (A_gut, C_plasma) | 5 (dose, F, Vd, Ka, Ke) | `gpu/ode_systems.rs` |
| `TwoCompartmentOde` | 2 (C1, C2) | 4 (k10, k12, k21, V1) | `gpu/ode_systems.rs` |

Each provides:
- `wgsl_derivative()` — WGSL source for GPU kernel generation via `BatchedOdeRK4::generate_shader()`
- `cpu_derivative()` — CPU fallback matching the wetSpring pattern

**Action for barraCuda**: These ODE systems are health-domain. Consider absorbing them into `barracuda::numerical::ode_bio::` alongside the existing wetSpring systems (PhageDefense, Bistable, etc.). This would make them available ecosystem-wide for any spring doing PK modeling.

### 2. Capability Registry (IPC dispatch evolution)

The 46-arm `match` in `ipc/dispatch/mod.rs` was replaced with a static `REGISTRY: &[CapabilityEntry]` table. Each entry maps `(method, handler_fn, domain)`. New public function `registered_capabilities()` exposes the registry for machine introspection.

**Action for toadStool/biomeOS**: healthSpring now exposes `registered_capabilities()` which returns `Vec<(&str, &str)>` — method→domain pairs. This aligns with `capability.list` and enables programmatic capability discovery.

### 3. Uncertainty Quantification Module

Absorbed from groundSpring's measurement-science patterns:

| Function | Source Pattern | healthSpring Use |
|----------|--------------|-----------------|
| `bootstrap_mean/median` | groundSpring `bootstrap.rs` | Population PK CI estimation |
| `jackknife_mean_variance` | groundSpring `jackknife.rs` | Microbiome index variance |
| `decompose_error(mbe, rmse)` | groundSpring `decompose.rs` | Model bias-variance partitioning |
| `monte_carlo_propagate` | groundSpring MC pattern | PK parameter uncertainty propagation |
| `mbe` | groundSpring `stats.rs` | Mean Bias Error for model evaluation |

**Action for barraCuda**: `monte_carlo_propagate` is a general pattern (perturb→model→summarize). Consider a GPU-accelerated version in barraCuda for batched uncertainty propagation.

### 4. IPC Cast Safety

Added safe parameter extraction helpers to `ipc/dispatch/handlers/mod.rs`:
- `sz(params, key) -> Option<usize>` — uses `usize::try_from` with saturation
- `sz_or(params, key, default) -> usize` — with default
- `sza(params, key) -> Option<Vec<usize>>` — array variant

Eliminated ~40 raw `as usize` casts across all IPC handler files. Added `len_f64()` utility for safe precision-annotated `usize→f64` conversion.

### 5. `core::` Imports

`std::fmt` → `core::fmt`, `std::f64` → `core::f64` in `validation.rs` and `biosignal/fft.rs`. Preparing for potential `no_std` library extraction.

---

## Metrics

| Metric | V25 | V27 |
|--------|-----|-----|
| Tests | 501 | 601 |
| Experiments | 73 | 73 |
| Capabilities | 55+ | 55+ (registry-backed) |
| ODE systems (OdeSystem trait) | 0 | 3 |
| Uncertainty functions | 0 | 6 |
| Raw `as usize` in IPC | ~40 | 0 |
| Clippy warnings | 0 | 0 |
| Unsafe blocks | 0 | 0 |

---

## GPU Candidates (New from V27)

| Candidate | Current | GPU Path |
|-----------|---------|----------|
| `MichaelisMentenOde` | CPU via `BatchedOdeRK4` | `generate_shader()` → wgpu dispatch |
| `OralOneCompartmentOde` | CPU via `BatchedOdeRK4` | `generate_shader()` → wgpu dispatch |
| `TwoCompartmentOde` | CPU via `BatchedOdeRK4` | `generate_shader()` → wgpu dispatch |
| `monte_carlo_propagate` | CPU loop | Batched perturbation kernel |

---

## Cross-Spring Absorption Status

| Source | Absorbed | Module |
|--------|----------|--------|
| wetSpring `generate_shader()` pattern | `OdeSystem` impls | `gpu/ode_systems.rs` |
| groundSpring `bootstrap/jackknife/decompose` | Full absorption | `uncertainty.rs` |
| groundSpring agreement stats (V26) | `rmse/mae/nse/r²/d` | `validation.rs` |
| wetSpring `AttentionState` (V26) | Hysteresis model | `biosignal/attention.rs` |
| neuralSpring gut params (V26) | Transfer params | `microbiome_transfer.rs` |

---

## Next Evolution Targets

1. **GPU dispatch for ODE systems**: Wire `generate_shader()` → `wgpu::ComputePipeline` for batched PK parameter sweeps
2. **Tier B shader absorption**: MM PK, SCFA, beat classify → barraCuda canonical ops
3. **HMM absorption from neuralSpring**: Hidden Markov Models for biosignal regime detection
4. **ESN classifier from neuralSpring**: Echo State Network for attention state prediction
5. **Tissue Anderson from groundSpring**: 3D Anderson lattice for tissue heterogeneity modeling
