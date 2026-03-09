<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Evolution Map — Rust Module → WGSL Shader → Pipeline Stage

**Last Updated**: March 9, 2026
**Status**: V7. Tier 0+1+2 complete. 3 WGSL shaders live (Hill, PopPK, Diversity). GpuContext + fused pipeline. Full petalTongue visualization (22 nodes, 62 channels). petalTongue absorption complete.

---

## Evolution Path

```
Python baseline → Rust CPU (Tier 1) → barraCuda CPU parity (Exp040)
  → GPU via WGSL (Tier 2) → metalForge dispatch (Tier 3)
```

---

## Tier Assessment

### Tier A — Ready for GPU (Embarrassingly Parallel)

| Rust Module | Function | GpuOp | WGSL Shader | Pipeline Pattern | Priority |
|------------|----------|-------|-------------|-----------------|----------|
| `pkpd::population_pk_cpu` | Per-patient Bateman ODE | `PopulationPkBatch` | `population_pk_f64.wgsl` | 1 workgroup/patient, independent | **P0** |
| `pkpd::hill_dose_response` | Per-concentration Hill eq | `HillSweep` | `hill_dose_response_f64.wgsl` | Element-wise, f32 pow() intermediates | **P0 — LIVE** |
| `microbiome::shannon_index` | -Σ p·ln(p) batch | `DiversityBatch` | `diversity_f64.wgsl` | Workgroup reduction | **P0 — LIVE** |
| `endocrine::lognormal_params` | μ,σ from typical+CV | — | CPU-side utility | Parameter setup, not GPU | P2 |

### Tier B — Adapt (Decompose into Existing Primitives)

| Rust Module | Function | barraCuda Primitive | Notes |
|------------|----------|-------------------|-------|
| `microbiome::simpson_index` | 1 - Σ p² | `FusedMapReduceF64` (square→sum) | |
| `pkpd::auc_trapezoidal` | Σ (c_i+c_{i+1})/2 · Δt | `FusedMapReduceF64` (diff→mul→sum) | Parallel prefix |
| `endocrine::anderson_localization_length` | ξ(W) power-law | Extend hotSpring Anderson | Parameterize for gut substrate |
| `biosignal::ppg_r_value` | AC/DC ratio per channel | `batched_elementwise_f64.wgsl` | Element-wise division |
| `microbiome::bray_curtis` | Community dissimilarity | `FusedMapReduceF64` (abs_diff→sum) | Pairwise matrix |

### Tier C — New Shader Required

| Rust Module | Function | Shader Design | Blocking |
|------------|----------|--------------|----------|
| `biosignal::pan_tompkins_qrs` | Streaming detect pipeline | Custom streaming shader or NPU | NPU dispatch path in toadStool |
| `biosignal::fuse_channels` | Multi-modal ECG+PPG+EDA | toadStool pipeline (3-stage) | Pipeline execution on GPU |
| `pkpd::pbpk_iv_simulate` | PBPK multi-compartment ODE | Euler/RK4 ODE shader | wetSpring ODE absorption status |
| `endocrine::hrv_trt_response` | Exponential saturation | `batched_elementwise_f64.wgsl` | Trivial once shader exists |

---

## GPU Dispatch Layer (`barracuda/src/gpu.rs`) — LIVE (V6+)

`GpuContext` holds persistent `wgpu::Device`/`Queue`. `execute_fused()` dispatches multiple ops in a single encoder submission.

| GpuOp Variant | CPU Fallback | WGSL Shader | Status |
|--------------|-------------|-------------|--------|
| `HillSweep` | `pkpd::hill_dose_response` loop | `hill_dose_response_f64.wgsl` | **LIVE** — 17/17 parity, crossover at 100K |
| `PopulationPkBatch` | `pkpd::population_pk_cpu` | `population_pk_f64.wgsl` | **LIVE** — u32 xorshift32 PRNG, crossover at 5M |
| `DiversityBatch` | `microbiome::shannon_index` loop | `diversity_f64.wgsl` | **LIVE** — workgroup reduction |

Exp040 validates CPU parity (15 contracts). Exp053 validates GPU parity (17 checks). Exp054 validates fused pipeline (11 checks). Exp055 validates scaling to 10M elements.

---

## Absorption Candidates (healthSpring → barraCuda)

| Function | Current Location | Target | Status |
|----------|-----------------|--------|--------|
| `pk_multiple_dose` | `pkpd/compartment.rs` | `barraCuda::signal` | Ready |
| `hill_dose_response` | `pkpd/dose_response.rs` | `barraCuda::bio::pharmacology` | Ready |
| `allometric_scale` | `pkpd/allometry.rs` | `barraCuda::math::scale` | Ready |
| `lognormal_params` | `pkpd/population.rs` | `barraCuda::stats::distributions` | Ready |
| `population_pk_cpu` | `pkpd/population.rs` | `barraCuda::monte_carlo` | Ready |
| `hazard_ratio_model` | `endocrine.rs` | `barraCuda::epi` | Ready |
| `pan_tompkins_qrs` | `biosignal.rs` | `barraCuda::signal::detect` | Needs streaming design |
| `pbpk_iv_simulate` | `pkpd/pbpk.rs` | `barraCuda::bio::pbpk` | Ready (ODE integration) |
| `bray_curtis` | `microbiome.rs` | `barraCuda::bio::ecology` | Ready |
| `fmt_blend` | `microbiome.rs` | `barraCuda::bio::ecology` | Ready |
| `fuse_channels` | `biosignal.rs` | toadStool pipeline stage | Ready |
| `cardiac_risk_composite` | `endocrine.rs` | `barraCuda::epi::risk` | Ready |

---

## Absorption Candidates (healthSpring → toadStool/metalForge)

| Component | Current Location | Target | Status |
|-----------|-----------------|--------|--------|
| NUCLEUS atomics (Tower/Node/Nest) | `metalForge/forge/src/nucleus.rs` | toadStool core topology | Ready |
| PCIe P2P transfer planning | `metalForge/forge/src/transfer.rs` | toadStool scheduler | Ready |
| Pipeline/Stage/StageOp | `toadstool/src/` | toadStool core | Ready |
| Substrate/Workload dispatch | `metalForge/forge/src/lib.rs` | toadStool dispatch layer | Ready |

---

## metalForge Dispatch Targets

| Experiment | GPU (Server) | CPU (Desktop) | NPU (Wearable) |
|-----------|-------------|--------------|----------------|
| Exp005/036 Population PK | Batch 10K-1M patients | Single patient | — |
| Exp001 Hill sweep | 100K concentrations | 100 concentrations | — |
| Exp010/013 Diversity batch | 10K communities | 100 communities | — |
| Exp020 Pan-Tompkins | — | Offline analysis | Real-time ECG (Akida) |
| Exp023 Biosignal fusion | — | Multi-channel offline | Streaming 3-channel (Akida) |
| Exp006 PBPK | 10K patients × tissues | Single patient | — |

---

## petalTongue Evolution — COMPLETE (V6.1 lean, V7 visualization)

petalTongue absorbed all healthSpring prototypes (commit `037caaa`). healthSpring leaned in V6.1 (petaltongue-health removed). V7 added per-track scenario builders.

### Absorption Status

| Component | healthSpring Source | petalTongue Target | Status |
|-----------|--------------------|--------------------|--------|
| `DataChannel` enum | `visualization/types.rs` | `petal-tongue-core/data_channel.rs` | **Absorbed** |
| `ClinicalRange` struct | `visualization/types.rs` | `petal-tongue-core/data_channel.rs` | **Absorbed** (status: String) |
| Chart renderers | ~~petaltongue-health/render.rs~~ (removed) | `petal-tongue-graph/chart_renderer.rs` | **Absorbed** |
| Clinical theme | ~~petaltongue-health/theme.rs~~ (removed) | `petal-tongue-graph/clinical_theme.rs` | **Absorbed** |
| Version parsing fix | N/A | `dynamic_schema.rs` | **Fixed** |

### Per-Track Scenario Builders (V7)

| Track | Builder | Nodes | Channels | Experiments |
|-------|---------|-------|----------|-------------|
| PK/PD | `scenarios::pkpd_study()` | 6 | 18 | Exp001-006 |
| Microbiome | `scenarios::microbiome_study()` | 4 | 10 | Exp010-013 |
| Biosignal | `scenarios::biosignal_study()` | 4 | 15 | Exp020-023 |
| Endocrinology | `scenarios::endocrine_study()` | 8 | 19 | Exp030-038 |
| Full Study | `scenarios::full_study()` | 22 | 62 | All 4 tracks |

---

## Tier 2 Status — LIVE

All previous blocking items resolved:

1. ~~metalForge routing logic empty~~ → NUCLEUS atomics + transfer planning built (27 tests) ✓
2. ~~No GPU dispatch abstraction~~ → `GpuContext` + `execute_fused` ✓
3. ~~`population_pk_f64.wgsl` not written~~ → LIVE with u32 xorshift32 PRNG ✓
4. ~~No fused-op chain~~ → `execute_fused()` single encoder submission ✓
5. ODE solver absorption status from wetSpring TBD
6. NPU dispatch path in toadStool not production-ready
7. ~~coralReef `df64_core.wgsl` preamble~~ → `strip_f64_enable()` workaround ✓
