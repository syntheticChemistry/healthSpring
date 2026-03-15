<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V23 → barraCuda/toadStool: Deep Remediation + Absorption Handoff

**Date**: March 15, 2026
**From**: healthSpring V23 (435 tests, 61 experiments, 55+ wired capabilities)
**To**: barraCuda team, toadStool team
**License**: CC-BY-SA-4.0
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: V22 Niche Deployment Handoff (Mar 15), V15 Absorption Handoff (Mar 10)
**barraCuda pin**: `a60819c` (v0.3.5)

---

## Executive Summary

V23 is a zero-debt deep evolution. All V22 audit findings are resolved: license compliance, clippy nursery enforcement, `#[allow]` elimination, `UniBin` compliance, 13 new dispatch handlers, three-tier fetch implementation, tolerance centralization, and `ValidationHarness` adoption. This handoff documents what barraCuda and toadStool should absorb from healthSpring's validated work.

---

## Part 1: Tier A GPU Ops — Ready for Rewire

healthSpring validates three GPU ops that have canonical upstream implementations in barraCuda. The CPU reference (`execute_cpu`) and GPU path (local WGSL shaders) both produce validated results. The next step is rewiring healthSpring to consume barraCuda's ops directly.

| healthSpring Op | Local Shader | barraCuda Op | Status |
|-----------------|-------------|--------------|--------|
| `HillSweep` | `hill_dose_response_f64.wgsl` | `barracuda::ops::HillFunctionF64` | Rewire ready |
| `PopulationPkBatch` | `population_pk_f64.wgsl` | `barracuda::ops::PopulationPkF64` | Rewire ready |
| `DiversityBatch` | `diversity_f64.wgsl` | `barracuda::ops::bio::DiversityFusionGpu` | Rewire ready |

**Rewire path**: When `gpu` feature is active, `GpuContext::execute()` should delegate to `barracuda::ops::*` instead of local dispatch. The CPU fallback (`execute_cpu`) remains as the reference implementation.

**Validation**: Exp053 (GPU parity, 42/42 checks) and Exp060 (CPU vs GPU, 27 checks) confirm bit-identical results within `GPU_F32_TRANSCENDENTAL` tolerance (1e-4).

---

## Part 2: Tier B Absorption Candidates

Three local WGSL shaders are candidates for upstream absorption into barraCuda:

| Shader | Domain | Pattern | barraCuda Target |
|--------|--------|---------|-----------------|
| `michaelis_menten_batch_f64.wgsl` | Nonlinear PK | Euler ODE per patient, parallel | `barracuda::ops::bio::MichaelisMentenBatch` |
| `scfa_batch_f64.wgsl` | Gut metabolism | Element-wise Michaelis-Menten ×3 | `barracuda::ops::bio::ScfaBatch` |
| `beat_classify_batch_f64.wgsl` | Biosignal | Template correlation + argmax | `barracuda::ops::bio::BeatClassifyBatch` |

**f64 precision**: All shaders use f64 with f32 transcendental workarounds (`pow` → `exp(f32(n*log(c)))`). coralReef Phase 10 f64 lowering should replace these workarounds.

---

## Part 3: toadStool Pipeline Evolution

healthSpring's toadStool integration (`toadstool/src/`) provides:

- **Pipeline execution**: `execute()`, `execute_gpu()`, `execute_streaming()`, `execute_auto()` (764 LOC)
- **Stage ops**: 6 `StageOp` variants mapped to healthSpring science modules (596 LOC)
- **Auto-dispatch**: CPU/GPU selection based on workload size
- **Streaming callbacks**: Real-time progress reporting

**Absorption candidates for toadStool upstream**:
1. `StageOp::to_gpu_op()` mapping pattern — generalizable to any spring
2. `failed_stage_result()` pattern — safe GPU fallback without panic
3. Streaming progress callback pattern for long-running operations

---

## Part 4: NLME Population PK — New barraCuda Primitive Candidates

healthSpring implements FOCE and SAEM population PK estimation (`pkpd/nlme/`). These are computationally intensive and GPU-parallelizable:

| Algorithm | CPU Implementation | GPU Opportunity |
|-----------|-------------------|-----------------|
| FOCE inner loop | Per-subject eta optimization | Embarrassingly parallel across subjects |
| SAEM E-step | Monte Carlo eta sampling | GPU batch sampling |
| VPC simulation | N replicate datasets | Parallel simulation |
| Population Monte Carlo | N virtual patients | Embarrassingly parallel |

**Recommendation**: barraCuda should absorb the NLME inner loop as `barracuda::ops::nlme::FoceInnerLoop` — this is the bottleneck in population PK analysis and maps directly to GPU batch processing.

---

## Part 5: Quality Standards Achieved

| Check | V22 | V23 |
|-------|-----|-----|
| `cargo test` | 414 pass | **435 pass** |
| `cargo fmt` | 46 diffs in 9 files | **0 diffs** |
| `cargo clippy` (pedantic) | 0 warnings | **0 warnings** |
| `cargo clippy` (nursery) | Not enforced | **0 warnings** (`#![deny]`) |
| `#[allow()]` in production | 6 | **0** |
| `TODO` in production | 3 stubs | **0** |
| `unwrap/expect` in production | Several | **0** (safe patterns) |
| License | AGPL-3.0-only | **AGPL-3.0-or-later** (all files) |
| Files over 1000 LOC | 1 (dispatch.rs 1193) | **0** (max 968) |
| Inline magic tolerances | ~20 experiments | **0** (all `tolerances::*`) |
| `UniBin` compliance | No | **Yes** (clap, --help, --version) |

---

## Part 6: Verified barraCuda Usage

healthSpring consumes barraCuda `a60819c` (v0.3.5) via path dependency:

| barraCuda Module | healthSpring Usage |
|------------------|--------------------|
| `barracuda::rng::lcg_step` | Deterministic PRNG in all Monte Carlo experiments |
| `barracuda::rng::LCG_MULTIPLIER` | Centralized in `rng.rs` |
| `barracuda::special::tridiagonal_ql` | Anderson eigensolver in microbiome lattice |
| `barracuda::special::anderson_diagonalize` | Gut colonization resistance |
| `barracuda::stats::{shannon, simpson, chao1, pielou, bray_curtis}` | All diversity index calculations |

**No duplicate math**: All shared numerical primitives delegate to barraCuda. Zero local reimplementations.

---

## Part 7: What barraCuda Should Evolve

1. **NLME inner loop GPU op** — FOCE per-subject optimization as batch GPU kernel
2. **Michaelis-Menten batch op** — Absorb `michaelis_menten_batch_f64.wgsl`
3. **SCFA batch op** — Absorb `scfa_batch_f64.wgsl`
4. **Beat classify op** — Absorb `beat_classify_batch_f64.wgsl`
5. **TensorSession** — healthSpring's `GpuContext::execute_fused()` is the Spring-side pattern; barraCuda should provide `TensorSession` as the upstream equivalent

## What toadStool Should Evolve

1. **Spring StageOp registry** — generalize healthSpring's `StageOp` mapping so springs can register ops without modifying toadStool source
2. **Streaming progress API** — healthSpring's callback pattern should become toadStool's standard streaming interface
3. **Safe GPU fallback** — `failed_stage_result()` pattern should be upstream default behavior
