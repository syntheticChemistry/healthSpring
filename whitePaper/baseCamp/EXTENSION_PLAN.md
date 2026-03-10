<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Extension Plan — Datasets, New Systems, and Evolution Paths

**Last Updated**: March 10, 2026
**Status**: V19 baseline — 59 experiments, 395 tests, 6 WGSL shaders, 30/30 papers complete. Full-stack portability proven (CPU → GPU → toadStool → NUCLEUS).

This document surveys how each track can extend beyond the current validated experiments
using open datasets, new computational systems, and cross-track integration.

---

## Track 1: PK/PD Extensions

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **Michaelis-Menten population** | Phenytoin TDM (therapeutic drug monitoring) | Published TDM studies (Winter 2009) | Low (parameters) | **High** — batch ODE GPU (already shader-ready via `michaelis_menten_batch_f64.wgsl`) |
| **Drug panel expansion** | IC50/EC50 for 50+ compounds | ChEMBL (open) + DrugBank (CC-BY-NC) | ~2GB download | Low (parameter extraction) |
| **PBPK tissue expansion** | Tissue:plasma partition coefficients | PK-Sim open library | ~50MB | Medium (multi-compartment ODE) |
| **Real population PK** | MIMIC-IV clinical data (vancomycin, etc.) | PhysioNet (credentialed) | ~6GB subset | **High** — NLME on real data |
| **FDA FAERS safety signals** | Adverse event reports | openFDA API | ~10GB quarterly | Medium (text + aggregation) |

### Implementation Notes

- **ChEMBL** provides IC50/Ki data for >2M bioactivities. Filter by human target, confidence score ≥ 7.
  NestGate fetch via REST API: `https://www.ebi.ac.uk/chembl/api/data/activity.json`
- **PK-Sim** open library contains tissue Kp values for ~30 compounds. Extend Exp006 PBPK to
  multi-drug comparison using these reference partitions.
- **MIMIC-IV** requires PhysioNet credentialed access. Start with vancomycin TDM subset
  (~6GB) — AUC-guided dosing is a clinical standard. Validates NLME (Exp075) on real data.
- **openFDA FAERS** enables safety signal detection for TRT drugs (Mok Ch. 2 FDA concerns).
  Disproportionality analysis (PRR, EBGM) on adverse event/drug pairs.

---

## Track 2: Microbiome Extensions

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **Real 16S gut profiles** | Human Microbiome Project (HMP) | NCBI SRA | ~50GB | **High** — OTU tables, diversity, Anderson |
| **Antibiotic recovery 16S** | Dethlefsen time-series (ciprofloxacin) | NCBI SRA (SRP...) | ~10GB | Medium — time-series Anderson |
| **FMT donor-recipient 16S** | Published FMT trials (McGill etc.) | NCBI SRA | ~10GB | Medium — Bray-Curtis, engraftment |
| **QS gene profiling** (see Phase 3) | AHL synthase, AI-2 genes per microbe | NCBI Gene/Protein + UniProt | ~5GB | Medium — feature matrix + Anderson |
| **KEGG metabolic pathways** | SCFA production, tryptophan metabolism | KEGG REST API (academic) | ~1GB | Low — pathway mapping |
| **SILVA 16S taxonomy** | Reference taxonomy for OTU classification | SILVA (open) | ~2GB | Medium — Smith-Waterman classification |
| **Anderson 3D gut lattice** | Extend L=200 1D to 20³ 3D | Computed from 16S profiles | ~500MB VRAM | **Very High** — hotSpring `BatchedEighGpu` needed |

### Implementation Notes

- **HMP** is the gold-standard human microbiome reference. BioProject PRJNA48479 (HMP1) and
  PRJNA275349 (iHMP). Process through DADA2 → OTU tables → Shannon/Pielou → Anderson pipeline.
- **Dethlefsen 2011** (SRP...) provides the real data behind Exp078's synthetic model.
  Time-series 16S before/during/after ciprofloxacin → validate recovery dynamics.
- **QS gene profiling** extends the Anderson disorder parameter from purely structural
  (Pielou evenness) to functional (signaling capacity). See Phase 3 below.
- **Anderson 3D** requires hotSpring's `BatchedEighGpu` for the 8000×8000 Hamiltonian
  eigendecomposition. Target: Northgate RTX 5090 (32GB VRAM, ~500MB needed).

---

## Track 3: Biosignal Extensions

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **Full MIT-BIH** | 48 records × 30min × 360Hz | PhysioNet (open, WFDB ready) | ~100MB | Low — streaming |
| **MIMIC-III waveforms** | Thousands of ICU records | PhysioNet (credentialed) | ~3TB total, ~50GB subset | **High** — batch processing on Strandgate |
| **PTB Diagnostic ECG** | 549 records, 15-lead ECG | PhysioNet (open) | ~2GB | Medium |
| **Apnea-ECG** | 70 records with apnea annotations | PhysioNet (open) | ~500MB | Medium — new detection model |
| **Sleep-EDF** | 197 sleep recordings | PhysioNet (open, needs EDF parser) | ~15GB | Medium — new format + new model |

### Implementation Notes

- **MIT-BIH** is immediately accessible via the WFDB parser (already in `barracuda/src/wfdb.rs`).
  Exp082's template-matching beat classification can be validated against MIT-BIH annotations.
  GPU dispatch via `beat_classify_batch_f64.wgsl` for full-database batch.
- **MIMIC-III waveforms** provide ICU-scale biosignal data. Start with ~50GB subset of
  ECG + ABP waveforms. Requires PhysioNet credentialed access + Strandgate (256GB RAM).
- **Sleep-EDF** requires a new EDF parser (European Data Format). Pattern: streaming
  header + channel decode similar to WFDB. Sleep staging would be a new experiment.

---

## Track 4: Endocrinology Extensions

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **GEO androgen receptor** | AR gene expression across tissues | NCBI GEO | ~50GB raw | **High** — differential expression |
| **16S gut + testosterone** | Cross-study microbiome + hormone | NCBI SRA | ~10GB | **High** — Anderson + endocrine covariate |
| **dbGaP hypogonadism GWAS** | GWAS summary statistics | dbGaP (public summaries) | ~1GB | Medium — association analysis |
| **D4 cross-track Monte Carlo** | 10K patients × (PK + Anderson + metabolic) | Computed from above | ~500MB VRAM | **Very High** — needs Northgate RTX 5090 |

### Implementation Notes

- **GEO androgen receptor** datasets: search NCBI GEO for "androgen receptor" tissue expression.
  Process: GEO Series Matrix → limma-style differential expression in Rust. Validates
  tissue-specific TRT response mechanisms (Mok Ch. 4 weight, Ch. 6 cardiovascular).
- **16S gut + testosterone** — search SRA for studies with both microbiome sequencing and
  testosterone measurements. Validates cross-track hypothesis D1/D2 (Pielou → Anderson → metabolic).
- **D4 cross-track Monte Carlo** is the crown computation: 10K virtual patients, each with
  PK parameters (Track 1), gut microbiome state (Track 2), and metabolic response (Track 4).
  Requires 3 GPU shaders running in sequence per patient. Target: Northgate RTX 5090.

---

## Track 5: NLME Extensions

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **Real PK trial data** | Published popPK datasets | Monolix/NONMEM tutorials (open) | ~10MB | Medium — FOCE/SAEM on real data |
| **NLME 10K subjects** | Simulated from validated models | Computed | ~100MB VRAM | **High** — GPU FOCE gradient |
| **VPC 1K simulations** | Monte Carlo VPC | Computed | ~50MB VRAM | Medium — embarrassingly parallel |

### Implementation Notes

- **Real PK trial data** from published tutorial datasets (e.g., theophylline, warfarin).
  Validates Exp075 FOCE/SAEM on real data with known NONMEM/Monolix reference results.
- **NLME 10K** requires GPU FOCE: per-subject gradient is embarrassingly parallel.
  Each subject's individual objective function evaluated independently → reduce.
- **VPC 1K** is Monte Carlo simulation — already GPU-ready pattern (see `population_pk_f64.wgsl`).
  Extend to NLME parameterization for proper visual predictive checks.

---

## Phase 3: QS Gene Profiling Plan

baseCamp Paper 16 (Anaerobic-Aerobic QS Phase Transition) formalizes QS propagation along the
gut oxygen gradient. The extension adds a **functional dimension** to the Anderson disorder
parameter.

### QS Gene Families to Profile

| Gene Family | Signal | Microbes | Database |
|-------------|--------|----------|----------|
| **LuxI/LuxR** (AHL) | N-acyl homoserine lactones | Gram-negative anaerobes | NCBI Gene, UniProt |
| **LuxS** (AI-2) | Autoinducer-2 (universal) | Broad (>50% of gut bacteria) | NCBI Gene |
| **AgrBDCA** | Autoinducing peptide | Staphylococcus, Clostridium | UniProt |
| **ComABCDE** | Competence-stimulating peptide | Streptococcus | NCBI Gene |
| **LasI/RhlI** | 3-oxo-C12-HSL, C4-HSL | Pseudomonas | NCBI Gene |

### Integration with Anderson Model

Currently, the Anderson disorder parameter W is derived solely from Pielou evenness
(community structure). QS gene profiling adds a **functional dimension**:

```
Current:  Pielou(J) → W → Hamiltonian → eigensolve → ξ → CR
Extended: Pielou(J) + QS_density(community) → W_effective → ...
```

Where `QS_density` is the fraction of community members carrying specific QS gene families.
This creates a biologically richer disorder parameter that captures both structural diversity
and functional signaling capacity.

### Implementation Path

1. Build a QS gene presence/absence matrix from NCBI Gene for ~200 common gut microbes
2. Extend `microbiome.rs` with `qs_gene_density()` and `effective_disorder()`
3. Modify Anderson Hamiltonian to incorporate QS-weighted hopping terms
4. Validate: higher QS density should correlate with stronger inter-species signaling
   (lower effective disorder, more extended states)

**Data**: ~5GB from NCBI Gene/Protein, cached via NestGate

---

## Aggregate Data and Compute Hunger

| Category | Data Size | Storage Target | Compute Target |
|----------|-----------|----------------|----------------|
| **Runs locally (Eastgate)** | <1GB | Local NVMe | All Tier 0+1, basic GPU |
| **NCBI literature (PubMed)** | ~50MB | Westgate ZFS | Strandgate EPYC |
| **NCBI 16S/SRA** | ~80GB | Westgate ZFS | Strandgate EPYC + NestGate |
| **PhysioNet waveforms** | ~55GB (subset) | Westgate ZFS | Strandgate batch |
| **NCBI GEO expression** | ~50GB | Westgate ZFS | Strandgate + Northgate GPU |
| **QS gene matrix** | ~5GB | Westgate ZFS | Low (lookup + matrix) |
| **ChEMBL/DrugBank** | ~3GB | Westgate ZFS | Low (parameter extraction) |
| **GPU compute (population MC)** | Computed | Northgate VRAM (32GB) | RTX 5090 |
| **GPU compute (Anderson 3D)** | Computed | Northgate/biomeGate VRAM | RTX 5090 / Titan V |
| **Total new data** | **~245GB** | **Westgate ZFS (76TB available)** | — |

### Timeline by Hardware Phase

```
Phase 1 (Now, Eastgate only):
  - Synthetic data, published parameters
  - All existing 59 experiments
  - QS gene matrix building (NCBI Gene API, small)
  - ChEMBL drug panel extraction
  - MIT-BIH full dataset (100MB, open, WFDB parser ready)

Phase 2 (LAN — Northgate + Eastgate):
  - Population MC 100K+ patients (RTX 5090)
  - Anderson L=1000 eigensolve (RTX 5090)
  - 10G backbone: < 0.1s per dispatch

Phase 3 (LAN — add Strandgate + Westgate):
  - NCBI bulk download: 60GB in ~1min at 10GbE
  - NestGate content-addressed caching on ZFS
  - 16S OTU tables + Anderson pipeline
  - PhysioNet MIMIC-III batch processing (64 cores)

Phase 4 (Full mesh — all gates):
  - biomeOS NUCLEUS: Tower(Eastgate) + Node(Northgate,biomeGate) + Nest(Strandgate,Westgate)
  - D4 cross-track Monte Carlo (10K patients)
  - Anderson 3D gut lattice (hotSpring GPU shader)
  - Real-time biosignal streaming via NPU
```

---

## Priority Ordering

Sorted by scientific value / effort ratio:

1. **MIT-BIH full validation** — data ready, parser ready, beat classify shader ready. Immediate.
2. **QS gene matrix** — NCBI Gene API calls, small data, high scientific novelty (functional Anderson).
3. **ChEMBL drug panel** — API-accessible, extends Track 1 breadth with minimal compute.
4. **Real PK trial data** — small datasets, validates NLME against NONMEM reference.
5. **HMP 16S** — large download but well-documented pipeline. Anderson on real gut data.
6. **Dethlefsen 16S** — validates Exp078 antibiotic perturbation model against real time-series.
7. **GEO androgen receptor** — larger data, validates cross-track biology.
8. **MIMIC-III waveforms** — requires credentialed access + Strandgate.
9. **D4 cross-track Monte Carlo** — requires Northgate GPU.
10. **Anderson 3D gut** — requires Northgate GPU + hotSpring `BatchedEighGpu`.
