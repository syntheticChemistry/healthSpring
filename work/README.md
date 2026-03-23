<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# work/ — Computational Exploration Lab

Active scientific explorations using computation as a **preprocessor** for
analysis rather than a post-processor. Each subdirectory is a research thread
that may span multiple experiments, modules, and springs.

## Philosophy

Traditional science pipeline:
```
wet lab → data → computation → analysis → paper
```

ecoPrimals pipeline:
```
computation → predict → design experiment → validate → iterate
```

Computation tells us where to look. The experiment confirms what we found.
Every exploration here produces Rust modules, validation experiments, and
Python baselines — not just documents. The documents explain *why* the code
exists and *what* it teaches us.

## Active Explorations

### `low_affinity_binding/` — The Weak Binder Thesis

What if the HTS data we discard (IC50 > 10 µM) contains the treatments
we need? Explores how deliberately weak, distributed binding can achieve
selectivity through coincidence detection, not affinity.

| Document | Purpose |
|----------|---------|
| `STUDY.md` | Thesis: computation as preprocessor, low-affinity landscape |
| `TOXICITY.md` | The body's burden: Anderson delocalization of toxicity |
| `JOINT_EXPERIMENT.md` | healthSpring × wetSpring: colonization resistance surface |
| `CROSS_SPRING_HORMESIS.md` | Hormesis unifies pesticides, CR, mithridatism, hygiene |
| `CAUSAL_TERRARIUM.md` | Mechanistic internals: 7-level causal chain from molecular to ecosystem |

#### Causal Chain Built

```
dose → binding → stress sensing → pathway activation → cellular fitness
  → tissue integration → organism fitness → population → ecosystem
```

| Scale | Spring | Module | Experiment | Status |
|-------|--------|--------|------------|--------|
| Molecular (binding) | healthSpring | `discovery::affinity_landscape` | exp097 | Complete |
| Cellular (stress) | healthSpring | `simulation` | exp111 | Complete |
| Tissue (integration) | healthSpring | `toxicology` | exp098 | Complete |
| Organism (hormesis) | healthSpring | `toxicology` | exp099 | Complete |
| Population | wetSpring, groundSpring | `simulation` (basic) | exp111 | Framework |
| Ecosystem | all springs | `simulation` (framework) | exp111 | Framework |
| Environment | airSpring | — | — | Conceptual |

#### Key Results (Computational Evidence)

- **26× selectivity** from weak binding: a compound with IC50=20µM across
  15 cancer markers achieves composite binding 26× higher than on 4 normal
  markers — from coincidence detection, not affinity.

- **Delocalization advantage**: distributed weak binding keeps per-tissue
  burden below repair capacity while strong localized binding overwhelms it.
  Anderson IPR quantifies the difference (0.003 vs 0.695).

- **Hormetic biphasic curve** emerges mechanistically from the competition
  between saturating repair pathways (HSP, SOD, p53, mTOR) and unbounded
  damage accumulation — explaining caloric restriction, mithridatism,
  pesticide hormesis, and the hygiene hypothesis with one mathematical shape.

- **Ecosystem reshaping**: weak pesticide exposure shifts competitive balance
  between resistant and sensitive species (Lotka-Volterra), potentially
  increasing pest populations through competitive release.

#### Cross-Spring Mapping

| Spring | Hormesis Example | Module |
|--------|-----------------|--------|
| healthSpring | Caloric restriction → longevity | `toxicology::caloric_restriction_fitness` |
| healthSpring | Mithridatism → poison tolerance | `toxicology::mithridatism_adaptation` |
| healthSpring | Hygiene hypothesis → immune calibration | `toxicology::immune_calibration` |
| groundSpring | Weak pesticide → more grasshoppers | `toxicology::ecological_hormesis` |
| airSpring | Radiation hormesis → DNA repair upregulation | Planned |
| wetSpring | Low-dose antibiotic → resistance selection | Planned |

## Reproducibility

Every exploration has:

1. **Rust modules** — the actual science code (`ecoPrimal/src/`)
2. **Validation experiments** — binary pass/fail checks (`experiments/`)
3. **Python baselines** — control scripts with provenance (`control/`)
4. **Provenance records** — traceable to git commit, date, command (`provenance/registry.rs`)
5. **IPC capabilities** — discoverable via JSON-RPC for niche integration

## Adding a New Exploration

1. Create `work/<topic>/STUDY.md` with thesis and design
2. Build Rust module in `ecoPrimal/src/`
3. Create validation experiment in `experiments/`
4. Write Python baseline in `control/`
5. Add provenance record to `provenance/registry.rs`
6. Wire IPC handlers in `ipc/dispatch/handlers/`
7. Update `specs/EVOLUTION_MAP.md` with GPU tier assessment
8. Update `specs/TOLERANCE_REGISTRY.md` with tolerance entries
9. Document cross-spring connections
