# healthSpring V32 — toadStool / barraCuda Absorption Handoff

**Date:** March 16, 2026
**From:** healthSpring (V32)
**To:** barraCuda team, toadStool team, coralReef team
**License:** AGPL-3.0-or-later
**Covers:** healthSpring V22–V32 (full biomeOS niche lifecycle)

---

## Executive Summary

- healthSpring is a biomeOS niche with **618 tests**, **73 experiments**, **57+ JSON-RPC capabilities**, and **6 WGSL compute shaders**
- **3 Tier A ops rewired** to barraCuda upstream (Hill, PopPK, Diversity) — Write → Absorb → **Lean** complete
- **3 Tier B shaders** ready for barraCuda absorption (MM batch, SCFA batch, Beat classify)
- **3 health-domain ops** already contributed to `barracuda::health::*` (mm_auc, scr_rate, antibiotic_perturbation)
- Typed `compute.dispatch.*` IPC client ready for toadStool integration
- 6 GPU learnings documented (f64 portability, PRNG, fused pipelines, transcendental workarounds)

---

## Part 1: barraCuda Usage — Current State

### Modules Consumed

| barraCuda Module | healthSpring Usage | Files |
|-----------------|-------------------|-------|
| `barracuda::ops::hill_f64::HillFunctionF64` | Tier A GPU Hill dose-response | `gpu/barracuda_rewire.rs` |
| `barracuda::ops::population_pk_f64::PopulationPkF64` | Tier A GPU population PK Monte Carlo | `gpu/barracuda_rewire.rs` |
| `barracuda::ops::bio::diversity_fusion::DiversityFusionGpu` | Tier A GPU Shannon/Simpson diversity | `gpu/barracuda_rewire.rs` |
| `barracuda::device::WgpuDevice` | GPU device for all Tier A ops | `gpu/barracuda_rewire.rs`, `gpu/context.rs` |
| `barracuda::numerical::OdeSystem` | ODE trait for PK models (3 implementations) | `gpu/ode_systems.rs` |
| `barracuda::numerical::BatchedOdeRK4` | ODE → WGSL codegen | `gpu/ode_systems.rs` (tests) |
| `barracuda::stats::{hill, shannon_from_frequencies, simpson, chao1_classic, bray_curtis, mean}` | CPU math delegation | `microbiome/`, `pkpd/`, `uncertainty.rs` |
| `barracuda::health::pkpd::mm_auc` | Michaelis-Menten AUC | `pkpd/nonlinear.rs` |
| `barracuda::health::microbiome::antibiotic_perturbation` | Antibiotic perturbation abundances | `microbiome/clinical.rs` |
| `barracuda::health::biosignal::scr_rate` | Skin conductance response rate | `biosignal/stress.rs` |
| `barracuda::special::anderson_diagonalize` | Anderson localization eigensolver | `microbiome/anderson.rs` |
| `barracuda::rng::{LCG_MULTIPLIER, lcg_step, state_to_f64, uniform_f64_sequence}` | Deterministic PRNG | `rng.rs` |

### Version Pin

```toml
barracuda = { path = "../../barraCuda/crates/barracuda", default-features = false }
barracuda-core = { path = "../../barraCuda/crates/barracuda-core", default-features = false }
```

Validated against barraCuda rev `a60819c3` (2026-03-14). `PopulationPkF64::simulate()` takes `u32` params (aligned in V31).

---

## Part 2: WGSL Shaders — Absorption Candidates

### 6 Local Shaders in `ecoPrimal/shaders/health/`

| Shader | Lines | Tier | Status | barraCuda Target |
|--------|-------|------|--------|------------------|
| `hill_dose_response_f64.wgsl` | ~50 | A | **Rewired** — barraCuda `HillFunctionF64` used when `barracuda-ops` enabled | Already upstream |
| `population_pk_f64.wgsl` | ~80 | A | **Rewired** — barraCuda `PopulationPkF64` used | Already upstream |
| `diversity_f64.wgsl` | ~60 | A | **Rewired** — barraCuda `DiversityFusionGpu` used | Already upstream |
| `michaelis_menten_batch_f64.wgsl` | ~70 | B | **Absorption candidate** | `BatchedOdeRK4` + `MichaelisMentenOde` codegen |
| `scfa_batch_f64.wgsl` | ~80 | B | **Absorption candidate** | New `ScfaBatch` or `ElementwiseBatch` op |
| `beat_classify_batch_f64.wgsl` | ~90 | B | **Absorption candidate** | New `bio::signal::BeatClassifyBatch` op |

### Tier A (Lean Phase Complete)

The three original shaders are retained as fallback when `barracuda-ops` feature is disabled. When enabled, `gpu/barracuda_rewire.rs` delegates to upstream barraCuda ops. The fused pipeline (`GpuContext::execute_fused()`) still uses local WGSL — **toadStool action:** consider supporting mixed-source fused pipelines where some stages use barraCuda ops and others use local shaders.

### Tier B: Absorption Request

**barraCuda action: absorb these 3 health-domain shaders**

1. **`michaelis_menten_batch_f64.wgsl`** — Capacity-limited elimination PK. Can be generated from `OdeSystem` + `BatchedOdeRK4::generate_shader()`. healthSpring already has `MichaelisMentenOde` implementing the trait. Absorb as `barracuda::ops::health::MichaelisMentenBatchF64`.

2. **`scfa_batch_f64.wgsl`** — Short-chain fatty acid production kinetics (Michaelis-Menten: acetate, propionate, butyrate). Embarrassingly parallel — each workgroup processes one substrate. Absorb as `barracuda::ops::bio::ScfaBatchF64` or generalize to `ElementwiseBatchF64`.

3. **`beat_classify_batch_f64.wgsl`** — Template-correlation arrhythmia beat classification (Normal, PVC, PAC). Signal processing pattern: sliding window correlation with 3 templates. Absorb as `barracuda::ops::bio::signal::BeatClassifyBatch`.

### Tier C: New Shaders Needed (Future)

| Module | Function | Blocker | Priority |
|--------|----------|---------|----------|
| `biosignal::pan_tompkins_qrs` | Streaming QRS detection | NPU dispatch path | P2 |
| `biosignal::fuse_channels` | Multi-modal ECG+PPG+EDA | Multi-stage pipeline | P3 |
| `pkpd::pbpk_iv_simulate` | PBPK multi-compartment ODE | ODE batch codegen | P2 |
| `endocrine::hrv_trt_response` | Exponential saturation | Elementwise batch | P3 |
| `pkpd::foce_estimate` | Per-subject gradient batch | Parallel prefix | P2 |
| `pkpd::vpc_simulate` | Monte Carlo population sim | Embarrassingly parallel | P1 |
| `pkpd::saem_estimate` | SAEM E-step batched MC | Same as vpc_simulate | P2 |

---

## Part 3: healthSpring Contributions to barraCuda

### Already in `barracuda::health::*`

| Function | barraCuda Location | healthSpring Source |
|----------|-------------------|-------------------|
| `mm_auc()` | `barracuda::health::pkpd::mm_auc` | `pkpd/nonlinear.rs` |
| `scr_rate()` | `barracuda::health::biosignal::scr_rate` | `biosignal/stress.rs` |
| `antibiotic_perturbation_abundances()` | `barracuda::health::microbiome::antibiotic_perturbation` | `microbiome/clinical.rs` |

### Ready for Upstream Contribution

| Function | Domain | GPU Candidate | Priority |
|----------|--------|--------------|----------|
| `auc_trapezoidal()` | PK/PD | Parallel prefix sum | P1 |
| `pbpk_iv_simulate()` | PK/PD | ODE batch | P1 |
| `chao1_classic()` | Microbiome | Already in `barracuda::stats` | Done |
| `bray_curtis()` | Microbiome | Pairwise dissimilarity kernel | P2 |
| `pan_tompkins_qrs()` | Biosignal | NPU-first design | P2 |
| `fuse_channels()` | Biosignal | Multi-modal | P3 |
| `PatientTrtProfile` | Clinical | N/A (application layer) | — |

---

## Part 4: GPU Learnings for barraCuda / coralReef

These learnings were earned during healthSpring's GPU evolution and are relevant for upstream:

### 1. f64 Support is Device-Dependent

`enable f64;` is not needed in WGSL source — wgpu/naga handles f64 via device feature negotiation. Strip this directive from all shaders. If the device doesn't support f64, fall back to f32 with documented precision bounds.

### 2. `pow(f64, f64)` Portability

`pow(f64, f64)` is unsupported on NVIDIA via NVVM backend (coralReef transcendental poisoning — hotSpring Exp 053). Workaround: `exp(n * log(c))` via f32 intermediate. **coralReef action:** document supported transcendental functions per backend.

### 3. u64 PRNG Not Portable

u64 operations are not universally available in WGSL compute. All healthSpring shaders use u32-only xorshift32 + Wang hash for deterministic PRNG. **barraCuda action:** standardize on u32 PRNG for portable shaders.

### 4. Fused Pipeline Overhead

Single GPU encoder submission (fused pipeline) cuts ~30× overhead at small input sizes compared to per-op submission. At 10M+ elements, memory bandwidth dominates regardless. The `GpuContext::execute_fused()` pattern submits all stages in a single encoder pass.

### 5. IPC Response Buffer

Capability responses from `biomeOS` can exceed 16KB. healthSpring uses 64KB (`IPC_RESPONSE_BUF = 65_536`). **toadStool action:** ensure response buffers are adequate for large capability lists.

### 6. Non-Async GPU Ops

barraCuda GPU ops are synchronous (device.poll blocks). healthSpring V31 stripped `async` from Tier A GPU functions — false async overhead eliminated. The toadStool pipeline dispatch should not assume GPU ops are async.

---

## Part 5: toadStool Integration Status

### Typed Compute Dispatch Client

`ecoPrimal/src/ipc/compute_dispatch.rs` provides typed wrappers for the toadStool `compute.dispatch.*` protocol:

```rust
pub fn submit_dispatch(socket: &Path, plan: &DispatchPlan) -> Result<Value, IpcError>;
pub fn query_dispatch_result(socket: &Path, job_id: &str) -> Result<Value, IpcError>;
pub fn query_dispatch_capabilities(socket: &Path) -> Result<Value, IpcError>;
```

Discovery is capability-based — no hardcoded primal names. `discover_compute_primal()` probes the socket directory.

### Pipeline Integration

| Component | Status | File |
|-----------|--------|------|
| `Pipeline::execute_cpu()` | Live | `toadstool/src/pipeline.rs` |
| `Pipeline::execute_gpu()` | Live | `toadstool/src/pipeline.rs` |
| `Pipeline::execute_streaming()` | Live | `toadstool/src/pipeline.rs` |
| `Pipeline::execute_auto()` | Live | `toadstool/src/pipeline.rs` |
| `StageOp::to_gpu_op()` | Live | `toadstool/src/stage.rs` |
| metalForge NUCLEUS routing | Live | `metalForge/forge/src/` |

### Enriched Capability Response

V31 added `operation_dependencies` and `cost_estimates` to `capability.list` response:

```json
{
  "operation_dependencies": {
    "science.diagnostic.assess_patient": [
      "science.pkpd.one_compartment_pk",
      "science.microbiome.shannon_index",
      "science.biosignal.hrv_metrics",
      "science.endocrine.testosterone_pk"
    ]
  },
  "cost_estimates": {
    "science.pkpd.hill_dose_response": {"cpu_ms": 0.01, "gpu_eligible": true},
    "science.diagnostic.population_montecarlo": {"cpu_ms": 100.0, "gpu_eligible": true}
  }
}
```

**biomeOS action:** Pathway Learner can use `operation_dependencies` for DAG planning and `cost_estimates` for substrate selection.

---

## Part 6: V32 Evolution — What Changed

| Feature | Impact for barraCuda/toadStool |
|---------|-------------------------------|
| Structured `tracing` | Log output is now structured JSON (via `RUST_LOG`). Other primals can parse healthSpring logs. |
| `health.liveness` probe | Lightweight process-alive check — coralReef/biomeOS can probe without blocking. |
| `health.readiness` probe | Reports subsystem availability (trio, compute, data) — enables smart routing. |
| Resilient trio IPC | Circuit breaker prevents cascading failures when provenance trio is down. 5s cooldown, auto-recovery. |
| `IpcError` type | Structured error enum with `RpcError{code, message}` and `Timeout` variants. All springs converging on this. |

---

## Part 7: Action Items

### barraCuda Team

1. **P1: Absorb MM batch shader** — `michaelis_menten_batch_f64.wgsl` → `barracuda::ops::health::MichaelisMentenBatchF64`
2. **P1: Absorb SCFA batch shader** — `scfa_batch_f64.wgsl` → `barracuda::ops::bio::ScfaBatchF64`
3. **P1: Absorb beat classify shader** — `beat_classify_batch_f64.wgsl` → `barracuda::ops::bio::signal::BeatClassifyBatch`
4. **P2: `auc_trapezoidal` GPU** — Parallel prefix sum for AUC computation at population scale
5. **P2: `vpc_simulate` GPU** — Embarrassingly parallel Monte Carlo for VPC
6. **P2: Standardize u32 PRNG** — All WGSL shaders should use u32-only Wang hash / xorshift32
7. **P3: `bray_curtis` pairwise GPU** — Pairwise dissimilarity matrix for microbiome analytics

### toadStool Team

1. **P1: Mixed-source fused pipeline** — Support fused encoder submissions mixing barraCuda ops and local WGSL
2. **P1: NPU dispatch path** — Pan-Tompkins streaming QRS detection is the target use case
3. **P2: `DispatchPlan` planner** — Route stages to CPU/GPU/NPU based on `cost_estimates` from capability response
4. **P2: Response buffer sizing** — Ensure ≥64KB for capability-rich primals
5. **P3: Non-async GPU awareness** — barraCuda ops block on device.poll; pipeline should not assume async

### coralReef Team

1. **P1: Document transcendental support matrix** — Which functions (pow, exp, log, sin, cos) are supported per backend (naga/NVVM/SPIRV)
2. **P2: `health.liveness`/`health.readiness` probe standard** — healthSpring implements per Iter 51; confirm alignment

---

## Appendix: Write → Absorb → Lean Cycle Status

```
Write (local WGSL)  →  Validate  →  Handoff  →  barraCuda Absorbs  →  Lean on upstream
     ↑                                                                        │
     └────────────── local shader retained as fallback ───────────────────────┘
```

| Phase | Shader | Status |
|-------|--------|--------|
| **Lean** | hill_dose_response_f64 | barraCuda `HillFunctionF64` — upstream used |
| **Lean** | population_pk_f64 | barraCuda `PopulationPkF64` — upstream used |
| **Lean** | diversity_f64 | barraCuda `DiversityFusionGpu` — upstream used |
| **Handoff** | michaelis_menten_batch_f64 | Ready for barraCuda absorption |
| **Handoff** | scfa_batch_f64 | Ready for barraCuda absorption |
| **Handoff** | beat_classify_batch_f64 | Ready for barraCuda absorption |
| **Write** | (Tier C candidates) | Local development pending |
