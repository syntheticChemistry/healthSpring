<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V25 → barraCuda/toadStool: Track 6+7 Complete — Absorption + Evolution Handoff

**Date**: March 15, 2026
**From**: healthSpring V25 (501 tests, 73 experiments, 173 new validation checks)
**To**: barraCuda team, toadStool team
**License**: CC-BY-SA-4.0
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: V24 Audit Execution Handoff (Mar 15, 2026)
**barraCuda pin**: `a60819c` (v0.3.5)

---

## Executive Summary

V25 completes the Track 6 (Comparative Medicine) and Track 7 (Drug Discovery) paper queues:

- **12 new experiment binaries** (Exp090–094, Exp100–106) with 173 validation checks, all green
- **8 new library modules** in `discovery/` (matrix_score, hts, compound, fibrosis) and `comparative/` (species_params, canine, feline)
- **7 Python Tier 0 baselines** providing cross-validation targets
- **17 new named tolerances** with documented justification
- **501 library tests + 173 validation checks** — zero failures, zero clippy warnings (pedantic + nursery)

Key findings relevant to barraCuda/toadStool:
1. **Batch Hill sweep** (`discovery::compound::batch_ic50_sweep`) is a direct GPU candidate — reuses the `HillSweep` pattern validated in Exp053, applied to compound library screening
2. **MATRIX panel scoring** (`discovery::matrix_score::score_panel`) is a compound of existing ops: pathway selectivity (Hill) × tissue geometry (Anderson) × disorder impact
3. **Feline Michaelis-Menten PK** reuses the same nonlinear ODE pattern as `MichaelisMentenBatchF64` shader — species parameters change, math is identical
4. **Anderson gut lattice** works cross-species: canine gut diversity → same Shannon/Pielou/W pipeline as human gut (Exp010–013)
5. **Fibrotic geometry** inverts the standard Anderson tissue geometry — `exp(-ξ/L)` instead of `1 - exp(-ξ/L)` — same shader, different sign

---

## Part 1: New Tier A GPU Candidates

V25 identifies new operations that map directly to existing barraCuda GPU ops.

| healthSpring Function | Maps To | Pattern | GPU Priority |
|----------------------|---------|---------|:------------:|
| `discovery::compound::batch_ic50_sweep` | `HillSweep` (reuse) | N compounds × K targets Hill fit | **P0** |
| `discovery::hts::classify_hits` | Element-wise threshold | Per-well classification | P1 |
| `comparative::feline::methimazole_simulate` | `MichaelisMentenBatchF64` (reuse) | Per-patient Euler ODE | P2 |

### barraCuda action

`batch_ic50_sweep` is the highest priority. It's the same `HillSweep` shader dispatched per-compound instead of per-dose. The compound loop is embarrassingly parallel. At ADDRC scale (8,000 compounds × 4 JAK targets × 10 concentrations), this is 320,000 independent Hill evaluations — ideal for GPU.

---

## Part 2: New Library Modules — What They Do

### discovery/ (Track 7: Drug Discovery)

| Module | Functions | Key Types | GPU Pattern |
|--------|-----------|-----------|-------------|
| `matrix_score` | `pathway_selectivity_score`, `tissue_geometry_factor`, `disorder_impact_factor`, `matrix_combined_score`, `score_compound`, `score_panel` | `MatrixEntry`, `TissueContext` | Compound of Hill + Anderson — CPU orchestration, GPU sub-ops |
| `hts` | `z_prime_factor`, `ssmd`, `percent_inhibition`, `classify_hits` | `HitClass`, `HitResult` | Element-wise — straightforward GPU port |
| `compound` | `estimate_ic50`, `batch_ic50_sweep`, `selectivity_index`, `rank_by_selectivity` | `CompoundProfile`, `Ic50Estimate`, `CompoundScorecard` | `estimate_ic50` is bisection search (GPU-unfriendly); `batch_ic50_sweep` wraps it in embarrassingly parallel loop |
| `fibrosis` | `fractional_inhibition`, `score_anti_fibrotic`, `fibrotic_geometry_factor`, `fibrosis_matrix_score` | `AntiFibroticCompound`, `FibrosisPathwayScore` | Element-wise — reuses Hill + inverted Anderson geometry |

### comparative/ (Track 6: Comparative Medicine)

| Module | Functions | Key Types | GPU Pattern |
|--------|-----------|-----------|-------------|
| `species_params` | `allometric_clearance`, `allometric_volume`, `allometric_half_life`, `scale_across_species` | `Species`, `SpeciesPkParams` | CPU-only parameter lookup — not GPU |
| `canine` | `il31_serum_kinetics`, `pruritus_vas_response`, `pruritus_time_course`, `lokivetmab_pk`, `lokivetmab_effective_duration`, `lokivetmab_onset_hr`, `canine_jak_ic50_panel`, `human_jak_reference_panels` | `CanineIl31Treatment`, `JakIc50Panel` | Time-course is batch-parallel across patients/treatments |
| `feline` | `methimazole_simulate`, `methimazole_apparent_half_life`, `methimazole_css`, `t4_response` | `FelineMethimazoleParams` | Same MM ODE as `MichaelisMentenBatchF64` — parameter swap |

---

## Part 3: TissueContext Pattern — Reducing Argument Count

V25 introduced `TissueContext` struct to group tissue/microbiome parameters:

```rust
pub struct TissueContext {
    pub localization_length: f64,
    pub tissue_thickness: f64,
    pub w_baseline: f64,
    pub w_treated: f64,
}
```

This replaces 4 function arguments with a single `&TissueContext` reference. When porting `score_compound` / `score_panel` to GPU, the `TissueContext` maps to a uniform buffer.

### toadStool action

When building GPU dispatch for MATRIX scoring, use the `TissueContext` pattern: struct → uniform buffer. This avoids the "too many kernel arguments" problem that plagues GPU compute APIs.

---

## Part 4: Cross-Species PK — Same Math, Different Parameters

The allometric scaling bridge validates that species is a parameter, not a code branch:

```
allometric_clearance(reference_cl, target_bw, reference_bw, exponent)
allometric_volume(reference_vd, target_bw, reference_bw, exponent)
allometric_half_life(cl, vd) → 0.693 × vd / cl
```

For barraCuda: no new shader needed. The existing `MichaelisMentenBatchF64` shader works for feline methimazole, phenytoin, or any capacity-limited drug — just change the `Vmax`, `Km`, `Vd` parameters in the uniform buffer.

---

## Part 5: Fibrotic Anderson Geometry — Sign Inversion

Standard tissue geometry: `factor = 1 - exp(-ξ/L)` (long ξ → good drug distribution)
Fibrotic tissue geometry: `factor = exp(-ξ/L)` (short ξ → good anti-fibrotic targeting)

For barraCuda: this is a compile-time variant, not a new shader. A `#[cfg]` flag or a uniform boolean selects the sign.

---

## Part 6: Evolution Path — What barraCuda/toadStool Should Absorb

### Immediate (P0): Already-proven patterns

| What | Where | Absorb As |
|------|-------|-----------|
| Batch Hill sweep for compound libraries | `discovery::compound::batch_ic50_sweep` | Reuse existing `HillSweep` with compound-level parallelism |
| HTS hit classification | `discovery::hts::classify_hits` | Element-wise threshold op |

### Near-term (P1): New patterns

| What | Where | Absorb As |
|------|-------|-----------|
| MATRIX panel scoring | `discovery::matrix_score::score_panel` | Compound op: Hill + Anderson + product |
| Fibrotic geometry variant | `discovery::fibrosis::fibrotic_geometry_factor` | Boolean flag on existing `tissue_geometry_factor` |

### Future (P2): When TensorSession lands

| What | Where | Notes |
|------|-------|-------|
| Pruritus time-course | `comparative::canine::pruritus_time_course` | Batch across patients × treatments × time points |
| Cross-species scaling | `comparative::species_params::scale_across_species` | CPU orchestration of per-species GPU sweeps |

---

## Part 7: Learnings for toadStool

### Unidirectional streaming is a game-changer

The pruritus time-course (`pruritus_time_course`) computes VAS trajectories by composing `il31_serum_kinetics` → `pruritus_vas_response` at each time point. With toadStool's unidirectional streaming, this becomes:

```
il31_kinetics_stream → pruritus_vas_map → output_stream
```

No round-trips. No intermediate buffers held in CPU memory. This is the pattern for all time-course pharmacology.

### Species as a dispatch parameter

toadStool's `plan_dispatch` should treat species as a workload parameter, not a separate code path. The math is species-agnostic; only the parameter buffers change. This means:

```
plan_dispatch(workload: Workload::MichaelisMenten { species: "feline" })
  → same shader, different uniform buffer
```

---

## Part 8: Quality Metrics (V25)

| Metric | Value |
|--------|-------|
| Tests | 501 |
| Experiments | 73 |
| Validation checks | 173 (new) |
| New library modules | 8 |
| Named tolerances | 17 (new) |
| `#[allow()]` in production | 0 |
| TODO/FIXME | 0 |
| Unsafe blocks | 0 |
| Clippy | 0 warnings (pedantic + nursery) |
| Python Tier 0 baselines | 7 (new) |

---

## Action Items Summary

### barraCuda team

| Priority | Action |
|----------|--------|
| **P0** | Reuse `HillSweep` for compound-level batch IC50 (320K evaluations at ADDRC scale) |
| **P0** | Absorb 3 V16 Tier B shaders (reiterated from V24): `MichaelisMentenBatchF64`, `ScfaBatchF64`, `BeatClassifyBatchF64` |
| **P1** | Add fibrotic geometry boolean variant to `tissue_geometry_factor` |
| **P1** | Design `TensorSession` API using `fused.rs` + `TissueContext` as reference input |
| **P2** | Implement `FoceInnerLoop` GPU primitive for NLME population PK |

### toadStool team

| Priority | Action |
|----------|--------|
| **P0** | Respond to `capability.list` JSON-RPC (reiterated from V24) |
| **P0** | Support unidirectional streaming for time-course pharmacology |
| **P1** | Treat species as dispatch parameter (same shader, different uniform buffer) |
| **P2** | Support `TensorSession`-based dispatch when available upstream |

### healthSpring (self)

| Priority | Action |
|----------|--------|
| **P0** | Rewire Tier A ops to barraCuda upstream (Hill, PopPK, Diversity) |
| **P1** | Complete Track 7 DD-006 (iPSC validation) and DD-007 (Ellsworth med chem) |
| **P1** | Complete Track 6 CM-008 (equine laminitis) |
| **P2** | barraCuda CPU parity benchmarks for Track 6+7 primitives |
