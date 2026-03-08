# healthSpring V3 → barraCuda + toadStool Evolution Handoff

**Date**: March 8, 2026
**From**: healthSpring (human health applications)
**To**: barraCuda (GPU math), toadStool (heterogeneous dispatch)
**License**: AGPL-3.0-or-later
**healthSpring Version**: V3 (Tier 0+1 complete — all 4 tracks)
**barraCuda Version**: v0.3.3 (`a898dee`)
**toadStool Version**: S130+ (`bfe7977b`)
**Supersedes**: V1 handoff (archived)

---

## Executive Summary

- healthSpring has completed **17 experiments** across 4 tracks at Tier 0 (Python) + Tier 1 (Rust CPU)
- **192 Python checks, 179 Rust binary checks, 103 Rust lib unit tests** — all green
- Zero `unsafe` code, zero clippy warnings, zero TODO/FIXME in Rust source
- **Zero barraCuda GPU primitives consumed** — entire Tier 1 is pure CPU Rust
- Track 4 (Endocrinology) adds **population Monte Carlo** and **cross-track microbiome** models
- Ready for **Tier 2 (GPU)** via barraCuda WGSL shaders and **Tier 3 (metalForge)** dispatch

---

## Part 1: What healthSpring Built (V1 → V3)

### 1.1 Track Summary

| Track | Domain | Experiments | Python | Rust Binary | Lib Tests |
|-------|--------|:-----------:|:------:|:-----------:|:---------:|
| 1 — PK/PD | Pharmacokinetics, dose-response, population | 5 (001-005) | 55 | 43 | 39 |
| 2 — Microbiome | Gut diversity, Anderson lattice, C. diff | 3 (010-012) | 33 | 27 | 12 |
| 3 — Biosignal | Pan-Tompkins QRS detection | 1 (020) | 11 | 10 | 5 |
| 4 — Endocrinology | Testosterone PK, TRT outcomes, gut axis | 8 (030-037) | 93 | 99 | 47 |
| **Total** | | **17** | **192** | **179** | **103** |

### 1.2 New Since V1 (Tracks 2-4, 12 experiments)

V1 covered 5 PK/PD experiments. V3 adds:

**Track 2 — Microbiome (3 experiments)**:
- Exp010: Shannon/Simpson/Pielou/Chao1 diversity indices
- Exp011: Anderson localization in gut lattice (1D localization length ξ)
- Exp012: C. difficile colonization resistance score

**Track 3 — Biosignal (1 experiment)**:
- Exp020: Pan-Tompkins QRS detection (bandpass + derivative + moving average + threshold)

**Track 4 — Endocrinology (8 experiments)**:
- Exp030: Testosterone PK — IM injection steady-state (weekly vs biweekly)
- Exp031: Testosterone PK — pellet depot (5-month zero-order release)
- Exp032: Age-related testosterone decline (Harman 2001 BLSA model)
- Exp033: TRT metabolic response — weight/BMI/waist (Saad 2013 registry)
- Exp034: TRT cardiovascular — lipids + CRP + BP (Sharma 2015)
- Exp035: TRT diabetes — HbA1c + insulin sensitivity (Kapoor 2006 RCT)
- Exp036: Population TRT Monte Carlo (10K virtual patients, lognormal IIV, age-adjusted)
- Exp037: Testosterone-gut axis — microbiome stratification (Pielou evenness → Anderson ξ → metabolic response)

### 1.3 Rust API Surface (`healthspring-barracuda`)

4 modules, 40+ public functions:

| Module | Functions | Unit Tests | Key Types |
|--------|:---------:|:----------:|-----------|
| `pkpd` | 18 | 39 | `LognormalParam`, `PopResult`, `ImRegimen` |
| `microbiome` | 6 | 12 | — |
| `biosignal` | 5 | 5 | — |
| `endocrine` | 19 | 47 | `ImRegimen`, `GutAxisParams` |

---

## Part 2: GPU Workload Candidates for barraCuda

### 2.1 Embarrassingly Parallel (Priority 1)

These workloads are independent per-element — ideal for GPU `@workgroup` dispatch.

| Workload | Track | N | Pattern | barraCuda Primitive |
|----------|-------|---|---------|-------------------|
| Population PK Monte Carlo | 1+4 | 10K-1M patients | Each patient = independent Bateman ODE | `FusedMapReduceF64` or custom `population_pk_sample.wgsl` |
| Hill dose-response sweep | 1 | 10K-100K concentrations | Element-wise `E_max * C^n / (C^n + EC50^n)` | `BatchedElementwiseF64` (exp, pow, div) |
| Diversity indices | 2 | 1K-10K samples | `p * ln(p)` → sum | `FusedMapReduceF64` (log-multiply-reduce) |
| Anderson gut eigensolve | 2+4 | 100-1K lattice sites | Tridiagonal eigenvalue / localization length | `Lanczos` + `SpMV` (exists in hotSpring) |

**Exp005/036 (population PK Monte Carlo) is the recommended first GPU experiment.**

### 2.2 Fused Operation Chains (Priority 2)

These decompose into existing barraCuda primitives without custom shaders:

| Chain | Decomposes To |
|-------|--------------|
| Shannon H' = -Σ p·ln(p) | `log` → `element_mul` → `negate` → `sum` |
| Simpson D = 1 - Σ p² | `square` → `sum` → `subtract_scalar` |
| AUC (trapezoidal) | `diff` → `element_mul` → `sum` (parallel prefix) |
| Bateman concentration | `exp(-k_e*t)` → `exp(-k_a*t)` → `subtract` → `scale` |

If barraCuda supports fused op chains (like wetSpring's `FusedMapReduceF64`), healthSpring can skip local WGSL entirely for these patterns.

### 2.3 ODE Solvers (Priority 3)

Complex PBPK and tissue-lattice models need GPU ODE integration. wetSpring already wrote RK4/Euler shaders. **Check absorption status**.

---

## Part 3: Absorption Candidates (healthSpring → barraCuda)

Validated local Rust functions that generalize and could be absorbed upstream:

| Function | Current Location | Generalizes To | Absorption Target |
|----------|-----------------|---------------|-------------------|
| `pk_multiple_dose` | `pkpd.rs` | Any superposition of repeated signals | `barraCuda::signal` or `pharmacology` |
| `hill_dose_response` | `pkpd.rs` | Any sigmoidal transfer function | `barraCuda::bio` or `pharmacology` |
| `allometric_scale` | `pkpd.rs` | Power-law cross-domain scaling | `barraCuda::math::scale` |
| `shannon_index` | `microbiome.rs` | Information entropy | Already in wetSpring — cross-spring reuse |
| `anderson_localization_length` | `endocrine.rs` | Power-law ξ(W) model | Extend existing Anderson in hotSpring |
| `lognormal_params` | `endocrine.rs` | Lognormal parameterization (mean + CV → μ,σ) | `barraCuda::stats::distributions` |
| `population_pk_cpu` | `pkpd.rs` | Deterministic population sweep pattern | `barraCuda::monte_carlo` template |
| `hazard_ratio_model` | `endocrine.rs` | Any exposure-outcome risk model | `barraCuda::epi` or `stats` |
| `pan_tompkins_qrs` | `biosignal.rs` | Real-time signal detection pipeline | `barraCuda::signal::detect` |

### Absorption Priority

1. `population_pk_cpu` → GPU template (highest impact, embarrassingly parallel)
2. `hill_dose_response` → GPU element-wise (universal in pharmacology)
3. `lognormal_params` → utility (needed by any population model)
4. `anderson_localization_length` → extends existing Anderson framework

---

## Part 4: What We Learned (Evolution Guidance)

### 4.1 For barraCuda

1. **PK models are embarrassingly parallel across patients** — each patient's concentration-time curve is independent. This is the ideal first GPU workload: 10K independent Bateman equations with different parameters. No inter-thread communication needed.

2. **Bateman equation does not need an ODE solver** — the IM depot model `C(t) = (D·ka)/(Vd·(ka-ke))·(exp(-ke·t) - exp(-ka·t))` is analytical. GPU only needs `exp()` and arithmetic. ODE solvers are for multi-compartment PBPK.

3. **Lognormal inter-individual variability (IIV) is standard** — every population model uses lognormal parameters (CV → μ,σ). A `lognormal_params(typical, cv)` utility in barraCuda core would benefit all springs doing Monte Carlo.

4. **Anderson localization generalizes from soil to gut to skin** — the same `ξ(W)` model with different parameters works for wetSpring soil, healthSpring gut, and Paper 12 dermal tissue. A parameterized Anderson module in barraCuda that takes substrate-specific constants would unify all three.

5. **Cross-validation Python↔Rust at f64 gives exact parity** — 17 experiments, 179 binary checks, zero numerical drift. The f64-canonical approach works. GPU tier will introduce shader fp64 variance — document tolerances per primitive.

6. **Superposition (multiple dosing) is a generic pattern** — `pk_multiple_dose` takes any single-dose function and superposes `n_doses` at `interval` spacing. This pattern (sum of time-shifted copies of a kernel) appears in signal processing, radiation dosimetry, and environmental accumulation. Worth generalizing.

### 4.2 For toadStool

1. **Population Monte Carlo is the first metalForge workload** — healthSpring's Exp005/036 scales to 100K-1M patients. toadStool should validate 10K-workgroup dispatch with per-workgroup reduction and unidirectional streaming (parameters in → statistics out, no round-trip per patient).

2. **NPU path for biosignal** — Pan-Tompkins QRS detection (Exp020) is a streaming signal processing pipeline. Akida AKD1000 is ideal: <1ms latency, microwatt power. toadStool's NPU dispatch path is the target.

3. **Clinical deployment = metalForge** — the same TRT population PK model should run on:
   - GPU (server, batch 10K patients for clinic-wide optimization)
   - CPU (desktop, single-patient dosing adjustment)
   - NPU (wearable, real-time biosignal → dosing alert)
   This is the metalForge cross-substrate value proposition.

4. **Unidirectional streaming reduces dispatch overhead** — population PK needs parameters streamed in and statistics streamed out. No intermediate readback. toadStool's streaming dispatch model is a perfect fit.

### 4.3 Cross-Track Discovery: Testosterone-Gut Axis (Exp037)

A novel hypothesis validated at Tier 0+1: gut microbiome diversity (Pielou evenness) correlates with TRT metabolic response via Anderson localization. High-diversity gut = high localization length ξ = better metabolic response to TRT.

This bridges:
- wetSpring Anderson QS (Paper 01/06)
- healthSpring gut microbiome (Track 2)
- healthSpring endocrine outcomes (Track 4)

The cross-track model in `endocrine::gut_metabolic_response` uses `anderson_localization_length` with power-law scaling, producing detectable effect sizes (Pearson r > 0.5 for evenness-response, Cohen's d > 0.8 for high/low strata).

**barraCuda implication**: the Anderson eigensolve pathway (`Lanczos` → `SpMV` → localization length) must be available as a reusable building block, not specific to any one spring's lattice model.

---

## Part 5: Status and Pins

| Component | Version | Pin |
|-----------|---------|-----|
| healthSpring | V3 | — |
| barraCuda | v0.3.3 | `a898dee` |
| toadStool | S130+ | `bfe7977b` |
| coralReef | Iteration 10 | `d29a734` |
| wetSpring | V99 | — |
| neuralSpring | V90 | — |
| groundSpring | V100 | — |
| hotSpring | v0.6.17+ | — |
| airSpring | v0.7.5 | — |

---

## Part 6: Recommended Next Steps

### For barraCuda team:

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 1 | Write `population_pk_sample.wgsl` (first healthSpring GPU experiment) | **P0** | Template: 10K workgroups × 1 patient/wg, Bateman equation, lognormal IIV |
| 2 | Confirm ODE solver absorption status (Euler, RK4 from wetSpring) | P1 | Needed for PBPK, not for analytical PK |
| 3 | Add `lognormal_params(typical, cv) → (μ, σ)` utility | P1 | Used by all population models |
| 4 | Parameterize Anderson `ξ(W)` as reusable module | P2 | Unifies soil/gut/skin substrates |
| 5 | Assess fused-op chain for Shannon/Simpson (log→mul→sum) | P2 | May eliminate need for custom diversity shaders |
| 6 | Consider `allometric_scale` in core math utilities | P3 | Universal in pharmacology |

### For toadStool team:

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 1 | Validate 10K-workgroup dispatch for population PK Monte Carlo | **P0** | Streaming: params in → stats out |
| 2 | NPU dispatch path for biosignal inference (Pan-Tompkins) | P2 | Akida AKD1000 target |
| 3 | metalForge routing: GPU (server) → CPU (desktop) → NPU (wearable) | P2 | Clinical PK cross-substrate |

---

**License:** AGPL-3.0-or-later
