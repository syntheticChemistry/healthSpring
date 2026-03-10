<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# QS Gene Profiling Design — Functional Anderson Disorder

**Last Updated**: March 10, 2026
**Status**: Design — extends baseCamp Paper 16 (Anaerobic-Aerobic QS Phase Transition)
**Depends on**: `barracuda/src/microbiome.rs`, NestGate NCBI provider, wetSpring Anderson framework

---

## Motivation

The Anderson localization model in healthSpring (Exp011–013) maps gut microbiome diversity
to colonization resistance via a single disorder parameter W, derived from Pielou evenness:

```
W = evenness_to_disorder(pielou_j)
```

This captures **structural** diversity (how evenly species are distributed) but ignores
**functional** diversity (what the species can do). Two communities with identical Pielou
evenness but different quorum sensing (QS) gene repertoires would have the same W — yet
their signaling landscapes differ fundamentally.

QS gene profiling adds a functional dimension: `W_effective` incorporates both structural
evenness and inter-species signaling capacity.

---

## QS Gene Families

### Target Families

| Family | Gene(s) | Signal Molecule | Mechanism | Gut Relevance |
|--------|---------|-----------------|-----------|---------------|
| **LuxI/LuxR** | luxI, luxR | N-acyl homoserine lactones (AHL) | Gram-negative cell-density signaling | Anaerobic gut commensals, Bacteroides signaling |
| **LuxS** (AI-2) | luxS | Autoinducer-2 (DPD-derived) | Universal inter-species signal | >50% of gut bacteria; cross-species communication |
| **Agr** | agrB, agrD, agrC, agrA | Autoinducing peptides (AIP) | Gram-positive virulence regulation | C. difficile virulence, Staphylococcus colonization |
| **Com** | comA, comB, comC, comD, comE | Competence-stimulating peptide (CSP) | Genetic competence induction | Streptococcus, horizontal gene transfer |
| **Las/Rhl** | lasI, rhlI, lasR, rhlR | 3-oxo-C12-HSL, C4-HSL | Hierarchical QS cascade | Pseudomonas (rare in healthy gut; marker of dysbiosis) |
| **Fsr** | fsrA, fsrB, fsrC | Gelatinase biosynthesis-activating pheromone | Enterococcal virulence | E. faecalis colonization |

### NCBI Data Sources

| Source | Query Strategy | Expected Volume | Fields |
|--------|---------------|-----------------|--------|
| **NCBI Gene** | `luxI[Gene Name] AND "Bacteria"[Organism]` | ~2,000 gene records per family | GeneID, Organism, Symbol, Description |
| **NCBI Protein** | BLASTP of reference QS proteins against nr | ~10,000 hits per family | Accession, Organism, E-value, Identity |
| **UniProt** | Keyword: "quorum sensing" + reviewed (Swiss-Prot) | ~3,000 entries | Entry, Organism, Gene names, EC number |
| **KEGG Orthology** | KO entries: K13060 (luxI), K10125 (luxS), etc. | ~200 KO entries | KO ID, organisms, pathways |

**Total estimated download**: ~5GB (mostly protein sequences; gene metadata is small)

---

## Data Model

### QS Gene Presence/Absence Matrix

For ~200 common human gut microbes × 6 QS gene families:

```
Species               | LuxI/R | LuxS | Agr | Com | Las/Rhl | Fsr
----------------------|--------|------|-----|-----|---------|----
Bacteroides fragilis  |   1    |  1   |  0  |  0  |    0    |  0
Clostridium difficile |   0    |  1   |  1  |  0  |    0    |  0
E. coli (commensal)   |   0    |  1   |  0  |  0  |    0    |  0
Faecalibacterium      |   0    |  1   |  0  |  0  |    0    |  0
Enterococcus faecalis |   0    |  0   |  0  |  0  |    0    |  1
...
```

### Rust Data Structures

```rust
pub struct QsGeneMatrix {
    pub species: Vec<String>,
    pub families: Vec<QsFamily>,
    pub presence: Vec<Vec<bool>>,  // species × families
}

pub enum QsFamily {
    LuxIR,
    LuxS,
    Agr,
    Com,
    LasRhl,
    Fsr,
}

pub struct QsProfile {
    pub family_densities: [f64; 6],  // fraction of community with each family
    pub total_qs_density: f64,       // fraction of community with ANY QS gene
    pub signaling_diversity: f64,    // Shannon entropy across QS families
}
```

---

## Integration with Anderson Model

### Current Pipeline

```
Community abundances → Pielou(J) → W = evenness_to_disorder(J)
    → Hamiltonian H(W) → eigensolve → IPR → ξ → CR
```

### Extended Pipeline

```
Community abundances → Pielou(J) → W_structural
Community + QS matrix → QS_density(community) → W_functional
W_effective = α·W_structural + (1-α)·W_functional
    → Hamiltonian H(W_effective) → eigensolve → IPR → ξ → CR
```

Where:
- `W_structural` = current `evenness_to_disorder()` output
- `W_functional` = QS-weighted disorder (higher QS density → lower disorder, because
  more signaling → more coordination → more "extended" community behavior)
- `α` = mixing parameter (fit from data, expected ~0.6-0.8 structural-dominant)

### New Functions in `microbiome.rs`

```rust
/// Fraction of community members carrying genes from a specific QS family.
pub fn qs_gene_density(abundances: &[f64], qs_matrix: &QsGeneMatrix, family: QsFamily) -> f64

/// Complete QS profile for a community: per-family densities, total, and diversity.
pub fn qs_profile(abundances: &[f64], qs_matrix: &QsGeneMatrix) -> QsProfile

/// Effective disorder incorporating both structural (Pielou) and functional (QS) dimensions.
/// Higher QS density reduces effective disorder (more signaling = more coordination).
pub fn effective_disorder(pielou_j: f64, qs_profile: &QsProfile, alpha: f64) -> f64
```

### Biological Prediction

- **Healthy gut** (high Pielou, high QS density) → low `W_effective` → extended states → high CR
- **Dysbiotic gut** (low Pielou, low QS density) → high `W_effective` → localized states → low CR
- **Antibiotic-treated** (low Pielou, disrupted QS) → very high `W_effective` → deeply localized → vulnerable
- **Post-FMT recovery** — QS density recovers BEFORE Pielou evenness (signaling networks
  rewire faster than abundance profiles equilibrate)

The last prediction is novel and testable: QS gene recovery kinetics differ from diversity
recovery kinetics after antibiotic perturbation.

---

## Implementation Phases

### Phase A: QS Gene Matrix Construction (Eastgate, ~2 days)

1. Query NCBI Gene for each QS family → species list
2. Cross-reference with ~200 common gut microbes (from HMP reference)
3. Build presence/absence matrix as `data/qs_gene_matrix.json`
4. NestGate caching: `ncbi:gene:{gene_id}` content-addressed storage

### Phase B: Rust Module (`microbiome.rs` extension, ~1 day)

1. Add `QsGeneMatrix`, `QsFamily`, `QsProfile` types
2. Implement `qs_gene_density()`, `qs_profile()`, `effective_disorder()`
3. Unit tests: known communities with known QS gene complements
4. Validate: healthy community should have lower `W_effective` than dysbiotic

### Phase C: Anderson Integration (Exp084, ~1 day)

1. New experiment: `exp084_qs_anderson`
2. Compare `W` (structural only) vs `W_effective` (structural + QS) for:
   - Healthy gut community
   - Dysbiotic community (C. diff dominated)
   - Post-antibiotic community
   - Post-FMT community (Exp013 extension)
3. Validate: `W_effective` should separate healthy/dysbiotic MORE than `W` alone

### Phase D: Real Data Validation (requires NestGate + Strandgate, ~1 week)

1. Download HMP 16S data via NestGate
2. Process through DADA2 → OTU tables → species abundances
3. Look up QS gene presence for each detected species
4. Compute `W_effective` for HMP samples
5. Correlate with known health/disease status

---

## Dependencies

| Component | Location | Status |
|-----------|----------|--------|
| `evenness_to_disorder()` | `barracuda/src/microbiome.rs` | Existing |
| `anderson_hamiltonian_1d()` | `barracuda/src/microbiome.rs` | Existing |
| `colonization_resistance()` | `barracuda/src/microbiome.rs` | Existing |
| Anderson eigensolver (QL) | `barracuda/src/microbiome.rs` | Existing |
| NCBI Gene API | NestGate provider | Existing in wetSpring, planned for healthSpring |
| QS gene matrix | `data/qs_gene_matrix.json` | **New** |
| `qs_gene_density()` | `barracuda/src/microbiome.rs` | **New** |
| `qs_profile()` | `barracuda/src/microbiome.rs` | **New** |
| `effective_disorder()` | `barracuda/src/microbiome.rs` | **New** |
