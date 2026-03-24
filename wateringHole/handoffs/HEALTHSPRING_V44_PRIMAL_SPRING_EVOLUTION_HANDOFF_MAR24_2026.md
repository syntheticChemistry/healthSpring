<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V44 — Primal & Spring Evolution Handoff

**Date**: March 24, 2026
**From**: healthSpring V44 (928 tests, 83 experiments, 54 Python baselines)
**To**: All primal teams + sibling spring teams
**Supersedes**: V43 Primal & Spring Evolution Handoff (archived)
**Scope**: Deep debt resolution, modern idiomatic Rust evolution, primal name centralization, GPU module refactoring, tolerance hardening, coverage expansion

---

## Summary

V44 executed a comprehensive audit with 11 remediation actions. Key themes:
centralize all hardcoded values, smart-refactor large modules, expand test
coverage into untested corners, and document architectural decisions for
upstream teams. 928 tests pass, zero clippy (pedantic+nursery), zero unsafe,
zero `#[expect]` in production library code.

---

## 1. For barraCuda Team

### Absorption Requests (Active — carried from V42)

All 6 WGSL shaders remain ready for absorption. No changes to shader content since V42:

| # | Shader | Status |
|---|--------|--------|
| 1 | `hill_dose_response_f64.wgsl` | Ready — 7 validation checks |
| 2 | `population_pk_f64.wgsl` | Ready — 12 validation checks |
| 3 | `diversity_f64.wgsl` | Ready — 15 validation checks |
| 4 | `michaelis_menten_batch_f64.wgsl` | Ready — 12 validation checks |
| 5 | `scfa_batch_f64.wgsl` | Ready — 10 validation checks |
| 6 | `beat_classify_batch_f64.wgsl` | Ready — 12 validation checks |

### New V44: PopulationPK Config Provenance

`PopulationPkConfig` defaults are now traced to named tolerance constants:

| Constant | Value | Source |
|----------|-------|--------|
| `POP_PK_BASE_CL` | 10.0 | Rowland & Tozer Ch. 3 — typical clearance |
| `POP_PK_CL_LOW` | 0.5 | Lower IIV bound (50% of base) |
| `POP_PK_CL_HIGH` | 1.5 | Upper IIV bound (150% of base) |

**Request**: When absorbing `population_pk_f64.wgsl`, carry these named constants
into the upstream shader config defaults with the same provenance comments.

### New V44: TensorSession Evaluation

healthSpring evaluated `TensorSession` for its fused GPU pipeline. Finding:
the current healthSpring pipeline dispatches **independent parallel ops**
(Hill + PopPK + Diversity in a single encoder pass). `TensorSession` is
designed for **dependent operation chains** (output of op A feeds input of op B).
These are complementary patterns. Integration is deferred until healthSpring
has dependent multi-op chains (e.g., NLME per-subject gradient → population update).

**Recommendation**: Document `TensorSession` as the pattern for dependent chains
and add a `FusedEncoder` or `ParallelBatch` API for independent parallel dispatch.

### New V44: FFT Absorption Path

`barraCuda`'s FFT (`spectral` module) is GPU-only. healthSpring's `biosignal::fft`
is CPU-only (DFT/IDFT for HRV power spectrum). These are complementary — no
duplication. When `barraCuda` ships a CPU FFT, healthSpring will delegate.
Until then, the local CPU FFT is the correct implementation.

### Cast Module (Carried from V43)

The cast module request remains active: absorb `usize_f64`, `usize_u32`,
`u64_u32_truncate`, `f64_usize`, `u32_f64` into `barraCuda::cast`.

### Upstream Contract Tolerances (Carried from V43)

5 cross-spring agreed tolerance values documented in `tolerances.rs`. Request
for barraCuda to document these as contract guarantees.

---

## 2. For toadStool Team

### New V44: Coverage Expanded to 51 Tests

toadStool's local crate gained 31 new tests covering:

| Area | Tests Added | What They Validate |
|------|:-----------:|-------------------|
| Michaelis-Menten batch | 4 | Substrate depletion, Vmax scaling, low Km |
| SCFA batch | 3 | Acetate/propionate/butyrate production |
| Beat classification | 4 | Normal/PVC/PAC templates, threshold rejection |
| Biosignal fusion | 3 | Multi-channel weighted assessment |
| AUC trapezoidal | 3 | Linear, single-point, high-resolution |
| Bray-Curtis | 3 | Identical, disjoint, partial overlap communities |
| Variance | 3 | Constant, two-element, multi-element |
| GPU mappability | 8 | All `StageOp` variants map to correct `GpuOp` |

**Recommendation**: When toadStool absorbs healthSpring's pipeline stages,
these tests provide the validation baseline for each stage's correctness.

### Self-Knowledge Compliance (Carried from V43)

Zero cross-primal names in production error strings. MCP tool descriptions
use capability-based language. 7-primal capability discovery active.

---

## 3. For Sibling Springs

### New V44 Patterns Available for Absorption

| Pattern | Source | What It Does |
|---------|--------|-------------|
| `primal_names` module | `ecoPrimal/src/primal_names.rs` | Canonical primal name constants + `socket_env_var()` / `prefix_env_var()` helpers |
| GPU smart refactor | `ecoPrimal/src/gpu/{types,cpu_fallback}.rs` | Extract types + CPU fallbacks from large GPU orchestrator (696→413 LOC) |
| Provenance registry accessors | `ecoPrimal/src/provenance/registry.rs` | `records_for_track()`, `record_for_experiment()`, `distinct_tracks()` for data-heavy files |
| Tolerance migration | 8 experiment files | Systematic replacement of inline literals with `tolerances::*` constants |
| WFDB annotation tests | `ecoPrimal/src/wfdb/annotations.rs` | 11 tests covering parser edge cases (empty, terminators, AUX/SKIP, truncation) |

### Smart Refactoring Principle

V44 established a principle for "smart refactoring" that applies across springs:

1. **Data-heavy files** (provenance registries, tolerance tables): keep data consolidated, add query accessors
2. **Logic-heavy files** (GPU orchestrators): extract by responsibility (types, CPU fallbacks, dispatch logic)
3. **Never split just to reduce line count** — split by cohesion and responsibility

### What healthSpring Absorbed This Version

| Source | Pattern | V44 Application |
|--------|---------|----------------|
| Audit methodology | V42/V43 8-axis audit | Executed all 11 identified remediation actions |
| groundSpring V122 | Named constant centralization | `primal_names`, `PopulationPkConfig` defaults |
| wetSpring V134 | Smart module refactoring | `gpu/mod.rs` → 3-module structure |
| Ecosystem standard | Zero hardcoded primal names | `primal_names` module eliminates all inline strings |

---

## 4. For biomeOS Team

### Capability Surface Stable

59 capabilities (46 science + 13 infra) registered via `capability.register`.
No changes to the capability surface in V44.

### Graph/Niche Integration

Deploy graph, graph status, and graph teardown methods remain wired but
await biomeOS deploy API stabilization for integration testing.

---

## 5. Debris Status

| Metric | Value |
|--------|-------|
| TODOs/FIXMEs in Rust | 0 |
| Temp files | 0 |
| Dead directories | 0 |
| `.bak`/`.old`/`.tmp` files | 0 |
| Stale archive | `wateringHole/handoffs/archive/` (40 files, preserved as fossil record) |
| Python controls | 54 (legitimate baseline provenance) |
| Shell scripts | 4 (operational: `visualize.sh`, `sync_scenarios.sh`, `live_dashboard.sh`, `compute_dashboard.sh`) |

---

## 6. Metrics Summary (V44)

| Metric | V43 | V44 | Delta |
|--------|:---:|:---:|:-----:|
| Tests | 888 | 928 | +40 |
| Experiments | 83 | 83 | — |
| Python baselines | 54 | 54 | — |
| JSON-RPC capabilities | 59 | 59 | — |
| Named tolerances | 87+ | 90+ | +3 |
| toadStool tests | 20 | 51 | +31 |
| WFDB annotation tests | 0 | 11 | +11 |
| `gpu/mod.rs` LOC | 696 | 413 | -283 |
| GPU ops (barraCuda) | 6/6 | 6/6 | — |
| Unsafe blocks | 0 | 0 | — |
| Clippy warnings | 0 | 0 | — |
| Max file LOC | 732 | 732 | — |

---

**License**: AGPL-3.0-or-later (code), CC-BY-SA-4.0 (this document)
