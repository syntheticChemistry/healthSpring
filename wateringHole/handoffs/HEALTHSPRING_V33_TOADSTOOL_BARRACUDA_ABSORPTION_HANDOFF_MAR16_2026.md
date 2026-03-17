# healthSpring V33 → toadStool/barraCuda Absorption Handoff

**Date:** March 16, 2026
**From:** healthSpring V33 (ecoPrimals)
**To:** toadStool, barraCuda, coralReef (informational)
**License:** AGPL-3.0-or-later (scyBorg Provenance Trio)
**Status:** Active handoff

---

## Executive Summary

- **635 tests**, 73 experiments, 42 Python baselines with provenance, 113/113 cross-validation
- healthSpring consumes **12+ barraCuda modules** across 6 science domains
- **6 WGSL shaders** (3 Tier A rewired to upstream, 3 Tier B ready for absorption)
- **3 health ops already contributed** to upstream barraCuda; **7 more ready**
- **V33** adds `DispatchOutcome` (protocol error classification), `IpcError::is_recoverable()`, generic discovery helpers, centralized `cast` module
- Zero unsafe, zero clippy warnings, zero `#[allow()]`, all files < 1000 LOC

---

## Part 1: barraCuda Primitive Consumption Map

### 1.1 Heavy-Use Modules

| barraCuda Module | healthSpring Usage | Delegation Count |
|------------------|--------------------|:----------------:|
| `stats::hill` | Hill dose-response (4-param) | 1 |
| `stats::shannon_from_frequencies` | Shannon diversity index | 1 |
| `stats::simpson` | Simpson diversity index | 1 |
| `stats::chao1_classic` | Chao1 richness estimator | 1 |
| `stats::bray_curtis` | Bray-Curtis dissimilarity | 2 |
| `stats::mean` | Mean for UQ bootstrap/jackknife | 1 |
| `health::pkpd::mm_auc` | Michaelis-Menten AUC | 1 |
| `health::microbiome::antibiotic_perturbation` | Gut perturbation model | 1 |
| `health::biosignal::scr_rate` | SCR rate for EDA stress detection | 1 |
| `special::anderson_diagonalize` | Anderson localization for gut colonization | 1 |
| `rng::lcg_step`, `state_to_f64`, `uniform_f64_sequence` | LCG PRNG (re-exported via `rng.rs`) | 4 |
| `numerical::OdeSystem` | Trait for ODE→WGSL codegen | 3 |
| `device::WgpuDevice` | GPU context creation | 1 |
| `ops::hill_f64::HillFunctionF64` | Tier A GPU rewire (Hill) | 1 |
| `ops::population_pk_f64::PopulationPkF64` | Tier A GPU rewire (PopPK) | 1 |
| `ops::bio::diversity_fusion::DiversityFusionGpu` | Tier A GPU rewire (Diversity) | 1 |

### 1.2 ODE Systems (Write → Absorb → Lean)

healthSpring defines 3 `OdeSystem` implementations for barraCuda codegen:

| ODE System | Domain | WGSL Shader | Status |
|-----------|--------|-------------|--------|
| Michaelis-Menten batch | PK/PD | `michaelis_menten_batch_f64.wgsl` | **Tier B** — ready for upstream |
| SCFA production batch | Microbiome | `scfa_batch_f64.wgsl` | **Tier B** — ready for upstream |
| Beat classification | Biosignal | `beat_classify_batch_f64.wgsl` | **Tier B** — ready for upstream |

### 1.3 Compose Phase (Wiring Multiple Primitives)

healthSpring's `GpuContext::execute_fused()` composes multiple GPU ops into a
single submission:

```
Hill → PopPK → Diversity → MM → SCFA → BeatClassify
```

All 6 ops in one encoder, one readback. This is healthSpring's fused pipeline,
which should inform the design of barraCuda's `TensorSession` API.

---

## Part 2: WGSL Shader Census

### 2.1 Tier A — Absorbed Upstream (rewire to barraCuda ops)

| Shader | Op | Status |
|--------|-----|--------|
| `hill_dose_response_f64.wgsl` | `HillFunctionF64` | **Rewired** — `barracuda_rewire::execute_hill_barracuda()` |
| `population_pk_f64.wgsl` | `PopulationPkF64` | **Rewired** — `barracuda_rewire::execute_pop_pk_barracuda()` |
| `diversity_f64.wgsl` | `DiversityFusionGpu` | **Rewired** — `barracuda_rewire::execute_diversity_barracuda()` |

### 2.2 Tier B — Ready for Upstream Absorption

| Shader | Domain | Complexity | Notes |
|--------|--------|:----------:|-------|
| `michaelis_menten_batch_f64.wgsl` | PK/PD | Medium | Per-patient Euler ODE, capacity-limited elimination. `OdeSystem` trait. |
| `scfa_batch_f64.wgsl` | Microbiome | Medium | 3× Michaelis-Menten kinetics (acetate, propionate, butyrate). `OdeSystem` trait. |
| `beat_classify_batch_f64.wgsl` | Biosignal | Low | Template-matching beat classification (normal, PVC, PAC). |

**barraCuda action:** Absorb these 3 Tier B shaders into `barracuda::ops::health::*`.
They follow the same `OdeSystem` → WGSL codegen pattern as existing ops. Once
absorbed, healthSpring switches from local WGSL to `barracuda_rewire` delegation.

---

## Part 3: GPU Learnings for barraCuda Evolution

From 42/42 GPU parity checks and 6 shaders:

| Learning | Detail | Impact |
|----------|--------|--------|
| **f64 precision critical for PK/PD** | Hill dose-response and population PK require f64; f32 accumulates >1% error over 1000-patient MC simulations | `Fp64Strategy` is correct default for health domain |
| **`pow()` precision varies across GPU** | `pow(x, n)` on some GPUs deviates from CPU `f64::powf`; `exp(n * log(x))` is more portable | Document in barraCuda precision guide |
| **PRNG state must be u64** | LCG with `u64` state → `f64` via `(state >> 33) as f64 / u32::MAX as f64` gives sufficient uniformity | Current barraCuda LCG is correct |
| **Fused pipeline needs TensorSession** | 6-op fused submission via single encoder is 30-40% faster than individual dispatches | Use healthSpring `fused.rs` as design input for `TensorSession` |
| **IPC buffer size matters** | Default 4KB buffer truncates large capability responses; we use `IPC_PROBE_BUF` (configurable) | All IPC consumers should configurable buffer sizes |
| **Non-async barraCuda ops** | `HillFunctionF64::compute`, `PopulationPkF64::simulate`, `DiversityFusionGpu::compute` are synchronous — don't wrap in `async` | Document in barraCuda API guide |

---

## Part 4: IPC Patterns for toadStool Absorption

### 4.1 `DispatchOutcome` (V33 — from groundSpring V112)

```rust
pub enum DispatchOutcome {
    Ok(serde_json::Value),
    ProtocolError { code: i64, message: String },
    ApplicationError { code: i64, message: String },
}
```

Classifies RPC responses by error code range. Protocol errors (-32700..=-32600)
are retryable; application errors are domain-specific and should propagate.

**toadStool action:** Absorb this pattern into the compute dispatch client.
When a spring returns a protocol error, retry; when it returns an application
error, surface to the caller.

### 4.2 `IpcError::is_recoverable()` (V33 — from neuralSpring S161)

```rust
impl IpcError {
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::Timeout | Self::Write(_) | Self::Read(_))
    }
}
```

**toadStool action:** Add `is_recoverable()` to toadStool's error type for
retry logic in `compute.dispatch` and pipeline orchestration.

### 4.3 Generic Discovery Helpers (V33)

```rust
pub fn socket_from_env(env_var: &str) -> Option<PathBuf>
pub fn discover_primal_socket(env_override: &str, name_prefix: &str) -> Option<PathBuf>
```

Replaces per-primal env-var functions. Resolution order: env var → socket dir scan.

**toadStool action:** Consider absorbing this as a shared utility for any primal
needing multi-primal socket discovery.

### 4.4 Structured `tracing` (V32)

healthSpring's primal binary uses `tracing` with `EnvFilter` (default
`healthspring=info`, configurable via `RUST_LOG`). All `eprintln!` eliminated.

### 4.5 Health Probes (V32 — from coralReef Iter 51)

- `health.liveness` — `{"alive": true}` unconditionally
- `health.readiness` — subsystem availability check

**toadStool action:** Implement liveness/readiness probes in toadStool's
JSON-RPC interface for orchestrator health monitoring.

### 4.6 Resilient Provenance IPC (V32 — from sweetGrass v0.7.18)

Circuit breaker (5s cooldown) + exponential backoff retry (50ms base, 2 retries)
for provenance trio calls. Transient failures retried; permanent failures
propagated immediately.

### 4.7 Enriched Capability Response (V31)

`capability.list` returns `operation_dependencies` (DAG) and `cost_estimates`
(CPU ms, GPU eligibility) for biomeOS Pathway Learner integration.

**toadStool action:** Parse `cost_estimates` from spring capability responses
to inform dispatch routing decisions.

---

## Part 5: What healthSpring Does NOT Need from barraCuda

- **`nn` module** — no neural network inference in health domain (ML delegated to squirrel/neuralSpring)
- **`spectral::fft`** — healthSpring has its own pure-Rust FFT (biosignal-specific, 213 LOC)
- **`nautilus`** — not currently used; future candidate for biosignal anomaly detection
- **`TensorSession`** — not yet available; healthSpring's `fused.rs` serves as interim + design input

---

## Part 6: Recommended Upstream Actions

### For barraCuda

1. **Absorb 3 Tier B shaders** — `michaelis_menten_batch_f64.wgsl`, `scfa_batch_f64.wgsl`, `beat_classify_batch_f64.wgsl`. All follow `OdeSystem` trait; each is 50-100 LOC of WGSL.
2. **Document `pow()` precision** — GPU `pow(x, n)` differs from CPU `f64::powf`; recommend `exp(n * log(x))` for portability.
3. **Design `TensorSession` from healthSpring `fused.rs`** — 6-op fused pipeline, single encoder, one readback. Pattern is proven (42/42 parity).
4. **Add `inverse_simpson`** to `stats` module — healthSpring implements locally; barraCuda has `simpson` but not the reciprocal.
5. **Add `normalized_correlation`** to `stats` module — Pearson correlation for biosignal template matching.
6. **Expose `solve_triangular`** as public API — healthSpring's NLME `cholesky_solve` could delegate.
7. **Document non-async nature of ops** — `HillFunctionF64::compute` etc. are sync; callers shouldn't wrap in `async`.

### For toadStool

1. **Absorb `DispatchOutcome`** — classify protocol vs application errors in dispatch client.
2. **Add `is_recoverable()`** to toadStool's IPC error type for retry decisions.
3. **Implement `health.liveness` / `health.readiness`** probes (coralReef Iter 51).
4. **Parse `cost_estimates`** from spring capability responses for routing.
5. **Consider generic discovery helpers** (`socket_from_env` / `discover_primal_socket`) as shared utility.

### For coralReef

1. **Validate Tier B WGSL shaders** through sovereign compiler — `michaelis_menten_batch_f64.wgsl`, `scfa_batch_f64.wgsl`, `beat_classify_batch_f64.wgsl`.
2. **Confirm f64 precision** for health-domain shaders through coralReef's precision tier routing.

---

## Test Evidence

| Metric | Value |
|--------|-------|
| Workspace tests | **635** (567 lib + 33 forge + 30 toadStool + 5 doc) |
| Experiments | 73 (Tracks 1–7) |
| Python baselines | 42 with provenance |
| Cross-validation | 113/113 checks |
| GPU parity | 42/42 checks |
| Clippy warnings | 0 |
| Unsafe blocks | 0 |
| `#[allow()]` | 0 |
| TODO/FIXME | 0 |
| Max file LOC | ~767 |

---

## Superseded

- V32 toadStool/barraCuda Absorption → `archive/`
