<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# Sub-Thesis: QS Gene Profiling — Functional Anderson Disorder

**Last Updated**: March 24, 2026
**Status**: V43 — 59 genera, 6 QS families, effective disorder validated (12 unit tests); healthSpring 888 tests, 54 baselines. Cross-spring absorption; self-knowledge compliance; simulation/validation refactoring.
**Tracks**: 2 (Microbiome), 6 (Comparative Medicine), 7 (Drug Discovery)
**Cross-Paper**: Paper 01 (Anderson-QS), Paper 16 (Anaerobic-Aerobic QS)

---

## What QS Gene Profiling Adds

Anderson localization in the gut microbiome currently uses **structural** disorder — Pielou evenness J maps to disorder parameter W. Two communities with identical evenness but different signaling repertoires are treated as equivalent.

QS gene profiling adds a **functional** dimension: the fraction of community members carrying quorum sensing genes. This captures inter-species coordination capacity — a biologically meaningful property that structural diversity alone misses.

```
Current:  abundances → Pielou(J) → W → Hamiltonian → eigensolve → ξ → CR
Extended: abundances + QS matrix → W_effective → Hamiltonian → eigensolve → ξ → CR
```

Where:
```
W_effective = alpha * W_structural + (1 - alpha) * W_functional
W_structural = J * w_scale
W_functional = (1 - total_qs_density) * w_scale
```

Higher QS density means more inter-species signaling — lower functional disorder — more "extended" community behavior (analogous to metallic conduction in physics).

---

## Current Implementation

### QS Gene Matrix (59 genera, 6 families)

The matrix is built by `data/fetch_qs_genes.py` from NCBI Gene and UniProt. Each row is a genus; each column is a QS gene family.

| Family | Signal | Gut Relevance |
|--------|--------|---------------|
| LuxI/LuxR | AHL | Gram-negative cell-density sensing |
| LuxS (AI-2) | DPD-derived | Universal inter-species (>50% of gut bacteria) |
| Agr | AIP | *C. difficile* virulence regulation |
| Com | CSP | *Streptococcus* competence |
| Las/Rhl | HSL cascade | *Pseudomonas* pathogenesis (dysbiosis marker) |
| Fsr | GBAP | *Enterococcus* virulence |

### Phyla Covered

- **Firmicutes** (26 genera): Clostridium, Clostridioides, Faecalibacterium, Roseburia, Eubacterium, Ruminococcus, Coprococcus, Dorea, Blautia, Lachnospira, Anaerostipes, Butyrivibrio, Lactobacillus, Limosilactobacillus, Lacticaseibacillus, Lactiplantibacillus, Enterococcus, Streptococcus, Staphylococcus, Peptoclostridium, Peptostreptococcus, Veillonella, Megasphaera, Dialister, Acidaminococcus, Christensenella
- **Bacteroidetes** (8 genera): Bacteroides, Prevotella, Parabacteroides, Alistipes, Porphyromonas, Barnesiella, Odoribacter, Tannerella
- **Actinobacteria** (6 genera): Bifidobacterium, Collinsella, Eggerthella, Actinomyces, Atopobium, Slackia
- **Proteobacteria** (15 genera): Escherichia, Klebsiella, Enterobacter, Citrobacter, Proteus, Salmonella, Shigella, Campylobacter, Helicobacter, Desulfovibrio, Bilophila, Sutterella, Parasutterella, Pseudomonas, Acinetobacter
- **Verrucomicrobia** (1): Akkermansia
- **Fusobacteria** (1): Fusobacterium
- **Synergistetes** (1): Synergistes
- **Euryarchaeota** (1): Methanobrevibacter

### Core Functions (ecoPrimal/src/qs.rs)

| Function | Input | Output |
|----------|-------|--------|
| `qs_gene_density(abundances, matrix, family)` | Community + QS matrix + family | Fraction carrying that family |
| `qs_profile(abundances, matrix)` | Community + QS matrix | Per-family densities, total density, signaling diversity (Shannon) |
| `effective_disorder(pielou_j, profile, alpha, w_scale)` | Structural + functional | Combined W_effective |

### Validated Results (12 tests in qs.rs)

- Healthy gut (uniform, high QS density): high W_effective (extended states)
- Dysbiotic gut (Clostridioides dominant, Agr-heavy): lower W_effective
- Empty community: zero disorder
- Alpha clamping: [0, 1]
- Determinism: bit-identical across runs

---

## Extension Plan: 59 → 200+ Genera

### New Body Site Coverage

| Body Site | Source | New Genera | Accession |
|-----------|--------|:----------:|-----------|
| Oral microbiome | HMP1 | ~40 | PRJNA48479 |
| Skin microbiome | HMP1 | ~30 | PRJNA48479 |
| Vaginal microbiome | HMP1 | ~20 | PRJNA48479 |
| Dog gut | Published canine studies | ~40 | PRJNA322554 |
| Soil-gut crossover | wetSpring Anderson data | ~30 | Various |

### New QS Families

| Family | Signal | Relevance |
|--------|--------|-----------|
| QseBC | Epinephrine/norepinephrine | Inter-kingdom signaling (host stress → gut bacteria) |
| VqsM | Vibrio quorum signal | Cholera-associated dysbiosis |
| PqsABCDE | Pseudomonas quinolone signal | Pseudomonas pathogenesis |

### Data Pipeline

```
NestGate data.ncbi_search → NCBI Gene bulk query (200+ genera × 9 families)
    → NestGate storage.store (cache results)
    → Rebuild qs_gene_matrix.json
    → Run effective_disorder on real HMP communities
    → Validate: QS-augmented Anderson predicts colonization resistance better than structural-only
```

### Experimental Validation (Exp107)

**Hypothesis**: QS-augmented Anderson disorder (W_effective) predicts colonization resistance better than structural disorder (W_structural) alone.

**Method**:
1. Build expanded QS matrix (200+ genera, 9 families)
2. Compute W_structural and W_effective for each HMP sample
3. Compute Anderson localization length xi for both
4. Correlate xi with known clinical outcomes (healthy vs dysbiotic)
5. Compare predictive power: R-squared for W_effective vs W_structural

---

## Cross-Paper Connections

- **Paper 01 (Anderson-QS)**: Provides the theoretical framework — QS as the functional dimension of Anderson disorder
- **Paper 04 (Sentinel Microbes)**: QS-active genera as sentinel indicators
- **Paper 12 (Immunological Anderson)**: QS in immunological tissue lattices
- **Paper 16 (Anaerobic-Aerobic QS)**: QS propagation along the gut oxygen gradient

---

## Data Source Provenance

| Source | Accession | Method | Cached Via |
|--------|-----------|--------|-----------|
| NCBI Gene | Per-genus gene IDs | E-utilities ESearch + EFetch | NestGate `ncbi:gene:{id}` |
| UniProt | Per-genus protein IDs | REST API | NestGate `uniprot:{id}` |
| KEGG | Pathway maps | REST API (academic license) | NestGate `kegg:{pathway}` |
| HMP BioProject | PRJNA48479 | SRA metadata fetch | NestGate `ncbi:sra:{SRR}` |

All data from public repositories with documented accession numbers.
