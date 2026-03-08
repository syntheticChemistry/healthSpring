# healthSpring: Human Health Applications of Sovereign Scientific Computing

**Version**: 0.1 (scaffold)
**Date**: March 8, 2026

---

## Abstract

healthSpring applies the validated scientific computing pipelines of the ecoPrimals spring ecosystem to human health problems. Where wetSpring reproduced 52 published life science papers, neuralSpring validated ML primitives against 31 scholarly works, and groundSpring established uncertainty quantification across 35 experiments — healthSpring builds **usable clinical applications** from those validated foundations.

Three application domains define the initial scope:

1. **Pharmacokinetic/pharmacodynamic modeling** — Pure Rust tools for drug dosing, population PK simulation, and therapeutic drug monitoring, replacing NONMEM/Python dependency chains with GPU-accelerated, zero-dependency binaries.

2. **Gut microbiome analytics** — Anderson localization as a quantitative metric for colonization resistance, with applications to *Clostridioides difficile* risk assessment, defined consortium design, and antibiotic impact prediction.

3. **Biosignal processing** — Real-time physiological signal analysis (ECG, PPG, glucose) on sovereign hardware, leveraging BarraCUDA attention mechanisms and neuromorphic NPU inference for edge health monitoring.

A fourth cross-cutting concern — **medical device software foundation** — addresses the regulatory path enabled by Rust's memory safety guarantees and the Ferrocene IEC 62304 Class C qualification.

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

### 2.4 Track 4: Medical Device Foundation

**Foundation**: Pure Rust ecosystem (zero unsafe code), BarraCUDA (deterministic math), ecoBin (reproducible builds).

**Context**: Ferrocene achieved IEC 62304 Class C qualification (January 2025). FDA cybersecurity guidance strengthened (February 2026). Rust's compiler eliminates ~90% of traditional safety analysis requirements.

**Deliverable**: Documentation and validation evidence for the IEC 62304 compliance path. Not the certification itself — that requires a regulatory partner — but the technical foundation that makes certification viable.

---

## 3. Validation Protocol

Same four-tier protocol as all ecoPrimals springs:

| Tier | Description | Acceptance |
|------|-------------|------------|
| 0 | Python control (reference implementation from published paper) | Reproduces published results |
| 1 | Rust CPU (Pure Rust, f64-canonical) | Matches Python within documented tolerance |
| 2 | Rust GPU (BarraCUDA WGSL shaders) | Matches CPU within documented tolerance |
| 3 | metalForge (ToadStool dispatch, cross-substrate) | Matches GPU, routing validated |

---

## References

- wetSpring V99: 8,886+ checks, 52 papers, Anderson lattice validated
- neuralSpring V90: 4,100+ checks, nS-601–605 PK/PD modeling
- groundSpring V100: 824+ checks, uncertainty quantification
- airSpring v0.7.1: 2,591+ checks, CytokineBrain
- hotSpring: 697+ checks, lattice methods
- Ferrocene IEC 62304: https://ferrous-systems.com/ferrocene/
- Jenior et al. 2021: doi:10.1371/journal.pcbi.1008782
- McGill et al. 2025: Cell Host & Microbe
- Dsouza et al. 2024: doi:10.1038/s41591-024-03337-4
