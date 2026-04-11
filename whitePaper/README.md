<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring White Paper

**Date:** April 11, 2026
**Status:** V50 — Composition Evolution. Python was the validation target for Rust. Now Rust and Python are both validation targets for NUCLEUS composition patterns. 89 experiments, 960+ tests, 80+ JSON-RPC capabilities (62 science + 22 infra), 54 Python baselines, 89 provenance entries (100% coverage), 6 Tier 4 composition experiments validating IPC dispatch parity. ecoBin harvested to plasmidBin. barraCuda v0.3.11. Zero clippy, zero unsafe.
**License:** scyBorg (AGPL-3.0-or-later code + ORC mechanics + CC-BY-SA 4.0 creative content)

---

## Start Here

healthSpring is a **Spring** — a validation project within the ecoPrimals ecosystem.
Its job is to prove that published health science algorithms can be faithfully ported
from Python to Rust to GPU, producing identical results at every stage, composed into
NUCLEUS primal compositions via IPC, and deployed as sovereign health applications.

**The one-sentence version**: We reproduce published health science papers in Python,
rewrite them in Rust (with GPU acceleration), verify the results match, compose them
into primal services via JSON-RPC, and then deploy those validated compositions as
real tools — drug dosing, colonization resistance scoring, biosignal processing,
and drug discovery pipelines.

**The validation ladder**: Python was the validation target for Rust. Now both Python
and Rust are validation targets for the primal composition layer. Tier 4 experiments
(exp112–117) prove that dispatching science through JSON-RPC IPC produces bit-identical
results to calling Rust functions directly — the composition is faithful to the science.

**As of V21**, we are expanding from human health to the **health of living systems**.
The math is species-agnostic: the Hill equation, Anderson localization, Bateman PK,
and Shannon diversity work identically regardless of species. What changes between
a dog and a human is parameters, not equations. This lets us study disease where it
naturally occurs (e.g., atopic dermatitis in dogs) and translate causal insight to
humans via parameter substitution — rather than testing human drugs on animals
without establishing causality.

---

## Reading Order

### For newcomers (30 minutes)

| Order | Document | What you'll learn |
|:-----:|----------|------------------|
| 1 | **This file** | What healthSpring is, glossary, key insights |
| 2 | [STUDY.md](STUDY.md) | The scientific narrative — why we built this, what domains we cover |
| 3 | [METHODOLOGY.md](METHODOLOGY.md) | The four-tier validation protocol (Python → Rust → GPU → dispatch) |

### For collaborators joining the project (1 hour)

| Order | Document | What you'll learn |
|:-----:|----------|------------------|
| 4 | [baseCamp/README.md](baseCamp/README.md) | All 83 experiments, validation counts, per-track status |
| 5 | [baseCamp/gonzales/README.md](baseCamp/gonzales/README.md) | PK/PD sub-thesis: Gonzales canine → human → drug discovery pipeline |
| 6 | [baseCamp/EXTENSION_PLAN.md](baseCamp/EXTENSION_PLAN.md) | Where we're going: Tracks 6–7, datasets, QS gene profiling, living systems |

### For drug discovery team (Gonzales/Lisabeth/ADDRC/Fajgenbaum)

| Order | Document | What you'll learn |
|:-----:|----------|------------------|
| 1 | **This file** | Project overview, glossary |
| 2 | [baseCamp/fajgenbaum/](baseCamp/fajgenbaum/) | MATRIX comparison + deep cost/access/methods breakdown |
| 3 | [baseCamp/gonzales/](baseCamp/gonzales/) | Gonzales lab sub-thesis + cost/access/methods vs. traditional PK |
| 4 | [../specs/PAPER_REVIEW_QUEUE.md](../specs/PAPER_REVIEW_QUEUE.md) | Track 7 (Drug Discovery) paper queue — front-loaded for ADDRC |
| 5 | [../specs/BARRACUDA_REQUIREMENTS.md](../specs/BARRACUDA_REQUIREMENTS.md) | GPU primitives for compound screening (Hill sweep, MATRIX scoring) |

### For technical deep-dive (specs)

| Document | Purpose |
|----------|---------|
| [../specs/README.md](../specs/README.md) | Master spec: metrics, paper queue summary, cross-spring dependencies |
| [../specs/PAPER_REVIEW_QUEUE.md](../specs/PAPER_REVIEW_QUEUE.md) | All 45 queued papers, per-track status, GPU/NUCLEUS validation |
| [../specs/EVOLUTION_MAP.md](../specs/EVOLUTION_MAP.md) | Rust module → WGSL shader → pipeline stage mapping |
| [../specs/TOLERANCE_REGISTRY.md](../specs/TOLERANCE_REGISTRY.md) | Every numerical tolerance with justification |
| [../specs/BARRACUDA_REQUIREMENTS.md](../specs/BARRACUDA_REQUIREMENTS.md) | GPU primitive inventory and absorption status |
| [../specs/COMPUTE_DATA_PROFILE.md](../specs/COMPUTE_DATA_PROFILE.md) | Data hunger, GPU memory, hardware allocation |

---

## Study Questions

### Tracks 1–5 (Answered — all complete)

1. Can validated PK/PD algorithms (Hill, compartmental, population Monte Carlo) be
   implemented in Pure Rust with GPU acceleration, replacing Python/NONMEM/R dependency
   chains while maintaining documented numerical tolerances?
   **Answer: Yes.** 84× faster than Python, 6 WGSL shaders, zero unsafe code.

2. Can Anderson localization transfer from soil microbiome (wetSpring) to gut colonization
   resistance, predicting *C. difficile* risk from 16S profiles?
   **Answer: Yes.** Same Hamiltonian, different substrate. Pielou → W → ξ pipeline validated.

3. Can real-time biosignal processing run on sovereign hardware at clinically useful latency?
   **Answer: Yes.** Pan-Tompkins QRS, HRV, SpO2, EDA — all running on CPU and GPU.

4. Can TRT clinical claims be systematically validated against published registry data?
   **Answer: Yes.** 9 experiments (Exp030–038), 182 checks, every claim sourced to peer-reviewed data.

5. Can GPU-accelerated population PK provide clinically actionable dosing recommendations?
   **Answer: Yes.** GPU crossover at 100K patients, linear scaling confirmed, FOCE/SAEM sovereign.

### Tracks 6–7 (Complete — V25)

6. Can species-agnostic mathematics validate disease models across species, enabling
   causal insight from naturally occurring animal disease (rather than testing human
   drugs on animals without causality)?

7. Can Anderson-augmented MATRIX scoring, combined with ADDRC high-throughput screening,
   identify novel therapeutics from an 8,000-compound library?

---

## What We Have Learned

### The math is species-agnostic

The Hill equation gives the same dose-response curve whether the IC50 comes from a
beagle or a human. The Anderson Hamiltonian localizes signals the same way whether
the lattice represents soil, gut, or immunological tissue. Population PK is the same
Bateman ODE regardless of species. This is not a claim — it is a validated result:
688/688 checks across wetSpring + neuralSpring prove the canine models are
mathematically faithful to the same equations used for human models.

This means studying disease where it naturally occurs (dogs get atopic dermatitis)
gives us causal insight that translates to humans via parameter substitution. The
alternative — testing human drugs on healthy animals — establishes correlation, not
causation.

### Validated science is portable across compute substrates

The same algorithm produces the same results on CPU, GPU, and mixed hardware
dispatch — within documented, justified tolerances. This was not obvious. WGSL f64
on GPU has different intermediate rounding than Rust f64 on CPU. The tolerance
registry documents every difference and why it is acceptable.

### Rust is 84× faster than Python and equally correct

Exp084 benchmarks six domain primitives head-to-head. The Rust implementations are
not just fast — they are verifiably identical to the Python baselines within machine
epsilon. Zero unsafe code. Zero clippy warnings under pedantic + nursery.

### Anderson localization applies to biology

The physics of wave localization in disordered media (Anderson 1958) provides a
quantitative model for colonization resistance in gut microbiomes, signal propagation
in immunological tissue, and drug penetration in tissue compartments. This is the
unifying mathematical framework across all healthSpring domains.

### The full pipeline terminates at a patient

Every computation pipeline in healthSpring terminates at a patient — parameterized
by age, weight, comorbidities, lab values — and rendered in petalTongue so a clinician
sees *this patient's data*, not abstract population statistics. Exp063 closes the
loop: a `PatientTrtProfile` generates a full scenario graph with clinical ranges and
risk annotations.

### Sovereign tools can replace commercial software

FOCE estimation replaces NONMEM ($2,000/year). SAEM estimation replaces Monolix.
NCA replaces WinNonlin. All validated, all AGPL-3.0, all zero-dependency Pure Rust.

---

## Glossary

| Term | Meaning |
|------|---------|
| **Spring** | A validation project in ecoPrimals. Each spring reproduces published science in Python, then evolves it to Rust → GPU. healthSpring is the sixth spring. |
| **ecoPrimals** | The parent ecosystem. Contains 6 springs (wet, neural, hot, air, ground, health) plus shared infrastructure (barraCuda, toadStool, metalForge, petalTongue, biomeOS). |
| **barraCuda** | Standalone Rust math library with vendor-agnostic GPU (WGSL) shaders. Provides Hill equation, reduction, linear algebra, statistics. healthSpring consumes it. |
| **toadStool** | Compute pipeline dispatch. Routes work to CPU, GPU, or NPU based on workload size and hardware availability. |
| **metalForge** | Hardware topology manager. NUCLEUS atomics (Tower/Node/Nest) for mixed-hardware orchestration across machines. |
| **petalTongue** | Universal visualization UI. 7 data channel types (TimeSeries, Bar, Scatter3D, Distribution, Spectrum, Heatmap, Gauge). healthSpring pushes clinical scenarios to it. |
| **biomeOS** | Deployment orchestration. Atomic graph-based deployment across hardware nodes. |
| **NUCLEUS** | metalForge's hierarchical hardware topology: Tower (machine group) → Node (machine) → Nest (compute unit). |
| **Tier 0** | Python reference implementation. The ground truth. Every experiment starts here. |
| **Tier 1** | Rust CPU. Pure Rust reimplementation validated against Python within documented tolerances. |
| **Tier 2** | Rust GPU. WGSL shader validated against CPU within documented tolerances. |
| **Tier 3** | metalForge dispatch. Cross-substrate routing validated — same results regardless of CPU/GPU/NPU. |
| **WGSL** | WebGPU Shading Language. The GPU compute language used by barraCuda. Vendor-agnostic (NVIDIA, AMD, Intel, Apple). |
| **Anderson localization** | Physics phenomenon (Anderson 1958): waves in disordered media become confined to local regions. Applied here to gut microbiome (diversity = disorder), tissue lattice (cytokine signals), and drug penetration. |
| **Hill equation** | Dose-response model: R = C^n / (C^n + IC50^n). Used for drug potency modeling across all species. |
| **Pielou evenness** | J = H'/ln(S). Measures how evenly distributed species abundances are. Maps to Anderson disorder parameter W. |
| **MATRIX** | Fajgenbaum drug repurposing framework. Scores drugs against diseases by mechanism overlap. Anderson geometry augments it. |
| **ADDRC** | Assay Development and Drug Repurposing Core (MSU). Lisabeth's high-throughput screening lab. 8,000-compound library. |
| **FOCE** | First-Order Conditional Estimation. Population PK method (replaces NONMEM). |
| **SAEM** | Stochastic Approximation Expectation-Maximization. Population PK method (replaces Monolix). |
| **NCA** | Non-Compartmental Analysis. PK metrics: AUC, Cmax, half-life, clearance (replaces WinNonlin). |
| **QS** | Quorum Sensing. Bacterial cell-to-cell communication. Gene families (LuxI/LuxR, AI-2, Agr) inform the functional dimension of Anderson disorder. |
| **Track 6** | Comparative Medicine / One Health. Species-agnostic validation: study disease in animals for their own sake, translate to humans via parameter substitution. |
| **Track 7** | Drug Discovery / ADDRC / MATRIX. Computational drug screening pipeline → ADDRC wet-lab HTS → Gonzales iPSC validation → Ellsworth medicinal chemistry. |
| **scyBorg** | ecoPrimals triple-copyleft licensing framework: AGPL-3.0 (code) + ORC (game mechanics) + CC-BY-SA 4.0 (creative content). No single entity can rug-pull any layer. Provenance trio (sweetGrass/rhizoCrypt/loamSpine) makes it machine-verifiable. See `wateringHole/SCYBORG_PROVENANCE_TRIO_GUIDANCE.md`. |
| **Write → Absorb → Lean** | The ecoPrimals cycle: Write locally (validate), Absorb upstream (into barraCuda), Lean locally (remove local copy, consume from upstream). |

---

## Document Index

| Document | Purpose | Audience |
|----------|---------|----------|
| [STUDY.md](STUDY.md) | Main narrative — abstract, domains, validation, what we learned | Reviewers, collaborators, clinicians |
| [METHODOLOGY.md](METHODOLOGY.md) | Validation protocol — four-tier design, acceptance criteria, tolerances | Technical validation |
| [baseCamp/README.md](baseCamp/README.md) | Experiment inventory, per-track validation summary, GPU pipeline | All |
| [baseCamp/gonzales/README.md](baseCamp/gonzales/README.md) | PK/PD + drug discovery sub-thesis (Gonzales, Lisabeth, Neubig, Ellsworth) | Drug discovery team |
| [baseCamp/mok/README.md](baseCamp/mok/README.md) | TRT claim verification sub-thesis (Mok) | Endocrinology |
| [baseCamp/cdiff_colonization.md](baseCamp/cdiff_colonization.md) | Anderson → gut colonization resistance | Microbiome / infectious disease |
| [baseCamp/biosignal_sovereign.md](baseCamp/biosignal_sovereign.md) | Edge biosignal processing (ECG, PPG, EDA) | Biosignal / wearables |
| [baseCamp/fajgenbaum/](baseCamp/fajgenbaum/) | MATRIX comparison + cost/access/methods ($48.3M vs. ~$5K) | Drug discovery, Fajgenbaum, funders |
| [baseCamp/gonzales/](baseCamp/gonzales/) | Gonzales lab sub-thesis + cost/access/methods | Gonzales lab, ADDRC |
| [baseCamp/mok/](baseCamp/mok/) | Mok TRT sub-thesis | Endocrinology |
| [baseCamp/EXTENSION_PLAN.md](baseCamp/EXTENSION_PLAN.md) | Extension datasets, new tracks, living systems roadmap | All — future planning |

---

## baseCamp Sub-Theses (per-org directories)

| Sub-thesis | Domain | Faculty | Status |
|-----------|--------|---------|--------|
| [gonzales/](baseCamp/gonzales/) | PK/PD → living systems + drug discovery | Gonzales, Lisabeth, Neubig, Ellsworth | **Complete** (Track 1), **Complete** (Tracks 6–7) |
| [fajgenbaum/](baseCamp/fajgenbaum/) | MATRIX drug repurposing + Anderson geometry | Fajgenbaum (Every Cure) | **Ingested + Extended** (Track 7) |
| [mok/](baseCamp/mok/) | TRT claim verification + endocrine modeling | Mok (Allure Medical) | **Complete** — Exp030-038 (Track 4) |
| [cdiff_colonization.md](baseCamp/cdiff_colonization.md) | Anderson localization → gut colonization resistance | (TBD) | **Complete** — Exp010-013 (Track 2) |
| [biosignal_sovereign.md](baseCamp/biosignal_sovereign.md) | Edge biosignal processing | (TBD) | **Complete** — Exp020-023 (Track 3) |

---

## Faculty Network

| Faculty | Affiliation | Domain | Tracks |
|---------|-------------|--------|--------|
| Andrea J. Gonzales | MSU Pharmacology & Toxicology | Comparative pharmacology, JAK inhibitors, iPSC skin models | 1, 6, 7 |
| Erika Lisabeth | ADDRC, MSU | HTS assay development, drug repurposing, EphA3 | 7 |
| Richard Neubig | Drug Discovery, MSU | GPCR signaling, Rho/MRTF/SRF, skin fibrosis | 7 |
| Edmund Ellsworth | Drug Discovery, MSU | Medicinal chemistry, niclosamide, terpene biosynthesis | 7 |
| Charles Mok | Allure Medical | Testosterone replacement therapy | 4 |
| Wei Liao | ADREC, MSU BAE | Anaerobic digestion (gut-digester analogy) | 2 |

---

## Quick Metrics

| Metric | Value |
|--------|-------|
| Experiments | 83 complete (Tracks 1–9), 15 queued |
| Rust tests | 928 (lib + proptest + IPC fuzz + doc + experiment bins) |
| Python checks | 194 cross-validation |
| Paper queue | 30/30 complete, 15 new queued |
| GPU shaders | 6 WGSL (Hill, PopPK, Diversity, MM batch, SCFA batch, Beat classify) |
| CPU speedup | Rust 84× faster than Python |
| Unsafe code | 0 blocks |
| Clippy warnings | 0 (pedantic + nursery) |
| File size | All under 1000 lines (wateringHole standard) |
| License | **scyBorg**: AGPL-3.0 (code) + CC-BY-SA 4.0 (docs) |
