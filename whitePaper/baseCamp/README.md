# healthSpring baseCamp

Faculty-linked sub-theses documenting how healthSpring extends validated science into human health applications.

**Last Updated:** March 8, 2026
**Status:** V6 — 4 tracks + diagnostics + GPU pipeline, 30 experiments complete (200 unit tests, 346 binary checks, 104 cross-validation checks). Tier 2 GPU live.

---

## Tracks

| Track | Domain | Experiments | Status |
|-------|--------|-------------|--------|
| 1 — PK/PD | Pharmacokinetics, dose-response, population modeling, PBPK | Exp001-006 | **Complete** (Tier 0+1) |
| 2 — Microbiome | Gut diversity, Anderson lattice, colonization resistance, FMT | Exp010-013 | **Complete** (Tier 0+1) |
| 3 — Biosignal | ECG detection, HRV, PPG SpO2, EDA, multi-channel fusion | Exp020-023 | **Complete** (Tier 0+1) |
| 4 — Endocrinology | Testosterone PK, TRT outcomes, gut axis, HRV cross-track | Exp030-038 | **Complete** (Tier 0+1) |

---

## Sub-Theses

| # | File | Faculty | Domain | Status | Python | Rust |
|---|------|---------|--------|--------|:------:|:----:|
| 01 | [gonzales.md](gonzales.md) | Gonzales, Lisabeth, Neubig | PK/PD + immunology → human therapeutics | **Complete** | 73 | 79 |
| 02 | cdiff_colonization.md | TBD | Anderson localization → gut colonization resistance, FMT | **Complete** | 36 | 48 |
| 03 | biosignal_sovereign.md | TBD | Edge biosignal processing, PPG SpO2, fusion | **Complete** | 44 | 44 |
| 04 | [mok_testosterone.md](mok_testosterone.md) | Dr. Charles Mok | Testosterone PK, TRT outcomes, HRV cross-track | **Complete** | 96 | 86 |

---

## Validation Summary

| Sub-thesis | Upstream Checks | Tier 0 (Python) | Tier 1 (Rust) | Total |
|------------|:---------------:|:---------------:|:-------------:|:-----:|
| Gonzales PK/PD (Exp001-006) | 688 | 84 | 84 | 856 |
| Microbiome / C. diff / FMT (Exp010-013) | wetSpring Anderson | 48 | 51 | 99+ |
| Biosignal / ECG+PPG+EDA (Exp020-023) | — | 44 | 44 | 88 |
| Mok Testosterone + D3 (Exp030-038) | — | 96 | 86 | 182 |
| Validation / Parity (Exp040) | — | 15 | 15 | 30 |
| **Lib unit tests** | — | — | **139** | 139 |
| **metalForge tests** | — | — | **27** | 27 |
| **Total** | **688** | **287** (Tier 0) | **280** (binary) + **166** (lib+forge) = **446** | **1,421+** |

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

### Track 2 — Microbiome

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 010 | Diversity indices (Shannon/Simpson/Pielou/Chao1) | 14 | 12 |
| 011 | Anderson gut lattice (localization → resistance) | 12 | 14 |
| 012 | C. diff colonization resistance score | 10 | 10 |
| 013 | FMT microbiota transplant for rCDI | 12 | 12 |

### Track 3 — Biosignal

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 020 | Pan-Tompkins QRS detection | 12 | 12 |
| 021 | HRV metrics (SDNN, RMSSD, pNN50) | 10 | 10 |
| 022 | PPG SpO2 R-value calibration | 11 | 11 |
| 023 | Multi-channel fusion (ECG + PPG + EDA) | 11 | 11 |

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
wetSpring (V99, 8,886 checks)
    └─ 16S diversity → Exp010 (Shannon/Simpson/Pielou/Chao1)
    └─ Anderson lattice → Exp011 (gut colonization)
    └─ Gonzales immunology (Exp273-286) → baseline for all Track 1

neuralSpring (V90, 2,279 checks)
    └─ nS-601 (Hill/IC50) → Exp001 (human JAK inhibitors)
    └─ nS-603 (lokivetmab PK) → Exp004 (mAb cross-species transfer)
    └─ nS-604 (tissue lattice) → planned tissue lattice extension
    └─ nS-605 (MATRIX) → planned ADDRC integration

groundSpring (V100) → planned uncertainty propagation on clinical models
hotSpring → planned lattice tissue finite-size scaling
```

---

## GPU Pipeline (Tier 2) — LIVE

All 24 Tier 0+1 experiments validated. GPU pipeline now live with 3 additional experiments:

### WGSL Shaders (f64 precision, compiled into binary)

| Shader | Operation | Pattern | Status |
|--------|-----------|---------|--------|
| `hill_dose_response_f64.wgsl` | E(c) = Emax·c^n / (c^n + EC50^n) | Element-wise | **Validated** (Exp053) |
| `population_pk_f64.wgsl` | AUC = F·Dose / CL(random) | Embarrassingly parallel MC | **Validated** (Exp053) |
| `diversity_f64.wgsl` | Shannon + Simpson indices | Workgroup reduction | **Validated** (Exp053) |

### GPU Architecture

| Component | Purpose | Location |
|-----------|---------|----------|
| `GpuContext` | Persistent device/queue, shader reuse | `barracuda/src/gpu.rs` |
| `execute_fused()` | All ops in one encoder, no CPU roundtrips | `barracuda/src/gpu.rs` |
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

## Next Steps: Tier 3 (metalForge Live Dispatch) and Field

1. **metalForge** — wire `select_substrate()` to live GPU dispatch (currently routing-only)
2. **Anderson eigensolve** — GPU shader for gut lattice localization length (Exp011/037)
3. **Biosignal FFT** — GPU radix-2 FFT for real-time ECG/PPG processing (Exp020-023)
4. **Field deployment** — validate on Raspberry Pi + eGPU (same WGSL, portable pipeline)
5. **TPU/NPU** — toadStool backend swap for Coral TPU, Akida NPU (Pan-Tompkins streaming)

---

## Integration Points

- **NestGate**: `data.ncbi_search` / `data.ncbi_fetch` for PubMed literature
- **biomeOS NUCLEUS**: Atomic deployment (Nest for data storage, Node for GPU compute)
- **toadStool/barraCuda**: GPU population PK Monte Carlo, fused diagnostic pipeline
- **wetSpring**: Diversity primitives (reuse `science.diversity`), Anderson lattice (cross-spring)
