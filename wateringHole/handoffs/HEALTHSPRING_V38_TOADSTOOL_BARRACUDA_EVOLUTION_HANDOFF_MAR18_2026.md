<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# healthSpring V38 → toadStool / barraCuda Evolution Handoff

**Date**: 2026-03-18
**From**: healthSpring V38
**To**: toadStool team, barraCuda team, coralReef team
**License**: AGPL-3.0-or-later
**Pins**: barraCuda v0.3.5 (rev a60819c3), toadStool S158+, coralReef Phase 10 Iteration 55+
**Supersedes**: HEALTHSPRING_V37_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR18_2026.md

---

## Executive Summary

healthSpring V38 completes the deep debt audit: all 79 experiments use standardized `ValidationHarness`, all test tolerances are named constants, all provenance records have DOI citations, typed IPC dispatch replaces raw `rpc::send()`, and clippy nursery enforced workspace-wide. This handoff focuses on what barraCuda/toadStool should absorb.

- **719 tests** (up from 706), 79 experiments, 80 capabilities
- **All 79 experiments → `ValidationHarness`** — zero ad-hoc validation patterns remain
- **~120+ inline magic tolerances → named constants** in 30+ test files
- **Typed IPC dispatch** — `routing.rs` uses `compute_dispatch`, `shader_dispatch`, `inference_dispatch`, `data_dispatch`
- **Provenance DOI citations** — all provenance records enhanced with literature references
- **`clippy::nursery`** enforced across all 79 experiments (already had `pedantic`)

---

## Part 1: barraCuda Primitive Consumption (same as V37 — no new ops consumed)

No new barraCuda ops consumed in V38 — this was a debt completion sprint.

### CPU Primitives Used

| Category | Primitive | healthSpring Module | Usage |
|----------|-----------|-------------------|-------|
| **rng** | `lcg_step`, `LCG_MULTIPLIER`, `state_to_f64`, `uniform_f64_sequence` | `rng.rs` (re-export) | PPG, EDA, ECG signal generation; SAEM Monte Carlo; bootstrap resampling |
| **stats** | `mean` | `uncertainty.rs` | Bootstrap CI, jackknife, bias-variance decomposition |
| **numerical** | `OdeSystem`, `BatchedOdeRK4` | `gpu/ode_systems.rs` | Michaelis-Menten, oral 1-compartment, 2-compartment ODE |

### GPU Ops Used (All 6 — Complete Rewire)

| healthSpring Op | barraCuda Op | Rewire Module | Status |
|-----------------|-------------|---------------|--------|
| `HillSweep` | `barracuda::ops::hill_f64::HillFunctionF64` | `barracuda_rewire.rs` | **Tier A — LIVE** |
| `PopulationPkBatch` | `barracuda::ops::population_pk_f64::PopulationPkF64` | `barracuda_rewire.rs` | **Tier A — LIVE** |
| `DiversityBatch` | `barracuda::ops::bio::diversity_fusion::DiversityFusionGpu` | `barracuda_rewire.rs` | **Tier A — LIVE** |
| `MichaelisMentenBatch` | `barracuda::ops::health::michaelis_menten_batch::MichaelisMentenBatchGpu` | `barracuda_rewire.rs` | **Tier B — LIVE (V36)** |
| `ScfaBatch` | `barracuda::ops::health::scfa_batch::ScfaBatchGpu` | `barracuda_rewire.rs` | **Tier B — LIVE (V36)** |
| `BeatClassifyBatch` | `barracuda::ops::health::beat_classify::BeatClassifyGpu` | `barracuda_rewire.rs` | **Tier B — LIVE (V36)** |

### What's NOT Delegated (Remaining Local Math)

| Local Implementation | File | LOC | Why Not Delegated | Recommended Action |
|---------------------|------|-----|-------------------|-------------------|
| `fn std_dev(data)` | `uncertainty.rs:290` | 8 | Uses `barracuda::stats::mean` internally, adds manual variance loop | **barraCuda: add `stats::sample_variance()`** |
| `fn fft_complex_inplace()`, `rfft()`, `irfft()` | `biosignal/fft.rs` | 208 | CPU radix-2 FFT for HRV analysis (<512 pts) | **barraCuda: GPU FFT for >512pt workloads** |
| `fn fused_pipeline()` | `gpu/fused.rs` | ~300 | Local multi-op upload→compute→readback | **barraCuda: stabilize `TensorSession` API** |

---

## Part 2: Recommended Actions for barraCuda

### P1 — High Priority (carry forward from V37)

| Action | Context | Effort |
|--------|---------|--------|
| **Add `stats::sample_variance(data) → f64`** | healthSpring `uncertainty.rs` manually computes sample variance from `mean()`. groundSpring likely duplicates. One function eliminates N local copies. | Small |
| **Validate Tier B GPU parity on hardware** | healthSpring rewired MM, SCFA, BeatClassify to `barracuda::ops::health::*`. Needs GPU hardware validation to confirm numerical parity. healthSpring tests are CPU-only. | 1 session |
| **`mul_add()` sweep in CPU reference code** | healthSpring (V37), neuralSpring (S165), airSpring (V090) have all done FMA sweeps. barraCuda's own CPU reference implementations should follow for consistency. | Small |

### P2 — Medium Priority (carry forward from V37)

| Action | Context | Effort |
|--------|---------|--------|
| **Stabilize `TensorSession` API** | healthSpring's `gpu/fused.rs` implements a local multi-op pipeline (upload → N compute → readback). This is the consumer use case for `TensorSession`. When the API stabilizes, healthSpring will rewire. | Medium |
| **GPU FFT (`spectral::fft_gpu`)** | healthSpring `biosignal/fft.rs` has 208-line CPU FFT. groundSpring also needs GPU FFT for spectral recon. Shared upstream implementation benefits both. | Medium |
| **`stats::kahan_sum()` absorption** | wetSpring V127 identified `kahan_sum` as generic math for upstream. healthSpring could use for large-array summation in population PK Monte Carlo. | Small |

### P2 — NEW from V38

| Action | Context | Effort |
|--------|---------|--------|
| **`ValidationHarness` pattern for upstream** | healthSpring proved that standardizing 79 experiments to a single harness with named tolerances and `.exit()` eliminates all manual counter bugs. barraCuda's own test infrastructure should consider adopting. | Medium |
| **Tolerance registry as named constants** | healthSpring now has 70+ named constants in `tolerances.rs`. barraCuda should consider a shared tolerance module for cross-spring consistency. | Small |

### P3 — Track (carry forward from V37)

| Action | Context | Effort |
|--------|---------|--------|
| **`BatchedOdeRK45F64` (adaptive step)** | airSpring V090 requested adaptive step-size ODE. healthSpring Michaelis-Menten with extreme Km/Vmax ratios would benefit. | Medium |
| **PRNG alignment** | groundSpring noted xorshift64 vs xoshiro128** divergence. healthSpring uses LCG via `barracuda::rng`. Unified PRNG policy would help reproducibility. | Discussion |

---

## Part 3: Recommended Actions for toadStool

### P1 — High Priority (carry forward from V37)

| Action | Context | Effort |
|--------|---------|--------|
| **Validate healthSpring dispatch thresholds** | healthSpring's element-count → substrate routing (CPU vs GPU) was tuned on RTX 4070. Real hardware benchmarks should confirm. exp085/087 provide the test matrix. | 1 session |
| **Standardize streaming callback API** | healthSpring exp072 uses `execute_streaming()` with `fn(stage_idx, total, result)` callback. This pattern should be the standard for all toadStool consumers. | Small |

### P2 — Medium Priority (carry forward from V37)

| Action | Context | Effort |
|--------|---------|--------|
| **Session-level provenance** | healthSpring now has 49 structured provenance records. toadStool's dispatch sessions should carry provenance metadata (pipeline ID, spring version, dispatch timestamp). | Medium |
| **MCP integration path** | healthSpring exposes `mcp.tools.list` returning 23 tool schemas. toadStool should consider a standard for advertising compute tools via MCP. | Discussion |

### P2 — NEW from V38

| Action | Context | Effort |
|--------|---------|--------|
| **Typed dispatch client as pattern** | healthSpring's typed clients (`compute_dispatch::submit`, `shader_dispatch::compile`, etc.) are a clean pattern toadStool consumers should adopt. | Small |

---

## Part 4: Learnings for All Primals

### NEW learnings from V38

1. **Standardized validation harness** — 23 experiments migrated in a single sprint by: (a) replacing `passed/failed` counters with `h = ValidationHarness::new()`, (b) replacing `if/else increment` with `h.check_bool()`/`h.check_abs()`/`h.check_rel()`, (c) replacing `process::exit()` with `h.exit()`. Pattern is trivially mechanical.

2. **Named tolerance elimination of magic numbers** — searching test code for `1e-N` patterns and mapping to named constants prevents tolerance drift.

3. **Typed IPC dispatch** — reduces 4 raw `rpc::send()` call sites to typed function calls with proper error types. Discovery is internal to the client.

---

## Part 5: Quality Metrics

| Metric | V37 | V38 |
|--------|-----|-----|
| Tests | 706 | 719 |
| Experiments | 79 | 79 |
| ValidationHarness adoption | partial | 79/79 |
| Named tolerance constants | ~55 | ~70+ |
| Provenance records with DOI | 0 | 49 |
| Typed IPC dispatch handlers | 0 | 4 |
| clippy::nursery experiments | 0 | 79 |
| Unsafe blocks | 0 | 0 |
| `#[allow()]` in production | 0 | 0 |
| Clippy warnings | 0 | 0 |
| TODO/FIXME | 0 | 0 |
| Files > 1000 LOC | 0 | 0 (max 800) |

---

## Verification

```bash
cargo fmt --check --all          # 0 diffs
cargo clippy --workspace --all-targets  # 0 warnings
cargo test --workspace           # 719 passed, 0 failed
cargo doc --workspace --no-deps  # 0 warnings
```

---

**healthSpring V38 | 719 tests | 79 experiments | 79/79 ValidationHarness | 80 capabilities | AGPL-3.0-or-later**
