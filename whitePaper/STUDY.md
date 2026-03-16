# healthSpring: Human Health Applications of Sovereign Scientific Computing

**Version**: 0.8 (V28 — Deep Debt + Ecosystem Maturity)
**Date**: March 16, 2026

---

## Abstract

healthSpring applies the validated scientific computing pipelines of the ecoPrimals
spring ecosystem to the **health of living systems**. Where wetSpring reproduced 52
published life science papers, neuralSpring validated ML primitives against 31 scholarly
works, and groundSpring established uncertainty quantification across 35 experiments —
healthSpring builds **usable clinical applications** from those validated foundations,
and extends them across species boundaries.

Five completed domains and two new domains define the scope:

1. **Pharmacokinetic/pharmacodynamic modeling** — Pure Rust tools for drug dosing,
   population PK simulation, and therapeutic drug monitoring, replacing NONMEM/Python
   dependency chains with GPU-accelerated, zero-dependency binaries.

2. **Gut microbiome analytics** — Anderson localization as a quantitative metric for
   colonization resistance, with applications to *C. difficile* risk assessment,
   defined consortium design, antibiotic impact prediction, and QS gene profiling.

3. **Biosignal processing** — Real-time physiological signal analysis (ECG, PPG, EDA)
   on sovereign hardware via BarraCUDA GPU and neuromorphic NPU.

4. **Endocrinology** — Testosterone PK, TRT clinical claim verification against
   published registry data, and cross-track integration (gut axis, HRV).

5. **NLME population pharmacokinetics** — Sovereign replacement for NONMEM (FOCE),
   Monolix (SAEM), and WinNonlin (NCA). Full population PK with diagnostics.

6. **Comparative medicine / One Health** (V21) — Species-agnostic mathematics validated
   on animal models for their own sake: study disease where it naturally occurs, gain
   causal understanding, translate to humans via parameter substitution.

7. **Drug discovery / ADDRC** (V21) — Anderson-augmented MATRIX scoring, high-throughput
   screening pipeline integration (Lisabeth ADDRC → Gonzales iPSC → Ellsworth med chem).

A cross-cutting concern — **medical device software foundation** — addresses the
regulatory path enabled by Rust's memory safety guarantees and the Ferrocene IEC 62304
Class C qualification.

---

## 1. Motivation

### 1.1 The Gap Between Science and Application

The ecoPrimals springs have demonstrated that Pure Rust + BarraCUDA GPU can faithfully reproduce published scientific algorithms across eight domains (plasma physics, agriculture, life science, uncertainty, machine learning, immunology, pharmacology, drug repurposing). The validation evidence is extensive: 11,161+ checks across 70+ papers, 13 faculty connections, ~$0.93 total compute cost.

But reproduction is not application. A clinician cannot use a validated lattice QCD simulation. A doctor cannot prescribe a 16S diversity index. The gap between "we can compute this" and "a patient benefits from this computation" is where healthSpring operates.

### 1.2 Why Pure Rust Matters for Health

Existing PK/PD tools (NONMEM, Pharmpy, Chi, rPBK) depend on C libraries, Python runtimes, or commercial licenses. This creates barriers:

- **NONMEM**: Commercial license (~$2,000/year), FORTRAN core, requires trained pharmacometrician
- **Pharmpy**: Python + SymPy + NumPy + SciPy + NONMEM backend
- **Chi**: Python + Myokit + Sundials (C library for ODE solving)
- **rPBK**: R + RStan + Rcpp (compiled C++ bridge)

A Pure Rust PK/PD tool has zero external dependencies, cross-compiles to any platform (including mobile and embedded), and the BarraCUDA GPU path provides population PK simulation at speeds Python cannot match. For Monte Carlo population PK (thousands of virtual patients), GPU acceleration is a genuine clinical differentiator — dosing decisions that take hours in NONMEM take seconds on a consumer GPU.

### 1.3 Anderson Localization in the Gut

The Anderson localization framework, validated in soil microbiome (wetSpring Papers 01 and 06) and immunological signaling (Paper 12), provides a physics-based model for colonization resistance. A diverse gut microbiome is an Anderson localizer: pathogenic signals (toxins, colonization attempts) are confined to local regions and cannot propagate. Antibiotic disruption reduces diversity, lengthens the localization length, and pathogens spread.

This is not a metaphor. The mathematics is identical. The 1D/2D Anderson lattice code in wetSpring models localization length as a function of disorder strength. In soil, disorder = microbial diversity; in gut, disorder = microbial diversity. The substrate changes; the physics does not. A patient's 16S profile provides a measured disorder parameter, and the localization length provides a quantitative colonization resistance score.

### 1.4 Sovereign Health Monitoring

Current wearable health devices depend on cloud processing — data leaves the device, travels to a corporate server, and results return. This creates latency, privacy exposure, and single-point-of-failure dependency.

BarraCUDA's attention mechanisms (all 7 types validated in neuralSpring) combined with ToadStool's Akida NPU driver (<1ms inference, 50-400x power efficiency) enable on-device health monitoring. ECG anomaly detection, SpO2 estimation, glucose prediction — all running locally on sovereign hardware, no cloud required.

---

## 2. Domains

### 2.1 Track 1: PK/PD Modeling

**Foundation**: neuralSpring nS-601 (Hill dose-response), nS-603 (lokivetmab PK), nS-604 (three-compartment tissue).

**Extension**: Veterinary PK models validated against Gonzales publications (G1–G6) are mathematically identical to human PK. The Hill equation, compartmental models, and population PK methods are species-agnostic. healthSpring extends these to human drug parameters.

**Deliverable**: A Pure Rust binary that accepts drug parameters (clearance, volume of distribution, bioavailability), patient parameters (weight, age, renal function), and dosing schedule — and outputs concentration-time curves, AUC, Cmax, trough levels, and dose adjustment recommendations. GPU path for population simulations.

### 2.2 Track 2: Gut Microbiome Analytics

**Foundation**: wetSpring Track 1 (16S pipeline, 10 papers), Papers 01/06 (Anderson localization in microbial communities), Exp273–286 (immunological Anderson).

**Extension**: Transfer the Anderson lattice from soil to gut. Replace soil diversity parameters with gut diversity parameters. Validate against published C. diff colonization data (Jenior 2021, McGill 2025, Dsouza 2024).

**Deliverable**: A tool that takes a patient's 16S profile, computes Anderson localization length, outputs a colonization resistance score, and suggests which consortium strains (e.g., VE303 components) would restore localization.

### 2.3 Track 3: Biosignal Processing

**Foundation**: neuralSpring attention mechanisms, BarraCUDA CNN operations, ToadStool Akida driver.

**Extension**: Apply validated ML primitives to physiological signals. ECG → QRS detection → arrhythmia classification. PPG → SpO2 → heart rate. Glucose → trend prediction.

**Deliverable**: A BarraCUDA pipeline that processes raw biosignals on CPU, GPU, or NPU and outputs clinical alerts. Runs on Raspberry Pi, phone, or full workstation.

### 2.4 Track 4: Endocrinology (Exp030–038)

**Foundation**: Mok clinical reference, published TRT registry data (Harman 2001,
Saad 2013/2016, Kapoor 2006, Sharma 2015).

**Extension**: Extract quantifiable clinical claims about TRT from published literature,
build computational models for each claim, validate against open registry data. Cross-track
integration: testosterone-gut axis (Track 2 × Track 4), HRV-TRT (Track 3 × Track 4).

**Deliverable**: A claim verification pipeline: published claim → model → validation
against open data → patient-parameterized clinical scenario → petalTongue visualization.

### 2.5 Track 5: NLME Population Pharmacokinetics (Exp075–076)

**Foundation**: Beal & Sheiner (FOCE), Kuhn & Lavielle (SAEM), Gabrielsson & Weiner (NCA).

**Extension**: Sovereign replacement for NONMEM ($2,000/year), Monolix, and WinNonlin.
Full population PK estimation with diagnostics (CWRES, VPC, GOF).

**Deliverable**: Pure Rust FOCE + SAEM estimation, NCA metrics, diagnostic plots —
all validated, all AGPL-3.0, all zero-dependency.

### 2.6 Track 6: Comparative Medicine / One Health (V21)

**Foundation**: Gonzales canine work (G1–G6, 688/688 checks across wetSpring + neuralSpring),
species-agnostic PK/PD mathematics.

**V25: VALIDATED.** 7 experiments (Exp100–106) validate canine IL-31 kinetics, JAK1
selectivity, pruritus time-course, lokivetmab dose-duration, cross-species allometric PK,
canine gut Anderson, and feline hyperthyroidism PK.

**The causal inversion principle**: Current practice tests human drugs on animals — this
establishes correlation, not causation. PCP makes a chimp sleepy. healthSpring inverts
this: study disease where it naturally occurs (dogs get atopic dermatitis, cats get
hyperthyroidism, horses get laminitis). The species-native disease model yields causal
insight. The math is the same; parameters change. Translate to humans via parameter
substitution from the same equation set.

**Deliverable**: Species-agnostic PK parameter registry, cross-species Anderson for gut
and tissue lattices, comparative gut microbiome pipelines, and validated cross-species
allometric bridges.

### 2.7 Track 7: Drug Discovery / ADDRC (V21)

**Foundation**: Fajgenbaum MATRIX framework (nS-605), Anderson geometry scoring (Exp011),
Lisabeth ADDRC HTS infrastructure, Gonzales iPSC skin models.

**V25: Partially VALIDATED.** 5 experiments (Exp090–094) validate Anderson-augmented
MATRIX scoring, ADDRC HTS pipeline, compound IC50 profiling, ChEMBL JAK panel, and
Rho/MRTF/SRF fibrosis scoring. Remaining: iPSC validation (DD-006), Ellsworth med chem
(DD-007).

**Extension**: Anderson-augmented MATRIX scoring ranks compounds by predicted tissue
penetration. ADDRC screens the top candidates. Gonzales validates hits in iPSC skin
models. Ellsworth optimizes leads via medicinal chemistry.

**Pipeline**:
```
healthSpring MATRIX scoring → Lisabeth ADDRC HTS → Gonzales iPSC → Ellsworth med chem → Preclinical
```

**Deliverable**: Computational scoring of ADDRC's 8,000-compound library, HTS data
analysis pipeline, and integration of QS gene profiling for microbial drug targets.

### Medical Device Software Foundation (cross-cutting)

**Foundation**: Pure Rust ecosystem (zero unsafe code), BarraCUDA (deterministic math), ecoBin (reproducible builds).

**Context**: Ferrocene achieved IEC 62304 Class C qualification (January 2025). FDA cybersecurity guidance strengthened (February 2026). Rust's compiler eliminates ~90% of traditional safety analysis requirements.

**Deliverable**: Documentation and validation evidence for the IEC 62304 compliance path. Not the certification itself — that requires a regulatory partner — but the technical foundation that makes certification viable.

---

## 3. What We Have Learned

Six months of validation across 73 experiments, 603 tests, and 45 reproduced papers
have produced several key insights:

### 3.1 Species-agnostic mathematics works

The Hill equation, Anderson Hamiltonian, Bateman PK, and Shannon diversity index produce
identical results for canine and human parameters. This is not assumed — it is validated
across 688/688 checks in two springs. The practical implication: disease biology studied
in dogs (where atopic dermatitis occurs naturally) translates directly to human
therapeutics via parameter substitution.

### 3.2 Anderson localization is a universal biological framework

Originally a condensed matter physics result (Anderson 1958), wave localization in
disordered media provides quantitative models for:
- **Colonization resistance** in gut microbiomes (Pielou diversity → disorder parameter W)
- **Cytokine signal propagation** in immunological tissue
- **Drug penetration** in tissue compartments (localized = poor distribution, extended = good)
- **Quorum sensing propagation** along gut oxygen gradients

The mathematics is identical across all four applications. Only the substrate changes.

### 3.3 Validated science is compute-substrate-portable

The same algorithm produces the same results on CPU, GPU, and mixed hardware dispatch —
within documented tolerances. Every discrepancy is explained (IEEE 754 rounding, GPU
intermediate precision, Monte Carlo sampling variance) and catalogued in
`specs/TOLERANCE_REGISTRY.md`.

### 3.4 Pure Rust + scyBorg eliminates the dependency tax

NONMEM costs $2,000/year and requires FORTRAN. Pharmpy needs Python + SymPy + NumPy +
SciPy. healthSpring's FOCE, SAEM, and NCA run in Pure Rust with zero external
dependencies, zero unsafe code, and 84× faster execution than Python.

The scyBorg triple-copyleft framework (AGPL-3.0 code + CC-BY-SA 4.0 docs) ensures
these tools remain sovereign: no single entity — corporate, government, or academic —
can rug-pull the license. The provenance trio (sweetGrass/rhizoCrypt/loamSpine) makes
attribution and derivation machine-verifiable.

### 3.5 The drug discovery pipeline is computational — and scales beyond $48.3M MATRIX

Every Cure's MATRIX platform ($48.3M ARPA-H, Fajgenbaum) scores ~3,000 FDA-approved
drugs against ~12,000 diseases using AI on knowledge graphs. healthSpring extends this
with three dimensions MATRIX lacks — **now VALIDATED** (not just proposed): Track 7
DD-001–005 has implemented and validated the Anderson-augmented MATRIX scoring, HTS
analysis, compound IC50 profiling, and fibrosis pathway scoring.

- **Physics**: Anderson geometry scoring predicts tissue-specific drug penetration from
  first principles — not just pathway overlap from literature
- **Species**: Scoring across all species with naturally occurring disease (not just human)
- **Populations**: GPU population PK Monte Carlo per scored pair — not just binary match scores

At full scale (3K drugs × 12K diseases × 5 species × 20 tissue geometries), the
Anderson-augmented scoring takes ~2 seconds on a consumer GPU. Population PK for the
top 1% takes ~100 seconds. The entire Every Cure scale, extended across species and
tissues, fits on hardware costing less than one month of ARPA-H funding.

See [baseCamp/fajgenbaum/README.md](baseCamp/fajgenbaum/README.md) for the full
comparative analysis.

---

## 4. Validation Protocol

Same four-tier protocol as all ecoPrimals springs:

| Tier | Description | Acceptance |
|------|-------------|------------|
| 0 | Python control (reference implementation from published paper) | Reproduces published results |
| 1 | Rust CPU (Pure Rust, f64-canonical) | Matches Python within documented tolerance |
| 2 | Rust GPU (BarraCUDA WGSL shaders) | Matches CPU within documented tolerance |
| 3 | metalForge (ToadStool dispatch, cross-substrate) | Matches GPU, routing validated |

---

## References

### ecoPrimals Springs
- wetSpring V101: 9,060+ checks, Anderson lattice validated, 16S pipeline, Gonzales immunology
- neuralSpring V90: 4,100+ checks, nS-601–605 PK/PD modeling, MATRIX drug repurposing
- groundSpring V100: 824+ checks, uncertainty quantification
- airSpring v0.7.5: CytokineBrain, immunological Anderson
- hotSpring v0.6.17+: lattice methods, SU(3) → tissue lattice, BatchedEighGpu

### Regulatory
- Ferrocene IEC 62304: https://ferrous-systems.com/ferrocene/

### Key Papers
- Anderson P.W. (1958) *Phys Rev* 109:1492 — Absence of diffusion in certain random lattices
- Gonzales AJ et al. (2014) *J Vet Pharmacol Ther* 37:317 — Oclacitinib JAK1 selectivity
- Fleck/Gonzales (2021) *Vet Dermatol* 32:681 — Lokivetmab dose-duration
- Fajgenbaum DC et al. (2018) *NEJM* 379:1941 — MATRIX drug repurposing framework
- Jenior et al. (2021) *PLoS Comput Biol* doi:10.1371/journal.pcbi.1008782
- Lisabeth et al. (2024) *Front Microbiol* — Brucella small molecule screen
- McGill et al. (2025) *Cell Host & Microbe*
- Dsouza et al. (2024) *Nature Medicine* doi:10.1038/s41591-024-03337-4
- Harman SM et al. (2001) *JCEM* — BLSA longitudinal testosterone
