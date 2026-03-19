<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Sub-Thesis 06: Low-Affinity Binding, Toxicology, and Hormesis

**Track 9** — Computation as Preprocessor for Living Systems

**Faculty**: Gonzales (MSU Pharm/Tox), Lisabeth (ADDRC HTS)
**Date**: March 19, 2026
**Status**: Complete (V39) — 4 experiments, 3 modules, 65+ unit tests

---

## Thesis

Standard drug discovery optimizes for strong, specific binding. But the immune
system, gut colonization, and tissue homeostasis rely on weak, distributed,
combinatorial binding that HTS screens classify as "inactive." What if the
data we discard is the data we need?

This sub-thesis uses computation as a **preprocessor** — predicting which
low-affinity regimes are worth exploring *before* running wet lab experiments.

## Key Results

### 1. Low-Affinity Selectivity (exp097)

A compound with IC50=20µM across 15 cancer markers achieves **26× selectivity**
over normal tissue (4 markers at IC50=200µM) through coincidence detection,
not affinity. The Gini coefficient measures binding breadth: uniform binding
(Gini≈0) indicates a broad-spectrum binder; concentrated binding (Gini>0.5)
indicates traditional high-affinity.

### 2. Delocalized Toxicity (exp098)

Anderson localization distinguishes localized toxicity (strong binder →
hepatotoxicity, IPR≈0.695) from delocalized toxicity (weak distributed →
manageable load, IPR≈0.003). The body handles distributed weak binding
because per-tissue burden stays below repair capacity and clearance remains
in the linear kinetics regime.

### 3. Hormesis (exp099)

The biphasic dose-response curve — low-dose benefit, high-dose harm — appears
across biology: caloric restriction (longevity), mithridatism (poison tolerance),
pesticide hormesis (more grasshoppers), hygiene hypothesis (immune calibration).
Same mathematical shape, different parameters.

### 4. Causal Terrarium (exp111)

The biphasic curve is the terrarium glass. Inside: molecular binding → stress
pathway activation (HSP, SOD, p53, mTOR) → cellular fitness → tissue integration
→ organism fitness → population dynamics → ecosystem structure. The curve
EMERGES from the competition between saturating repair pathways and unbounded
damage accumulation.

## Modules

| Module | Functions | Tests | Experiment |
|--------|-----------|-------|------------|
| `discovery::affinity_landscape` | fractional_occupancy, composite_binding_score, cross_reactivity_matrix, low_affinity_selectivity, colonization_resistance, disorder_adhesion_profile | 15 | exp097 |
| `toxicology` | systemic_burden_score, toxicity_ipr, delocalization_advantage, biphasic_dose_response, hormetic_optimum, mithridatism, caloric_restriction, ecological_hormesis, hormesis_localization | 32 | exp098, exp099 |
| `simulation` | standard_eukaryotic_pathways, mechanistic_cell_fitness, ecosystem_simulate, causal_chain | 18 | exp111 |

## Cross-Spring Connections

| Spring | Concept | Module |
|--------|---------|--------|
| wetSpring | Colonization resistance, probiotic adhesion, `bio::hormesis`, `bio::binding_landscape` | Joint experiment |
| groundSpring | Pesticide hormesis, plant growth, soil community | `toxicology::ecological_hormesis` |
| airSpring | Environmental chemical exposure, hygiene hypothesis | `toxicology::immune_calibration` |
| All springs | Causal terrarium — multi-scale simulation | `simulation::causal_chain` |

## Testable Predictions for Wet Lab

1. **Broad-weak binders show higher cancer selectivity than narrow-strong** when
   cancer markers outnumber normal markers (coincidence detection). Testable via
   ADDRC HTS panels with deliberate inclusion of "inactive" compounds.

2. **Delocalized binding produces lower organ-specific toxicity** at equivalent
   total exposure. Testable via tissue-specific biomarker panels in preclinical.

3. **Sub-toxic exposure upregulates stress pathways** (HSP, SOD) measurably.
   Testable via qPCR on stress pathway genes at multiple dose levels.

## What This Means for wetSpring Collaboration

wetSpring V130 already built `bio::hormesis` and `bio::binding_landscape` modules.
These are the mathematical twins of healthSpring's `toxicology` and
`discovery::affinity_landscape`. The shared primitive is the biphasic
dose-response function — a strong candidate for upstream `barraCuda::bio::hormesis`.

The joint experiment sweeps a 3D parameter space (adhesion strength × species
diversity × epithelial disorder) to compute a colonization resistance surface.
healthSpring owns the organism-level PK/PD; wetSpring owns the community-level
ecology. The handoff happens through wateringHole.
