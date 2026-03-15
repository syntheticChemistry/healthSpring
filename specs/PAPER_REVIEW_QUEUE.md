<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring Paper Review Queue

**Last Updated**: March 15, 2026
**Status**: V25 — 73 experiments complete (Tracks 1–7), 501 library tests + 173 validation checks, 55+ wired JSON-RPC capabilities. All 30 Track 1–5 papers complete. Track 7 DD-001–DD-005 complete. Track 6 CM-001–CM-007 complete. New modules: `discovery/fibrosis`, `comparative/feline`.

---

## Queue Summary

| Track | Papers Queued | Started | Complete |
|-------|:------------:|:-------:|:--------:|
| Track 1: PK/PD | 7 | 7 | 7 |
| Track 2: Microbiome | 7 | 7 | 7 |
| Track 3: Biosignal | 6 | 6 | 6 |
| Track 4: Endocrinology | 9 | 9 | 9 |
| Validation | 1 | 1 | 1 |
| **Tracks 1–5 Total** | **30** | **30** | **30** |
| Track 6: Comparative Medicine | 8 | 3 | 3 |
| Track 7: Drug Discovery | 7 | 4 | 4 |
| **Grand Total** | **45** | **37** | **37** |

---

## Completed

| ID | Paper | Experiment | Python | Rust Binary | Tier |
|----|-------|-----------|:------:|:-----------:|:----:|
| PK-007 | Gonzales oclacitinib → human JAK inhibitor PK | Exp001 | 19 | 18 | 0,1,2 |
| PK-001 | Rowland & Tozer Ch. 3 — one-compartment PK | Exp002 | 12 | 18 | 0,1 |
| PK-001 | Rowland & Tozer Ch. 19 — two-compartment PK | Exp003 | 15 | 11 | 0,1 |
| PK-006 | Gonzales lokivetmab → human mAb PK | Exp004 | 12 | 7 | 0,1 |
| PK-002 | Mould & Upton 2013 — Population PK Monte Carlo | Exp005 | 15 | 12 | 0,1,2 |
| PK-004 | Gabrielsson & Weiner PBPK compartments | Exp006 | 13 | 13 | 0,1 |
| MB-007 | wetSpring 16S → diversity indices | Exp010 | 14 | 12 | 0,1,2 |
| MB-006 | wetSpring Anderson soil → gut colonization | Exp011 | 12 | 14 | 0,1 |
| MB-001 | Jenior et al. C. diff metabolic modeling | Exp012 | 10 | 10 | 0,1 |
| MB-002 | McGill synthetic microbiota for rCDI | Exp013 | 12 | 12 | 0,1 |
| BS-001 | Pan & Tompkins QRS detection | Exp020 | 12 | 12 | 0,1 |
| BS-002 | MIT-BIH HRV metrics (SDNN, RMSSD, pNN50) | Exp021 | 10 | 10 | 0,1 |
| BS-003 | Wearable PPG → SpO2 via R-value calibration | Exp022 | 11 | 11 | 0,1 |
| BS-004 | Multi-channel biosignal fusion (ECG + PPG + EDA) | Exp023 | 11 | 11 | 0,1 |
| EN-001 | Mok Ch.11 + Shoskes 2016 — IM testosterone PK | Exp030 | 12 | 11 | 0,1 |
| EN-002 | Testopel label + Cavender 2009 — Pellet PK | Exp031 | 10 | 10 | 0,1 |
| EN-003 | Harman 2001 (BLSA) — Age testosterone decline | Exp032 | 10 | 8 | 0,1 |
| EN-004 | Saad 2013 registry — TRT weight trajectory | Exp033 | 10 | 7 | 0,1 |
| EN-005 | Saad 2016 + Sharma 2015 — TRT cardiovascular | Exp034 | 10 | 10 | 0,1 |
| EN-006 | Kapoor 2006 + Dhindsa 2016 — TRT diabetes | Exp035 | 10 | 10 | 0,1 |
| EN-007 | Composite registries — Population TRT Monte Carlo | Exp036 | 12 | 10 | 0,1 |
| EN-008 | Cross-track D1/D2 — Testosterone-gut axis | Exp037 | 12 | 10 | 0,1 |
| EN-D3 | Mok D3: HRV × TRT cardiovascular cross-track | Exp038 | 10 | 10 | 0,1 |
| PK-003 | Rowland & Tozer Ch. 20 — Michaelis-Menten nonlinear PK | Exp077 | 10 | 12 | 0,1 |
| MB-003 | Dethlefsen & Relman 2011 — Antibiotic perturbation recovery | Exp078 | 10 | 10 | 0,1 |
| MB-004 | den Besten 2013 / Cummings 1987 — SCFA production | Exp079 | 10 | 10 | 0,1 |
| MB-005 | Yano 2015 / Clarke 2013 — Gut-brain serotonin pathway | Exp080 | 10 | 10 | 0,1 |
| BS-005 | Boucsein 2012 — EDA autonomic stress detection | Exp081 | 11 | 11 | 0,1 |
| BS-006 | MIT-BIH / AAMI EC57 — Arrhythmia beat classification | Exp082 | 12 | 12 | 0,1 |
| VAL-001 | barraCuda CPU parity — analytical contracts | Exp040 | 15 | 15 | 0,1 |
| DD-001 | Fajgenbaum 2018 — Anderson-augmented MATRIX scoring | Exp090 | 15 | 15 | 0,1 |
| DD-002 | Lisabeth 2024 — ADDRC HTS analysis pipeline | Exp091 | 12 | 12 | 0,1 |
| DD-003 | ADDRC compound library — batch IC50 profiling | Exp092 | 12 | 12 | 0,1 |
| DD-004 | ChEMBL JAK inhibitor bioactivity panel | Exp093 | 21 | 21 | 0,1 |
| CM-001 | Gonzales 2013 — Canine IL-31 serum kinetics | Exp100 | 14 | 14 | 0,1 |
| CM-002 | Gonzales 2014 — Oclacitinib JAK1 selectivity | Exp101 | 15 | 15 | 0,1 |
| CM-005 | Cross-species allometric PK bridge | Exp104 | 15 | 15 | 0,1 |

---

## NLME Population PK Validation (V14)

| ID | Paper / Reference | Experiment | Checks | Tier |
|----|-------------------|-----------|:------:|:----:|
| NLME-001 | Beal & Sheiner (NONMEM) — FOCE estimation | Exp075 | 19 | 0,1 |
| NLME-002 | Kuhn & Lavielle (Monolix) — SAEM estimation | Exp075 | (included above) | 0,1 |
| NCA-001 | Gabrielsson & Weiner — NCA (λz, AUC∞, MRT, CL, Vss) | Exp075 | (included above) | 0,1 |
| WFDB-001 | Goldberger et al. — PhysioNet WFDB format specification | Exp076 | structural | 0,1 |
| PIPE-001 | Full pipeline petalTongue scenario validation | Exp076 | 197 | 0,1 |

---

## GPU Tier 2 Validation (V8-V12)

Papers PK-007, PK-002, and MB-007 are validated at Tier 2 via WGSL shaders:

| Paper | GPU Shader | Parity | Experiment |
|-------|-----------|--------|-----------|
| PK-007 (Hill dose-response) | `hill_dose_response_f64.wgsl` | < 1e-4 | Exp053, Exp060 |
| PK-002 (Population PK) | `population_pk_f64.wgsl` | < 1e-4 | Exp053, Exp060 |
| MB-007 (Diversity indices) | `diversity_f64.wgsl` | < 1e-4 | Exp053, Exp060 |

CPU vs GPU parity matrix (Exp060): 27/27 checks across 3 kernels × 3 scales.

---

## Mixed Hardware Dispatch (V8-V12)

metalForge NUCLEUS topology validation:

| Validation | Experiment | Checks |
|-----------|-----------|--------|
| NUCLEUS dispatch routing | Exp061 | 22/22 |
| PCIe P2P transfer planning | Exp062 | 26/26 |
| PCIe P2P bypass (NPU→GPU) | Exp070 | structural |
| Mixed system pipeline (CPU+GPU+NPU) | Exp071 | structural |
| toadStool dispatch matrix | Exp069 | structural |

---

## Cross-Track: Gut–Digester Analogy (baseCamp Paper 16)

The gut is an anaerobic digester. The same microbial ecology that determines
biogas yield in an ADREC digester determines nutrient extraction, immune
modulation, and pathogen resistance in the gut. Experiments Exp011 (Anderson
gut colonization), Exp012 (C. diff metabolic modeling), Exp013 (synthetic
microbiota for rCDI), and Exp037 (testosterone-gut axis) all model the
colon as a largely anaerobic microbial ecosystem.

baseCamp Paper 16 (Anaerobic-Aerobic QS Phase Transition) formalizes this:
the Anderson disorder parameter W at the gut mucosal surface (aerobic) differs
from W in the lumen (anaerobic). The oxygen gradient creates a spatial
gradient in QS propagation. healthSpring's Anderson gut lattice (Exp011)
provides the test case; wetSpring Track 6 (Liao/ADREC) provides the
controlled anaerobic comparison.

Faculty anchor: Wei Liao (ADREC, MSU BAE). See `whitePaper/attsi/non-anon/contact/liao/README.md`.

---

## Track 6: Comparative Medicine / One Health (NEW — V21)

The **causal inversion principle**: study disease directly in animal models for their
own sake to gain causal understanding, then apply species-agnostic mathematics to
humans. Dogs with naturally occurring atopic dermatitis give better causal data than
testing human drugs on animals without causality.

### Queued Papers — Comparative Medicine

| ID | Paper / System | What to Validate | Priority |
|----|---------------|-----------------|----------|
| CM-001 | Gonzales 2013 — IL-31 elevated in AD dog serum | Canine IL-31 serum kinetics, pruritus dose-response (species-native model) | **P0** |
| CM-002 | Gonzales 2014 — Oclacitinib JAK1 selectivity | Canine IC50 panel as independent validation (not just human bridge) | **P0** |
| CM-003 | Gonzales 2016 — IL-31 pruritus model in beagles | Time-course pruritus recovery, treatment comparison (canine-native) | **P1** |
| CM-004 | Fleck/Gonzales 2021 — Lokivetmab dose-duration | mAb PK in dogs: onset 3hr, duration 14/28/42 days dose-dependent | **P1** |
| CM-005 | Cross-species allometric PK bridge | Species-agnostic PK refactor: canine ↔ human ↔ feline parameter scaling | **P1** |
| CM-006 | Canine gut microbiome + AD severity | Cross-species Anderson: dog gut Pielou → W → colonization resistance | **P2** |
| CM-007 | Feline hyperthyroidism PK (methimazole) | Species extension: capacity-limited PK in cats (MM kinetics) | **P2** |
| CM-008 | Equine laminitis inflammatory cascade | Multi-species tissue Anderson: hoof lamellae as lattice substrate | **P3** |

### Principle: Species-Agnostic Mathematics

The Hill equation, Anderson localization, Bateman PK, and Shannon diversity are
species-invariant mathematical structures. What changes between species is parameters:
- `IC50_canine` vs `IC50_human` (same Hill equation)
- `CL_canine` vs `CL_human` (same compartment ODE, allometric scaling)
- `W_gut_canine` vs `W_gut_human` (same Anderson Hamiltonian, different Pielou input)

healthSpring Track 6 validates the math on animal models directly, gaining causal
insight from species with naturally occurring disease, then translates via parameter
substitution — not by testing human therapies on animals.

---

## Track 7: Drug Discovery / ADDRC / MATRIX (NEW — V21)

Front-loaded for Gonzales/Lisabeth/ADDRC meeting (March 2026). The pipeline:
```
healthSpring MATRIX scoring → Lisabeth ADDRC HTS → Gonzales iPSC validation → Ellsworth med chem → Preclinical
```

### Queued Papers — Drug Discovery (FRONT-LOADED)

| ID | Paper / System | What to Validate | Priority |
|----|---------------|-----------------|----------|
| DD-001 | Fajgenbaum 2018 — MATRIX drug repurposing framework | Anderson-augmented MATRIX scoring for AD targets | **P0 — FRONT** |
| DD-002 | Lisabeth et al. 2024 — Brucella host-cellular small molecule screen | ADDRC HTS data analysis pipeline, hit scoring | **P0 — FRONT** |
| DD-003 | ADDRC compound library (8,000 compounds) | IC50/EC50 batch computation, Anderson geometry scoring | **P0 — FRONT** |
| DD-004 | ChEMBL JAK inhibitor bioactivity panel | 50+ compound IC50/Ki sweep, selectivity profiling across kinases | **P1** |
| DD-005 | Neubig — Rho/MRTF/SRF inhibitors for skin fibrosis | Fibrosis ↔ AD barrier model cross-talk, Anderson scoring | **P1** |
| DD-006 | iPSC skin model validation protocol | Gonzales iPSC viability/cytokine readout → computational validation | **P2** |
| DD-007 | Ellsworth — Niclosamide delivery optimization | Medicinal chemistry optimization downstream of HTS hits | **P2** |

### ADDRC Meeting Preparation — Deliverables

For the Gonzales/Lisabeth meeting this week:

1. **Anderson-augmented MATRIX**: Show how Anderson geometry scoring identifies promising
   candidates from the ADDRC 8,000-compound library by predicting which compounds create
   "extended states" (good drug penetration) vs "localized states" (poor tissue distribution)
2. **Species-agnostic validation**: 688/688 checks across wetSpring + neuralSpring prove
   the canine models are mathematically faithful — the ADDRC screening benefits from
   disease biology studied in dogs for dogs
3. **GPU-accelerated screening**: Population PK on GPU (100K virtual patients per compound)
   enables rapid triage of the 8,000-compound library
4. **Pipeline integration**: healthSpring computational models → ADDRC HTS → Gonzales iPSC → Ellsworth med chem

### Scaling Context — vs. Every Cure MATRIX ($48.3M ARPA-H)

Every Cure scores ~3K drugs × ~12K diseases = 36M pairs (human only, pathway + ML).
healthSpring extends with Anderson physics, species-agnostic scoring, and GPU population PK:

- Full Every Cure scale + 5 species + 20 tissue geometries = **3.6B scored combinations**
- Population PK for top 1% (100K virtual patients each) = **36B PK evaluations**
- Total compute time on single RTX 5090: **under 1 minute**
- Data pipeline (ChEMBL + NCATS Translator + FDA CVM) needed, not compute

See `whitePaper/baseCamp/drug_matrix_comparison.md` for the full comparative analysis.

---

## Next Evolution Targets

All 30 queued papers complete at Tier 0+1. Six have Tier 2 GPU validation. Full-stack portability proven (V19). Next:

1. ~~**barraCuda CPU parity benchmarks**~~: **DONE** (V18) — Exp084, Rust 84× faster
2. ~~**GPU scaling + fused pipeline**~~: **DONE** (V19) — Exp085, linear scaling confirmed at 4 scales
3. ~~**toadStool V16 dispatch**~~: **DONE** (V19) — Exp086, streaming + callbacks + GPU-mappability
4. ~~**metalForge NUCLEUS V16 dispatch**~~: **DONE** (V19) — Exp087, PCIe P2P bypass, mixed pipeline
4b. ~~**petalTongue V16 visualization**~~: **DONE** (V20) — Exp088 (326/326), Exp089 (14/14), 34-node full study, 16 scenarios
5. ~~**Track 7 DD-001–004**~~: **DONE** (V25) — Exp090–093, MATRIX scoring + HTS + compound library + ChEMBL panel
6. ~~**Track 6 CM-001–007**~~: **DONE** (V25) — Exp100–106, canine IL-31/JAK1/pruritus/lokivetmab/gut + feline MM PK + cross-species PK
6b. ~~**Track 7 DD-001–005**~~: **DONE** (V25) — Exp090–094, MATRIX/HTS/compound/ChEMBL/fibrosis
6c. **Track 7 DD-006–007**: iPSC validation, Ellsworth med chem (next)
7. **QS gene profiling**: Extend Anderson disorder with functional dimension (NCBI Gene)
8. **Species-agnostic PK refactor**: Parameterize compartment models by species
9. **GPU Tier 2**: Anderson eigensolve (Exp011) → `anderson_lyapunov_f64.wgsl`
10. **GPU Tier 2**: Biosignal FFT (Exp020-023) → GPU radix-2 FFT for real-time ECG/PPG
11. **GPU Tier 2**: Michaelis-Menten population (Exp077) → batch parallel ODE
12. **NPU Tier 3**: Pan-Tompkins streaming via Akida NPU (toadStool NPU dispatch path)
13. **NLME GPU**: FOCE per-subject gradient shader, VPC Monte Carlo shader
14. **DNA/protein integration**: neuralSpring sequence analysis → drug target genomics
15. **Cross-species microbiome**: wetSpring QS gene profiling → comparative gut Anderson
16. **biomeOS integration**: NUCLEUS local deployment graph orchestration

---

## Open Data Audit

All experiments use publicly accessible data. No proprietary dependencies.

| Experiment | Open Data Source | Access Method |
|-----------|-----------------|---------------|
| Exp001 | Published IC50 values (Gonzales 2014, FDA labels) | Literature |
| Exp002 | Textbook equations (Rowland & Tozer) | Published reference |
| Exp003 | Textbook equations (Rowland & Tozer Ch. 19) | Published reference |
| Exp004 | Fleck/Gonzales 2021 + Kabashima 2020 Phase III | Peer-reviewed papers |
| Exp005 | Published population PK parameters | Literature + simulation |
| Exp006 | Gabrielsson & Weiner PBPK model | Published tissue parameters |
| Exp010 | Synthetic communities (defined abundances) | Self-generated from literature |
| Exp011 | wetSpring validated lattice code (V99) | ecoPrimals internal |
| Exp012 | Published CDI clinical parameters | Literature |
| Exp013 | McGill FMT parameters (literature) | Peer-reviewed transplant studies |
| Exp020 | MIT-BIH Arrhythmia Database | PhysioNet (open) |
| Exp021 | Synthetic ECG (internal) | Generated from Gaussian P-QRS-T model |
| Exp022 | Beer-Lambert PPG calibration | Published SpO2 calibration curves |
| Exp023 | Synthetic multi-channel (ECG+PPG+EDA) | Internal generated signals |
| Exp030 | Shoskes 2016, Ross 2004 (IM PK) | Peer-reviewed papers |
| Exp031 | Testopel PI, Cavender 2009 | FDA label + literature |
| Exp032 | Harman 2001 (BLSA, n=890), Feldman 2002 | Peer-reviewed longitudinal |
| Exp033 | Saad 2013 registry (n=411) | Published figures |
| Exp034 | Saad 2016, Sharma 2015 (VA, n=83,010) | Published registry/cohort |
| Exp035 | Kapoor 2006 (RCT), Dhindsa 2016, Hackett 2014 | Peer-reviewed RCTs |
| Exp036 | Composite registries (simulation) | Literature parameters |
| Exp037 | Cross-track (ecoPrimals thesis) | Internal + Ridaura 2013, Tremellen 2012 |
| Exp038 | Kleiger 1987 HRV mortality + Mok TRT claims | Peer-reviewed + clinical |
| Exp040 | Analytical identities (textbook) | Mathematical definitions |
| Exp050-052 | Composite diagnostic (all tracks) | Same open sources as Tracks 1-4 |
| Exp053-055 | GPU parity (no new data) | CPU baseline as ground truth |
| Exp056 | Visualization (no new data) | Scenario builders use existing data |
| Exp060-062 | Hardware dispatch (no new data) | CPU/GPU parity as ground truth |
| Exp063-065 | Clinical TRT (published registries) | Same open sources as Track 4 |
| Exp066-072 | Compute benchmarks (no new data) | Existing pipelines, timing only |
| Exp073-074 | petalTongue evolution (no new data) | Existing scenarios + mock server |
| Exp075 | NONMEM/Monolix methods (published algorithms) | Beal & Sheiner, Kuhn & Lavielle (literature) |
| Exp076 | Full pipeline validation (no new data) | All existing track data + scenario builders |
| Exp077 | Phenytoin PK parameters (Ludden 1977) | Published textbook parameters |
| Exp078 | Ciprofloxacin perturbation (Dethlefsen & Relman 2011) | Published diversity time-course |
| Exp079 | SCFA ratios (Cummings 1987, den Besten 2013) | Published molar ratios |
| Exp080 | Serotonin production (Yano 2015, Clarke 2013) | Published tryptophan metabolism |
| Exp081 | EDA stress norms (Boucsein 2012) | Published SCR/SCL reference values |
| Exp082 | MIT-BIH beat morphology (Moody & Mark 2001) | PhysioNet (open) |

| Exp090 | Fajgenbaum 2018 + Anderson 1958 (pathway × geometry) | Published IC50 + analytical |
| Exp091 | Zhang 1999 (Z'-factor) + Lisabeth 2024 (HTS) | Published HTS methods |
| Exp092 | Hill 1910 (dose-response) + synthetic library | Analytical + literature IC50 |
| Exp093 | ChEMBL bioactivity (JAK1/2/3/TYK2) | ChEMBL (EBI, CC-BY-SA) + literature |
| Exp100 | Gonzales 2013 (IL-31 in AD dogs) | Peer-reviewed (Vet Dermatol) |
| Exp101 | Gonzales 2014 (oclacitinib JAK1) | Peer-reviewed (JVPT) + FDA CVM |
| Exp102 | Gonzales 2016 (IL-31 pruritus beagle model) | Peer-reviewed (Vet Dermatol) |
| Exp103 | Fleck/Gonzales 2021 (lokivetmab dose-duration) | Peer-reviewed (Vet Dermatol) |
| Exp104 | Mahmood 2006 (allometric exponents) | Published textbook parameters |
| Exp105 | Shannon 1948 + Anderson 1958 (canine gut diversity) | Analytical + published |
| Exp106 | Trepanier 2006 (feline methimazole PK) | Peer-reviewed (JVIM) |
| Exp083 | GPU V16 parity (no new data) | CPU baseline as ground truth |
| Exp084 | V16 CPU parity bench (no new data) | Python baseline as timing ground truth |
| Exp085 | GPU vs CPU V16 scaling bench (no new data) | CPU execute_cpu as GPU ground truth |
| Exp086 | toadStool V16 streaming dispatch (no new data) | Pipeline CPU reference as ground truth |
| Exp087 | Mixed NUCLEUS V16 dispatch (no new data) | Tower topology + PCIe Gen4 specs |
| Exp088 | Unified dashboard (no new data) | All existing scenarios |
| Exp089 | Patient explorer (no new data) | Diagnostic pipeline + V16 primitives |

See `specs/README.md` for full provenance table.
