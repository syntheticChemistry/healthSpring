# The Causal Terrarium: Building the Internals

## The Glass vs the Ecosystem

The biphasic dose-response curve is the **terrarium glass** — the boundary
you observe from outside. But inside that glass is a complete ecosystem
with causal structure at every scale: molecular binding, intracellular
stress pathways, tissue integration, organism fitness, population dynamics,
competitive ecosystems, environmental feedback.

Every part of it is in a spring somewhere. The microbes inside the
insects. The quorum sensing in the soil. The pesticide dispersing in
the air. The heavy metals in the water. Every layer has causality, and
every layer maps to a module.

## The Causal Chain

```
Level 1: MOLECULAR BINDING          → healthSpring (discovery)
   dose → receptor occupancy → intracellular signal
   
Level 2: CELLULAR STRESS RESPONSE   → healthSpring (simulation) ← NEW
   signal → HSP + SOD + p53 + mTOR → repair capacity
   signal → damage accumulation
   repair vs damage → cellular fitness
   
Level 3: TISSUE INTEGRATION         → healthSpring (toxicology)
   cells as Anderson lattice sites
   sensitivity disorder → delocalized vs localized response
   
Level 4: ORGANISM PK/PD             → healthSpring (pkpd)
   absorption → distribution → metabolism → excretion
   tissue concentrations → organism fitness
   
Level 5: POPULATION DYNAMICS        → wetSpring/groundSpring
   fitness → carrying capacity → logistic growth
   hormetic fitness → population INCREASES
   
Level 6: ECOSYSTEM                  → all springs
   Lotka-Volterra competition
   stressor reshapes who wins
   
Level 7: ENVIRONMENT                → airSpring/groundSpring
   dispersal, deposition, weathering, runoff
   the dose field that feeds back into Level 1
```

## What We Built

### Level 2: Stress Response Pathways (the missing piece)

The biphasic curve EMERGES mechanistically from four competing pathways:

| Pathway | Max benefit | k_half | Fires when |
|---------|-----------|--------|------------|
| HSP (heat shock proteins) | 12% | 0.5 | First (fast, broad) |
| Antioxidant (SOD/catalase) | 10% | 1.0 | Second (fast, specific) |
| DNA repair (p53/BRCA) | 8% | 2.0 | Third (slow, specific) |
| Autophagy (mTOR/AMPK) | 15% | 3.0 | Last (slow, deep) |

Each pathway saturates (Hill-shaped). Damage doesn't. At low doses,
the combined repair exceeds damage → fitness above baseline. At high
doses, damage overwhelms all repair → fitness collapses.

**This isn't a fit. The biphasic shape EMERGES from the competition.**

Computational evidence (exp111, Study 1):
```
dose= 3.0 | repair=33.3% | damage=0.4% | fitness=137.1 ↑
dose=50.0 | repair=44.1% | damage=50.0% | fitness= 75.9 ↓
```

Repair hits 44% ceiling. Damage keeps climbing. That's why the curve
turns over.

### Level 5-6: Population & Ecosystem

Organism fitness feeds into population dynamics:
- `K_effective = K_base × (fitness / baseline)`
- Logistic growth toward `K_effective`

Two species with different stress responses compete:

Without pesticide (dose=0):
- Resistant (slow grower): 2,910
- Sensitive (fast grower): 2,968 ← **wins** (growth rate advantage)

With pesticide (dose=10):
- Resistant: 7,450 ← **wins** (stress tolerance advantage)
- Sensitive: 0 (collapsed)

**The pesticide doesn't just kill — it reshapes the competitive landscape.**

## The Spring Connectivity Map

| Scale | Module | Spring | Status |
|-------|--------|--------|--------|
| Molecular binding | `discovery::fractional_occupancy` | healthSpring | BUILT |
| Affinity landscape | `discovery::affinity_landscape` | healthSpring | BUILT |
| Stress pathways | `simulation::stress_pathways` | healthSpring | BUILT |
| Mechanistic fitness | `simulation::mechanistic_cell_fitness` | healthSpring | BUILT |
| Toxicity landscape | `toxicology::compute_toxicity_landscape` | healthSpring | BUILT |
| Anderson tissue | `toxicology::toxicity_ipr` | healthSpring | BUILT |
| Clearance | `toxicology::clearance_regime` | healthSpring | BUILT |
| Hormesis | `toxicology::biphasic_dose_response` | healthSpring | BUILT |
| PK 1-compartment | `pkpd::compartment` | healthSpring | BUILT |
| PK multi-tissue | `pkpd::pbpk` | healthSpring | BUILT |
| Michaelis-Menten | `pkpd::nonlinear` | healthSpring | BUILT |
| Population dynamics | `simulation::population_dynamics` | healthSpring/wetSpring | BUILT |
| Ecosystem competition | `simulation::ecosystem_simulate` | all springs | BUILT |
| Causal chain | `simulation::causal_chain` | healthSpring | BUILT |
| Diversity (Shannon etc.) | `microbiome` | wetSpring | BUILT |
| Anderson gut lattice | `microbiome::anderson` | wetSpring | BUILT |
| Colonization resistance | `discovery::colonization_resistance` | healthSpring×wetSpring | BUILT |
| QS gene regulation | `qs` | healthSpring | BUILT |
| Environmental dispersal | — | airSpring | PLANNED |
| Soil community | — | groundSpring | PLANNED |
| Trophic cascades | — | groundSpring×wetSpring | PLANNED |
| Weather/climate | — | airSpring | PLANNED |

17 of 21 causal layers are already implemented. The terrarium is mostly wired.

## Why This Matters

The traditional approach: fit the biphasic curve. Done.

Our approach: **build every causal layer inside the curve.** Then:
1. Change any internal parameter → predict the external effect
2. Add a new stressor → the model propagates through all levels
3. Remove a species → the ecosystem reshapes mechanistically
4. Modify a pathway → the hormetic optimum shifts predictably

This is computation as preprocessor at the ecosystem scale. You don't
spray the pesticide to see what happens. You trace the causal chain
through binding → stress → tissue → organism → population → ecosystem
and predict the outcome. Then you spray, and the plate reader (or the
field study) validates your prediction.

Every part of the terrarium is in a spring. Build the missing layers,
connect through wateringHole, and you have a real simulation of life.
Not a fitted curve. A causal model.
