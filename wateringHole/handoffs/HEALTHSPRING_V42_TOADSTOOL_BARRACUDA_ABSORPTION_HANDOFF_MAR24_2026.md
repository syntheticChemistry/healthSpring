<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring V42 → toadStool / barraCuda Absorption Handoff

**Date**: March 24, 2026
**From**: healthSpring V42 (Comprehensive Audit & Deep Debt Resolution)
**To**: barraCuda team, toadStool team
**Status**: Active

---

## Summary

healthSpring V42 completed a full-stack audit and 13 remediation actions.
877 tests pass, zero clippy (pedantic+nursery), zero unsafe, zero `#[expect]`
in production library code. This handoff consolidates all absorption requests,
evolution insights, and GPU readiness status for the upstream teams.

---

## Current barraCuda Usage (v0.3.7, rev c04d848)

### Primitives Consumed (Direct)

| Category | What healthSpring Uses |
|----------|----------------------|
| Core math | `exp`, `log`, `pow`, `sqrt`, `abs`, `clamp` |
| Reduction | `sum`, `mean`, `variance` |
| Statistics | `percentile` (population PK) |
| RNG | `lcg_step`, `state_to_f64`, `uniform_f64_sequence`, `LCG_MULTIPLIER` |
| ODE | `BatchedOdeRK4` codegen (3 `OdeSystem` impls: MM, Oral1C, Two-compartment) |
| Precision | `Fp64Strategy` (f64 shader selection, emulation path routing) |

### 6 GPU Ops — All LIVE

| Op | WGSL Shader | Dispatch Path | Validation |
|----|-------------|---------------|------------|
| `HillSweep` | `hill_dose_response_f64.wgsl` | barraCuda Tier A + local WGSL | Exp053 (crossover 100K) |
| `PopulationPkBatch` | `population_pk_f64.wgsl` | barraCuda Tier A + local WGSL | Exp053 (crossover 5M) |
| `DiversityBatch` | `diversity_f64.wgsl` | barraCuda Tier A + local WGSL | Exp053 (workgroup reduction) |
| `MichaelisMentenBatch` | `michaelis_menten_batch_f64.wgsl` | barraCuda Tier B + local WGSL | Exp083/085-087 |
| `ScfaBatch` | `scfa_batch_f64.wgsl` | barraCuda Tier B + local WGSL | Exp083/085-087 |
| `BeatClassifyBatch` | `beat_classify_batch_f64.wgsl` | barraCuda Tier B + local WGSL | Exp083/085-087 |

### ODE Codegen Registry (BatchedOdeRK4)

3 ODE systems wired to barraCuda's codegen:

```text
MichaelisMentenOde   → GpuOp::MichaelisMentenBatch
OralOneCompOde       → future GpuOp (PK oral batch)
TwoCompartmentOde    → future GpuOp (PK 2-comp batch)
```

---

## Absorption Requests

### P0 — Immediate (healthSpring shaders ready for upstream)

These 6 WGSL shaders are validated, tested, and ready for direct absorption
into barraCuda's default dispatch. healthSpring will then lean on upstream.

| # | Shader | Lines | Key Algorithm | Validation Checks |
|---|--------|-------|---------------|-------------------|
| 1 | `hill_dose_response_f64.wgsl` | 58 | Hill 4-param dose-response | 7 (Exp001) |
| 2 | `population_pk_f64.wgsl` | 76 | LCG Monte Carlo + one-comp PK | 12 (Exp005) |
| 3 | `diversity_f64.wgsl` | 64 | Shannon/Simpson workgroup reduce | 15 (Exp010) |
| 4 | `michaelis_menten_batch_f64.wgsl` | 72 | Per-patient MM ODE (Euler) | 12 (Exp077) |
| 5 | `scfa_batch_f64.wgsl` | 56 | Michaelis-Menten fermentation | 10 (Exp079) |
| 6 | `beat_classify_batch_f64.wgsl` | 68 | Template correlation + argmax | 12 (Exp082) |

**Post-absorption**: healthSpring drops local WGSL copies, rewires to
`barracuda::dispatch::execute_*`, and local `execute_fused` converges to
barraCuda's `TensorSession`.

### P1 — New Primitives Requested

| # | Primitive | Domain | Use Case |
|---|-----------|--------|----------|
| 1 | `pbpk_tissue_partition` | PK/PD | Multi-compartment PBPK (5+ tissues) — parallel per-tissue ODE |
| 2 | `auc_parallel_prefix` | PK/PD | Trapezoidal AUC via parallel prefix scan |
| 3 | `foce_gradient_batch` | NLME | Per-subject objective function gradient (population PK) |
| 4 | `anderson_hamiltonian_gpu` | Physics | 1D Anderson Hamiltonian eigenvalue (tridiagonal) |
| 5 | `bray_curtis_pairwise` | Microbiome | All-pairs community dissimilarity matrix |
| 6 | `pan_tompkins_streaming` | Biosignal | 5-stage QRS detection as streaming pipeline |

### P2 — TensorSession (Fused Pipeline)

healthSpring's `execute_fused_local` currently uses a local single-encoder
fusion path for all 6 ops. When barraCuda ships `TensorSession`, all three
dispatch paths (stateless, session-based, fused) converge to upstream.

**Requested TensorSession API**:
```rust
let session = TensorSession::new(&device);
session.add_op(HillSweep { ... });
session.add_op(DiversityBatch { ... });
session.add_op(PopulationPkBatch { ... });
let results = session.execute().await?;
```

---

## Evolution Insights for barraCuda Team

### 1. Pass-by-Value for Copy Types

During the V42 audit, we found that extracted GPU dispatch helpers were
passing `&f64`, `&u64`, `&u32` by reference. Clippy `trivially_copy_pass_by_ref`
caught this. **Recommendation**: barraCuda dispatch APIs should prefer
`fn execute_hill(emax: f64, ec50: f64, n: f64, ...)` over references for
scalar params. Slices (`&[f64]`) remain by reference.

### 2. Precision Routing Already Works

healthSpring's `metalForge::PrecisionRouting` enum (F64Native /
F64NativeNoSharedMem / Df64Only / F32Only) maps directly to barraCuda's
`Fp64Strategy`. The runtime probe in `Capabilities::discover()` correctly
identifies shared-memory reliability. No changes needed upstream.

### 3. ValidationHarness Pattern

healthSpring's `ValidationHarness` (register checks, pass/fail, exit 0/1)
has proven effective across 83 experiments. **Recommendation**: consider
standardizing this as a barraCuda test utility for shader validation binaries.

### 4. Tolerance Registry Pattern

Centralized named tolerances (`tolerances.rs` — 87+ constants with classes:
machine epsilon, numerical method, statistical, GPU/CPU parity, clinical) with
a companion `TOLERANCE_REGISTRY.md` has eliminated magic numbers across the
codebase. **Recommendation**: barraCuda could adopt a similar pattern for
its shader validation thresholds.

### 5. Provenance Tracking

`ProvenanceRecord` structs with `python_script`, `git_commit`, `run_date`,
`exact_command`, `checks`, `baseline_source` provide complete traceability.
**Recommendation**: upstream should adopt this for any new barraCuda op that
has a Python/NumPy reference implementation.

---

## For toadStool Team

### Pipeline Dispatch Status

healthSpring's local `toadstool/` crate implements a unidirectional streaming
pipeline with stage-based dispatch (`execute_cpu`, `execute_streaming`,
`execute_auto`). This is ready for absorption into upstream toadStool.

### metalForge Refactor Complete

The metalForge `forge` crate was refactored from 524 LOC → 4 files:
- `types.rs` — `Substrate`, `Workload`, `PrecisionRouting`, GPU/NPU info
- `discovery.rs` — `Capabilities::discover()` with feature-gated probes
- `routing.rs` — `select_substrate*` with configurable `DispatchThresholds`
- `lib.rs` — 51 LOC re-export hub

This structure maps well to toadStool's dispatcher architecture.

### NUCLEUS Topology

`metalForge::nucleus` defines Tower → Node → Nest hierarchy with:
- `plan_dispatch` for stage → Nest assignment
- PCIe P2P DMA transfer planning (31.5 GB/s Gen4 estimate)
- Host-staged and network IPC fallback paths

### NPU Path (Future)

`metalForge::probe_npu()` returns `None` unconditionally — Akida driver
integration is feature-gated behind `npu`. The `BiosignalDetect` and
`BiosignalFusion` workload variants route to NPU when available (via
`Workload::prefers_npu()`).

---

## Cross-Spring Context

- **wetSpring V123**: `ValidationSink` trait absorbed from wetSpring
- **hotSpring v0.6.31**: `ValidationHarness` follows hotSpring canonical form
- **neuralSpring S157**: Hill dose-response, PK decay shared via IPC
- **groundSpring V109**: Uncertainty propagation patterns referenced
- **ludoSpring V22**: Session decomposition patterns inform TensorSession design
