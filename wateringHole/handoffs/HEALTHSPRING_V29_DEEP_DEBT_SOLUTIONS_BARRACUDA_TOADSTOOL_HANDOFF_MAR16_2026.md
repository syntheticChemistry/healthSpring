# HEALTHSPRING V29 — Deep Debt Solutions + barraCuda Absorption Handoff

**Date:** March 16, 2026
**From:** healthSpring V29
**To:** barraCuda, toadStool, All Springs
**Supersedes:** V28 (Deep Debt + Ecosystem Maturity)
**License:** scyBorg (AGPL-3.0-or-later code + ORC mechanics + CC-BY-SA 4.0 creative content)
**Covers:** V28 → V29 (full audit remediation, tolerance centralization, barraCuda delegation, GPU context rewire)

---

## Executive Summary

- **Full audit remediation** executed — every finding from V28 comprehensive audit resolved
- **Zero clippy warnings** workspace-wide (86 crates, pedantic + nursery)
- **70+ named tolerance constants** — zero inline magic numbers remaining in library or experiments
- **`mean()` delegated** to `barracuda::stats::mean` — zero duplicate math between healthSpring and upstream
- **`GpuContext::execute()`** now wires Tier A ops to barraCuda upstream when `barracuda-ops` enabled
- **Python tolerance mirror** (`control/tolerances.py`) — all 70+ constants mirrored for baseline reproducibility
- **IPC error extraction** centralized — `extract_rpc_error()` replaces 4 scattered patterns
- **603 tests pass**, 73 experiments, 113/113 cross-validation checks

---

## Part 1: What Changed (V28 → V29)

### 1.1 Experiment Binary Refactoring

4 experiment binaries failed `clippy::too_many_lines`. Refactored by domain concern:

| Binary | Before | After | Strategy |
|--------|--------|-------|----------|
| `exp090_matrix_scoring` | 187-line `main()` | `validate_pathway_selectivity`, `validate_tissue_geometry`, `validate_disorder_impact`, `validate_combined_scoring` | By MATRIX component |
| `exp092_compound_library` | 157-line `main()` | `build_panel`, `validate_batch_ranking` | Panel construction vs ranking validation |
| `exp093_chembl_jak_panel` | 196-line `main()` | `validate_hill_properties`, `validate_matrix_panel` | Hill pharmacology vs MATRIX scoring |
| `exp100_canine_il31` | 163-line `main()` | `validate_il31_kinetics`, `validate_pruritus_vas` | Kinetics vs VAS response |

### 1.2 Tolerance Centralization

All inline numeric constants replaced with named constants from `tolerances.rs`:

| Location | Before | After |
|----------|--------|-------|
| `validation.rs` `check_rel()` | `1e-15` | `MACHINE_EPSILON_STRICT` |
| `validation.rs` `nse()` | `1e-15` | `MACHINE_EPSILON_STRICT` |
| `validation.rs` `index_of_agreement()` | `1e-15` | `MACHINE_EPSILON_STRICT` |
| `uncertainty.rs` `decompose_error()` | `1e-30` | `DECOMPOSITION_GUARD` |
| `rng.rs` `BOX_MULLER_CLAMP` | local `const` | `tolerances::BOX_MULLER_CLAMP` |
| `ipc/socket.rs` | hardcoded `8192`, `500` | `IPC_PROBE_BUF`, `IPC_TIMEOUT_MS` |
| `visualization/capabilities.rs` | `RPC_RESPONSE_BUF` local | `IPC_RESPONSE_BUF` from tolerances |

New constants added to `tolerances.rs`:
- `DECOMPOSITION_GUARD` (1e-30) — RMSE decomposition near-zero guard
- `BOX_MULLER_CLAMP` (1e-30) — prevents `ln(0)` in normal sampling
- `IPC_PROBE_BUF` (8192) — capability probe buffer
- `IPC_RESPONSE_BUF` (4096) — Songbird/petalTongue RPC buffer
- `IPC_TIMEOUT_MS` (500) — socket read/write timeout

### 1.3 IPC Error Extraction

New `extract_rpc_error()` in `ipc/rpc.rs`:

```rust
pub fn extract_rpc_error(error: &serde_json::Value) -> (i64, String)
```

Replaces identical `unwrap_or(-1)` / `unwrap_or("unknown")` patterns in:
- `data/rpc.rs`
- `data/provenance.rs`
- `visualization/capabilities.rs`
- `visualization/ipc_push/client.rs`

### 1.4 barraCuda `mean()` Delegation

`uncertainty.rs` local `mean()` replaced with `barracuda::stats::mean`. The function signatures are identical; this eliminates duplicate math.

### 1.5 GPU Context Tier A Rewire

`GpuContext` now holds an `Option<Arc<WgpuDevice>>` for barraCuda when `barracuda-ops` is enabled. `execute()` delegates Tier A ops (Hill, PopPK, Diversity) to `barracuda_rewire::execute_*_barracuda()` before falling through to local WGSL. Previously only `dispatch::execute_gpu()` had this delegation.

### 1.6 NLME Cholesky Decision

Local `cholesky_solve()` in `pkpd/nlme/solver.rs` documented as intentional optimization — 2×2/3×3 `Vec<Vec<f64>>` matrices with integrated diagonal fallback. barraCuda's `cholesky_f64_cpu` targets larger flat-layout matrices; wrapping it would add conversion overhead with no benefit at this scale.

### 1.7 Hardcoding Evolution

Health response fields `nestgate` / `toadstool` → `data_provider` / `compute_provider`. Primal code has self-knowledge only; discovers others via capability probing at runtime.

---

## Part 2: Patterns Worth Absorbing

### For barraCuda

1. **Tolerance module pattern**: healthSpring's `tolerances.rs` with 70+ named constants, domain-categorized sections, ordering tests, and Python mirror is a mature pattern. Consider absorbing a similar `barracuda::tolerances` module for upstream precision constants.

2. **`extract_rpc_error()` pattern**: Generic JSON-RPC error extraction with safe defaults. Useful if barraCuda ever has IPC consumers.

3. **Python tolerance mirror**: Single-source-of-truth tolerance constants mirrored to Python for baseline reproducibility. Cross-spring adoption would prevent tolerance drift.

### For toadStool

1. **Capability-based health response**: healthSpring's `lifecycle.health` response now reports `data_provider` / `compute_provider` availability without naming specific primals. toadStool could adopt this for its own health endpoint.

2. **IPC timeout centralization**: Named constants for buffer sizes and timeouts instead of scattered magic numbers.

---

## Part 3: barraCuda Absorption Priorities

### P0 — Ready Now (Tier A rewire complete)

| healthSpring Op | barraCuda Target | Status |
|-----------------|-----------------|--------|
| `HillSweep` | `barracuda::ops::HillFunctionF64` | **Rewired** in `GpuContext::execute()` + `dispatch::execute_gpu()` |
| `PopulationPkBatch` | `barracuda::ops::PopulationPkF64` | **Rewired** |
| `DiversityBatch` | `barracuda::ops::bio::DiversityFusionGpu` | **Rewired** |
| `mean()` | `barracuda::stats::mean` | **Delegated** (V29) |

### P1 — Absorption Candidates (local WGSL, design pending)

| Shader | Domain | Absorption Path |
|--------|--------|-----------------|
| `michaelis_menten_batch_f64.wgsl` | PK ODE (Euler per patient) | `barracuda::ops::bio::MichaelisMentenBatchF64` |
| `scfa_batch_f64.wgsl` | Metabolic (element-wise MM ×3) | `barracuda::ops::bio::ScfaBatchF64` |
| `beat_classify_batch_f64.wgsl` | Biosignal (template correlation + argmax) | `barracuda::ops::biosignal::BeatClassifyF64` |

### P2 — Future (healthSpring designs, barraCuda absorbs)

| Module | Domain | Notes |
|--------|--------|-------|
| `pkpd/nlme` (FOCE/SAEM) | Population PK | GPU-promotable inner loop (Cholesky, eta optimization). Needs `TensorSession` for fused multi-subject dispatch. |
| `discovery/matrix_score` | Drug repurposing | Simple element-wise — low-priority shader candidate. |
| `comparative/species_params` | Allometric scaling | `powf`-heavy — candidate for precision routing. |

---

## Part 4: toadStool Evolution Targets

| Target | Priority | Description |
|--------|----------|-------------|
| **IPC timeout constants** | P1 | Adopt centralized timeout/buffer constants pattern from healthSpring. |
| **Capability-based discovery** | P1 | toadStool's health endpoint should use capability-based field names (not specific primal names). |
| **ODE batch dispatch** | P2 | `MichaelisMentenBatch` GPU op needs toadStool `StageOp` variant for streaming dispatch. |
| **NLME multi-subject dispatch** | P3 | When `TensorSession` lands, NLME inner loop can parallelize across subjects on GPU. |

---

## Part 5: Learnings for coralReef

1. **f32 transcendental workarounds**: All 6 WGSL shaders use `f64` with f32 cast for `pow`/`log`/`exp`. coralReef's DFMA polynomial lowering will replace these — ~7 decimal digits → full f64 precision.

2. **Wang hash PRNG**: `population_pk_f64.wgsl` uses a 32-bit Wang hash for per-patient variation. coralReef's native PRNG would be a direct replacement.

3. **`strip_f64_enable()` preprocessor**: healthSpring's WGSL preprocessor (`enable f64;` stripping for naga compatibility) is a candidate for coralReef's naga pass pipeline.

---

## Part 6: Metrics

| Metric | V28 | V29 |
|--------|-----|-----|
| Tests | 603 | 603 |
| Clippy warnings | 4 (experiment `too_many_lines`) | **0** (workspace-wide) |
| Inline magic numbers (library) | ~8 | **0** |
| Duplicate `mean()` | 1 (local) | **0** (delegated to barraCuda) |
| `GpuContext` Tier A rewire | No | **Yes** |
| Python tolerance mirror | No | **Yes** (70+ constants) |
| IPC error extraction patterns | 4 (scattered) | **1** (centralized `extract_rpc_error()`) |
| Named tolerance constants | ~65 | **70+** |
| `barracuda-core` dep documented | No | **Yes** (feature resolution) |
| NLME Cholesky documented | No | **Yes** (intentional local) |

---

## Part 7: Next Evolution Targets

1. **TensorSession adoption**: When `barracuda::session::TensorSession` lands, wire `GpuContext::execute_fused()` through it instead of local fused pipeline.
2. **Tier B shader absorption**: Push `michaelis_menten_batch_f64.wgsl`, `scfa_batch_f64.wgsl`, `beat_classify_batch_f64.wgsl` upstream to barraCuda.
3. **NLME GPU promotion**: Parallelize FOCE/SAEM inner loops across subjects using GPU compute.
4. **llvm-cov integration**: Target 90%+ line coverage with `cargo llvm-cov`.
5. **musl cross-compile CI**: Add `x86_64-unknown-linux-musl` target to CI matrix per ecoBin standard.
6. **MCP tool definitions**: Generate MCP tool schemas from IPC capability registry for LLM-orchestrated pipelines.

---

## Appendix: barraCuda Consumption Map

| healthSpring Module | barraCuda Primitive | Path | Status |
|--------------------|--------------------|------|--------|
| `rng.rs` | `barracuda::rng::{lcg_step, state_to_f64, LCG_MULTIPLIER}` | Re-export | **Live** |
| `uncertainty.rs` | `barracuda::stats::mean` | Direct call | **Live** (V29) |
| `microbiome/anderson.rs` | `barracuda::special::{tridiagonal_ql, anderson_diagonalize}` | Direct call | **Live** |
| `microbiome/mod.rs` | `barracuda::stats::{shannon, simpson, chao1, pielou, bray_curtis}` | Direct call | **Live** |
| `gpu/barracuda_rewire.rs` | `barracuda::ops::{HillFunctionF64, PopulationPkF64}`, `barracuda::ops::bio::DiversityFusionGpu` | Async GPU | **Live** (gated) |
| `gpu/context.rs` | `barracuda::device::WgpuDevice` | Persistent context | **Live** (V29, gated) |
| `gpu/ode_systems.rs` | Absorbed `OdeSystem` pattern from wetSpring | Trait impl | **Live** |
| `pkpd/nlme/solver.rs` | **Intentionally local** `cholesky_solve()` | N/A | Documented |
