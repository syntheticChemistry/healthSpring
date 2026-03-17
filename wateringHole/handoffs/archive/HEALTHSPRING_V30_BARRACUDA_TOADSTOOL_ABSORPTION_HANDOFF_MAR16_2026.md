# HEALTHSPRING V30 — barraCuda / toadStool Absorption Handoff

**Date:** March 16, 2026
**From:** healthSpring V30 (611 tests, 73 experiments, 7 tracks)
**To:** barraCuda team, toadStool team
**License:** scyBorg (AGPL-3.0-or-later code + ORC mechanics + CC-BY-SA 4.0 creative content)
**Scope:** Complete absorption guide — what to absorb, what to evolve, what we learned

---

## Executive Summary

healthSpring has reached maturity across all 7 science tracks. This handoff provides the
barraCuda and toadStool teams with everything needed to absorb our validated science into
upstream primitives. Three layers:

1. **barraCuda**: 6 WGSL shaders ready for upstream absorption, 3 CPU health delegations
   already leaning on `barracuda::health::*`, 20+ CPU primitives validated and ready
2. **toadStool**: `compute.dispatch.*` protocol typed and tested from client side,
   streaming dispatch needed for real-time biosignal, workload registry spec
3. **GPU learnings**: f64 portability, PRNG, fused pipeline, scaling analysis

---

## Part 1: barraCuda Absorption — Complete Inventory

### 1.1 Already Absorbed (Lean Phase — healthSpring delegates to barraCuda)

| healthSpring | barraCuda | Since | Type |
|-------------|-----------|-------|------|
| `pkpd::hill_dose_response()` | `barracuda::stats::hill` | V13 | CPU |
| `pkpd::auc_trapezoidal()` (reference) | `barracuda::health::pkpd::mm_auc` | V30 | CPU |
| `microbiome::shannon_index()` | `barracuda::stats::shannon_from_frequencies` | V13 | CPU |
| `microbiome::simpson_index()` | `barracuda::stats::simpson` | V13 | CPU |
| `microbiome::chao1_richness()` | `barracuda::stats::chao1_classic` | V13 | CPU |
| `microbiome::bray_curtis_distance()` | `barracuda::stats::bray_curtis` | V28 | CPU |
| `microbiome::anderson_diagonalize()` | `barracuda::special::anderson_diagonalize` | V13 | CPU |
| `uncertainty::mean()` | `barracuda::stats::mean` | V29 | CPU |
| `biosignal::scr_rate()` | `barracuda::health::biosignal::scr_rate` | V30 | CPU |
| `clinical::antibiotic_perturbation_abundances()` | `barracuda::health::microbiome::antibiotic_perturbation` | V30 | CPU |
| `rng::*` | `barracuda::rng::*` (LCG, lcg_step, state_to_f64) | V13 | CPU |
| `gpu::ode_systems::*` | `barracuda::numerical::OdeSystem` | V27 | GPU codegen |
| `gpu::barracuda_rewire::hill_sweep()` | `barracuda::ops::HillFunctionF64` | V24 | GPU |
| `gpu::barracuda_rewire::pop_pk_batch()` | `barracuda::ops::PopulationPkF64` | V24 | GPU |
| `gpu::barracuda_rewire::diversity_batch()` | `barracuda::ops::bio::DiversityFusionGpu` | V24 | GPU |

### 1.2 WGSL Shaders — Tier B Absorption Candidates

These 6 local shaders are validated and ready for absorption into `barracuda::ops::health`:

| Shader | Location | Validation | GPU Pattern | Recommendation |
|--------|----------|-----------|-------------|----------------|
| `hill_dose_response_f64.wgsl` | `ecoPrimal/shaders/health/` | Exp053 (17/17) | Element-wise | Already absorbed into `barracuda::ops::hill_f64` — keep local as reference |
| `population_pk_f64.wgsl` | `ecoPrimal/shaders/health/` | Exp053 (17/17) | MC parallel | Already absorbed into `barracuda::ops::population_pk_f64` — keep local as reference |
| `diversity_f64.wgsl` | `ecoPrimal/shaders/health/` | Exp053 (17/17) | Workgroup reduction | Already absorbed into `barracuda::ops::bio::diversity_fusion` — keep local as reference |
| **`michaelis_menten_batch_f64.wgsl`** | `ecoPrimal/shaders/health/` | Exp083 (25/25) | Per-patient ODE | **ABSORB** → `barracuda::ops::health::mm_batch_f64` |
| **`scfa_batch_f64.wgsl`** | `ecoPrimal/shaders/health/` | Exp083 (25/25) | Element-wise 3-output | **ABSORB** → `barracuda::ops::health::scfa_batch_f64` |
| **`beat_classify_batch_f64.wgsl`** | `ecoPrimal/shaders/health/` | Exp083 (25/25) | Per-beat correlation | **ABSORB** → `barracuda::ops::health::beat_classify_f64` |

**barraCuda action**: Absorb the 3 bold shaders into `crates/barracuda/src/ops/health/`. All three use Wang hash PRNG (u32-only, GPU-portable), f64 precision, and have documented numerical tolerances.

### 1.3 CPU Primitives — Validated & Ready for Absorption

These are validated in healthSpring but not yet in barraCuda. Ordered by absorption priority:

**P0 — Core PK/PD** (extends `barracuda::health::pkpd`):
| Primitive | Module | Tests | Why Absorb |
|-----------|--------|-------|------------|
| `pk_iv_bolus()` | `pkpd/one_compartment.rs` | Exp002 (18 checks) | Foundational PK, used by all springs |
| `pk_oral_one_compartment()` | `pkpd/one_compartment.rs` | Exp002 (18 checks) | Oral PK with Bateman function |
| `pk_two_compartment_iv()` | `pkpd/two_compartment.rs` | Exp003 (11 checks) | Biexponential α/β phases |
| `allometric_scale()` | `comparative/species_params.rs` | Exp104 (12 checks) | Cross-species PK scaling (5 species) |
| `pbpk_iv_simulate()` | `pkpd/pbpk.rs` | Exp006 (13 checks) | 5-tissue PBPK ODE |

**P1 — Microbiome/Biosignal** (extends `barracuda::health`):
| Primitive | Module | Tests | Why Absorb |
|-----------|--------|-------|------------|
| `scfa_production()` | `microbiome/clinical.rs` | Exp079 (11 checks) | Gut SCFA kinetics (matches SCFA shader) |
| `gut_serotonin_production()` | `microbiome/clinical.rs` | Exp080 (10 checks) | Gut-brain axis |
| `fmt_blend()` | `microbiome/clinical.rs` | Exp013 (12 checks) | FMT transplant model |
| `pan_tompkins_qrs()` | `biosignal/ecg.rs` | Exp020 (12 checks) | QRS detection (5-stage) |
| `eda_stress_pipeline()` | `biosignal/eda.rs`, `stress.rs` | Exp081 (11 checks) | EDA autonomic stress |
| `classify_beats()` | `biosignal/mod.rs` | Exp082 (11 checks) | Template beat classification |

**P2 — Clinical/NLME** (new `barracuda::health::nlme` or `barracuda::clinical`):
| Primitive | Module | Tests | Why Absorb |
|-----------|--------|-------|------------|
| `foce_estimate()` | `pkpd/nlme/solver.rs` | Exp075 (19 checks) | Sovereign NONMEM replacement |
| `saem_estimate()` | `pkpd/nlme/solver.rs` | Exp075 (19 checks) | Sovereign Monolix replacement |
| `nca_analysis()` | `pkpd/nca.rs` | Exp075 (19 checks) | Sovereign WinNonlin replacement |

### 1.4 Local Implementations Kept (Divergent Signatures)

These intentionally differ from barraCuda's copies and should NOT be absorbed:

| Function | Rationale |
|----------|-----------|
| `mm_pk_simulate()` | healthSpring uses dose-based API (clinical context); barraCuda uses concentration-based (pure math) |
| `nlme::cholesky_solve()` | Optimized for 2×2/3×3 matrices with fallback — not worth generalizing |
| `fuse_channels()` | healthSpring uses `FusedHealthAssessment` struct; barraCuda has no health-specific fusion type |
| EDA decomposition | healthSpring adds `min_interval_samples` and `moving_window_integration` parameters |

---

## Part 2: toadStool Absorption

### 2.1 `compute.dispatch.*` Protocol

healthSpring V30 added a typed client in `ipc/compute_dispatch.rs`:

```
compute.dispatch.submit    → { workload, params } → { job_id }
compute.dispatch.result    → { job_id }           → { result or error }
compute.dispatch.capabilities → {}                → { workloads: [...] }
```

**toadStool action**: Implement the server side of this protocol. healthSpring has 3 tests
that validate the client, and 4 experiment binaries (exp069, exp086, exp087, exp060) that
exercise dispatch through the existing `toadstool/src/pipeline.rs` `execute_auto()` path.

### 2.2 Streaming Dispatch

`exp065_live_dashboard` needs streaming dispatch for real-time biosignal (250 Hz ECG/PPG/EDA).
Current `StreamSession` with backpressure is wired but needs toadStool server support.

**toadStool action**: Add `compute.dispatch.stream` method that returns a streaming handle
with backpressure control. healthSpring's `StreamSession` in `visualization/stream.rs` is
the client pattern to follow.

### 2.3 Workload Registry

healthSpring defines these GPU workload types:

| Workload Type | Description | WGSL |
|---------------|-------------|------|
| `hill_sweep` | Vectorized Hill dose-response | `hill_dose_response_f64.wgsl` |
| `pop_pk_batch` | Population PK Monte Carlo | `population_pk_f64.wgsl` |
| `diversity_batch` | Shannon + Simpson diversity | `diversity_f64.wgsl` |
| `mm_pk_batch` | Michaelis-Menten per-patient ODE | `michaelis_menten_batch_f64.wgsl` |
| `scfa_batch` | SCFA production kinetics | `scfa_batch_f64.wgsl` |
| `beat_classify` | Template-matching beat classification | `beat_classify_batch_f64.wgsl` |
| `foce_gradient` | NLME per-subject gradient (future) | Codegen via `OdeSystem` |
| `vpc_monte_carlo` | VPC simulation batch (future) | Codegen via `OdeSystem` |

**toadStool action**: Include these in `compute.dispatch.capabilities` response.

---

## Part 3: GPU Learnings

### 3.1 f64 Portability

WGSL `enable f64;` must NOT appear in shader source — `wgpu`/`naga` adds it based on
device features. healthSpring stripped this in V17 after 3 days debugging. All 6 shaders
use `f64` types directly (vec2<f64>, etc.) without the enable directive.

### 3.2 `pow()` on NVIDIA

`pow(f64, f64)` is unsupported on NVIDIA via NVVM. Use `exp(n * log(c))` instead. For Hill
equation: `let cn = exp(n * log(c));` not `let cn = pow(c, n);`. Cast through f32 for the
transcendental if needed (precision loss is <1e-7 at pharmacological concentrations).

### 3.3 GPU PRNG

u64 operations are not portable across GPU vendors. Use u32-only Wang hash:
```
fn wang_hash(seed: u32) -> u32 {
    var s = seed;
    s = (s ^ 61u) ^ (s >> 16u);
    s = s * 9u;
    s = s ^ (s >> 4u);
    s = s * 0x27d4eb2du;
    s = s ^ (s >> 15u);
    return s;
}
```

### 3.4 Fused Pipeline

Single-encoder submission (all ops in one `CommandEncoder`) is 30× faster at small sizes vs
individual dispatches. healthSpring's `GpuContext::execute_fused()` demonstrates this pattern.
When barraCuda ships `TensorSession`, healthSpring will migrate.

### 3.5 Scaling Analysis

| Operation | Crossover (CPU→GPU) | Peak Speedup | Notes |
|-----------|-------------------|:------------:|-------|
| Hill sweep | 100K elements | 2.0× at 5M | Memory-bound above 10M |
| Population PK MC | 5M elements | 1.15× at 5M | ODE overhead dominates |
| Fused (all 3) | ~10K | 31.7× vs individual | Encoder overhead dominates below |

**barraCuda action**: Use these crossover points when deciding default CPU/GPU dispatch thresholds.

---

## Part 4: Ecosystem Patterns We Evolved

### 4.1 Dual-Format Capability Parsing

`extract_capability_strings()` in `ipc/socket.rs` handles 4 response formats. Every primal
in the ecosystem should converge on one format. Recommendation: `result.capabilities` (flat
array) as canonical, with `result.science` / `result.infrastructure` as optional enrichment.

### 4.2 Zero-Panic Validation

All ~100 panic sites replaced with `let Ok/Some(...) else { eprintln!(); exit(1); }`. This
pattern is spreading across springs (groundSpring V109 originated it, wetSpring V123 uses
`OrExit` trait, healthSpring V30 uses let-else). Recommend standardizing on one approach
for the ecosystem.

### 4.3 `deny.toml` for Dep Hygiene

```toml
[bans]
wildcards = "deny"
```
Catches `*` version specs that break reproducibility. All springs should have this.

---

## Part 5: What We Depend On That Should Not Break

| Dependency | Used For | Breakage Impact |
|-----------|----------|----------------|
| `barracuda::stats::hill` | Hill dose-response across all 7 tracks | All PK/PD experiments fail |
| `barracuda::stats::shannon_from_frequencies` | All microbiome experiments | Track 2 fails |
| `barracuda::special::anderson_diagonalize` | Anderson lattice (gut, tissue, cross-species) | Tracks 2, 6, 7 fail |
| `barracuda::health::pkpd::mm_auc` | AUC computation | Track 1 fails |
| `barracuda::health::biosignal::scr_rate` | EDA stress detection | Track 3 fails |
| `barracuda::health::microbiome::antibiotic_perturbation` | Antibiotic perturbation model | Track 2 fails |
| `barracuda::numerical::OdeSystem` | GPU ODE codegen (3 systems) | GPU pipeline breaks |
| `barracuda::ops::HillFunctionF64` | GPU Hill sweep | Tier 2 experiments fail |
| `barracuda::ops::PopulationPkF64` | GPU population PK | Tier 2 experiments fail |
| `barracuda::ops::bio::DiversityFusionGpu` | GPU diversity | Tier 2 experiments fail |
| `barracuda::device::WgpuDevice` | GPU context initialization | All GPU experiments fail |

**barraCuda action**: Do not rename or remove any of the above without coordinating via wateringHole handoff. healthSpring's 611 tests act as integration tests for these APIs.

---

## Part 6: Metrics

| Metric | Value |
|--------|-------|
| Total barraCuda imports | **15 distinct modules** |
| CPU delegations (Lean) | **11** (stats, health, special, rng) |
| GPU delegations (Lean) | **4** (HillF64, PopPkF64, DiversityGpu, OdeSystem) |
| Local WGSL shaders | **6** (3 absorbed, 3 ready for absorption) |
| Validated CPU primitives ready for absorption | **20+** |
| Tests exercising barraCuda APIs | **200+** (direct + transitive) |
| healthSpring version | V30 |
| barraCuda version pinned | v0.3.5 (rev a60819c3) |
