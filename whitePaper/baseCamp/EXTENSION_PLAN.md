<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring Extension Plan — Datasets, New Systems, and Evolution Paths

**Last Updated**: March 18, 2026
**Status**: V38 — Deep Debt Completion Sprint. 79 experiments, 719 tests, 49 Python baselines with structured provenance registry. MCP tool definitions (23 tools). 80 capabilities.

This document surveys how each track can extend beyond the current validated experiments
using open datasets, new computational systems, cross-track integration, and
**cross-species biology**. The guiding principle is species-agnostic mathematics:
study disease in the species where it naturally occurs, extract causal understanding,
then translate via parameter substitution rather than testing human drugs on animals.

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

- **MIT-BIH** is immediately accessible via the WFDB parser (already in `ecoPrimal/src/wfdb.rs`).
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

## Track 6: Comparative Medicine / One Health Extensions (V21)

The **causal inversion principle**: study disease directly in animal models for their
own sake, gain causal understanding, then apply species-agnostic mathematics for
translation. Dogs with naturally occurring atopic dermatitis yield better mechanistic
insight than testing human drugs on healthy animals.

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **Canine AD natural history (Gonzales)** | IL-31 serum kinetics, pruritus scoring | G1–G6 published data | Low (parameters) | Low |
| **Species-agnostic PK refactor** | Canine ↔ human ↔ feline PK parameters | Gonzales + FDA CVM Green Book + literature | ~100MB | Medium |
| **Canine gut microbiome** | Dog 16S gut profiles + AD severity correlation | NCBI SRA (published canine studies) | ~10GB | **High** — Anderson pipeline |
| **Feline hyperthyroidism PK** | Methimazole capacity-limited elimination in cats | Published veterinary PK studies | Low (parameters) | Medium (MM kinetics) |
| **Equine laminitis inflammatory cascade** | Hoof lamellae tissue lattice | Published equine pathology | Low (parameters) | Medium (tissue Anderson) |
| **Cross-species microbiome comparison** | Dog/human/mouse gut Pielou + Anderson | NCBI SRA multi-species | ~30GB | **High** — comparative Anderson |
| **Veterinary drug database** | FDA CVM Green Book (approved veterinary drugs) | FDA public | ~200MB | Low (parameter extraction) |
| **Cross-species immune lattice** | Canine/human cytokine receptor densities | Published immunology | Low | Medium (tissue lattice) |

### Implementation Notes

- **Species-agnostic PK refactor** is the critical infrastructure change: parameterize all compartment
  models by species rather than hardcoding human parameters. `SpeciesParams { clearance, volume, ..}`
  makes every existing experiment reusable across species.
- **Canine gut microbiome** extends Track 2's Anderson pipeline to dog gut. Same math, different
  species parameters. Validates the gut-as-digester analogy (baseCamp Paper 16) across species.
- **Gonzales canine data** (G1–G6) was already validated in wetSpring (359/359) and neuralSpring
  (329/329). Track 6 reframes it: the canine models are not bridges to human — they are
  independent validation systems providing causal insight into AD biology.

---

## Track 7: Drug Discovery / ADDRC / MATRIX Extensions (V21 — FRONT-LOADED)

Pipeline: healthSpring MATRIX scoring → Lisabeth ADDRC HTS → Gonzales iPSC → Ellsworth med chem.
Front-loaded for Gonzales/ADDRC meeting (March 2026).

| Extension | Dataset | Source | Size | Compute |
|-----------|---------|--------|------|---------|
| **MATRIX + Anderson scoring** | Fajgenbaum MATRIX framework + Anderson geometry | nS-605 + Exp011 | Low (computed) | **High** — batch scoring |
| **ADDRC 8K compound screening** | 8,000-compound library IC50/EC50 | ADDRC (MSU collaboration) | ~500MB | **High** — GPU Hill sweep (8K × N concentrations) |
| **ChEMBL JAK inhibitor panel** | IC50/Ki/EC50 for JAK1/2/3/TYK2 across 50+ compounds | ChEMBL REST API (open) | ~2GB | Medium (batch parameter extraction) |
| **PubChem BioAssay integration** | HTS assay data for AD-relevant targets | PubChem (NCBI, open) | ~5GB | Medium (hit scoring) |
| **Lisabeth Brucella screen analysis** | Small molecule host-cellular pathway screen | Lisabeth 2024 published data | Low | Medium (pathway scoring) |
| **iPSC skin model readout** | Cytokine levels, viability assays from Gonzales iPSC | Collaboration data | Low | Low (structured analysis) |
| **Neubig Rho/MRTF/SRF inhibitors** | Fibrosis pathway inhibitors for AD barrier model | Neubig published + FibrosIX | Low (parameters) | Medium (Anderson scoring) |
| **QS-informed drug targets** | QS gene profiling → microbial drug targets | NCBI Gene + UniProt | ~5GB | Medium (matrix + Anderson) |

### Implementation Notes

- **ADDRC 8K compound screening** is the highest-impact near-term computation: run GPU Hill
  dose-response sweep for 8,000 compounds (8K × 10 concentrations × 6 cytokine targets = 480K
  evaluations). Existing `hill_dose_response_f64.wgsl` handles this directly.
- **MATRIX + Anderson scoring** extends nS-605 Fajgenbaum MATRIX: each compound gets an
  Anderson geometry score predicting tissue penetration. Compounds creating "extended states"
  (delocalized eigenstates) are predicted to distribute well across tissue compartments.
- **ChEMBL** provides the reference IC50/Ki data. NestGate fetch via REST API. Filter by
  human JAK targets, confidence score ≥ 7. Compare against ADDRC hit IC50s.
- **QS-informed drug targets**: microbial QS genes inform which microbial processes are
  druggable. QS disruption compounds (quorum quenchers) scored via Anderson framework.

### Scaling to Full Every Cure MATRIX Scope

Every Cure's MATRIX ($48.3M ARPA-H) scores ~3,000 drugs × ~12,000 diseases = ~36M pairs.
healthSpring can match and extend this scope:

| Scale Target | Pairs | GPU Time (RTX 4070) | GPU Time (RTX 5090) |
|-------------|:-----:|:-------------------:|:-------------------:|
| Every Cure equivalent (3K × 12K, human only) | 36M | ~0.2s (Hill sweep) | ~0.1s |
| + 5 species (canine, human, feline, equine, murine) | 180M | ~0.9s | ~0.4s |
| + 20 tissue geometries per disease | 3.6B | ~17s | ~7s |
| + Population PK for top 1% (100K patients each) | 36B PK evals | ~99s | ~40s |
| + Microbiome dysbiosis scoring per drug | 36M gut Anderson | ~5s | ~2s |
| **Total** | **3.6B+ scored** | **~2 min** | **~50s** |

**Data requirements to reach full scale**:
- Drug parameter database: ChEMBL REST API (open, 2M+ bioactivities) + ADDRC 8K + FDA labels
- Disease pathway profiles: NCATS Translator (open, same source as Every Cure)
- Species PK parameters: FDA CVM Green Book (veterinary) + published literature
- Disease ontology: MONDO / DO (same as Every Cure uses)

None of the blockers are algorithmic — the GPU shaders exist, the math is validated.
The bottleneck is data pipeline construction.

See [fajgenbaum/README.md](fajgenbaum/README.md) for the full comparative analysis (healthSpring vs Every Cure MATRIX).

### DNA/Protein Integration Path (future — via neuralSpring + wetSpring)

Eventually, drug targets and MATRIX scores tie to underlying genomes:

```
neuralSpring protein structure → drug binding pocket geometry
wetSpring microbial genomics   → QS gene presence/absence per species
                               ↓
healthSpring discovery::qs_drug_target → QS-informed MATRIX scoring
                               ↓
Cross-species genome comparison → ortholog mapping (canine/human/feline)
                               ↓
Species-agnostic drug target identification → ADDRC screening
```

This integration connects the drug discovery pipeline to the genomic substrate,
enabling precision targeting based on genome + microbiome + disease phenotype.

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
| **QseBC** | Epinephrine/norepinephrine | Inter-kingdom signaling | NCBI Gene |
| **VqsM** | Vibrio quorum signal | Vibrio (cholera dysbiosis) | NCBI Gene |
| **PqsABCDE** | PQS (Pseudomonas quinolone signal) | Pseudomonas pathogenesis | NCBI Gene, UniProt |

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

## NestGate Integration Plan

healthSpring's data module (`ecoPrimal/src/data/`) already implements three-tier fetch:
1. biomeOS `capability.call("data.ncbi_fetch", ...)` via Neural API
2. NestGate direct JSON-RPC (`data.ncbi_search`, `storage.store/retrieve`)
3. Local cache at `$HEALTHSPRING_DATA_ROOT/ncbi_cache/`

### Missing: Sovereign HTTP Tier

The sovereign tier (direct NCBI HTTP when no ecosystem services are available) needs `ureq` for E-utilities access:
- `efetch(db, id, api_key)` — NCBI EFetch for sequences, gene records
- `esearch(db, query, max_results)` — NCBI ESearch for accession discovery
- `sra_metadata(accession)` — SRA Run Info for 16S datasets

Content key format: `ncbi:{db}:{id}` (e.g., `ncbi:gene:12345`, `ncbi:sra:SRR000001`).

### NestGate Storage API

When NestGate is available, fetched data is cached via:
- `storage.store(key, value, family_id)` — persist content
- `storage.retrieve(key, family_id)` — fetch cached content
- `storage.exists(key, family_id)` — check cache hit

This pattern is absorbed from wetSpring V127's `ncbi/nestgate/` module.

---

## NUCLEUS Local Deployment Plan

### Phase 0 — Tower Atomic (eastGate)

```
Tower Atomic = BearDog (crypto) + Songbird (discovery) + Neural API
```

Update `graphs/healthspring_niche_deploy.toml` to bootstrap Tower Atomic. Gives:
- Automatic capability discovery (no manual socket paths)
- Encrypted IPC (BearDog lineage-based auto-trust)
- Neural API routing for any primal

### Phase 1 — Node Atomic (add northGate)

```
Node Atomic = Tower Atomic + ToadStool (compute)
```

GPU job dispatch to northGate RTX 5090 via `compute.dispatch.submit`. Precision routing: f64 on Titan V, df64 on consumer GPUs.

### Phase 2 — Nest Atomic (add westGate)

```
Nest Atomic = Tower Atomic + NestGate (storage)
```

Content-addressed storage on westGate ZFS (76TB). Blob replication. NCBI bulk downloads (~290GB) cached and replicated.

### Phase 3 — Full NUCLEUS (all gates)

```
NUCLEUS = Tower + Node + Nest + Squirrel (AI)
```

All gates orchestrated via biomeOS. D4 cross-track Monte Carlo, Anderson 3D, real-time wearable streaming.

---

## Aggregate Data and Compute Hunger

| Category | Data Size | Storage Target | Compute Target |
|----------|-----------|----------------|----------------|
| **Runs locally (Eastgate)** | <1GB | Local NVMe | All Tier 0+1, basic GPU |
| **NCBI literature (PubMed)** | ~50MB | Westgate ZFS | Strandgate EPYC |
| **NCBI 16S/SRA (human + canine + multi-species)** | ~120GB | Westgate ZFS | Strandgate EPYC + NestGate |
| **PhysioNet waveforms** | ~55GB (subset) | Westgate ZFS | Strandgate batch |
| **NCBI GEO expression** | ~50GB | Westgate ZFS | Strandgate + Northgate GPU |
| **QS gene matrix** | ~5GB | Westgate ZFS | Low (lookup + matrix) |
| **ChEMBL/DrugBank (Track 7)** | ~5GB | Westgate ZFS | Low (parameter extraction) |
| **PubChem BioAssay (Track 7)** | ~5GB | Westgate ZFS | Medium (hit scoring) |
| **ADDRC compound library (Track 7)** | ~500MB | Westgate ZFS | **High** (GPU Hill sweep) |
| **FDA CVM Green Book (Track 6)** | ~200MB | Westgate ZFS | Low (parameter extraction) |
| **Cross-species PK literature (Track 6)** | ~100MB | Westgate ZFS | Low (parameters) |
| **GPU compute (population MC)** | Computed | Northgate VRAM (32GB) | RTX 5090 |
| **GPU compute (Anderson 3D)** | Computed | Northgate/biomeGate VRAM | RTX 5090 / Titan V |
| **GPU compute (ADDRC 8K compound screen)** | Computed | Northgate VRAM | RTX 5090 |
| **Total new data** | **~290GB** | **Westgate ZFS (76TB available)** | — |

### Timeline by Hardware Phase

```
Phase 1 (Now, Eastgate only):
  - Synthetic data, published parameters
  - All existing 73 experiments (Tracks 1–7)
  - FRONT-LOAD: MATRIX + Anderson scoring (DD-001) for ADDRC meeting
  - FRONT-LOAD: ADDRC 8K compound IC50 sweep (DD-003) — GPU Hill shader ready
  - FRONT-LOAD: Lisabeth Brucella screen analysis (DD-002)
  - Reframe Gonzales canine models as Track 6 (CM-001/002) — data already validated
  - Species-agnostic PK refactor (CM-005) — infrastructure change
  - QS gene matrix building (NCBI Gene API, small)
  - ChEMBL JAK panel extraction (DD-004)
  - MIT-BIH full dataset (100MB, open, WFDB parser ready)

Phase 2 (LAN — Northgate + Eastgate):
  - Population MC 100K+ patients (RTX 5090)
  - ADDRC compound library GPU screen (8K × 480K evaluations)
  - Anderson L=1000 eigensolve (RTX 5090)
  - Cross-species gut Anderson (canine + human 16S)
  - 10G backbone: < 0.1s per dispatch

Phase 3 (LAN — add Strandgate + Westgate):
  - NCBI bulk download: 120GB in ~2min at 10GbE (expanded for multi-species 16S)
  - NestGate content-addressed caching on ZFS
  - 16S OTU tables + Anderson pipeline (human + canine + murine)
  - PhysioNet MIMIC-III batch processing (64 cores)
  - PubChem BioAssay data for ADDRC hit comparison

Phase 4 (Full mesh — all gates):
  - biomeOS NUCLEUS: Tower(Eastgate) + Node(Northgate,biomeGate) + Nest(Strandgate,Westgate)
  - D4 cross-track Monte Carlo (10K patients, multi-species)
  - Anderson 3D gut lattice (hotSpring GPU shader)
  - DNA/protein drug target integration (neuralSpring + wetSpring convergence)
  - Real-time biosignal streaming via NPU
```

---

## Priority Ordering

Sorted by scientific value / effort ratio. **Drug discovery front-loaded** for
Gonzales/ADDRC meeting (March 2026).

### Tier 1 — Front-Loaded (Drug Discovery + Comparative Medicine)

1. **DD-001 MATRIX + Anderson scoring** — nS-605 validated, extend to ADDRC compound set. Meeting deliverable.
2. **DD-002 Lisabeth Brucella screen analysis** — demonstrates HTS data analysis pipeline. Meeting deliverable.
3. **DD-003 ADDRC 8K compound screen** — GPU Hill sweep for 8,000 compounds. Meeting deliverable.
4. **CM-001/002 Gonzales canine models** — reframe existing nS-601–605 as Track 6 comparative medicine.
5. **DD-004 ChEMBL JAK panel** — 50+ compound IC50 sweep, validates Track 7 breadth.
6. **CM-005 Species-agnostic PK refactor** — infrastructure change enabling all cross-species work.

### Tier 2 — Near-Term (QS + Real Data + Comparative)

7. **QS gene matrix** — NCBI Gene API calls, small data, high scientific novelty (functional Anderson).
8. **MIT-BIH full validation** — data ready, parser ready, beat classify shader ready.
9. **CM-006 Canine gut microbiome** — cross-species Anderson pipeline.
10. **Real PK trial data** — small datasets, validates NLME against NONMEM reference.
11. **HMP 16S** — large download but well-documented pipeline. Anderson on real gut data.

### Tier 3 — Medium-Term (Data-Heavy + GPU)

12. **Dethlefsen 16S** — validates Exp078 antibiotic perturbation model against real time-series.
13. **DD-005 Neubig Rho/MRTF/SRF** — fibrosis ↔ AD barrier cross-talk.
14. **GEO androgen receptor** — larger data, validates cross-track biology.
15. **MIMIC-III waveforms** — requires credentialed access + Strandgate.

### Tier 4 — Long-Horizon (Large Compute + Integration)

16. **D4 cross-track Monte Carlo** — requires Northgate GPU.
17. **Anderson 3D gut** — requires Northgate GPU + hotSpring `BatchedEighGpu`.
18. **DNA/protein drug target integration** — neuralSpring + wetSpring convergence.
19. **Cross-species genome comparison** — ortholog mapping for drug target translation.
