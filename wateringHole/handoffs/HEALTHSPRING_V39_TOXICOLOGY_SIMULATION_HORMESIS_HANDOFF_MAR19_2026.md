<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# healthSpring V39 → toadStool / barraCuda / coralReef Toxicology, Simulation, Hormesis Handoff

**Date**: 2026-03-19
**From**: healthSpring V39
**To**: toadStool team, barraCuda team, coralReef team
**License**: AGPL-3.0-or-later
**Pins**: barraCuda v0.3.5, toadStool S158+, coralReef Phase 10 Iter 55+
**Supersedes**: HEALTHSPRING_V38_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR18_2026.md

---

## Executive Summary

- **809 tests** (up from 719), **83 experiments** (up from 79), **85 capabilities** (up from 80)
- **NEW: 3 science domains** — toxicology (Anderson delocalization, hormesis, mithridatism), simulation (multi-scale causal chain), discovery::affinity_landscape (low-affinity binding, Gini breadth)
- **5 new IPC capabilities** (science.toxicology.*, science.simulation.*)
- **53 Python baselines** with structured provenance (up from 49)
- **Cross-spring hormesis framework**: healthSpring → groundSpring → airSpring → wetSpring

---

## Part 1: New Science Modules

| Module | Functions / Types | Unit Tests |
|--------|-------------------|------------|
| **discovery::affinity_landscape** | `fractional_occupancy`, `composite_binding_score`, `cross_reactivity_matrix`, `low_affinity_selectivity`, `colonization_resistance`, `disorder_adhesion_profile`, Gini analysis | 15 |
| **toxicology** | `systemic_burden_score`, `tissue_excess_burden`, `toxicity_ipr`, `toxicity_localization_length`, `delocalization_advantage`, `clearance_regime`, `compute_toxicity_landscape`, `biphasic_dose_response`, `hormetic_optimum`, `mithridatism_adaptation`, `mithridatism_fitness`, `immune_calibration`, `caloric_restriction_fitness`, `ecological_hormesis`, `hormesis_localization` | 32 |
| **simulation** | `StressPathway`, `standard_eukaryotic_pathways`, `pathway_activation`, `damage_accumulation`, `mechanistic_cell_fitness`, `population_dynamics`, `ecosystem_simulate`, `causal_chain` | 18 |

---

## Part 2: barraCuda Absorption Candidates

**NEW from V39:**

| Local Function | File | LOC | barraCuda Target | Priority |
|----------------|------|-----|------------------|----------|
| `composite_binding_score` | `discovery/affinity_landscape.rs` | 6 | `barraCuda::bio::binding::composite_score` | P2 |
| `cross_reactivity_matrix` | `discovery/affinity_landscape.rs` | 12 | `barraCuda::bio::binding::reactivity_matrix` — batched Hill sweep over N×M | P1 |
| `systemic_burden_score` | `toxicology.rs` | 5 | `barraCuda::bio::toxicology::systemic_burden` — FusedMapReduceF64 | P2 |
| `toxicity_ipr` | `toxicology.rs` | 12 | `barraCuda::bio::toxicology::ipr` — Anderson IPR specialization | P2 |
| `biphasic_dose_response` | `toxicology.rs` | 6 | `barraCuda::bio::hormesis::biphasic` — element-wise dose sweep | P1 |
| `mechanistic_cell_fitness` | `simulation.rs` | 10 | `barraCuda::bio::simulation::cell_fitness` — batch per-pathway Hill | P1 |
| `ecosystem_simulate` | `simulation.rs` | 30 | `barraCuda::bio::ecology::lotka_volterra` — ODE batch shader | P2 |

---

## Part 2b: toadStool Absorption Candidates

- **BiomeSweep** StageOp for dose-response sweeps (hormetic curve exploration)
- **EcosystemBatch** StageOp for Lotka-Volterra competition (N species × M environments)
- **Pipeline**: `BindingSweep → ToxicityLandscape → HormesisOptimum → EcosystemPredict`

---

## Part 3: Carry-Forward Actions (from V38)

| Priority | Action |
|----------|--------|
| **P1** | `stats::sample_variance()` — healthSpring still computes manually |
| **P1** | Validate Tier B GPU parity on hardware (MM, SCFA, BeatClassify) |
| **P2** | Stabilize TensorSession API |
| **P2** | GPU FFT for >512pt workloads |
| **P2** | `stats::kahan_sum()` absorption |

---

## Part 4: Cross-Spring Connections

**NEW:**

- **healthSpring × wetSpring**: Joint low-affinity binding experiment (colonization resistance surface). wetSpring V130 already has `bio::binding_landscape` and `bio::hormesis` modules — these are the twin of healthSpring's `toxicology` and `discovery::affinity_landscape`. Coordinate shared tolerance constants and biphasic model parameters.
- **healthSpring × groundSpring**: Pesticide hormesis (`ecological_hormesis` maps to plant/soil chemistry)
- **healthSpring × airSpring**: Environmental chemical exposure, hygiene hypothesis
- **Shared mathematical primitive**: The biphasic dose-response curve. Same function, different parameters, different springs. This is **THE** candidate for a shared `barraCuda::bio::hormesis` module.

---

## Part 5: Metrics Table (V38 vs V39)

| Metric | V38 | V39 | Δ |
|--------|-----|-----|---|
| Tests | 719 | 809 | +90 |
| Experiments | 79 | 83 | +4 |
| Python baselines | 49 | 53 | +4 |
| Provenance records | 49 | 53 | +4 |
| IPC capabilities | 80 | 85 | +5 |
| Science modules | ~25 | ~28 | +3 |
| Science domains | 6 | 8 | +2 (toxicology, simulation) |
| Clippy warnings | 0 | 0 | = |
| Unsafe blocks | 0 | 0 | = |
| TODO/FIXME | 0 | 0 | = |

---

## Part 6: Ecosystem Learnings

1. **The biphasic dose-response curve is universal**: It appears in pharmacology (hormesis), ecology (pesticide), immunology (hygiene hypothesis), nutrition (caloric restriction), and toxicology (mithridatism). Same math, different parameters. This argues for a **SHARED barraCuda primitive**.

2. **Anderson localization extends naturally**: From physics → gut colonization → toxicity distribution → hormesis transition. The IPR metric is the universal bridge. barraCuda should own the Anderson eigensolver and IPR calculation.

3. **Multi-scale causal simulation maps cleanly to springs**: Molecular → cellular → tissue → organism → population → ecosystem. healthSpring owns molecular-through-organism; wetSpring/groundSpring own population-through-ecosystem. The causal chain function is a prototype for cross-spring wateringHole pipeline.

4. **"Computation as preprocessor" paradigm proved out**: Computational evidence of 26× selectivity from weak binding is a testable experimental prediction that can be sent to Gonzales/Lisabeth for wet lab validation.

---

**healthSpring V39 | 809 tests | 83 experiments | 85 capabilities | AGPL-3.0-or-later**
