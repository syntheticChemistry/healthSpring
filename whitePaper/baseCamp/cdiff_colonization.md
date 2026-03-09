<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Sub-Thesis: Anderson Localization → Gut Colonization Resistance

**Faculty**: TBD (extends wetSpring Anderson framework)
**Track**: 2 — Microbiome
**Experiments**: Exp010 (diversity indices), Exp011 (Anderson gut lattice), Exp012 (C. diff resistance), Exp013 (FMT engraftment)
**Status**: Complete — 4 experiments, 48 binary checks, 48 Python cross-validation checks. Anderson eigenvalue spectrum and per-eigenstate IPR spectrum now visualized in petalTongue (6 channels on Anderson node). GPU Tier 2 validated for diversity indices. V13: Anderson eigensolver (QL algorithm) fixes IPR to use true eigenvectors; `shannon_index` doc-test added; `evenness_to_disorder` deduplicated.
**Last Updated**: March 9, 2026

---

## Thesis

The Anderson localization framework, validated in soil microbiome (wetSpring), transfers to
human gut microbiome. Pielou evenness maps to Anderson disorder W; localization length ξ
predicts colonization resistance; extended states resist C. difficile colonization while
localized states are vulnerable. FMT engraftment models the transition from localized
(dysbiotic) to extended (healthy) states.

---

## Experiments

| Exp | Title | Key Validation |
|-----|-------|----------------|
| 010 | Diversity Indices | Shannon H′, Simpson D, Pielou J, Chao1, inverse Simpson, Anderson W across 5 community types |
| 011 | Anderson Gut Lattice | Hamiltonian construction, IPR, ξ, colonization resistance CR(ξ), level spacing ratio |
| 012 | C. diff Resistance | Pielou → W → ξ → CR pipeline; healthy gut has higher CR than dysbiotic/C. diff colonized |
| 013 | FMT Engraftment | fmt_blend(donor, recipient, engraftment); Shannon/Pielou/Bray-Curtis vs engraftment fraction |

---

## Key Results

- Healthy gut: H′ ≈ 2.2, J ≈ 0.96, extended eigenstates → high colonization resistance
- Dysbiotic gut: H′ ≈ 0.81, J ≈ 0.51, localized eigenstates → low resistance
- FMT at 70% engraftment: Shannon recovers to ~2.1, Bray-Curtis to donor < 0.15
- Anderson W maps monotonically from Pielou J via `evenness_to_disorder()`

---

## Modules

- `barracuda/src/microbiome.rs`: `shannon_index`, `simpson_index`, `pielou_evenness`, `chao1`, `evenness_to_disorder`, `anderson_hamiltonian_1d`, `inverse_participation_ratio`, `localization_length_from_ipr`, `colonization_resistance`, `fmt_blend`, `bray_curtis`
- `control/microbiome/`: Python baselines for all 4 experiments

---

## petalTongue Visualization (V12)

`scenarios::microbiome_study()` produces a 4-node scenario with:
- Bar charts: Shannon, Simpson, Pielou across community types
- Gauges: IPR, ξ, colonization resistance
- TimeSeries: Shannon and Bray-Curtis vs FMT engraftment fraction
- **V12**: Spectrum channels on Anderson node: eigenvalue distribution (Hamiltonian diagonal) and per-eigenstate IPR values — enabling direct visualization of the localization landscape

---

## Cross-Spring Lineage

- **wetSpring**: Anderson localization framework (soil microbiome → gut substrate transfer)
- **hotSpring**: Anderson precision (f64 eigenvalue convergence)
- **groundSpring**: Uncertainty budgets for colonization resistance thresholds
