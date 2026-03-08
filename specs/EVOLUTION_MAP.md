<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Evolution Map — Rust Module → WGSL Shader → Pipeline Stage

**Last Updated**: March 8, 2026
**Status**: Tier 0+1 complete. GPU dispatch layer built (`gpu.rs`). No WGSL shaders executed yet.

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
| `pkpd::hill_dose_response` | Per-concentration Hill eq | `HillSweep` | `batched_elementwise_f64.wgsl` | Element-wise exp/pow/div | P1 |
| `microbiome::shannon_index` | -Σ p·ln(p) batch | `DiversityBatch` | `mean_variance_f64.wgsl` | Map-reduce per community | P1 |
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

## GPU Dispatch Layer (`barracuda/src/gpu.rs`)

Built in V4. Maps domain operations to WGSL shaders with CPU reference fallback.

| GpuOp Variant | CPU Fallback | Shader Path | Memory Estimate |
|--------------|-------------|-------------|----------------|
| `HillSweep` | `pkpd::hill_dose_response` loop | `batched_elementwise_f64.wgsl` | 8 bytes/concentration |
| `PopulationPkBatch` | `pkpd::population_pk_cpu` | `population_pk_f64.wgsl` | ~200 bytes/patient |
| `DiversityBatch` | `microbiome::shannon_index` loop | `mean_variance_f64.wgsl` | 8 bytes/taxon/community |

Exp040 validates analytical CPU parity (15 contracts) between direct function calls and `execute_cpu(&GpuOp)`.

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

## Blocking Items for Tier 2

1. ~~metalForge routing logic empty~~ → NUCLEUS atomics + transfer planning built (27 tests)
2. ~~No GPU dispatch abstraction~~ → `gpu.rs` GpuOp + execute_cpu + shader_for_op built
3. `population_pk_f64.wgsl` not yet written in barraCuda (P0 target)
4. No `FusedMapReduceF64` fused-op chain wiring in healthSpring
5. ODE solver absorption status from wetSpring TBD
6. NPU dispatch path in toadStool not production-ready
7. coralReef `df64_core.wgsl` preamble required for consumer GPUs
