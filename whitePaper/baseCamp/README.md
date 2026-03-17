# healthSpring baseCamp

Per-person translation of validated science into usable health applications. Metagenomics, pharmacokinetics, biosignals, and endocrine models mean nothing unless they produce actionable clinical insight for individual patients. Every pipeline here terminates at a patient — parameterized, visualized, and interpretable by the clinician standing in front of them.

**Last Updated:** March 16, 2026
**Status:** V31 — Deep Debt Solutions + Modern Idiomatic Rust Evolution. 616 tests, 73 experiments, 42 baselines with provenance, 113/113 cross-validation checks. OrExit trait, IpcError, enriched capability.list, magic number cleanup, forbid(unsafe_code), capability-based discovery, non-async Tier A GPU ops.

---

## Tracks

| Track | Domain | Experiments | Status |
|-------|--------|-------------|--------|
| 1 — PK/PD | Pharmacokinetics, dose-response, population modeling, PBPK, MM PK | Exp001-006, 077 | **Complete** (Tier 0+1+2) |
| 2 — Microbiome | Gut diversity, Anderson lattice, colonization resistance, FMT, antibiotics, SCFA, serotonin | Exp010-013, 078-080 | **Complete** (Tier 0+1+2) |
| 3 — Biosignal | ECG detection, HRV, PPG SpO2, EDA stress, arrhythmia classification, multi-channel fusion | Exp020-023, 081-082 | **Complete** (Tier 0+1+2) |
| 4 — Endocrinology | Testosterone PK, TRT outcomes, gut axis, HRV cross-track | Exp030-038 | **Complete** (Tier 0+1) |
| 5 — NLME | FOCE/SAEM population PK, NCA, CWRES/VPC/GOF diagnostics | Exp075-076 | **Complete** (Tier 0+1) |
| 6 — Comparative Medicine | Species-agnostic PK, cross-species Anderson, canine AD models | Exp100-106 | **Complete** (V25) |
| 7 — Drug Discovery | MATRIX scoring, ADDRC HTS, compound screening, iPSC validation | Exp090-094 | **Complete** (V25) |

---

## Sub-Theses (per-org subdirectories)

Each targeted organization/group has a dedicated subdirectory containing a README
(the sub-thesis narrative), a cost/access/methods comparison, and any useful data
for onboarding that group.

| # | Directory | Faculty | Domain | Status | Python | Rust |
|---|-----------|---------|--------|--------|:------:|:----:|
| 01 | [gonzales/](gonzales/) | Gonzales, Lisabeth, Neubig, Ellsworth | PK/PD → living systems + drug discovery (Tracks 1, 6, 7) | **Complete** (T1, T6, T7) | 73 | 79 |
| 02 | [fajgenbaum/](fajgenbaum/) | Fajgenbaum (Every Cure) | MATRIX drug repurposing + Anderson geometry (Track 7) | **Ingested + Extended** | — | — |
| 03 | [mok/](mok/) | Dr. Charles Mok | Testosterone PK, TRT outcomes, HRV cross-track (Track 4) | **Complete** | 96 | 86 |
| 04 | [cdiff_colonization.md](cdiff_colonization.md) | — | Anderson localization → gut colonization, FMT (Track 2) | **Complete** | 36 | 48 |
| 05 | [biosignal_sovereign.md](biosignal_sovereign.md) | — | Edge biosignal processing (Track 3) | **Complete** | 44 | 44 |

### Per-Org Directory Contents

```
baseCamp/
├── README.md                    ← This file
├── EXTENSION_PLAN.md            ← Datasets, new tracks, living systems roadmap
├── gonzales/
│   ├── README.md                ← Sub-thesis: PK/PD → human → living systems
│   └── cost_access_methods.md   ← Cost/access/methods vs. traditional PK pipeline
├── fajgenbaum/
│   ├── README.md                ← MATRIX comparison: healthSpring vs. $48.3M Every Cure
│   └── cost_access_methods.md   ← Deep cost/access/data/methods breakdown
├── mok/
│   └── README.md                ← Sub-thesis: TRT claim verification + endocrinology
├── cdiff_colonization.md        ← Sub-thesis: Anderson → gut colonization
└── biosignal_sovereign.md       ← Sub-thesis: Edge biosignal
```

The authoritative versions are in the per-org subdirectories.

---

## Per-Person Translation Pipeline

The entire validation chain exists to serve a single purpose: translate population-level science into individual patient insight.

```
Published literature → Claim extraction → Computational model
     → Population validation → Patient parameterization
          → Clinical scenario → petalTongue visualization
               → Clinician sees THIS patient's data
```

Exp063 closes this loop: a `PatientTrtProfile` (age, weight, testosterone level, comorbidities) generates an 8-node scenario graph with edges, clinical ranges, and risk annotations — all rendered in petalTongue's clinical mode (sidebars hidden, awakening skipped, graph fitted to view). The clinician sees the patient, not the infrastructure.

---

## Validation Summary

| Sub-thesis | Upstream Checks | Tier 0 (Python) | Tier 1+ (Rust) | Total |
|------------|:---------------:|:---------------:|:-------------:|:-----:|
| Gonzales PK/PD (Exp001-006) | 688 | 84 | 84 | 856 |
| Microbiome / C. diff / FMT (Exp010-013) | wetSpring Anderson | 48 | 51 | 99+ |
| Biosignal / ECG+PPG+EDA (Exp020-023) | — | 44 | 44 | 88 |
| Mok Testosterone + D3 (Exp030-038) | — | 96 | 86 | 182 |
| Validation / Parity (Exp040) | — | 15 | 15 | 30 |
| Diagnostics (Exp050-052) | — | — | 87 | 87 |
| GPU Pipeline (Exp053-055) | — | — | GPU live | — |
| Visualization (Exp056) | — | — | 50 | 50 |
| Mixed Dispatch (Exp060-062) | — | — | 75 | 75 |
| Clinical TRT + IPC + streaming (Exp063-065) | — | — | Structural | — |
| Compute + benchmark (Exp066-072) | — | — | Structural | — |
| petalTongue evolution (Exp073-074) | — | — | 19 | 19 |
| NLME + Full Pipeline (Exp075-076) | — | — | 216 | 216 |
| Paper Queue (Exp077-082) | — | 6 controls | 6 binaries | — |
| Comparative Medicine (Exp100-106) | Gonzales 688 | 7 controls | 7 binaries (103 checks) | — |
| Drug Discovery (Exp090-094) | Fajgenbaum MATRIX | 5 controls | 5 binaries (70 checks) | — |
| GPU V16 Parity (Exp083) | — | — | 25 | 25 |
| CPU Parity Bench (Exp084) | — | 17 | 33 | 50 |
| GPU Scaling Bench (Exp085) | — | 10 | 47 | 57 |
| toadStool V16 Dispatch (Exp086) | — | — | 24 | 24 |
| Mixed NUCLEUS V16 (Exp087) | — | — | 35 | 35 |
| **Lib unit tests** | — | — | **544** | 544 |
| **metalForge tests** | — | — | **33** | 33 |
| **toadStool tests** | — | — | **30** | 30 |
| **Doc-tests** | — | — | **4** | 4 |
| **Criterion benchmarks** | — | — | **14** | 14 |
| **Total** | **688** | **287+** (Tier 0) | **616** (tests) | **2,700+** |

---

## Experiment Inventory

### Track 1 — PK/PD

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 001 | Hill dose-response (JAK inhibitors) | 19 | 18 |
| 002 | One-compartment PK (IV + oral) | 12 | 18 |
| 003 | Two-compartment PK (biexponential) | 15 | 11 |
| 004 | mAb PK cross-species transfer (allometric) | 12 | 7 |
| 005 | Population PK Monte Carlo (1000 patients) | 15 | 12 |
| 006 | PBPK compartments (5-tissue physiological) | 13 | 13 |
| 077 | Michaelis-Menten nonlinear PK (phenytoin) | control | binary |

### Track 2 — Microbiome

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 010 | Diversity indices (Shannon/Simpson/Pielou/Chao1) | 14 | 12 |
| 011 | Anderson gut lattice (localization → resistance) | 12 | 14 |
| 012 | C. diff colonization resistance score | 10 | 10 |
| 013 | FMT microbiota transplant for rCDI | 12 | 12 |
| 078 | Antibiotic perturbation (ciprofloxacin) | control | binary |
| 079 | SCFA production (acetate/propionate/butyrate) | control | binary |
| 080 | Gut-brain serotonin axis (tryptophan → 5-HT) | control | binary |

### Track 3 — Biosignal

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 020 | Pan-Tompkins QRS detection | 12 | 12 |
| 021 | HRV metrics (SDNN, RMSSD, pNN50) | 10 | 10 |
| 022 | PPG SpO2 R-value calibration | 11 | 11 |
| 023 | Multi-channel fusion (ECG + PPG + EDA) | 11 | 11 |
| 081 | EDA electrodermal stress detection | control | binary |
| 082 | Arrhythmia beat classification (template matching) | control | binary |

### Track 4 — Endocrinology

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 030 | Testosterone PK: IM injection steady-state | 12 | 11 |
| 031 | Testosterone PK: pellet depot (5-month) | 10 | 10 |
| 032 | Testosterone decline: age-related hypogonadism | 10 | 8 |
| 033 | TRT metabolic response: weight/BMI/waist | 10 | 7 |
| 034 | TRT cardiovascular: lipids + CRP + BP | 10 | 10 |
| 035 | TRT diabetes: HbA1c + insulin sensitivity | 10 | 10 |
| 036 | Population TRT Monte Carlo (10K patients) | 12 | 10 |
| 037 | Testosterone–gut axis: microbiome stratification | 12 | 10 |
| 038 | HRV × TRT cardiovascular cross-track (Mok D3) | 10 | 10 |

### GPU V16 Parity

| Exp | Title | Checks |
|-----|-------|:------:|
| 083 | GPU V16 parity (3 shaders + metalForge + toadStool) | 25/25 |

### CPU Parity Benchmarks (V18)

| Exp | Title | Rust Checks | Python Checks | Bench Cases |
|-----|-------|:-----------:|:-------------:|:-----------:|
| 084 | V16 CPU parity bench (Rust 84× faster than Python) | 33/33 | 17/17 | 14 |

### GPU Scaling + toadStool Dispatch + NUCLEUS Routing (V19)

| Exp | Title | Checks |
|-----|-------|:------:|
| 085 | barraCuda GPU vs CPU V16 scaling bench (4 scales × 3 ops + fused + routing) | 47/47 |
| 086 | toadStool V16 streaming dispatch (execute_cpu + streaming callbacks + GPU-mappability) | 24/24 |
| 087 | metalForge mixed NUCLEUS V16 dispatch (Tower/Node/Nest + PCIe P2P + plan_dispatch) | 35/35 |

---

## Data Sources for Track 4 (Mok Testosterone)

The Mok reference is a clinical practice book, not a peer-reviewed paper.
The challenge is to **extract quantifiable claims** and **validate them against published registry data** using open sources.

| Claim (from Mok) | Validation Source | Data Type | Open? |
|---|---|---|---|
| T declines 1-3%/yr after 30 | Harman 2001 (BLSA), Feldman 2002 (MMAS) | Longitudinal serum T | Yes (published tables) |
| TRT causes sustained weight loss | Saad 2013 (n=411), Traish 2014 (n=261+260) | Registry BMI/waist time series | Yes (published figures) |
| TRT reduces cardiovascular mortality | Sharma 2015 (VA, n=83,010), Shores 2012 | Retrospective cohort | Yes (published) |
| TRT improves HbA1c in T2DM | Kapoor 2006 (RCT), Dhindsa 2016 | RCT endpoints | Yes (published) |
| 10mg/lb pellet dosing → therapeutic levels | Allure Medical clinical experience | Clinical PK | **No** (clinical) |
| Pellet PK: 5-month steady state | Testopel label + Cavender 2009 | PK profile | Yes (label data) |
| IM injection: weekly dosing better than biweekly | Ross 2004, Shoskes 2016 | PK time series | Yes (published) |

---

## Cross-Spring Provenance

Every healthSpring experiment inherits validated primitives from upstream springs:

```
wetSpring (V123, 1,703 tests, 376 experiments)
    └─ 16S diversity → Exp010 (Shannon/Simpson/Pielou/Chao1)
    └─ Anderson lattice → Exp011 (gut colonization)
    └─ Gonzales immunology (Exp273-286) → baseline for all Track 1
    └─ OrExit zero-panic pattern → V31 graceful validation

neuralSpring (S157)
    └─ nS-601 (Hill/IC50) → Exp001 (human JAK inhibitors)
    └─ nS-603 (lokivetmab PK) → Exp004 (mAb cross-species transfer)
    └─ nS-604 (tissue lattice) → planned tissue lattice extension
    └─ nS-605 (MATRIX) → **VALIDATED** — Exp090 MATRIX scoring
    └─ Dual-format capability parsing → V31 cross-primal discovery

groundSpring (V109) → uncertainty propagation, zero-panic pattern → V31
hotSpring (v0.6.31) → planned lattice tissue finite-size scaling
```

---

## GPU Pipeline (Tier 2+3) — LIVE

All 24 Tier 0+1 experiments validated. GPU pipeline live (Exp053-055). CPU vs GPU parity (Exp060, 27/27). Mixed hardware dispatch (Exp061, 22/22). PCIe P2P transfers (Exp062, 26/26).

### WGSL Shaders (f64 precision, compiled into binary)

| Shader | Operation | Pattern | Status |
|--------|-----------|---------|--------|
| `hill_dose_response_f64.wgsl` | E(c) = Emax·c^n / (c^n + EC50^n) | Element-wise | **Validated** (Exp053) |
| `population_pk_f64.wgsl` | AUC = F·Dose / CL(random) | Embarrassingly parallel MC | **Validated** (Exp053) |
| `diversity_f64.wgsl` | Shannon + Simpson indices | Workgroup reduction | **Validated** (Exp053) |
| `michaelis_menten_batch_f64.wgsl` | Per-patient MM ODE (Euler + Wang hash PRNG) | Embarrassingly parallel ODE | **Validated** (Exp083) |
| `scfa_batch_f64.wgsl` | Acetate/propionate/butyrate MM kinetics | Element-wise (3-output) | **Validated** (Exp083) |
| `beat_classify_batch_f64.wgsl` | Template-matching beat classification | Per-beat cross-correlation | **Validated** (Exp083) |

### GPU Architecture

| Component | Purpose | Location |
|-----------|---------|----------|
| `GpuContext` | Persistent device/queue, shader reuse | `ecoPrimal/src/gpu/context.rs` |
| `execute_fused()` | All ops in one encoder, no CPU roundtrips | `ecoPrimal/src/gpu/mod.rs` |
| `Pipeline::execute_gpu()` | toadStool dispatches stages via `GpuContext` | `toadstool/src/pipeline.rs` |
| `Stage::to_gpu_op()` | Stage → GpuOp conversion | `toadstool/src/stage.rs` |

### Scaling Results (RTX 4070, release build)

| Operation | Crossover | Peak Speedup | Peak Throughput |
|-----------|-----------|:------------:|:---------------:|
| Hill dose-response | 100K | 2.0x at 5M | 207 M/s |
| Population PK | 5M | 1.15x at 5M | 365 M/s |
| Fused pipeline (small) | — | 31.7x vs individual | — |

### Learnings for toadStool/barraCuda Team

1. `enable f64;` in WGSL must be stripped — wgpu/naga handles f64 via device features, not shader directives
2. `pow(f64, f64)` is unsupported on NVIDIA via NVVM — use `exp(n * log(c))` cast through f32
3. u64 PRNG not portable — use u32-only xorshift32 + Wang hash for GPU Monte Carlo
4. Fused pipeline (single encoder) eliminates ~30x overhead at small sizes vs individual dispatches
5. At 10M+ elements, memory bandwidth dominates — buffer streaming needed for next tier

---

## V22 — biomeOS Niche Architecture

healthSpring evolved from experiment binaries into a biomeOS niche in V22:

| Component | File | Purpose |
|-----------|------|---------|
| Primal binary | `ecoPrimal/src/bin/healthspring_primal.rs` | 55+ capabilities via JSON-RPC 2.0 over Unix socket |
| IPC dispatch | `ecoPrimal/src/ipc/dispatch.rs` | Method→science function routing for 6 domains |
| Niche manifest | `graphs/healthspring_niche.toml` | Declares the niche: primals + workflow graphs |
| Patient assessment | `graphs/healthspring_patient_assessment.toml` | ConditionalDag: 4 parallel science tracks → composite |
| TRT scenario | `graphs/healthspring_trt_scenario.toml` | Sequential TRT clinical workflow |
| Microbiome analysis | `graphs/healthspring_microbiome_analysis.toml` | Sequential diversity → Anderson → SCFA pipeline |
| Biosignal monitor | `graphs/healthspring_biosignal_monitor.toml` | Continuous 250 Hz real-time monitoring |

The primal provides the science. The graphs define the composition. biomeOS's Neural API orchestrates and optimizes via the Pathway Learner.

---

## Next Steps (Post V31)

### Science Extensions

1. **DD-006 iPSC validation** — Gonzales iPSC skin model validation
2. **DD-007 Ellsworth med chem** — Medicinal chemistry lead optimization
3. **CM-008 equine laminitis** — Species-agnostic laminitis model

### GPU + Scale

4. **NLME GPU shaders** — FOCE per-subject gradient, VPC Monte Carlo
5. **Anderson eigensolve** — GPU shader for gut lattice localization length
6. **Biosignal FFT** — GPU radix-2 FFT for real-time ECG/PPG
7. **TensorSession** — When barraCuda ships fused multi-op pipeline API

### Cross-Spring Absorption (Identified V31)

8. **HMM biosignal regime** — Absorb `HmmBatchForwardF64` from neuralSpring for cardiac state detection
9. **ESN clinical prediction** — Absorb Echo State Network from neuralSpring for time-series outcomes
10. **3D tissue Anderson** — Absorb from groundSpring for tissue heterogeneity modeling
11. **llvm-cov** — Target 90%+ line coverage (wetSpring pattern)

---

## MATRIX Comparison

See [fajgenbaum/](fajgenbaum/) for the full comparison of healthSpring's Anderson-augmented
MATRIX vs. Every Cure's $48.3M ARPA-H platform:
- [fajgenbaum/README.md](fajgenbaum/README.md) — What we do differently, scaling analysis, onboarding for ingested researchers
- [fajgenbaum/cost_access_methods.md](fajgenbaum/cost_access_methods.md) — Deep cost/access/data-source/methods breakdown ($48.3M vs. ~$5K)

---

## Integration Points

- **NestGate**: `data.ncbi_search` / `data.ncbi_fetch` for PubMed literature
- **biomeOS NUCLEUS**: Atomic deployment (Nest for data storage, Node for GPU compute)
- **biomeOS Neural API**: Graph execution, capability routing, Pathway Learner optimization
- **Provenance Trio**: `rhizoCrypt` (ephemeral DAG) + `loamSpine` (immutable ledger) + `sweetGrass` (semantic attribution)
- **toadStool/barraCuda**: GPU population PK Monte Carlo, fused diagnostic pipeline
- **wetSpring**: Diversity primitives (reuse `science.diversity`), Anderson lattice (cross-spring)
