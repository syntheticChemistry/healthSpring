# Joint Experiment: healthSpring × wetSpring — Colonization Resistance Surface

## Summary

Joint computational experiment exploring the manifold where gut microbiome
diversity, bacterial adhesion strength, and epithelial disorder interact to
produce colonization resistance. Combines healthSpring's binding landscape
models with wetSpring's Anderson lattice and diversity pipelines.

## Scientific Question

**How does the interplay between adhesion affinity, species diversity, and
epithelial disorder determine colonization resistance to pathogens like C. diff?**

Traditional view: strong probiotics outcompete pathogens.
Our hypothesis: many weak binders with diverse adhesion profiles create more
robust colonization resistance than few strong binders, analogous to Anderson
delocalization in disordered systems.

## Computational Model

### healthSpring Contributions
- `discovery::affinity_landscape` — binding landscape, colonization occupancy
- `discovery::affinity_landscape::disorder_adhesion_profile` — Anderson-disorder adhesion
- `discovery::affinity_landscape::colonization_resistance` — cumulative occupancy
- `microbiome::anderson_gut_lattice` — disorder-localization model
- `microbiome::cdiff_colonization` — C. diff resistance baselines

### wetSpring Contributions
- 16S diversity pipeline — real community structure
- Anderson lattice eigensolver — localization length computation
- ODE community dynamics — species competition/cooperation
- Diversity indices — Shannon, Simpson, Chao1 across disorder gradient

### Joint Computation
Sweep a 3D parameter space:
1. **Adhesion strength** (K_base): 0.1 to 100 (µM, IC50 analog)
2. **Species diversity** (N_species): 1 to 20
3. **Epithelial disorder** (W): 0.1 to 5.0 (Anderson disorder parameter)

At each point, compute:
- Colonization resistance fraction
- Localization length (Anderson)
- Shannon diversity maintained under competition
- Pathogen invasion probability

The **colonization resistance surface** is the manifold in this 3D space where
resistance exceeds 90%. The shape of this surface reveals:
- Whether diversity or adhesion strength matters more
- The critical disorder threshold where weak binders outperform strong ones
- Phase transitions between "diverse-delocalized" and "monoculture-localized" states

## Experiment Design

| ID | Spring | Description |
|----|--------|-------------|
| exp097 | healthSpring | Affinity landscape (done) |
| exp098 | healthSpring | Colonization resistance surface sweep |
| exp_joint_01 | wetSpring | Anderson lattice with adhesion-modulated hopping |
| exp_joint_02 | joint | Full 3D surface computation + phase diagram |

## Connection to Gonzales / Lisabeth

- **Lisabeth ADDRC data**: Full dose-response curves from 8K compound screen
  contain the low-affinity tail we need. Instead of discarding compounds with
  IC50 > 10 µM, compute their adhesion profiles and colonization resistance
  scores.
- **Gonzales iPSC**: iPSC skin models can validate adhesion-mediated effects —
  are compounds with broad-weak binding profiles less cytotoxic to healthy
  cells while maintaining composite activity on target cells?

## wateringHole Handoff

When ready, this becomes:
```
wateringHole/handoffs/HEALTHSPRING_V39_WETSPRING_COLONIZATION_RESISTANCE_HANDOFF_{DATE}.md
```
