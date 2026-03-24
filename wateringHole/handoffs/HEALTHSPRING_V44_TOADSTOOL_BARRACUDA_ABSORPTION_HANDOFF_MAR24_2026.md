<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring V44 → toadStool / barraCuda Absorption Handoff

**Date**: March 24, 2026
**From**: healthSpring V44 (Deep Debt Resolution & Modern Idiomatic Evolution)
**To**: barraCuda team, toadStool team
**Supersedes**: V42 toadStool / barraCuda Absorption Handoff (archived)
**Status**: Active

---

## Summary

V44 deepens the absorption readiness established in V42. GPU module refactoring
separates types, CPU fallbacks, and orchestration into clean boundaries. Test
coverage for toadStool stages expanded from 20 to 51 tests. Tolerance constants
for `PopulationPkConfig` now carry Python provenance. `TensorSession` and FFT
absorption paths are architecturally evaluated and documented.

---

## Current barraCuda Usage (v0.3.7)

### Primitives Consumed (Unchanged from V42)

| Category | What healthSpring Uses |
|----------|----------------------|
| Core math | `exp`, `log`, `pow`, `sqrt`, `abs`, `clamp` |
| Reduction | `sum`, `mean`, `variance` |
| Statistics | `percentile` (population PK) |
| RNG | `lcg_step`, `state_to_f64`, `uniform_f64_sequence`, `LCG_MULTIPLIER` |
| ODE | `BatchedOdeRK4` codegen (3 `OdeSystem` impls) |
| Precision | `Fp64Strategy` |
| Health | `pkpd::mm_auc`, `biosignal::scr_rate`, `microbiome::antibiotic_perturbation` |

### 6 GPU Ops — All LIVE (Unchanged)

| Op | Shader | Tier | Validation |
|----|--------|------|-----------|
| `HillSweep` | `hill_dose_response_f64.wgsl` | A | Exp053 |
| `PopulationPkBatch` | `population_pk_f64.wgsl` | A | Exp053 |
| `DiversityBatch` | `diversity_f64.wgsl` | A | Exp053 |
| `MichaelisMentenBatch` | `michaelis_menten_batch_f64.wgsl` | B | Exp083 |
| `ScfaBatch` | `scfa_batch_f64.wgsl` | B | Exp083 |
| `BeatClassifyBatch` | `beat_classify_batch_f64.wgsl` | B | Exp083 |

---

## V44 GPU Module Structure (Post-Refactor)

```
ecoPrimal/src/gpu/
├── mod.rs           (413 LOC — orchestration, shader mapping, availability)
├── types.rs         (105 LOC — GpuOp, GpuResult, GpuError)
├── cpu_fallback.rs  (175 LOC — all CPU reference implementations)
├── context.rs       (350 LOC — GpuContext single-op + fused)
├── fused.rs         (340 LOC — per-op buffer prep + readback)
├── dispatch.rs      (GPU dispatch routing)
├── barracuda_rewire.rs (barraCuda Tier A delegation)
└── sovereign.rs     (coralReef sovereign dispatch)
```

This structure maps cleanly to upstream absorption:
- `types.rs` → barraCuda's `health::ops` enum
- `cpu_fallback.rs` → barraCuda's CPU reference impls for each shader
- `context.rs` + `fused.rs` → toadStool pipeline orchestration

### `wang_hash_uniform` Scoping

V44 tightened `wang_hash_uniform` from `pub(crate)` to private within
`cpu_fallback.rs`. This function exists solely for WGSL parity (the GPU
shader uses Wang hash for per-invocation PRNG state). It is not a
general-purpose PRNG and should not be exposed as a public API. barraCuda
has no equivalent CPU API because the function only makes sense in a
GPU context.

---

## Absorption Requests (Updated)

### P0 — 6 WGSL Shaders (Unchanged, Ready)

All 6 shaders validated across exp053/exp083/exp085-087. Post-absorption,
healthSpring drops local WGSL copies and leans on upstream dispatch.

### P1 — New Primitives (Unchanged, Prioritized)

1. `pbpk_tissue_partition` — parallel per-tissue ODE
2. `auc_parallel_prefix` — trapezoidal AUC via prefix scan
3. `foce_gradient_batch` — per-subject NLME gradient
4. `anderson_hamiltonian_gpu` — tridiagonal eigenvalue
5. `bray_curtis_pairwise` — all-pairs dissimilarity
6. `pan_tompkins_streaming` — 5-stage QRS pipeline

### P2 — TensorSession (Architectural Evaluation Complete)

healthSpring's fused pipeline dispatches **independent parallel ops** (all 6 ops
in one encoder pass). `TensorSession` is designed for **dependent chains**
(output A → input B). These are complementary patterns:

| Pattern | Use Case | healthSpring Status |
|---------|----------|-------------------|
| `FusedEncoder` / `ParallelBatch` | Independent parallel ops | Current (local) |
| `TensorSession` | Dependent multi-op chains | Deferred (no current use case) |

**Recommendation**: Ship both patterns. healthSpring will adopt `TensorSession`
when NLME GPU pipelines (per-subject gradient → population update) require it.

### P3 — Cast Module (New Request)

Absorb core numeric cast functions into `barraCuda::cast`:

| Function | Conversion | Priority |
|----------|-----------|----------|
| `usize_f64` | `usize → f64` | High (every spring needs this) |
| `usize_u32` | `usize → u32` (saturating) | High |
| `u64_u32_truncate` | `u64 → u32` (PRNG seeds) | Medium |
| `f64_usize` | `f64 → usize` (truncation) | High |
| `u32_f64` | `u32 → f64` (lossless) | Medium |

groundSpring V122 independently recommends the same.

---

## For toadStool: Pipeline Stage Validation Baseline

51 tests now validate all healthSpring `StageOp` variants. This provides
the correctness baseline for upstream absorption:

| StageOp | CPU Tests | GPU Mappability Test | Target `GpuOp` |
|---------|:---------:|:-------------------:|----------------|
| `HillSweep` | analytical | `to_gpu_op()` validated | `GpuOp::HillSweep` |
| `PopulationPk` | Monte Carlo | `to_gpu_op()` validated | `GpuOp::PopulationPkBatch` |
| `Diversity` | Shannon/Simpson | `to_gpu_op()` validated | `GpuOp::DiversityBatch` |
| `MichaelisMenten` | substrate depletion | `to_gpu_op()` validated | `GpuOp::MichaelisMentenBatch` |
| `Scfa` | fermentation kinetics | `to_gpu_op()` validated | `GpuOp::ScfaBatch` |
| `BeatClassify` | template correlation | `to_gpu_op()` validated | `GpuOp::BeatClassifyBatch` |
| `BiosignalFusion` | weighted multi-channel | `to_gpu_op() == None` | CPU-only (no shader) |
| `AucTrapezoidal` | linear + high-res | `to_gpu_op() == None` | CPU-only (pending P1.2) |
| `BrayCurtis` | dissimilarity matrix | `to_gpu_op() == None` | CPU-only (pending P1.5) |
| `Variance` | statistical | `to_gpu_op() == None` | CPU-only |

---

## Evolution Insights (New in V44)

### 1. Smart Refactoring Over Arbitrary Splitting

`gpu/mod.rs` was 696 lines — over the 1000 LOC soft limit but still readable.
Rather than splitting at an arbitrary line number, we extracted by responsibility:
- **Types** (what the operations are) → `types.rs`
- **CPU fallbacks** (how to compute without GPU) → `cpu_fallback.rs`
- **Orchestration** (when and where to dispatch) → stays in `mod.rs`

This principle applies to any large module in the ecosystem.

### 2. Data Tables Deserve Query Accessors, Not Splitting

`provenance/registry.rs` is a large file (data-heavy). Rather than splitting
the provenance records across multiple files (fragmenting the data), we added
query accessors: `records_for_track()`, `record_for_experiment()`, `distinct_tracks()`.
This keeps data consolidated while providing programmatic access.

### 3. Named Constants Require Provenance

The 3 new tolerance constants (`POP_PK_BASE_CL`, `POP_PK_CL_LOW`, `POP_PK_CL_HIGH`)
demonstrate the standard: every extracted magic number carries a source comment
linking it to the published reference or Python baseline.

---

**License**: AGPL-3.0-or-later (code), CC-BY-SA-4.0 (this document)
