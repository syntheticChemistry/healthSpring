# Cross-Spring Shader Evolution — healthSpring Perspective

**Updated**: April 19, 2026 (V56)
**License**: AGPL-3.0-or-later

This document tracks how WGSL shaders and math primitives flow between
ecoPrimals springs, through barraCuda (the canonical math layer), toadStool
(the hardware orchestrator), and coralReef (the sovereign shader compiler).

---

## How the Ecosystem Works

```
Springs (domain science)
    ↓ validate + contribute
barraCuda (canonical math: 794 WGSL shaders, CPU+GPU ops)
    ↓ compile
coralReef (WGSL → native binary: NVIDIA SM70-SM89, AMD RDNA2)
    ↓ dispatch
toadStool (hardware orchestration: precision routing, streaming dispatch)
```

Each spring validates math at the CPU level, proves GPU parity, then
contributes (absorbs) its primitives into barraCuda. barraCuda becomes
the universal library. Other springs consume the same primitives — this
is cross-spring evolution.

---

## Shader Flow: What healthSpring Consumes and Contributes

### healthSpring → barraCuda (Absorbed)

| Primitive | Origin | barraCuda Module | Shared With | Date |
|-----------|--------|------------------|-------------|------|
| Hill dose-response | Exp001 | `ops::HillFunctionF64` | neuralSpring (kinetics), wetSpring (ecology) | Mar 2026 |
| Population PK Monte Carlo | Exp005 | `ops::PopulationPkF64` | (healthSpring-specific) | Mar 2026 |
| Shannon/Simpson diversity | Exp010 | `ops::bio::DiversityFusionGpu` | wetSpring (16S), neuralSpring (popgen) | Mar 2026 |
| LCG PRNG | V3 | `rng::{lcg_step, state_to_f64}` | ALL springs | Mar 2026 |
| Tridiagonal QL eigensolver | Exp011 | `special::tridiagonal_ql` | groundSpring (Anderson), hotSpring (lattice) | Mar 2026 |
| Anderson diagonalization | Exp011 | `special::anderson_diagonalize` | groundSpring, hotSpring | Mar 2026 |
| Michaelis-Menten batch PK | Exp077/083 | `shaders/health/michaelis_menten_batch_f64.wgsl` | (healthSpring-specific) | Mar 2026 |
| SCFA batch production | Exp079/083 | `shaders/health/scfa_batch_f64.wgsl` | wetSpring (microbiome) | Mar 2026 |
| Beat classification batch | Exp082/083 | `shaders/health/beat_classify_batch_f64.wgsl` | (healthSpring-specific) | Mar 2026 |
| Batch IC50 compound sweep | Exp092 | Reuses `ops::HillFunctionF64` | wetSpring (ecology), neuralSpring (kinetics) | Mar 2026 |
| MATRIX panel scoring | Exp090 | `ops::bio::MatrixPanelScore` (candidate) | (healthSpring-specific) | Mar 2026 |
| Fibrotic geometry | Exp094 | Sign variant of `tissue_geometry_factor` | (healthSpring-specific) | Mar 2026 |
| Feline MM PK | Exp106 | Reuses `MichaelisMentenBatchF64` params | (healthSpring-specific) | Mar 2026 |

### healthSpring ← Other Springs (Consumed via barraCuda)

| Primitive | Origin Spring | barraCuda Path | How healthSpring Uses It |
|-----------|---------------|----------------|--------------------------|
| **df64_core.wgsl** | hotSpring S58 | `shaders/math/df64_core.wgsl` | Double-float emulation for f64 on consumer GPUs; enables precision routing for PK/PD shaders on non-native f64 hardware |
| **df64_transcendentals.wgsl** | hotSpring S60 | `shaders/math/df64_transcendentals.wgsl` | `exp`, `log`, `pow` in double-float; replaces healthSpring's f32 transcendental workarounds |
| **chi_squared_f64.wgsl** | groundSpring V74 | `shaders/special/chi_squared_f64.wgsl` | Goodness-of-fit tests for PK/PD diagnostics (CWRES, VPC, GOF) |
| **welford_mean_variance_f64.wgsl** | groundSpring V80 | `shaders/reduce/welford_mean_variance_f64.wgsl` | Numerically stable online mean/variance for biosignal streaming |
| **fused_kl_divergence_f64.wgsl** | neuralSpring V24 | `shaders/special/fused_kl_divergence_f64.wgsl` | Model selection in NLME (FOCE vs SAEM comparison) |
| **smith_waterman_banded_f64.wgsl** | wetSpring V87 | `shaders/bio/smith_waterman_banded_f64.wgsl` | Sequence alignment for microbiome 16S classification |
| **hmm_forward_f64.wgsl** | wetSpring V90 | `shaders/bio/hmm_forward_f64.wgsl` | Hidden Markov model for biosignal state detection (ECG rhythm) |
| **fused_map_reduce_f64.wgsl** | wetSpring V87 | `shaders/reduce/fused_map_reduce_f64.wgsl` | Fused map+reduce pattern for diversity index GPU computation |
| **PrecisionRoutingAdvice** | groundSpring V84-V85 | `device::driver_profile::PrecisionRoutingAdvice` | f64 shader variant selection based on GPU hardware capability |

---

## Cross-Spring Pollination Map

### hotSpring → precision shaders → everyone

hotSpring (plasma physics, lattice QCD) has the strictest precision requirements.
Its innovations propagate outward:

1. **df64_core.wgsl** (S58): Double-float arithmetic using paired f32 — enables f64
   precision on GPUs without native f64 ALUs. Consumed by ALL 5 springs.
2. **df64_transcendentals.wgsl** (S60): `exp_df64`, `log_df64`, `pow_df64` —
   DFMA polynomial approximations. Consumed by hot, wet, neural, ground.
3. **Kahan summation**: hotSpring's lattice QCD requires compensated sums.
   Pattern absorbed into barraCuda's reduce shaders, benefits neuralSpring
   attention kernels (`sdpa_scores_f64.wgsl`).
4. **FMA control / NoContraction**: hotSpring requires bit-exact CPU parity.
   Drives coralReef's `FmaPolicy` enum. Benefits groundSpring spectral methods.
5. **NVVM poisoning workarounds** (v0.6.25): hotSpring discovered that NVIDIA's
   NVVM optimizer corrupts f64 computations with FMA fusion. Drove coralReef's
   `nvvm_poison_*` test fixtures and `FmaPolicy::NoContraction`.

### wetSpring → bio shaders → neuralSpring

wetSpring (agriculture, environmental science) pioneered biological computation
patterns that cross-pollinate:

1. **Smith-Waterman alignment**: Banded GPU sequence alignment. Consumed by
   neuralSpring for protein structure (pangenome classification).
2. **Gillespie SSA**: Stochastic simulation algorithm for chemical kinetics.
   Consumed by neuralSpring for directed evolution simulations.
3. **HMM forward**: Hidden Markov model forward pass. Consumed by neuralSpring
   for genomic annotation, and applicable to healthSpring biosignal rhythm detection.
4. **diversity_fusion**: Shannon + Simpson + Pielou in one GPU dispatch.
   Originated from healthSpring diversity indices, enriched by wetSpring's
   16S microbiome pipeline.
5. **Bray-Curtis**: Beta diversity metric. Absorbed from wetSpring (Feb 2026),
   shared with healthSpring gut microbiome analysis.

### neuralSpring → statistics shaders → hotSpring, groundSpring

neuralSpring (neural networks, directed evolution) contributes statistical
primitives that benefit physics:

1. **matrix_correlation_f64.wgsl** (S69): Pairwise correlation matrix.
   Consumed by groundSpring (uncertainty analysis), hotSpring (lattice
   observable correlations).
2. **fused_kl_divergence_f64.wgsl** (V24): KL divergence in single dispatch.
   Consumed by wetSpring (cross-entropy), groundSpring (fitness landscape).
3. **fused_chi_squared_f64.wgsl** (V24): Chi-squared test. Consumed by
   hotSpring (lattice statistics), wetSpring (GOF tests).
4. **batch_ipr_f64.wgsl** (V128): Inverse participation ratio. Consumed by
   hotSpring (localization diagnostics). healthSpring uses IPR for Anderson
   localization in gut microbiome models.
5. **ESN readout**: Echo state network training. Originated in hotSpring
   (Stanton-Murillo transport), absorbed by neuralSpring for ML inference.

### groundSpring → spectral shaders → ALL

groundSpring (geophysics, condensed matter) contributes universal primitives:

1. **anderson_lyapunov_f64.wgsl** (V74): Anderson localization Lyapunov
   exponent. Consumed by hotSpring and neuralSpring for disorder physics.
2. **chi_squared_f64.wgsl** (V74): Universal chi-squared test. Consumed
   by ALL 5 springs.
3. **welford_mean_variance_f64.wgsl** (V80): Numerically stable online
   statistics. Consumed by ALL 5 springs.
4. **PrecisionRoutingAdvice** (V84-V85): Discovered f64 shared-memory
   reduction bug on certain GPUs. Led to toadStool S128's precision
   routing enum. Benefits ALL springs doing f64 GPU compute.

### airSpring → hydrology shaders → wetSpring

airSpring (atmospheric science, IoT) contributes environmental computation:

1. **Hargreaves ET₀**: Reference evapotranspiration. Consumed by wetSpring
   for crop water balance.
2. **seasonal_pipeline.wgsl**: Multi-stage seasonal computation. Consumed
   by wetSpring for growing season analysis.
3. **moving_window_f64.wgsl**: Rolling statistics for sensor streams.
   Consumed by wetSpring (soil moisture) and neuralSpring (streaming inference).

---

## How healthSpring Benefits from the Ecosystem

| healthSpring Domain | What We Get | From Where |
|---------------------|-------------|------------|
| PK/PD modeling | Precision routing (f64/DF64/f32 shaders per hardware) | hotSpring → groundSpring → toadStool |
| PK/PD modeling | Chi-squared GOF tests on GPU | groundSpring → neuralSpring |
| Microbiome analytics | Smith-Waterman for 16S classification | wetSpring |
| Microbiome analytics | HMM for microbiome dynamics | wetSpring → neuralSpring |
| Biosignal processing | Welford online statistics for streaming ECG | groundSpring |
| Biosignal processing | KL divergence for rhythm classification | neuralSpring |
| GPU dispatch | Double-float emulation on consumer GPUs | hotSpring |
| GPU dispatch | NVVM poisoning protection | hotSpring → coralReef |
| Model selection | Fused KL divergence for FOCE vs SAEM | neuralSpring |
| All domains | Universal chi-squared test | groundSpring |

---

## coralReef Compilation Coverage

coralReef can compile cross-spring WGSL shaders to SM70 SASS.
healthSpring's 6 shaders are in the compilation corpus:

| Shader | coralReef Status | Notes |
|--------|------------------|-------|
| `hill_dose_response_f64.wgsl` | Compiles to SM70 | f32 transcendental intermediates |
| `population_pk_f64.wgsl` | Compiles to SM70 | u32 PRNG, no SHADER_INT64 |
| `diversity_f64.wgsl` | Compiles to SM70 | Workgroup reduction |
| `michaelis_menten_batch_f64.wgsl` | Compiles to SM70 | df64 ODE per workgroup item (V17) |
| `scfa_batch_f64.wgsl` | Compiles to SM70 | Element-wise Michaelis-Menten (V17) |
| `beat_classify_batch_f64.wgsl` | Compiles to SM70 | Normalized cross-correlation (V17) |

When coralReef f64 lowering replaces the f32 transcendental workarounds,
healthSpring's Hill shader will gain full f64 precision (~15 digits vs ~7).
The 3 V17 shaders already use df64 throughout.

---

## barraCuda Shader Registry

The canonical registry at `barracuda::shaders::provenance::REGISTRY` tracks
all 794 shaders with origin, consumers, and evolution notes. Query with:

```rust
use barracuda::shaders::provenance::{shaders_from, shaders_consumed_by, SpringDomain};

let contributed = shaders_from(SpringDomain::HEALTH_SPRING);
let consumed = shaders_consumed_by(SpringDomain::HEALTH_SPRING);
```

---

## V27 ODE→WGSL Codegen Absorption

V27 absorbs the `OdeSystem` trait pattern from barraCuda (validated by wetSpring).
Instead of hand-writing WGSL shaders, healthSpring now defines ODE systems as
Rust structs implementing `barracuda::numerical::OdeSystem`, and the WGSL is
generated automatically via `BatchedOdeRK4::generate_shader()`.

| ODE System | States | Params | WGSL Generated | CPU Validated |
|-----------|--------|--------|:--------------:|:------------:|
| `MichaelisMentenOde` | 1 | 3 | Yes | Yes (7 tests) |
| `OralOneCompartmentOde` | 2 | 5 | Yes | Yes |
| `TwoCompartmentOde` | 2 | 4 | Yes | Yes |

This replaces the Tier B `michaelis_menten_batch_f64.wgsl` hand-rolled shader
with a generated equivalent, and extends GPU capability to oral and two-compartment
PK models for the first time.
