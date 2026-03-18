<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# healthSpring V36 — Deep Debt + Ecosystem Maturity Handoff

**Date**: 2026-03-18
**From**: healthSpring V36
**To**: barraCuda team, toadStool team, coralReef team, All Springs
**License**: AGPL-3.0-or-later
**Covers**: V35 → V36 (deep debt elimination, Tier B GPU absorption, ecosystem maturity)
**Pins**: barraCuda v0.3.5 (rev a60819c3), toadStool S158+, coralReef Phase 10 Iteration 55+
**Supersedes**: HEALTHSPRING_V35_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR17_2026.md

---

## Executive Summary

- **Tier B GPU ops rewired** — `MichaelisMentenBatchGpu`, `ScfaBatchGpu`, `BeatClassifyGpu` now delegate to `barracuda::ops::health::*` instead of local WGSL. All 6 GPU ops use upstream barraCuda implementations.
- **Duplicate math eliminated** — local `lcg_step` in `uncertainty.rs` replaced with `barracuda::rng::lcg_step` (single source of truth)
- **Inline tolerances centralized** — magic numbers in exp095, exp096, exp107, exp109 migrated to `tolerances::*` named constants
- **IPC routing complete** — `compute.shader_compile` → coralReef forwarding, `model.inference_route` → Squirrel forwarding, both via capability-based discovery
- **Zero `#[allow()]`** — all 4 instances in exp109 migrated to `#[expect()]` with `reason` strings
- **Idiomatic Rust evolution** — `partial_cmp().unwrap()` → `f64::total_cmp()` in exp036/exp055; Mutex lock patterns documented with `#[expect]`
- **WGSL license headers corrected** — all 6 shaders now `AGPL-3.0-or-later` (was `AGPL-3.0-only`)
- **Doc-tests fixed** — `ignore` → `no_run` for 2 doc examples requiring runtime resources
- **CI hardened** — `cargo-deny` step added to quality gate
- **6 new experiments** — exp095 (iPSC skin), exp096 (niclosamide), exp107 (QS-augmented Anderson), exp108 (real 16S), exp109 (MIT-BIH arrhythmia), exp110 (equine laminitis)
- **617 tests**, 79 experiments, 42 Python baselines, 113/113 cross-validation, zero clippy, zero unsafe, zero `#[allow()]`

---

## Part 1: barraCuda Primitive Consumption (V36 State)

### CPU Primitives Used

| Category | Primitive | healthSpring Module |
|----------|-----------|-------------------|
| **stats** | `mean` | `uncertainty.rs` — bootstrap, jackknife, MBE, MC propagation |
| **rng** | `lcg_step`, `LCG_MULTIPLIER`, `state_to_f64`, `uniform_f64_sequence` | `rng.rs` — re-exported + Box-Muller normal sampling |
| **numerical** | `OdeSystem`, `BatchedOdeRK4` | `gpu/ode_systems.rs` — MichaelisMenten, OralOneCompartment, TwoCompartment |
| **spectral** | (via Anderson lattice) | `microbiome/anderson.rs` — gut localization |

### GPU Ops Used (All 6 Rewired via `barracuda_rewire.rs`)

| healthSpring Op | barraCuda Op | Feature Gate | Status |
|-----------------|-------------|-------------|--------|
| `HillSweep` | `barracuda::ops::HillFunctionF64` | `barracuda-ops` | **Tier A — LIVE** |
| `PopulationPkBatch` | `barracuda::ops::PopulationPkF64` | `barracuda-ops` | **Tier A — LIVE** |
| `DiversityBatch` | `barracuda::ops::bio::DiversityFusionGpu` | `barracuda-ops` | **Tier A — LIVE** |
| `MichaelisMentenBatch` | `barracuda::ops::health::MichaelisMentenBatchGpu` | `barracuda-ops` | **Tier B — REWIRED (V36)** |
| `ScfaBatch` | `barracuda::ops::health::ScfaBatchGpu` | `barracuda-ops` | **Tier B — REWIRED (V36)** |
| `BeatClassifyBatch` | `barracuda::ops::health::BeatClassifyGpu` | `barracuda-ops` | **Tier B — REWIRED (V36)** |

### What's NOT Delegated (And Why)

| Local Implementation | Reason | Evolution Path |
|---------------------|--------|---------------|
| `biosignal/fft.rs` (radix-2 Cooley-Tukey) | CPU-only, small HRV workloads (~256 pts) | Could rewire to `barracuda::spectral::fft` for GPU FFT on larger workloads |
| `pkpd/nlme/solver.rs` (2×2/3×3 Cholesky) | Intentional — NLME uses small `Vec<Vec<f64>>` with integrated fallback; barraCuda targets larger flat matrices | No change needed |
| `uncertainty.rs` (`std_dev`) | Thin helper using `barracuda::stats::mean` internally | Could use barraCuda `stats::variance` if API adds sample variance |
| `gpu/fused.rs` (fused pipeline) | Local multi-op pipeline | Migrate to `barracuda::session::TensorSession` when API stabilizes |

---

## Part 2: What Changed (V35 → V36)

### Tier B GPU Absorption

Added `execute_mm_batch_barracuda()`, `execute_scfa_batch_barracuda()`, and `execute_beat_classify_barracuda()` to `gpu/barracuda_rewire.rs`. These delegate to `barracuda::ops::health::{MichaelisMentenBatchGpu, ScfaBatchGpu, BeatClassifyGpu}` — the canonical upstream implementations that barraCuda absorbed from healthSpring V19.

The Write → Absorb → Lean cycle is now complete for all 6 GPU ops:
1. **Write**: healthSpring V15-V17 wrote local WGSL shaders
2. **Absorb**: barraCuda absorbed them into `ops::health`
3. **Lean**: healthSpring V36 rewires to upstream (local shaders retained as validation targets)

**Next step**: Remove local shader copies once upstream parity is CI-validated.

### Duplicate Math Elimination

`uncertainty.rs` had a local `const fn lcg_step()` with identical constants to `barracuda::rng::lcg_step`. Removed the local copy, now imports directly. Single source of truth for LCG constants.

### IPC Routing Completion

Two advertised capabilities had no routing handlers:
- `compute.shader_compile` — now forwards to coralReef via `socket::discover_shader_compiler()`
- `model.inference_route` — now forwards to Squirrel via `socket::discover_inference_primal()`

Both use capability-based discovery (no hardcoded primal names). If coralReef or Squirrel aren't running, returns structured error JSON with env-override hints.

### Idiomatic Rust Evolution

| Before | After | Files |
|--------|-------|-------|
| `partial_cmp(&b).unwrap()` | `f64::total_cmp(&b)` | exp036, exp055 |
| `#[allow(clippy::cast_*)]` | `#[expect(clippy::cast_*, reason = "...")]` | exp109 (4 instances) |
| `Mutex::lock().expect("lock")` | Crate-level `#[expect(clippy::expect_used, reason = "...")]` | exp072 |
| ````rust,ignore` | ````rust,no_run` | `data/mod.rs`, `ipc/tower_atomic.rs` |

---

## Part 3: Patterns Worth Absorbing

### For barraCuda

1. **`TensorSession` fused pipeline** — healthSpring's `gpu/fused.rs` implements a local multi-op fused pipeline (upload → N compute → readback). This is the healthSpring-specific use case for `barracuda::session::TensorSession`. When TensorSession stabilizes, healthSpring will migrate.

2. **`std_dev` with sample variance** — `uncertainty.rs` computes sample standard deviation using `barracuda::stats::mean` plus manual variance. If `barracuda::stats` adds a `sample_variance(data) → f64` function, healthSpring can delegate.

3. **FFT for biosignal** — `biosignal/fft.rs` has a CPU radix-2 FFT (208 lines). For GPU-scale HRV analysis, `barracuda::spectral::fft` could be rewired. The CPU version stays for small workloads (<512 points) where GPU dispatch overhead dominates.

### For toadStool

1. **Streaming pipeline validation** — healthSpring's exp072 demonstrates `execute_streaming()` with per-stage callbacks + petalTongue gauge push. The callback pattern (stage index, total, result) is a good API for all toadStool consumers.

2. **Mixed dispatch validation** — exp087 validates CPU+GPU+NPU routing with PCIe P2P bypass. The dispatch thresholds (element count → substrate) are healthSpring-specific but the pattern is reusable.

### For coralReef

1. **Shader compile forwarding** — healthSpring now forwards `compute.shader_compile` requests to coralReef via capability discovery. coralReef should ensure its `shader.compile` method accepts the standard `{source, target, options}` params.

### For All Springs

1. **`f64::total_cmp` over `partial_cmp().unwrap()`** — modern Rust idiom for NaN-safe float comparison. All springs should migrate `sort_by(|a, b| a.partial_cmp(b).unwrap())` to `sort_by(f64::total_cmp)`.

2. **`#[expect()]` with `reason`** — zero `#[allow()]` policy. Every lint suppression must use `#[expect()]` with a `reason` string documenting why.

3. **`cargo-deny` in CI** — healthSpring now enforces dependency hygiene via `EmbarkStudios/cargo-deny-action@v2`. All springs should add this.

4. **Named tolerances over magic numbers** — inline `1e-10` in validation checks should reference centralized `tolerances::MACHINE_EPSILON`. Makes auditing trivial.

---

## Part 4: Recommended Upstream Actions

### barraCuda Team

| Priority | Action | Effort |
|----------|--------|--------|
| **P1** | Validate Tier B rewire parity (MM, SCFA, BeatClassify) on GPU hardware | 1 session |
| **P2** | Add `sample_variance(data) → f64` to `barracuda::stats` | Small |
| **P2** | Stabilize `TensorSession` API for fused multi-op pipelines | Medium |
| **P3** | GPU FFT rewire path for biosignal workloads (>512 points) | Medium |

### toadStool Team

| Priority | Action | Effort |
|----------|--------|--------|
| **P1** | Verify healthSpring dispatch thresholds with real GPU benchmarks | 1 session |
| **P2** | Standardize streaming callback API (`fn(idx, total, result)`) | Small |

### coralReef Team

| Priority | Action | Effort |
|----------|--------|--------|
| **P1** | Verify `shader.compile` method accepts healthSpring's forwarded requests | Small |
| **P2** | Document expected params for `shader.compile` JSON-RPC method | Small |

---

## Part 5: Quality Metrics

| Metric | V35 | V36 | Δ |
|--------|-----|-----|---|
| Tests | 613 | 617 | +4 |
| Experiments | 73 | 79 | +6 |
| Python baselines | 42 | 42 | — |
| Cross-validation | 113/113 | 113/113 | — |
| JSON-RPC capabilities | 79 | 79 | — |
| GPU ops (rewired to barraCuda) | 3/6 | **6/6** | +3 |
| `#[allow()]` in production | 0 | 0 | — |
| `#[expect()]` without reason | 0 | 0 | — |
| Unsafe blocks | 0 | 0 | — |
| Clippy warnings | 0 | 0 | — |
| `cargo fmt` diffs | 0 | 0 | — |
| `cargo doc` warnings | 0 | 0 | — |
| Ignored doc-tests | 2 | 0 | -2 |
| `cargo-deny` in CI | No | **Yes** | New |
| WGSL license correct | No (AGPL-3.0-only) | **Yes** (or-later) | Fixed |

---

## Part 6: New Experiments (V36)

| Exp | Track | Title | Validation |
|-----|-------|-------|-----------|
| 095 | Drug Discovery (7) | iPSC skin cytokine/viability model | 15 checks |
| 096 | Drug Discovery (7) | Niclosamide PBPK delivery | 17 checks |
| 107 | Microbiome (2) | QS-augmented Anderson disorder | 24 checks |
| 108 | Microbiome (2) | Real 16S Anderson pipeline | — |
| 109 | Biosignal (3) | MIT-BIH arrhythmia beat classification | 20 checks |
| 110 | Comparative Med (6) | Equine laminitis PK model | — |

---

## Verification

```bash
cargo fmt --check --all          # 0 diffs
cargo clippy --workspace         # 0 warnings
cargo test --workspace           # 617 passed, 0 failed, 0 ignored
cargo doc --workspace --no-deps  # 0 warnings
```

---

**healthSpring V36 | 617 tests | 79 experiments | 6/6 GPU ops rewired | 79 capabilities | AGPL-3.0-or-later**
