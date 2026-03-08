# healthSpring Experiments

Validation experiments documenting the four-tier pipeline (Python → Rust CPU → GPU → metalForge) for each health application domain.

**Status**: V3 — 17 experiments complete (Tier 0+1), 103 Rust lib tests, 179 binary checks, 192 Python checks
**Last Updated**: March 8, 2026

---

## Completed Experiments

### Track 1: PK/PD Modeling

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 001 | Hill dose-response (4 human JAK inhibitors) | nS-601 extension | 0,1 | 19 | 18 |
| 002 | One-compartment PK (IV bolus + oral + multi-dose) | Rowland & Tozer Ch. 3 | 0,1 | 12 | 18 |
| 003 | Two-compartment PK (biexponential α/β) | Rowland & Tozer Ch. 19 | 0,1 | 15 | 11 |
| 004 | mAb PK cross-species transfer (lokivetmab → nemolizumab) | nS-603 extension | 0,1 | 12 | 7 |
| 005 | Population PK Monte Carlo (1,000 patients) | Mould & Upton 2013 | 0,1 | 15 | 12 |

### Track 2: Gut Microbiome

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 010 | Shannon/Simpson/Pielou/Chao1 diversity | wetSpring Track 1 | 0,1 | 14 | 12 |
| 011 | Anderson localization in gut lattice | wetSpring Exp107 extension | 0,1 | 12 | 14 |
| 012 | C. diff colonization resistance score | Jenior 2021 / Anderson ξ | 0,1 | 10 | 10 |

### Track 3: Biosignal Processing

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 020 | Pan-Tompkins QRS detection | Pan & Tompkins 1985 | 0,1 | 12 | 12 |

### Track 4: Endocrinology (Testosterone PK / TRT Outcomes)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 030 | Testosterone PK: IM injection steady-state | Shoskes 2016, Ross 2004 | 0,1 | 12 | 11 |
| 031 | Testosterone PK: pellet depot (5-month) | Testopel label, Cavender 2009 | 0,1 | 10 | 10 |
| 032 | Age-related testosterone decline | Harman 2001 (BLSA, n=890) | 0,1 | 10 | 8 |
| 033 | TRT metabolic response: weight/BMI/waist | Saad 2013 (n=411, 5yr) | 0,1 | 10 | 7 |
| 034 | TRT cardiovascular: lipids + CRP + BP | Sharma 2015 (VA, n=83,010) | 0,1 | 10 | 10 |
| 035 | TRT diabetes: HbA1c + insulin sensitivity | Kapoor 2006 (RCT) | 0,1 | 10 | 10 |
| 036 | Population TRT Monte Carlo (10K patients) | Lognormal IIV, age-adjusted | 0,1 | 12 | 10 |
| 037 | Testosterone–gut axis: microbiome stratification | Cross-track 2×4 hypothesis | 0,1 | 12 | 10 |

### Cross-Validation

| Test | Scope | Matches | Status |
|------|-------|:-------:|--------|
| cross_validate.py | Exp001 + Exp002 Python ↔ Rust | 17/17 | **Complete** |

---

## Directory Layout

```
experiments/
├── exp001_hill_dose_response/
├── exp002_one_compartment_pk/
├── exp005_population_pk/
├── exp011_anderson_gut_lattice/
├── exp012_cdiff_resistance/
├── exp020_pan_tompkins_qrs/
├── exp030_testosterone_im_pk/
├── exp031_testosterone_pellet_pk/
├── exp032_age_testosterone_decline/
├── exp033_trt_weight_trajectory/
├── exp034_trt_cardiovascular/
├── exp035_trt_diabetes/
├── exp036_population_trt_montecarlo/
├── exp037_testosterone_gut_axis/
└── results/                          # (populated by CI runs)
```

Controls live in `control/`:
```
control/
├── pkpd/
│   ├── exp001_hill_dose_response.py
│   ├── exp002_one_compartment_pk.py
│   ├── exp003_two_compartment_pk.py
│   ├── exp004_mab_pk_transfer.py
│   ├── exp005_population_pk.py
│   └── cross_validate.py
├── microbiome/
│   ├── exp010_diversity_indices.py
│   ├── exp011_anderson_gut_lattice.py
│   └── exp012_cdiff_resistance.py
├── biosignal/
│   └── exp020_pan_tompkins_qrs.py
└── endocrine/
    ├── exp030_testosterone_im_pk.py
    ├── exp031_testosterone_pellet_pk.py
    ├── exp032_age_testosterone_decline.py
    ├── exp033_trt_weight_trajectory.py
    ├── exp034_trt_cardiovascular.py
    ├── exp035_trt_diabetes.py
    ├── exp036_population_trt_montecarlo.py
    └── exp037_testosterone_gut_axis.py
```

---

## Numbering Convention

- **001–009**: Track 1 (PK/PD)
- **010–019**: Track 2 (Microbiome)
- **020–029**: Track 3 (Biosignal)
- **030–039**: Track 4 (Endocrinology)
- **040+**: Extensions and cross-spring validations

---

## How to Add a New Experiment

1. Write Python control in `control/{track}/exp{NNN}_{name}.py` — runs checks inline
2. Add Rust implementations to `barracuda/src/{module}.rs` with `#[cfg(test)]` unit tests
3. Create validation binary in `experiments/exp{NNN}_{name}/` (workspace member)
4. Run Python control → Rust unit tests → validation binary
5. Update this README, `specs/PAPER_REVIEW_QUEUE.md`, and `whitePaper/baseCamp/`
