<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V24 → barraCuda/toadStool: Audit Execution + Absorption Handoff

**Date**: March 15, 2026
**From**: healthSpring V24 (435 tests, 61 experiments, 55+ wired capabilities)
**To**: barraCuda team, toadStool team
**License**: CC-BY-SA-4.0
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: V23 Deep Remediation Handoff (Mar 15, 2026)
**barraCuda pin**: `a60819c` (v0.3.5)

---

## Executive Summary

V24 executes on the V23 comprehensive audit findings. Key changes relevant to barraCuda/toadStool:

- **toadStool Hill/AUC duplication eliminated** — `stage.rs` now delegates to `pkpd::hill_sweep()` and `pkpd::auc_trapezoidal()` instead of reimplementing. Zero duplicate math in the spring.
- **gpu/context.rs smart refactor** — 968 → 350 LOC. Per-op preparation extracted into `gpu/fused.rs`. The fused pipeline pattern (single encoder, N compute passes, one readback) is now clean 3-phase orchestration, ready for `TensorSession` migration.
- **Capability-based primal discovery** — Hardcoded `COMPUTE_PRIMAL_DEFAULT = "toadstool"` and `DATA_PRIMAL_DEFAULT = "nestgate"` removed. Discovery now probes socket dir via `capability.list` JSON-RPC, with well-known name fallback. Primals only have self-knowledge; they discover peers at runtime.
- **CI clippy::nursery enforced** — matches `lib.rs` `#![deny(clippy::nursery)]`. All unfulfilled expects cleaned.

---

## Part 1: Tier A GPU Ops — Rewire Status

Three GPU ops have canonical upstream implementations in barraCuda. healthSpring validates them locally but has NOT yet rewired to consume barraCuda ops. This is the P0 action item.

| healthSpring Op | Local Shader | barraCuda Op | Validated | Rewired |
|-----------------|-------------|--------------|-----------|---------|
| `HillSweep` | `hill_dose_response_f64.wgsl` | `barracuda::ops::HillFunctionF64` | Yes (42 parity checks) | **No** |
| `PopulationPkBatch` | `population_pk_f64.wgsl` | `barracuda::ops::PopulationPkF64` | Yes | **No** |
| `DiversityBatch` | `diversity_f64.wgsl` | `barracuda::ops::bio::DiversityFusionGpu` | Yes | **No** |

### toadStool action: Tier A rewire

When healthSpring rewires `GpuContext::execute()` and `execute_fused()` to use barraCuda ops, toadStool needs to support the same dispatch pattern. The current `StageOp::to_gpu_op()` mapping should remain the routing layer.

---

## Part 2: Tier B Absorption Candidates — New Shaders for barraCuda

Three WGSL shaders were written and validated in healthSpring V16-V19. They are ready for barraCuda absorption.

| Shader | Domain | Formula | Validation | GPU Parity |
|--------|--------|---------|------------|------------|
| `michaelis_menten_batch_f64.wgsl` | PK/PD | Euler ODE per patient: `dC/dt = -Vmax*C/(Km+C)/Vd` | 64 patients, exp077+exp083 | 25/25 |
| `scfa_batch_f64.wgsl` | Microbiome | Element-wise MM ×3: acetate, propionate, butyrate | exp079+exp083 | 25/25 |
| `beat_classify_batch_f64.wgsl` | Biosignal | Template correlation + argmax per beat window | exp082+exp083 | 25/25 |

### barraCuda action: absorb as `barracuda::ops::bio::*`

Suggested targets:
- `barracuda::ops::bio::MichaelisMentenBatchF64` — Euler ODE batch (Wang hash PRNG per patient, same as PopPK pattern)
- `barracuda::ops::bio::ScfaBatchF64` — element-wise MM kinetics ×3 channels
- `barracuda::ops::bio::BeatClassifyBatchF64` — template correlation with argmax

All three follow the same buffer layout pattern: uniform params + input storage → output storage. The `fused.rs` prep functions document the exact layout.

---

## Part 3: toadStool Evolution Patterns

### Pattern: Hill/AUC delegation (V24 fix)

V24 eliminated duplicate math in `toadstool/src/stage.rs`. The Hill transform and AUC trapezoidal now delegate to `healthspring_barracuda::pkpd::*` instead of reimplementing. This validates the **Write → Absorb → Lean** cycle: healthSpring wrote the math, barraCuda absorbed it, toadStool now leans on barraCuda via healthSpring.

### Pattern: `failed_stage_result()` safe fallback

`stage.rs` uses a `failed_stage_result()` helper that returns `(empty_output, success: false)` when a GPU-native stage can't map to a `GpuOp`. This safe fallback pattern should be adopted by toadStool's core dispatch.

### Pattern: `StageOp::to_gpu_op()` mapping

The `to_gpu_op()` method is the boundary between domain stages and GPU dispatch. It returns `Option<GpuOp>` — `None` for CPU-only operations (filter, biosignal fusion). This pattern is clean and should be preserved as toadStool absorbs more stage types.

---

## Part 4: Fused Pipeline → TensorSession Migration Path

The `GpuContext::execute_fused()` refactor (V24) makes the migration path to `barracuda::session::TensorSession` clear:

```
Current (healthSpring local):
  GpuContext::execute_fused(ops)
    → prepare_all_ops(ops)     → Vec<PreparedOp>     [fused.rs]
    → submit_compute_passes()  → single encoder       [context.rs]
    → readback_all()           → Vec<GpuResult>       [context.rs]

Target (barraCuda upstream):
  TensorSession::new(device)
    → session.add(op)          → Op handle
    → session.execute()        → single encoder
    → session.readback(handle) → typed result
```

The `PreparedOp` struct in `fused.rs` documents the exact buffer layouts, workgroup sizes, and bind group patterns that `TensorSession` needs to support.

### barraCuda action: `TensorSession` design

The fused pipeline pattern is now well-documented with 6 concrete op preparations. Use `fused.rs` as the design input for `TensorSession`'s multi-op API.

---

## Part 5: Capability-Based Discovery (V24 Pattern)

V24 replaces hardcoded primal names with runtime capability probes:

```rust
// OLD (V23): hardcoded name
discover_primal("toadstool")

// NEW (V24): capability probe → name fallback
discover_by_capability("compute")    // probes via capability.list
    .or_else(|| discover_primal("toadstool"))
    .or_else(|| discover_primal("node-atomic"))
```

### toadStool action: respond to `capability.list`

toadStool should respond to `capability.list` JSON-RPC with its `compute.*` capabilities so that healthSpring (and other primals) can discover it without hardcoded names.

---

## Part 6: NLME GPU Primitive (Reiterated from V23)

The FOCE inner loop (`pkpd::nlme::foce`) iterates 150 times over 30 subjects. The per-subject objective function evaluation is embarrassingly parallel and a strong GPU candidate.

### barraCuda action: `barracuda::ops::nlme::FoceInnerLoop`

- Input: population parameters (θ, Ω, Σ), per-subject data (times, concentrations)
- Compute: per-subject objective function + gradient
- Output: per-subject contribution to total objective

This would make healthSpring's NLME pipeline GPU-native without touching the outer optimization loop.

---

## Part 7: Quality Metrics (V24)

| Metric | Value |
|--------|-------|
| Tests | 435 |
| Experiments | 61 |
| `#[allow()]` in production | 0 |
| TODO/FIXME | 0 |
| Unsafe blocks | 0 |
| Max file LOC | 350 (gpu/context.rs) |
| Clippy | 0 warnings (pedantic + nursery, CI enforced) |
| `cargo fmt` | 0 diffs |
| Duplicate math | 0 (Hill/AUC now delegate) |
| Hardcoded primal names | 0 (capability-based discovery) |

---

## Action Items Summary

### barraCuda team

| Priority | Action |
|----------|--------|
| **P0** | Absorb 3 Tier B shaders: `MichaelisMentenBatchF64`, `ScfaBatchF64`, `BeatClassifyBatchF64` |
| **P1** | Design `TensorSession` API using `fused.rs` as reference input |
| **P2** | Implement `FoceInnerLoop` GPU primitive for NLME population PK |

### toadStool team

| Priority | Action |
|----------|--------|
| **P0** | Respond to `capability.list` JSON-RPC (required for capability-based discovery) |
| **P1** | Adopt `failed_stage_result()` safe fallback pattern |
| **P2** | Support `TensorSession`-based dispatch when available upstream |

### healthSpring (self)

| Priority | Action |
|----------|--------|
| **P0** | Rewire Tier A ops to barraCuda upstream (Hill, PopPK, Diversity) |
| **P1** | Migrate remaining ~48 experiments to `ValidationHarness` |
| **P2** | Begin Track 6 (Comparative Medicine) paper queue |
