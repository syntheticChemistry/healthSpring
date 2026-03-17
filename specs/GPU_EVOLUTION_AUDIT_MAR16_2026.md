# healthSpring GPU Evolution Readiness & barraCuda Dependency Audit

**Date**: March 16, 2026  
**Scope**: ecoPrimal GPU integration, WGSL shaders, barraCuda dependency health, metalForge Tier 3 dispatch

---

## 1. GPU Integration Status (ecoPrimal/src/gpu/)

### 1.1 GPU Operations Implemented

| GpuOp | WGSL Shader | CPU Fallback | barraCuda Rewire |
|-------|-------------|--------------|------------------|
| `HillSweep` | `hill_dose_response_f64.wgsl` | `pkpd::hill_dose_response` | `HillFunctionF64::dose_response()` |
| `PopulationPkBatch` | `population_pk_f64.wgsl` | LCG + AUC loop | `PopulationPkF64::simulate()` |
| `DiversityBatch` | `diversity_f64.wgsl` | `microbiome::shannon_index`, `simpson_index` | `DiversityFusionGpu::compute()` |
| `MichaelisMentenBatch` | `michaelis_menten_batch_f64.wgsl` | `pkpd::mm_pk_simulate` + `mm_auc` | — |
| `ScfaBatch` | `scfa_batch_f64.wgsl` | `microbiome::scfa_production` | — |
| `BeatClassifyBatch` | `beat_classify_batch_f64.wgsl` | `biosignal::classification::classify_beat` | — |

### 1.2 barraCuda Dispatch vs Local WGSL

**Tier A (barraCuda rewire available when `barracuda-ops` feature enabled):**
- `HillSweep` → `barracuda::ops::HillFunctionF64`
- `PopulationPkBatch` → `barracuda::ops::PopulationPkF64`
- `DiversityBatch` → `barracuda::ops::bio::DiversityFusionGpu`

**Critical gap**: The barraCuda rewire path is **only** used in `dispatch::execute_gpu()` (standalone op execution). It is **not** used in:
- `GpuContext::execute()` — always uses local WGSL dispatch
- `GpuContext::execute_fused()` — always uses local WGSL via `fused::prepare_*`

So the fused pipeline (single encoder, upload once → N passes → readback once) **never** benefits from barraCuda Tier A ops. All 6 ops use local shaders in the fused path.

### 1.3 TensorSession Usage

**Not used.** `GpuContext` implements its own fused pipeline:
- `prepare_all_ops()` → `fused::prepare_hill`, `prepare_pop_pk`, etc.
- `submit_compute_passes()` → single encoder, N compute passes
- `readback_all()` → per-op staging buffers, tokio oneshot map_async

The mod.rs comments state: *"GpuContext fused pipeline → barracuda::session::TensorSession"* as a **pending architectural** item. healthSpring does not yet use `TensorSession` for fused pipelines.

---

## 2. WGSL Shader Analysis (ecoPrimal/shaders/health/)

### 2.1 hill_dose_response_f64.wgsl

| Attribute | Value |
|-----------|-------|
| **Tier** | **A** — direct rewire to existing barraCuda op |
| **barraCuda primitive** | `HillFunctionF64::dose_response()` |
| **Pipeline stage** | DoseResponse (Track 1 PK/PD) |
| **Pattern** | Element-wise: E(c) = Emax × c^n / (c^n + EC50^n) |
| **Local workarounds** | `power_f64()` via `exp(n*log(c))` f32 cast (~7 decimal digits) |
| **Blocks promotion** | None — barraCuda already has canonical op |

### 2.2 population_pk_f64.wgsl

| Attribute | Value |
|-----------|-------|
| **Tier** | **A** — direct rewire to existing barraCuda op |
| **barraCuda primitive** | `PopulationPkF64::simulate()` |
| **Pipeline stage** | PopulationPk (Track 1 PK/PD) |
| **Pattern** | Embarrassingly parallel Monte Carlo: AUC = F×Dose/CL per patient |
| **Local workarounds** | Wang hash + xorshift32 (u32-only PRNG), no SHADER_INT64 |
| **Blocks promotion** | None — barraCuda already has canonical op |

### 2.3 diversity_f64.wgsl

| Attribute | Value |
|-----------|-------|
| **Tier** | **A** — direct rewire to existing barraCuda op |
| **barraCuda primitive** | `DiversityFusionGpu::compute()` |
| **Pipeline stage** | DiversityIndex (Track 2 Microbiome) |
| **Pattern** | Workgroup reduction: Shannon + Simpson per community |
| **Local workarounds** | `log_f64(x)` → `f64(log(f32(x)))` for driver portability |
| **Blocks promotion** | None — barraCuda already has canonical op |

### 2.4 michaelis_menten_batch_f64.wgsl

| Attribute | Value |
|-----------|-------|
| **Tier** | **B** — adapt existing shader / barraCuda design pending |
| **barraCuda primitive** | `BatchedOdeRK4::generate_shader()` + `MichaelisMentenOde` (OdeSystem trait) — **codegen path exists**; handwritten shader is alternative |
| **Pipeline stage** | MichaelisMentenBatch (Track 1 PK/PD) |
| **Pattern** | Per-patient Euler ODE + PRNG (Vmax variation) + trapezoidal AUC |
| **Local workarounds** | `wang_hash`, `xorshift32`, `u32_to_uniform` |
| **Blocks promotion** | barraCuda has `OdeSystem` trait impl in healthSpring; `BatchedOdeRK4::generate_shader()` produces valid WGSL. Handwritten shader could be replaced by codegen path. barraCuda bio module absorption candidate. |

### 2.5 scfa_batch_f64.wgsl

| Attribute | Value |
|-----------|-------|
| **Tier** | **B** — adapt existing shader / barraCuda design pending |
| **barraCuda primitive** | `BatchedOdeRK4` N/A — element-wise Michaelis-Menten ×3, no ODE. Would need `ElementwiseBatch` or `BioBatch` primitive |
| **Pipeline stage** | ScfaBatch (Track 2 Microbiome) |
| **Pattern** | Element-wise: 3× Michaelis-Menten kinetics per fiber input |
| **Local workarounds** | None — pure `vmax * s / (km + s)` |
| **Blocks promotion** | barraCuda has no direct equivalent. Element-wise MM kinetics is generic; could be absorbed into `barracuda::ops::bio::` as `ScfaBatch` or similar. |

### 2.6 beat_classify_batch_f64.wgsl

| Attribute | Value |
|-----------|-------|
| **Tier** | **B** — adapt existing shader / barraCuda design pending |
| **barraCuda primitive** | None — template correlation + argmax. Would need `TemplateMatchBatch` or `BiosignalBatch` |
| **Pipeline stage** | BeatClassifyBatch (Track 3 Biosignal) |
| **Pattern** | Per-beat: mean, variance, cross-correlation vs N templates, argmax |
| **Local workarounds** | `sqrt(f32(denom_sq))` for Pearson correlation denominator |
| **Blocks promotion** | barraCuda has no biosignal primitive. Domain-specific. Absorption candidate for `barracuda::ops::bio::` or `barracuda::ops::biosignal::`. |

---

## 3. barraCuda Dependency (ecoPrimal/Cargo.toml)

| Dependency | Version | Features |
|------------|---------|----------|
| `barracuda-core` | path `../../barraCuda/crates/barracuda-core` | `default-features = false` |
| `barracuda` | path `../../barraCuda/crates/barracuda` | `default-features = false` |

**Validation:** *"Validated against barraCuda rev a60819c3 (2026-03-14)"*

**Features used:**
- `gpu` = wgpu, tokio, bytemuck
- `barracuda-ops` = gpu (enables Tier A rewire in `execute_gpu` only)

**Cargo.toml does not specify barraCuda version** — uses path dependency. BARRACUDA_REQUIREMENTS.md states v0.3.5 pinned; Cargo.toml comment says rev a60819c3.

---

## 4. Local Reimplementations vs barraCuda Primitives

### 4.1 Already Using barraCuda

| Module | Usage |
|--------|-------|
| `microbiome` | `barracuda::stats::shannon_from_frequencies`, `simpson`, `chao1_classic`, `bray_curtis` |
| `microbiome/anderson` | `barracuda::special::anderson_diagonalize` |
| `pkpd/dose_response` | `barracuda::stats::hill` |
| `rng` | `barracuda::rng::{lcg_step, LCG_MULTIPLIER, state_to_f64, uniform_f64_sequence}` |
| `gpu/ode_systems` | `barracuda::numerical::OdeSystem`, `BatchedOdeRK4` |

### 4.2 Local Reimplementations (No barraCuda Equivalent Used)

| Location | Function | barraCuda Equivalent? | Notes |
|----------|----------|------------------------|-------|
| `uncertainty.rs` | `mean(data)` | `barracuda` reduction: sum/mean | Private helper for bootstrap; barraCuda has `mean` in reduction |
| `uncertainty.rs` | `mbe`, `bootstrap_mean`, `jackknife_mean_variance`, etc. | `barracuda::stats` has histogram, percentile, median | UQ patterns (bootstrap, jackknife) not in barraCuda stats |
| `biosignal/classification.rs` | `normalized_correlation(a, b)` | None | Pearson correlation; barraCuda has `dot` but not full correlation |
| `pkpd/nlme/solver.rs` | `cholesky_solve` | `barracuda::linalg::solve_triangular` | Small symmetric positive-definite; barraCuda linalg could replace |
| `rng.rs` | `box_muller`, `normal_sample` | None | Domain-specific; uses barraCuda LCG for uniforms |

### 4.3 ODE Solvers

| Usage | barraCuda Path |
|-------|----------------|
| `gpu/ode_systems.rs` | Implements `OdeSystem` for 3 PK models; uses `BatchedOdeRK4::integrate_cpu` and `generate_shader()` |
| `michaelis_menten_batch_f64.wgsl` | Handwritten Euler ODE; does **not** use `BatchedOdeRK4::generate_shader()` |

**V27 handoff**: ODE→WGSL codegen is ready via `OdeSystem` trait. The handwritten MM batch shader could be migrated to `BatchedOdeRK4::generate_shader()` for consistency.

---

## 5. metalForge Tier 3 Dispatch Layer (metalForge/forge/src/)

### 5.1 Architecture

| File | Purpose |
|------|---------|
| `lib.rs` | `Substrate`, `Workload`, `Capabilities`, `PrecisionRouting`, `select_substrate` |
| `dispatch.rs` | `DispatchPlan`, `StageAssignment`, `plan_dispatch` — maps stages to NUCLEUS Nests |
| `transfer.rs` | `TransferPlan`, `TransferMethod` (PcieP2p, HostStaged, NetworkIpc) |
| `nucleus.rs` | `Nest`, `Node`, `Tower`, `NestId`, `PcieGeneration` |

### 5.2 Workloads Mapped to GPU

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

### 5.3 Absorption Status (from lib.rs comments)

- `Substrate` + `Workload` → barraCuda workload classification
- `select_substrate()` → toadStool dispatcher
- `Capabilities::discover()` → barraCuda hardware probe
- `DispatchPlan` + NUCLEUS topology → biomeOS graph planner

**Pending:** `DispatchPlan` → toadStool planner; `StageAssignment` with `NestId` → biomeOS graph node annotations.

---

## 6. Specs Alignment

### BARRACUDA_REQUIREMENTS.md

- **Tier A rewire ready**: Hill, PopPK, Diversity — **confirmed** in code
- **Tier B absorption candidates**: MM batch, SCFA batch, Beat classify — **confirmed**
- **V27 ODE systems**: 3 `OdeSystem` impls — **confirmed** in `gpu/ode_systems.rs`
- **Write phase still needed**: `anderson_xi_1d_gut`, `bandpass_iir_gpu`, `qrs_detect_gpu`, `fft_radix2_f64`

### COMPUTE_DATA_PROFILE.md

- Shader inventory matches 6 WGSL files
- metalForge workload routing thresholds align with `select_substrate` logic
- GPU thresholds: PopulationPk 5M, DoseResponse 100K, DiversityIndex 10K, etc.

---

## 7. Active Handoff (V31)

**File**: `wateringHole/handoffs/HEALTHSPRING_V31_DEEP_DEBT_MODERN_RUST_HANDOFF_MAR16_2026.md`

**Key points:**
- Dual-format capability parsing, zero-panic validation, compute_dispatch client
- barracuda::health delegation, deny.toml
- All 6 WGSL shaders documented with literature provenance for magic numbers
- 616 tests, zero clippy, zero unsafe
- Next targets: GPU dispatch for ODE systems, Tier B shader absorption, HMM/ESN from neuralSpring

---

## 8. Complete Mapping: Rust Module → barraCuda op / WGSL → Pipeline Stage → Tier → Blocks

| Rust Module | barraCuda Op / WGSL Shader | Pipeline Stage | Tier | What Blocks Promotion? |
|-------------|----------------------------|----------------|------|------------------------|
| `gpu/dispatch/hill.rs` | `HillFunctionF64` / `hill_dose_response_f64.wgsl` | DoseResponse | A | Nothing — rewire ready; fused path not wired |
| `gpu/dispatch/pop_pk.rs` | `PopulationPkF64` / `population_pk_f64.wgsl` | PopulationPk | A | Nothing — rewire ready; fused path not wired |
| `gpu/dispatch/diversity.rs` | `DiversityFusionGpu` / `diversity_f64.wgsl` | DiversityIndex | A | Nothing — rewire ready; fused path not wired |
| `gpu/dispatch/batch_ops.rs` (MM) | — / `michaelis_menten_batch_f64.wgsl` | MichaelisMentenBatch | B | barraCuda bio module design; could use `BatchedOdeRK4` codegen |
| `gpu/dispatch/batch_ops.rs` (SCFA) | — / `scfa_batch_f64.wgsl` | ScfaBatch | B | barraCuda has no direct equivalent; bio module candidate |
| `gpu/dispatch/batch_ops.rs` (Beat) | — / `beat_classify_batch_f64.wgsl` | BeatClassifyBatch | B | barraCuda has no biosignal primitive |
| `gpu/ode_systems.rs` | `BatchedOdeRK4` (OdeSystem) | N/A (CPU) | — | GPU dispatch not wired; `generate_shader()` → wgpu pipeline |
| `gpu/context.rs` | `TensorSession` (planned) | Fused pipeline | — | All ops use local WGSL; no barraCuda in fused path |
| `gpu/fused.rs` | — | Fused pipeline | — | Buffer layouts documented for `TensorSession` design. No Tier A rewire. |

---

## 9. Recommendations

### 9.1 Immediate (Fused Pipeline + Tier A)

1. **Wire barraCuda rewire into `GpuContext::execute_fused()`**  
   When `barracuda-ops` is enabled and an op is Tier A, use barraCuda device + ops instead of local shaders. This requires either:
   - Mixed fused: barraCuda for Tier A, local for Tier B (two device contexts), or
   - barraCuda `TensorSession` for fused pipelines (single design).

2. **Wire barraCuda rewire into `GpuContext::execute()`**  
   Single-op path also uses local WGSL; should mirror `execute_gpu()` behavior when `barracuda-ops` is enabled.

### 9.2 Tier B Absorption

3. **MichaelisMentenBatch**: Migrate to `BatchedOdeRK4::generate_shader()` for `MichaelisMentenOde`; remove handwritten shader. Or propose `MichaelisMentenBatch` to barraCuda bio module.

4. **ScfaBatch**: Propose `ScfaBatch` or `ElementwiseBatch` to barraCuda bio module.

5. **BeatClassifyBatch**: Propose `TemplateMatchBatch` or `BiosignalBatch` to barraCuda.

### 9.3 barraCuda Consumption

6. **`uncertainty::mean`**: Consider `barracuda::reduction::mean` or equivalent for CPU path if available.

7. **`pkpd/nlme/solver::cholesky_solve`**: Evaluate `barracuda::linalg::solve_triangular` for small systems.

8. **`TensorSession`**: When barraCuda provides `TensorSession`, migrate `GpuContext` fused pipeline to use it.

### 9.4 Dependency Hygiene

9. **Pin barraCuda version** in Cargo.toml (git rev or crate version) for reproducible CI.

10. **Unify `enable f64` handling**: `strip_f64_enable()` is local; BARRACUDA_REQUIREMENTS.md notes coralReef naga pass as future replacement.

---

## 10. Summary Table

| Metric | Value |
|--------|-------|
| GPU ops | 6 |
| Tier A (rewire ready) | 3 |
| Tier B (absorption candidate) | 3 |
| barraCuda rewire path | `execute_gpu()` only; not in `GpuContext` |
| TensorSession | Not used |
| ODE systems (OdeSystem trait) | 3 |
| Local stats (barraCuda used) | shannon, simpson, bray_curtis, hill, anderson |
| Local reimplementations | mean (uncertainty), cholesky_solve (nlme), normalized_correlation (biosignal) |
| metalForge workloads | 8 GPU-related |
| barraCuda path dep | `../../barraCuda/crates/*` |
