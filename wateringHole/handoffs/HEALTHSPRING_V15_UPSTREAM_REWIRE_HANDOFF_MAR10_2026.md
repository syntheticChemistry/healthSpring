# healthSpring V15 — Upstream Rewire + Precision Routing Handoff

**Date**: March 10, 2026
**From**: healthSpring V15
**To**: barraCuda, toadStool, coralReef, metalForge
**License**: AGPL-3.0-only
**Covers**: V14.1→V15 — Upstream rewire: pull and integrate ecosystem evolutions from toadStool (S128–S142), barraCuda (absorption sprint), and coralReef (Phase 10). Precision routing, provenance documentation, dependency wiring, and handoff catch-up.

---

## Executive Summary

healthSpring V15 catches up to the ecosystem state after significant upstream evolution. barraCuda has absorbed healthSpring's core math primitives (Hill dose-response, Population PK Monte Carlo, diversity indices, LCG PRNG, eigensolver). toadStool has evolved `PrecisionRoutingAdvice` for f64 GPU dispatch decisions. coralReef has reached Phase 10 with full f64 transcendental support via DFMA polynomial lowering.

This handoff documents:
1. What was absorbed upstream and the provenance trail
2. Precision routing wired into metalForge (mirroring toadStool S128)
3. Feature-gated upstream dependency for canonical op consumption
4. Shader provenance mapping (local → barraCuda canonical)
5. Remaining rewiring work for future sessions

**Metrics**: 422 tests, 48 experiments, 0 clippy warnings, 0 format diffs, 0 unsafe code.

---

## Part 1: Ecosystem State (What Evolved Upstream)

### 1.1 barraCuda Absorption Sprint

barraCuda has absorbed the following from healthSpring:

| healthSpring Local | barraCuda Canonical | Status |
|---|---|---|
| `pkpd::hill_dose_response()` | `barracuda::ops::HillFunctionF64` | Absorbed |
| `gpu::GpuOp::HillSweep` + `hill_dose_response_f64.wgsl` | `barracuda::shaders::math::hill_f64.wgsl` | Absorbed |
| `pkpd::population_pk_cpu()` | `barracuda::ops::PopulationPkF64` | Absorbed |
| `gpu::GpuOp::PopulationPkBatch` + `population_pk_f64.wgsl` | `barracuda::shaders::science::population_pk_f64.wgsl` | Absorbed |
| `microbiome::{shannon_index, simpson_index}` | `barracuda::ops::bio::DiversityFusionGpu` / `barracuda::stats::*` | Absorbed |
| `diversity_f64.wgsl` | `barracuda::shaders::bio::diversity_fusion_f64.wgsl` | Absorbed |
| `rng::lcg_step()`, `LCG_MULTIPLIER` | `barracuda::rng::{lcg_step, LCG_MULTIPLIER, state_to_f64}` | Absorbed |
| `microbiome::tridiagonal_ql_eigen()` | `barracuda::special::tridiagonal_ql` | Absorbed |
| `microbiome::anderson_hamiltonian_1d()` | `barracuda::special::anderson_diagonalize` | Absorbed |

barraCuda also added:
- `SpringDomain::HEALTH_SPRING` provenance tracking
- `Fp64Strategy` (Sovereign, Native, Hybrid, Concurrent) precision model
- `PrecisionRoutingAdvice` re-exported from device driver profile
- `PopulationPkConfig` with full parameterization (`base_cl`, `cl_low`, `cl_high`)
- `DiversityResult` with Shannon + Simpson + Pielou evenness

### 1.2 toadStool S128–S142 Evolution

| Session | Key Evolution |
|---|---|
| S128 | `PrecisionRoutingAdvice` (F64Native, F64NativeNoSharedMem, Df64Only, F32Only); `GpuAdapterInfo` with f64 shared-memory reliability probe; `shader.compile.*` JSON-RPC (4 methods) |
| S129 | C deps eliminated (flate2 → pure Rust); zero-copy `Cow`/`Arc<str>` hot paths |
| S130 | `cross_spring_provenance.rs` with 17+ flows; coralReef proxy |
| S139 | `StreamingDispatch` absorbed from hotSpring; `PipelineGraph` DAG from neuralSpring |
| S140 | `StreamingDispatchContext` enriched with `StageProgress`/`ProgressCallback` (from healthSpring V13); hardcoding → `interned_strings` |
| S141 | 120+ clippy pedantic fixes (10 crates); `Vec<u8>` → `bytes::Bytes` zero-copy |
| S142 | `PcieTransport`; `ResourceOrchestrator`; GPU sysmon telemetry |

### 1.3 coralReef Phase 10

- WGSL → native GPU binary (NVIDIA SM70–SM89, AMD RDNA2/GFX1030)
- Full f64 transcendental support via DFMA polynomial lowering
- 13-tier numerical tolerance model (`tol::`)
- `FmaPolicy` (AllowFusion / NoContraction) for bit-exact CPU parity
- `coral-gpu` unified compile + dispatch API
- `shader.compile.{wgsl,spirv,status,capabilities}` IPC methods
- Pure Rust — no C dependencies

---

## Part 2: What Changed in healthSpring (V14.1→V15)

### 2.1 Precision Routing in metalForge

Added `PrecisionRouting` enum to `metalForge::forge` mirroring toadStool S128:

```rust
pub enum PrecisionRouting {
    F64Native,
    F64NativeNoSharedMem,
    Df64Only,
    F32Only,
}
```

`GpuInfo` now carries:
- `f64_shared_mem_reliable: bool` — naga/SPIR-V shared-memory reduction reliability
- `precision: PrecisionRouting` — derived from hardware probing

`probe_gpu()` populates precision routing based on adapter features. GPU dispatch code can now branch on `caps.gpu.as_ref().map(|g| g.precision)` to select shader variants.

### 2.2 Shader Provenance Documentation

`gpu/mod.rs` shader module now documents:
- Canonical upstream locations in barraCuda
- f64 precision workarounds (f32 transcendental intermediates)
- Migration path to `barracuda::ops::*` consumption

Local WGSL shaders are marked as Spring validation targets, bit-identical to absorbed versions.

### 2.3 Upstream Dependency Wiring

`ecoPrimal/Cargo.toml` now has:
```toml
barracuda = { path = "../../barraCuda/crates/barracuda", default-features = false, optional = true }

[features]
upstream-ops = ["dep:barracuda"]
```

Default build unchanged. `upstream-ops` feature gates consumption of canonical barraCuda ops for when healthSpring transitions from local shaders to upstream GPU dispatch.

### 2.4 GPU Module Absorption Status Update

Updated `gpu/mod.rs` module documentation from "ABSORPTION CANDIDATES" to "ABSORPTION STATUS" with specific upstream API references.

---

## Part 3: API Mapping (healthSpring → barraCuda)

For consumers implementing the `upstream-ops` feature:

| healthSpring | barraCuda Canonical | Notes |
|---|---|---|
| `pkpd::hill_dose_response(c, ec50, n, emax)` | `barracuda::stats::hill(x, k, n)` (scalar) | barraCuda normalizes to emax=1.0; multiply by emax |
| `gpu::GpuOp::HillSweep` | `barracuda::ops::HillFunctionF64::dose_response(device, ec50, n, emax)` | Full GPU dispatch with precision routing |
| `pkpd::population_pk_cpu()` | `barracuda::ops::PopulationPkF64::new(device, config).simulate(n, seed)` | `PopulationPkConfig` replaces inline params |
| `microbiome::shannon_index()` | `barracuda::stats::shannon()` | Identical algorithm |
| `microbiome::simpson_index()` | `barracuda::stats::simpson()` | Identical algorithm |
| `microbiome::chao1_estimate()` | `barracuda::stats::chao1()` | Identical algorithm |
| `microbiome::pielou_evenness()` | `barracuda::stats::pielou()` | Identical algorithm |
| `microbiome::bray_curtis_distance()` | `barracuda::stats::bray_curtis()` | Identical algorithm |
| `rng::lcg_step()` | `barracuda::rng::lcg_step()` | Same LCG constants |
| `microbiome::tridiagonal_ql_eigen()` | `barracuda::special::tridiagonal_ql()` | QL algorithm |
| `microbiome::anderson_hamiltonian_1d()` | `barracuda::special::anderson_diagonalize()` | Includes QL call |

---

## Part 4: For toadStool

### 4.1 Dispatch Thresholds (Kokkos Crossover)

From healthSpring's Kokkos-parity benchmarks, recommended thresholds for toadStool's dispatch planner:

| Pattern | GPU Crossover (N) | Workload Type |
|---|---|---|
| Reduction | > 10,000 | `Workload::Reduce` |
| Scatter | > 50,000 | `Workload::Parallel` (with atomics) |
| Monte Carlo | > 100,000 | `Workload::Parallel` |
| ODE batch | > 5,000 patients | `Workload::Sweep` |
| NLME iteration | > 100 subjects | `Workload::Sweep` |

### 4.2 Precision Routing Alignment

healthSpring's `PrecisionRouting` enum is semantically identical to toadStool S128's `PrecisionRoutingAdvice`. When healthSpring integrates with the ecosystem daemon, it should consume `PrecisionRoutingAdvice` from the toadStool runtime rather than probing locally.

### 4.3 StageProgress Pattern

healthSpring's `Pipeline::execute_streaming()` callback `FnMut(usize, usize, &StageResult)` aligns with toadStool's `StageProgress` pattern. healthSpring's callback provides `(stage_index, total_stages, result)` — toadStool's `StageProgress` adds `stage_name` and `elapsed_secs`. Consider adopting `StageProgress` for richer telemetry.

---

## Part 5: For coralReef

### 5.1 Shader Compilation Targets

All healthSpring WGSL shaders require f64 support:

| Shader | Entry Point | Dispatch | f64 Usage |
|---|---|---|---|
| `hill_dose_response_f64.wgsl` | `main` | `(ceil(N/256), 1, 1)` | f64 storage, f32 `exp`/`log` intermediates |
| `population_pk_f64.wgsl` | `main` | `(ceil(N/256), 1, 1)` | f64 storage, u32 PRNG |
| `diversity_f64.wgsl` | `main` | `(N_communities, 1, 1)` | f64 storage/reduction, f32 `log` intermediate |

### 5.2 f64 Workaround Patterns

These WGSL workarounds should be replaced by coralReef's f64 lowering:

- `power_f64(base, exp)` → `f64(exp(f32(exp * log_base)))` (~7 decimal digits)
- `log_f64(x)` → `f64(log(f32(x)))` for driver portability

coralReef Phase 10 provides DFMA polynomial lowering for proper f64 transcendentals. When healthSpring routes through coralReef, these workarounds can be removed.

---

## Part 6: For barraCuda

### 6.1 Remaining Absorption Candidates

| Module | Primitive | Priority | GPU Pattern |
|---|---|---|---|
| `pkpd/nlme.rs` | FOCE estimation | P0 | Per-subject gradient → batch parallel |
| `pkpd/nlme.rs` | SAEM estimation | P0 | E-step Metropolis-Hastings |
| `pkpd/diagnostics.rs` | VPC simulation | P0 | Embarrassingly parallel |
| `biosignal/fft.rs` | Cooley-Tukey FFT | P1 | Butterfly pattern |
| `biosignal/ecg.rs` | Pan-Tompkins QRS | P2 | Streaming pipeline (NPU candidate) |
| `biosignal/fusion.rs` | Multi-channel fusion | P2 | Reduction pattern |
| `endocrine.rs` | Testosterone PK | P2 | Compartmental ODE |

### 6.2 Precision Model Evolution

healthSpring's local shaders use f32 transcendental intermediates (~7 digits). barraCuda's `Fp64Strategy` should route these to:
- `F64Native` on GPUs with native f64 (e.g., NVIDIA A100, V100)
- `Df64Only` via coralReef's DFMA lowering on consumer GPUs
- `F32Only` with documented precision loss as last resort

---

## Part 7: Upstream Rewiring (V15.1)

### 7.1 Completed Rewiring

| Module | Before | After |
|---|---|---|
| `rng.rs` | Local LCG (63 lines) | `pub use barracuda::rng::*` + local `box_muller`/`normal_sample` |
| `microbiome::anderson_diagonalize` | Local 100-line QL eigensolver | Delegates to `barracuda::special::anderson_diagonalize` |
| `barracuda` dependency | Optional behind `upstream-ops` | Non-optional (CPU math always available) |

### 7.2 Cross-Validation Tests Added

| Test | Validates |
|---|---|
| `cross_validate_shannon_vs_upstream` | `shannon_index` == `barracuda::stats::shannon_from_frequencies` |
| `cross_validate_bray_curtis_vs_upstream` | `bray_curtis` == `barracuda::stats::bray_curtis` |
| `cross_validate_anderson_vs_upstream` | `anderson_diagonalize` == `barracuda::special::anderson_diagonalize` |
| `cross_validate_hill_vs_upstream` | `hill_dose_response` == `barracuda::stats::hill` (normalized) |

### 7.3 Upstream Parity Benchmarks

| Function | Local | Upstream | Delta |
|---|---|---|---|
| Shannon (7 genera) | 25.3 ns | 23.5 ns | upstream 7% faster |
| Simpson (7 genera) | 2.4 ns | 6.6 ns | local 2.7x faster (simpler impl) |
| Pielou (7 genera) | 26.7 ns | 30.3 ns | local 12% faster |
| Bray-Curtis (7 genera) | 5.6 ns | 3.5 ns | upstream 38% faster |
| Hill (1K concs) | 8.66 µs | 8.51 µs | upstream 2% faster |
| LCG (1M steps) | 102 µs | 102 µs | identical (same code path) |
| state_to_f64 | 818 ps | 817 ps | identical |
| Anderson (L=50) | 636 µs | 636 µs | identical (delegates to upstream) |

Simpson local is faster because healthSpring's implementation is a tight
one-liner (`1 - Σ p²`) while upstream computes from raw counts with
normalization. For healthSpring's use case (pre-normalized frequencies),
the local implementation is optimal.

### 7.4 Cross-Spring Shader Evolution Document

New: `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md` — comprehensive map of
shader and primitive flows between all 6 springs, documenting:
- hotSpring → precision shaders (df64, Kahan, NVVM poisoning) → everyone
- wetSpring → bio shaders (Smith-Waterman, Gillespie, HMM) → neuralSpring
- neuralSpring → statistics (KL divergence, chi-squared, correlation) → hot, ground
- groundSpring → spectral + universal (Anderson, chi-squared, Welford) → ALL
- airSpring → hydrology (ET₀, seasonal, moving window) → wetSpring
- healthSpring → clinical math (Hill, PopPK, diversity, PRNG, eigensolver) → barraCuda

---

## Part 8: Verification

| Check | Result |
|---|---|
| `cargo fmt --all --check` | 0 diffs |
| `cargo clippy --workspace -- -D warnings -W clippy::pedantic` | 0 warnings |
| `cargo doc --workspace --no-deps` | Clean |
| `cargo test --workspace` | 425 passed, 0 failed |
| All 48 experiments | PASSED |
| `#![forbid(unsafe_code)]` | All lib crates |
| Files > 1000 lines | None |
| Cross-validation tests | 4 new, all passing |
| Upstream parity benchmarks | 8 groups, performance-neutral |

---

## Part 9: What's Next (V16 Targets)

1. **coralReef integration** — route shader compilation through `shader.compile.wgsl` IPC (replaces local wgpu/naga)
2. **toadStool daemon integration** — consume `PrecisionRoutingAdvice` at runtime from ecosystem daemon
3. **NLME GPU shaders** — FOCE/SAEM/VPC GPU promotion to barraCuda (P0 candidates)
4. **Consume upstream GPU ops** — activate `barracuda::ops::HillFunctionF64`, `PopulationPkF64`, `DiversityFusionGpu` when GPU feature is enabled
5. **df64 transcendental upgrade** — replace f32 `exp`/`log` workarounds with hotSpring's `df64_transcendentals.wgsl` via coralReef lowering
6. **StageProgress alignment** — adopt toadStool's `StageProgress` struct for richer streaming telemetry
