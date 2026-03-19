# healthSpring GPU Evolution Readiness & barraCuda Dependency Audit

**Date**: March 19, 2026  
**Scope**: ecoPrimal GPU integration, WGSL shaders, barraCuda dependency health, metalForge Tier 3 dispatch, toadStool pipeline  
**Supersedes**: GPU_EVOLUTION_AUDIT_MAR16_2026.md

---

## 1. Executive Summary

| Metric | Value |
|--------|-------|
| **barraCuda version** | v0.3.5 (path dep `../../barraCuda/crates/*`, Cargo.lock) |
| **GPU ops** | 6 (Hill, PopPK, Diversity, MM, SCFA, BeatClassify) |
| **Tier A (direct rewire)** | 3 — Hill, PopPK, Diversity |
| **Tier B (absorbed, rewired)** | 3 — MM, SCFA, BeatClassify |
| **TensorSession** | **Not used** — fused pipeline uses local WGSL |
| **biosignal FFT** | **Local implementation** — radix-2 Cooley-Tukey in `biosignal/fft.rs`; barraCuda FFT not used |
| **WGSL shaders** | 6 in `ecoPrimal/shaders/health/` |
| **metalForge workloads** | 8 GPU-related (PopulationPk, DoseResponse, DiversityIndex, MM, SCFA, BeatClassify, BiosignalDetect, BiosignalFusion) |

---

## 2. barraCuda Dependency Health

### 2.1 Version & Resolution

| Dependency | Version | Source |
|------------|---------|--------|
| `barracuda` | 0.3.5 | Path `../../barraCuda/crates/barracuda` |
| `barracuda-core` | 0.3.5 | Path `../../barraCuda/crates/barracuda-core` |

**Cargo.toml**: Path dependency, no version constraint. Comment: *"Validated against barraCuda rev a60819c3 (2026-03-14)"*.

**Recommendation**: Pin barraCuda version (git rev or crate version) in Cargo.toml for reproducible CI.

### 2.2 Features Used

- `gpu` = wgpu, tokio, bytemuck
- `barracuda-ops` = gpu (enables Tier A+B rewire in `GpuContext::execute()`)
- `sovereign-dispatch` = coralReef compile path (HillSweep only, experimental)

---

## 3. Tier A/B Delegation Patterns (barracuda_rewire.rs)

### 3.1 Tier A — Direct barraCuda Ops (absorbed upstream)

| GpuOp | barraCuda Op | Delegation Path |
|-------|--------------|-----------------|
| `HillSweep` | `HillFunctionF64::dose_response()` | `execute_hill_barracuda()` |
| `PopulationPkBatch` | `PopulationPkF64::new()` + `simulate()` | `execute_pop_pk_barracuda()` |
| `DiversityBatch` | `DiversityFusionGpu::compute()` | `execute_diversity_barracuda()` |

### 3.2 Tier B — barraCuda health Module (absorbed upstream, rewired)

| GpuOp | barraCuda Op | Delegation Path |
|-------|--------------|-----------------|
| `MichaelisMentenBatch` | `MichaelisMentenBatchGpu::compute()` | `execute_mm_batch_barracuda()` |
| `ScfaBatch` | `ScfaBatchGpu::compute()` | `execute_scfa_batch_barracuda()` |
| `BeatClassifyBatch` | `BeatClassifyGpu::classify()` | `execute_beat_classify_barracuda()` |

### 3.3 Where Rewire Is Used

| Path | Tier A | Tier B | Notes |
|------|--------|--------|-------|
| `GpuContext::execute()` | ✅ barraCuda | ✅ barraCuda | When `barracuda-ops` + `barracuda_device` available |
| `GpuContext::execute_fused()` | ❌ Local WGSL | ❌ Local WGSL | **Never** uses barraCuda; always `fused::prepare_*` |
| `dispatch::execute_gpu()` | ✅ barraCuda | ❌ Local WGSL | Tier A only; Tier B falls through to local path |

**Critical gap**: The fused pipeline (single encoder, upload once → N passes → readback once) **never** benefits from barraCuda. All 6 ops use local WGSL in `execute_fused()`.

---

## 4. TensorSession Usage

**Status: NOT USED.**

`GpuContext` implements its own fused pipeline:

- `prepare_all_ops()` → `fused::prepare_hill`, `prepare_pop_pk`, `prepare_diversity`, etc.
- `submit_compute_passes()` → single encoder, N compute passes
- `readback_all()` → per-op staging buffers, tokio oneshot map_async

**Design rationale** (from `fused.rs`):

1. barraCuda's `TensorSession` is the intended evolution path.
2. Mixing barraCuda ops (own encoders) with local WGSL would break the unidirectional pattern.
3. When `TensorSession` is mature, the fused pipeline will migrate entirely.

---

## 5. biosignal/fft.rs — barraCuda FFT vs Local

**Answer: LOCAL implementation.**

`ecoPrimal/src/biosignal/fft.rs` implements:

- **Radix-2 Cooley-Tukey** in-place complex FFT
- **rfft** / **irfft** for real signals
- Zero-pad to power-of-two
- Pure Rust, no external dependency

**barraCuda FFT**: Not used. BARRACUDA_REQUIREMENTS.md lists `fft_radix2_f64` as "Still Needed: Write Phase" for GPU. EVOLUTION_MAP.md V14.1 notes: *"DFT deduplication: visualization/scenarios/biosignal.rs HRV power spectrum now delegates to `biosignal::fft::rfft` instead of local DFT reimplementation"* — i.e., HRV delegates to this local FFT, not barraCuda.

---

## 6. Math Duplication vs barraCuda

### 6.1 Already Delegating to barraCuda

| Module | Usage |
|--------|-------|
| `microbiome` | `barracuda::stats::shannon_from_frequencies`, `simpson`, `chao1_classic`, `bray_curtis` |
| `microbiome/anderson` | `barracuda::special::anderson_diagonalize` |
| `pkpd/dose_response` | `barracuda::stats::hill` |
| `rng` | `barracuda::rng::{lcg_step, LCG_MULTIPLIER, state_to_f64}` |
| `gpu/ode_systems` | `barracuda::numerical::OdeSystem`, `BatchedOdeRK4` |
| `uncertainty` | `barracuda::stats::mean` |
| `pkpd/nonlinear` | `barracuda::health::pkpd::mm_auc` |
| `biosignal/stress` | `barracuda::health::biosignal::scr_rate` |

### 6.2 Local Reimplementations (No barraCuda Equivalent or Intentional)

| Location | Function | barraCuda Equivalent? | Notes |
|----------|----------|------------------------|-------|
| `biosignal/fft.rs` | `rfft`, `irfft`, `fft_complex_inplace` | None | Local radix-2 FFT; barraCuda has no FFT primitive |
| `biosignal/classification.rs` | `normalized_correlation(a, b)` | `dot` only | Pearson correlation; barraCuda has `dot` but not full correlation |
| `pkpd/nlme/solver.rs` | `cholesky_solve` | `barracuda::linalg::solve_triangular` | **Intentional** — 2×2/3×3 matrices, integrated fallback; barraCuda targets larger GPU matrices |
| `uncertainty.rs` | `bootstrap_mean`, `jackknife_mean_variance`, etc. | `barracuda::stats` has histogram, percentile, median | UQ patterns (bootstrap, jackknife) not in barraCuda stats |
| `rng.rs` | `box_muller`, `normal_sample` | None | Domain-specific; uses barraCuda LCG for uniforms |

### 6.3 ODE Solvers

| Usage | barraCuda Path |
|-------|----------------|
| `gpu/ode_systems.rs` | Implements `OdeSystem` for 3 PK models; uses `BatchedOdeRK4::integrate_cpu` and `generate_shader()` |
| `michaelis_menten_batch_f64.wgsl` | Handwritten Euler ODE; does **not** use `BatchedOdeRK4::generate_shader()` |

**V27 handoff**: ODE→WGSL codegen is ready via `OdeSystem` trait. The handwritten MM batch shader could be migrated to `BatchedOdeRK4::generate_shader()` for consistency.

---

## 7. Complete Mapping: Rust Module → barraCuda op / WGSL → Pipeline Stage → Tier → Blocks

| Rust Module | barraCuda Op / WGSL Shader | Pipeline Stage | Tier | What Blocks Promotion? |
|-------------|----------------------------|----------------|------|------------------------|
| `gpu/dispatch/hill.rs` | `HillFunctionF64` / `hill_dose_response_f64.wgsl` | DoseResponse | A | Nothing — rewire ready; fused path not wired |
| `gpu/dispatch/pop_pk.rs` | `PopulationPkF64` / `population_pk_f64.wgsl` | PopulationPk | A | Nothing — rewire ready; fused path not wired |
| `gpu/dispatch/diversity.rs` | `DiversityFusionGpu` / `diversity_f64.wgsl` | DiversityIndex | A | Nothing — rewire ready; fused path not wired |
| `gpu/dispatch/batch_ops.rs` (MM) | `MichaelisMentenBatchGpu` / `michaelis_menten_batch_f64.wgsl` | MichaelisMentenBatch | B | barraCuda bio module design; could use `BatchedOdeRK4` codegen |
| `gpu/dispatch/batch_ops.rs` (SCFA) | `ScfaBatchGpu` / `scfa_batch_f64.wgsl` | ScfaBatch | B | barraCuda has no direct equivalent; bio module candidate |
| `gpu/dispatch/batch_ops.rs` (Beat) | `BeatClassifyGpu` / `beat_classify_batch_f64.wgsl` | BeatClassifyBatch | B | barraCuda has no biosignal primitive |
| `gpu/ode_systems.rs` | `BatchedOdeRK4` (OdeSystem) | N/A (CPU) | — | GPU dispatch not wired; `generate_shader()` → wgpu pipeline |
| `gpu/context.rs` | `TensorSession` (planned) | Fused pipeline | — | All ops use local WGSL; no barraCuda in fused path |
| `gpu/fused.rs` | — | Fused pipeline | — | Buffer layouts documented for `TensorSession` design. No Tier A rewire. |

---

## 8. WGSL Shader Inventory

| Shader | Tier | Pattern | Local Workarounds |
|--------|------|---------|-------------------|
| `hill_dose_response_f64.wgsl` | A | Element-wise | `power_f64()` via `exp(n*log(c))` f32 cast (~7 decimal digits) |
| `population_pk_f64.wgsl` | A | Embarrassingly parallel | Wang hash + xorshift32 (u32-only PRNG) |
| `diversity_f64.wgsl` | A | Workgroup reduction | `log_f64(x)` → `f64(log(f32(x)))` for driver portability |
| `michaelis_menten_batch_f64.wgsl` | B | Per-patient Euler ODE | Wang hash, xorshift32, u32_to_uniform |
| `scfa_batch_f64.wgsl` | B | Element-wise 3× MM | None — pure `vmax * s / (km + s)` |
| `beat_classify_batch_f64.wgsl` | B | Per-beat template correlation | `sqrt(f32(denom_sq))` for Pearson denominator |

---

## 9. metalForge & toadStool Integration

### 9.1 metalForge (forge/src/)

| File | Purpose |
|------|---------|
| `lib.rs` | `Substrate`, `Workload`, `Capabilities`, `PrecisionRouting`, `select_substrate` |
| `dispatch.rs` | `DispatchPlan`, `StageAssignment`, `plan_dispatch` — maps stages to NUCLEUS Nests |
| `transfer.rs` | `TransferPlan`, `TransferMethod` (PcieP2p, HostStaged, NetworkIpc) |
| `nucleus.rs` | `Nest`, `Node`, `Tower`, `NestId`, `PcieGeneration` |

### 9.2 Workload Thresholds (GPU)

| Workload | GPU Threshold | Substrate |
|----------|:-------------:|-----------|
| `PopulationPk` | 100 | Gpu |
| `DoseResponse` | 1000 | Gpu |
| `DiversityIndex` | 500 | Gpu |
| `MichaelisMentenBatch` | 100 | Gpu |
| `ScfaBatch` | 1000 | Gpu |
| `BeatClassifyBatch` | 500 | Gpu |
| `BiosignalDetect`, `BiosignalFusion` | — | Npu (preferred) |
| `EndocrinePk`, `Analytical` | — | Cpu |

### 9.3 toadStool (pipeline)

| File | Purpose |
|------|---------|
| `stage/mod.rs` | `Stage`, `StageOp`, `to_gpu_op()`, `execute()` |
| `stage/exec.rs` | CPU helpers: generate, transform, reduce, biosignal fusion, AUC, Bray-Curtis |
| `pipeline/mod.rs` | `Pipeline::execute_cpu()`, `execute_gpu()`, `execute_auto()` |
| `pipeline/gpu.rs` | `gpu_result_to_vec()` |
| `pipeline/workload.rs` | `stage_to_workload()` — maps Stage → metalForge Workload |

**StageOp → GpuOp mapping**: `PopulationPk`, `DiversityReduce`, `MichaelisMentenBatch`, `ScfaBatch`, `BeatClassifyBatch`, `ElementwiseTransform(Hill)` all map to `GpuOp` and dispatch via `GpuContext::execute_fused()`.

---

## 10. Tier C — New Shader Required (from EVOLUTION_MAP.md)

| Rust Module | Function | Shader Design | Blocking |
|-------------|----------|---------------|----------|
| `biosignal::pan_tompkins_qrs` | Streaming detect pipeline | Custom streaming shader or NPU | NPU dispatch path in toadStool |
| `biosignal::fuse_channels` | Multi-modal ECG+PPG+EDA | toadStool pipeline (3-stage) | Pipeline execution on GPU |
| `pkpd::pbpk_iv_simulate` | PBPK multi-compartment ODE | Euler/RK4 ODE shader | wetSpring ODE absorption status |
| `endocrine::hrv_trt_response` | Exponential saturation | `batched_elementwise_f64.wgsl` | Trivial once shader exists |

---

## 11. What Blocks GPU Promotion per Module

| Module | Blocks |
|--------|--------|
| **Tier A (Hill, PopPK, Diversity)** | Nothing — rewire ready. Fused path uses local WGSL by design until `TensorSession`. |
| **Tier B (MM)** | barraCuda has `OdeSystem` trait + `BatchedOdeRK4::generate_shader()`. Handwritten shader could be replaced by codegen. |
| **Tier B (SCFA)** | barraCuda has no direct equivalent. Element-wise MM kinetics is generic; absorption candidate for `barracuda::ops::bio::`. |
| **Tier B (BeatClassify)** | barraCuda has no biosignal primitive. Absorption candidate for `barracuda::ops::bio::` or `barracuda::ops::biosignal::`. |
| **biosignal FFT** | barraCuda has no `fft_radix2_f64`. BARRACUDA_REQUIREMENTS.md lists as "Write Phase" item. |
| **biosignal pan_tompkins** | Streaming design; NPU path preferred. |
| **biosignal fuse_channels** | toadStool pipeline stage; multi-stage GPU execution. |
| **PBPK** | ODE integration; `OdeSystem` trait path exists. |

---

## 12. Recommendations

### 12.1 Immediate (Fused Pipeline + Tier A)

1. **Wire barraCuda rewire into `GpuContext::execute_fused()`**  
   When `barracuda-ops` is enabled and an op is Tier A, use barraCuda device + ops. This requires either:
   - Mixed fused: barraCuda for Tier A, local for Tier B (two device contexts), or
   - barraCuda `TensorSession` for fused pipelines (single design).

2. **Wire barraCuda rewire into `GpuContext::execute()`**  
   Single-op path already delegates when `barracuda-ops` enabled. ✅ Done.

### 12.2 Tier B Absorption

3. **MichaelisMentenBatch**: Migrate to `BatchedOdeRK4::generate_shader()` for `MichaelisMentenOde`; remove handwritten shader. Or propose `MichaelisMentenBatch` to barraCuda bio module.

4. **ScfaBatch**: Propose `ScfaBatch` or `ElementwiseBatch` to barraCuda bio module.

5. **BeatClassifyBatch**: Propose `TemplateMatchBatch` or `BiosignalBatch` to barraCuda.

### 12.3 barraCuda Consumption

6. **`uncertainty::mean`**: ✅ Already delegates to `barracuda::stats::mean`.

7. **`pkpd/nlme/solver::cholesky_solve`**: Intentional local — 2×2/3×3 with fallback. No change needed.

8. **`TensorSession`**: When barraCuda provides `TensorSession`, migrate `GpuContext` fused pipeline to use it.

### 12.4 Dependency Hygiene

9. **Pin barraCuda version** in Cargo.toml (git rev or crate version) for reproducible CI.

10. **Unify `enable f64` handling**: `strip_f64_enable()` is local; BARRACUDA_REQUIREMENTS.md notes coralReef naga pass as future replacement.

### 12.5 FFT

11. **biosignal FFT**: Consider proposing `fft_radix2_f64` to barraCuda for GPU HRV power spectrum when barraCuda adds FFT primitive.

---

## 13. Summary Table

| Metric | Value |
|--------|-------|
| GPU ops | 6 |
| Tier A (rewire ready) | 3 |
| Tier B (absorption candidate) | 3 |
| barraCuda rewire path | `GpuContext::execute()` only; not in `execute_fused()` |
| TensorSession | Not used |
| ODE systems (OdeSystem trait) | 3 |
| Local stats (barraCuda used) | shannon, simpson, bray_curtis, hill, anderson, mean |
| Local reimplementations | FFT (biosignal), cholesky_solve (nlme, intentional), normalized_correlation (biosignal) |
| metalForge workloads | 8 GPU-related |
| barraCuda path dep | `../../barraCuda/crates/*` |
| barraCuda version (Cargo.lock) | 0.3.5 |
