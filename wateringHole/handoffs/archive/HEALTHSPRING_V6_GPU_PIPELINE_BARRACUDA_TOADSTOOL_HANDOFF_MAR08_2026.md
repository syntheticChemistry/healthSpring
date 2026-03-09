# healthSpring V6 → barraCuda / toadStool GPU Pipeline Handoff

**SPDX-License-Identifier:** AGPL-3.0-or-later
**Date:** March 8, 2026
**From:** healthSpring V6
**To:** barraCuda / toadStool / coralReef teams
**License:** AGPL-3.0-or-later
**Status:** Complete — Tier 2 GPU live, fused pipeline validated, scaling benchmarked

---

## Executive Summary

- **3 WGSL shaders** written, compiled, and validated for Hill dose-response, population PK Monte Carlo, and diversity indices — all f64 precision
- **`GpuContext`** persistent device/queue eliminates per-dispatch initialization (31.7x overhead reduction at small sizes)
- **`execute_fused()`** unidirectional pipeline: all ops in one command encoder, single submit, single readback — no CPU roundtrips between stages
- **toadStool `Pipeline::execute_gpu()`** dispatches stages via `GpuContext`, with automatic CPU fallback for non-GPU stages
- **GPU scaling validated**: crossover at 100K elements (Hill), peak 207 M elements/s, tested to 10M
- **17/17 GPU parity checks**, 11/11 fused pipeline checks, full scaling sweep — zero regressions on 200 unit tests + 346 binary checks

---

## 1. What healthSpring Built (For Absorption)

### 1.1 WGSL Shaders

| File | Op | Entry | Workgroup | Notes |
|------|----|-------|:---------:|-------|
| `barracuda/shaders/health/hill_dose_response_f64.wgsl` | E(c) = Emax·c^n / (c^n + EC50^n) | `main` | 256 | `power_f64` via f32 exp/log |
| `barracuda/shaders/health/population_pk_f64.wgsl` | AUC = F·Dose / CL(random) | `main` | 256 | u32 xorshift32 + Wang hash |
| `barracuda/shaders/health/diversity_f64.wgsl` | Shannon + Simpson | `main` | 256 | Workgroup-level reduction |

### 1.2 Rust GPU Infrastructure (`barracuda/src/gpu.rs`)

| Component | Description |
|-----------|-------------|
| `GpuOp` / `GpuResult` | Enum-dispatched operations with typed results |
| `execute_cpu()` | CPU reference fallback (always available) |
| `execute_gpu()` | Individual dispatch (feature-gated `gpu`) |
| `GpuContext` | Persistent wgpu device/queue, `SHADER_F64` feature |
| `GpuContext::execute()` | Single op on cached device |
| `GpuContext::execute_fused()` | Multiple ops, one encoder, one submit |
| `strip_f64_enable()` | Removes `enable f64;` from WGSL before compilation |
| `dispatch_and_readback()` | Generic shader compile → dispatch → staging readback |
| `HillParams` / `PkParams` / `DivParams` | `#[repr(C)] bytemuck::Pod` uniform buffer structs |

### 1.3 toadStool GPU Dispatch (`toadstool/src/`)

| Component | Description |
|-----------|-------------|
| `Stage::to_gpu_op()` | Converts toadStool stage to `GpuOp` (Hill currently) |
| `Pipeline::execute_gpu()` | Batches GPU-mappable stages, fused dispatch, CPU fallback |
| `Pipeline::execute_auto()` | metalForge routing → GPU or CPU per stage |
| `gpu_result_to_vec()` | Flattens `GpuResult` for pipeline data flow |
| `stage_to_workload()` | Maps stage to metalForge `Workload` for routing |

### 1.4 Experiments

| Exp | Purpose | Key Metric |
|-----|---------|-----------|
| 053 | GPU parity (shader output vs CPU) | 17/17 checks, max_rel < 1e-4 |
| 054 | Fused pipeline + toadStool dispatch | 11/11 checks, fused 31.7x faster |
| 055 | Scaling 1K→10M elements | Hill crossover 100K, PK crossover 5M |

---

## 2. Critical Learnings for barraCuda/toadStool Evolution

### 2.1 WGSL f64 Portability

**`enable f64;` must be stripped.** wgpu's naga parser does not accept the directive. f64 support is enabled at device level via `wgpu::Features::SHADER_F64`. healthSpring uses `strip_f64_enable()` to remove the line before `create_shader_module()`.

**Action for barraCuda**: Consider a `preprocess_wgsl()` step in `compile_shader_f64` that handles this transparently for all springs.

### 2.2 Transcendental Functions

**`pow(f64, f64)` crashes NVIDIA via NVVM.** The naga → NVVM path does not support f64 transcendentals on many GPU drivers. healthSpring works around this by casting through f32:

```wgsl
fn power_f64(base: f64, exponent: f64) -> f64 {
    if base <= 0.0 { return 0.0; }
    let log_base = f64(log(f32(base)));
    return f64(exp(f32(exponent * log_base)));
}
```

This gives ~7 decimal digits — sufficient for dose-response and diversity, but **not for high-precision physics**. hotSpring should be aware.

**Action for barraCuda**: Provide a `power_f64` / `log_f64` library in WGSL that all springs can include, with a flag for precision tier (f32-intermediate vs full f64 when driver supports).

### 2.3 GPU PRNG

**u64 not available without `SHADER_INT64`.** healthSpring's CPU uses an LCG with u64 state. The GPU shader uses u32-only xorshift32 + Wang hash for seed mixing. This produces statistically different distributions (different PRNG families), so GPU/CPU parity is validated via statistical properties (mean, range, std dev) rather than bit-exact matching.

**Action for toadStool**: Document that Monte Carlo GPU/CPU parity is statistical, not bitwise, when PRNGs differ.

### 2.4 wgpu API Changes (v28)

- `wgpu::Maintain::Wait` → `wgpu::PollType::Wait { submission_index: None, timeout: None }`
- `DeviceDescriptor` requires `experimental_features` and `trace` fields
- `PipelineCompilationOptions::default()` now required

**Action for barraCuda**: Pin wgpu version or abstract these into `WgpuDevice` helper methods.

### 2.5 Workgroup Dispatch Limits

Max 65,535 workgroups per dimension. With `@workgroup_size(64)`, 5M elements exceeds the limit. healthSpring bumped to `@workgroup_size(256)` (39K workgroups at 10M). For larger sizes, 2D dispatch or multi-element-per-thread is needed.

**Action for barraCuda**: `PipelineBuilder` should auto-select workgroup dispatch strategy based on element count.

### 2.6 Fused Pipeline Architecture

The unidirectional pattern (upload → N compute passes → readback) provides:
- **31.7x** overhead reduction at small sizes (eliminating 2 extra device roundtrips)
- **No advantage** at large sizes where compute dominates (GPU can pipeline individual submits)

The optimal strategy is **fused for small/medium, pipelined individual for large**. `GpuContext` supports both patterns.

---

## 3. Absorption Candidates

### 3.1 For barraCuda (math library)

| Candidate | Type | Complexity | Notes |
|-----------|------|:----------:|-------|
| `power_f64` WGSL function | Utility | Low | All springs need portable f64 pow |
| `strip_f64_enable()` | Preprocessor | Low | Should be in wgpu device layer |
| `GpuContext` pattern | Architecture | Medium | Cached device + fused dispatch |
| `dispatch_and_readback()` | Helper | Medium | Generic compute dispatch pattern |
| `WG_SIZE` constant | Convention | Low | 256 standard for health/bio workloads |

### 3.2 For toadStool (compute dispatch)

| Candidate | Type | Complexity | Notes |
|-----------|------|:----------:|-------|
| `Pipeline::execute_gpu()` | Pipeline | Medium | GPU-aware stage dispatch |
| `Stage::to_gpu_op()` | Bridge | Low | Stage → GPU op conversion |
| `execute_auto()` | Routing | Medium | metalForge-driven substrate selection |
| GPU fallback pattern | Resilience | Low | CPU fallback on GPU failure |

### 3.3 For coralReef (shader compilation)

| Candidate | Type | Notes |
|-----------|------|-------|
| f32 transcendental workaround | Portability | NVIDIA NVVM f64 pow/exp/log limitations |
| u32-only PRNG pattern | Portability | No SHADER_INT64 dependency |
| workgroup_size(256) convention | Performance | Avoids 65K dispatch limit to 10M+ |

---

## 4. Files Changed (V5 → V6)

| File | Change |
|------|--------|
| `barracuda/shaders/health/hill_dose_response_f64.wgsl` | **New** — Hill GPU shader |
| `barracuda/shaders/health/population_pk_f64.wgsl` | **New** — PopPK GPU shader |
| `barracuda/shaders/health/diversity_f64.wgsl` | **New** — Diversity GPU shader |
| `barracuda/src/gpu.rs` | `GpuContext`, `execute_fused()`, `WG_SIZE` const |
| `barracuda/Cargo.toml` | `bytemuck` dep, `gpu` feature |
| `toadstool/src/pipeline.rs` | `execute_gpu()`, `execute_auto()` |
| `toadstool/src/stage.rs` | `to_gpu_op()`, `GpuOp` import |
| `toadstool/Cargo.toml` | `gpu` feature, `tokio` dep |
| `experiments/exp053_gpu_parity/` | **New** — GPU parity validation |
| `experiments/exp054_gpu_pipeline/` | **New** — Fused pipeline + toadStool |
| `experiments/exp055_gpu_scaling/` | **New** — 1K→10M scaling benchmark |
| `Cargo.toml` | Workspace: +exp053, +exp054, +exp055 |

---

## 5. Status and Verification

```
cargo test --workspace           # 200 tests pass
cargo clippy --workspace -- -D warnings  # zero warnings
cargo run --release --bin exp053_gpu_parity   # 17/17 pass
cargo run --release --bin exp054_gpu_pipeline # 11/11 pass
cargo run --release --bin exp055_gpu_scaling  # Full scaling sweep
```

---

This handoff is unidirectional: healthSpring → barraCuda / toadStool / coralReef. No response expected.
