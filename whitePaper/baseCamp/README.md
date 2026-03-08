# healthSpring baseCamp

Faculty-linked sub-theses documenting how healthSpring extends validated science into human health applications.

**Last Updated:** March 8, 2026
**Status:** V3 — 4 tracks active, 17 experiments complete (103 Rust lib tests, 179 binary checks, 192 Python checks)

---

## Tracks

| Track | Domain | Experiments | Status |
|-------|--------|-------------|--------|
| 1 — PK/PD | Pharmacokinetics, dose-response, population modeling | Exp001-005 | **Complete** (Tier 0+1) |
| 2 — Microbiome | Gut diversity, Anderson lattice, colonization resistance | Exp010-012 | **Complete** (Tier 0+1) |
| 3 — Biosignal | ECG detection, HRV, wearable fusion | Exp020 | **Complete** (Tier 0+1) |
| 4 — Endocrinology | Testosterone PK, TRT outcomes, gut axis, population MC | Exp030-037 | **Complete** (Tier 0+1) |

---

## Sub-Theses

| # | File | Faculty | Domain | Status | Python | Rust |
|---|------|---------|--------|--------|:------:|:----:|
| 01 | [gonzales.md](gonzales.md) | Gonzales, Lisabeth, Neubig | PK/PD + immunology → human therapeutics | **Complete** | 73 | 66 |
| 02 | cdiff_colonization.md | TBD | Anderson localization → gut colonization resistance | **Complete** | 36 | 36 |
| 03 | biosignal_sovereign.md | TBD | Edge biosignal processing on sovereign hardware | **Complete** | 12 | 12 |
| 04 | [mok_testosterone.md](mok_testosterone.md) | Dr. Charles Mok | Testosterone PK, TRT outcomes, clinical claim verification | **Complete** | 86 | 76 |

---

## Validation Summary

| Sub-thesis | Upstream Checks | Tier 0 (Python) | Tier 1 (Rust) | Total |
|------------|:---------------:|:---------------:|:-------------:|:-----:|
| Gonzales PK/PD (Exp001-005) | 688 | 73 | 66 | 827 |
| Microbiome / C. diff (Exp010-012) | wetSpring Anderson | 36 | 36 | 72+ |
| Biosignal / QRS (Exp020) | — | 12 | 12 | 24 |
| Mok Testosterone (Exp030-037) | — | 86 | 76 | 162 |
| **Lib unit tests** | — | — | **103** | 103 |
| **Total** | **688** | **192** (Tier 0) | **179** (binary) + **103** (lib) = **282** | **1,162+** |

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

### Track 2 — Microbiome

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 010 | Diversity indices (Shannon/Simpson/Pielou/Chao1) | 14 | 12 |
| 011 | Anderson gut lattice (localization → resistance) | 12 | 14 |
| 012 | C. diff colonization resistance score | 10 | 10 |

### Track 3 — Biosignal

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 020 | Pan-Tompkins QRS detection | 12 | 12 |

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

## Next Steps: Tier 2 (GPU) and Beyond

All 17 experiments are Tier 0 (Python control) and Tier 1 (Rust CPU) validated.
The path forward:

1. **barraCuda CPU** — already proving pure Rust math matches Python (Tier 1 complete)
2. **barraCuda GPU** — WGSL shaders for embarrassingly parallel workloads:
   - `population_pk_sample.wgsl` (Exp005/036 — first GPU experiment)
   - `hill_equation_gpu.wgsl` (Exp001 vectorized)
   - `anderson_xi_1d_gut.wgsl` (Exp011/037 eigensolve)
3. **toadStool** — unidirectional streaming dispatch (CPU→GPU, no round-trips)
4. **metalForge** — cross-substrate (GPU→NPU→CPU) heterogeneous deployment

---

## Integration Points

- **NestGate**: `data.ncbi_search` / `data.ncbi_fetch` for PubMed literature, SRA androgen receptor expression data
- **biomeOS NUCLEUS**: Atomic deployment (Nest for data storage, Node for GPU compute), population PK as distributed workload
- **ToadStool/BarraCUDA**: GPU population PK Monte Carlo, Anderson gut eigensolve, FFT for biosignal
- **wetSpring**: Diversity primitives (reuse `science.diversity`), Anderson lattice (cross-spring)
