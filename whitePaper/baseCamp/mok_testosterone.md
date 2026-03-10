# Sub-Thesis 04: Testosterone Replacement Therapy — Clinical Claim Verification Pipeline

**Source**: Dr. Charles Mok, *If Your Testosterone Is Low, You're Gonna Get Fat* (Allure Medical Publishing, 2018, 196 pages)
**Status**: Complete — 9 experiments validated (Exp030–038) + NLME population PK (Exp075), 96 Python + 86 Rust binary + 51 lib unit tests + 19 NLME checks. Patient-parameterized clinical scenarios (Exp063) close the per-person translation loop: `PatientTrtProfile` → scenario graph → petalTongue clinical mode. Live clinical TRT dashboard (Exp073) streams PK trough, HRV improvement, HbA1c trajectory, and cardiac risk comparison via `replace` stream operation. 5 TRT archetypes wired into `dump_scenarios`. Interaction roundtrip validated (Exp074). V14: NLME population PK (FOCE + SAEM), NCA (λz, AUC∞, MRT, CL, Vss), NLME diagnostics (CWRES, VPC, GOF) — sovereign NONMEM/Monolix/WinNonlin replacement.
**Last Updated**: March 10, 2026

---

## The Challenge

This is **not a standard scientific paper reproduction**. Mok's book is a clinical practice reference that aggregates ~200 published studies into narrative claims about testosterone replacement therapy (TRT). The challenge is twofold:

1. **Extract quantifiable claims** from narrative text
2. **Validate each claim** against the cited primary literature using open data

This creates a novel pipeline: **Narrative → Claim Extraction → Primary Source Verification → Computational Modeling → Prediction**. If successful, this pipeline generalizes to any clinical practice book or review article.

---

## Book Structure (11 Chapters, 196 pages)

| Ch | Title | Key Quantifiable Claims | Primary Sources Cited |
|----|-------|------------------------|----------------------|
| 1 | How the FDA Changed Everything | TRT prescription rates, FDA 2014 rule change, label restrictions | FDA BRUDAC Sept 2014 |
| 2 | What the FDA Doesn't Want You to Know | Risk-benefit of TRT, flawed study methodology | Vigen 2013 (retracted concerns), Finkle 2014 |
| 3 | How to Diagnose Low T | Threshold levels (280/200 ng/dL), testing protocols, free T vs total T | Endocrine Society 2010 |
| 4 | T and Weight Gain | TRT → weight loss, dose-response, registry data | **Saad 2013** (n=411), **Saad 2016** (n=411), Traish 2014 |
| 5 | T and Type 2 Diabetes | TRT → HbA1c reduction, insulin sensitivity | Kapoor 2006 (RCT), Dhindsa 2016, Hackett 2014 |
| 6 | T and Cardiovascular Disease | TRT → reduced MI/stroke, lipids, CRP, BP | **Sharma 2015** (VA n=83,010), Shores 2012, Muraleedharan 2013 |
| 7 | T and Urinary Tract | TRT and BPH/LUTS improvement | Karazindiyanoğlu 2008, Shigehara 2011 |
| 8 | T and Musculoskeletal | Bone density, muscle mass, sarcopenia | Snyder 2016 (TTrials), Bhasin 2010 |
| 9 | T and Sexual Function | IIEF scores, libido, ED improvement | Snyder 2016, Hackett 2016 |
| 10 | T and Mood/Brain | Depression scores, cognitive function | Snyder 2016, Zarrouf 2009 meta-analysis |
| 11 | Treating Low T | Pellet/injection/gel PK, dosing, steady-state | Testopel label, Ross 2004 |

---

## Quantifiable Claims for Computational Modeling

### Category A: PK/PD (Direct Modeling — healthSpring Track 1 extension)

| # | Claim | Model | Parameters | Open Data Source |
|---|-------|-------|------------|------------------|
| A1 | Age-related T decline: 1-3%/yr after age 30 | Exponential decay: T(age) = T₀ · exp(-k · (age - 30)) | T₀ = 600-700 ng/dL, k = 0.01-0.03/yr | Harman 2001 (BLSA, n=890), Feldman 2002 (MMAS, n=1,709) |
| A2 | IM injection PK: weekly vs biweekly steady-state | One-compartment IM depot: C(t) = (D·k_a)/(Vd·(k_a-k_e))·(e^(-k_e·t) - e^(-k_a·t)) | k_a from Shoskes 2016, t½ ≈ 8 days (cypionate) | Ross 2004 (buccal PK), Testopel label |
| A3 | Pellet depot PK: 5-month sustained release | Zero-order release → first-order absorption | Dose 2000mg, duration 150 days, target 700-1000 ng/dL | Cavender 2009, Testopel PI |
| A4 | Undecanoate IM: 10-week long-acting PK | Two-compartment depot with slow absorption | Nebido/Aveed label data | Behre 2004 (n=19) |
| A5 | Topical gel: twice-daily application PK | Transdermal first-order with skin depot | AndroGel 1.62% label | Swerdloff 2000 (pharmacokinetics) |
| A6 | Dose-response: 10mg/lb pellet → therapeutic | Population PK with BW covariate | Weight-based dosing model | Allure clinical (not open) + population PK simulation |

### Category B: Metabolic Outcomes (Time-Series Modeling)

| # | Claim | Model | Data Source |
|---|-------|-------|-------------|
| B1 | TRT → sustained weight loss (years) | Longitudinal mixed-effects: ΔBW(t) = β₀ + β₁·log(t) + u_i | Saad 2013 (n=411, 5yr, mean -16kg) |
| B2 | TRT → waist circumference reduction | Same model, waist endpoint | Saad 2016 (n=411, -12cm mean) |
| B3 | TRT → HbA1c reduction in T2DM | Exponential decay to new setpoint | Kapoor 2006 (RCT, -0.37%), Hackett 2014 |
| B4 | TRT → improved insulin sensitivity | HOMA-IR model | Dhindsa 2016, Rubinow 2012 |
| B5 | Interrupted TRT → rebound (weight regain) | Saad 2016 continuous vs interrupted | Saad 2016 (continuous n=115, interrupted n=147) |

### Category C: Cardiovascular (Survival/Hazard Modeling)

| # | Claim | Model | Data Source |
|---|-------|-------|-------------|
| C1 | TRT normalization → reduced MI incidence | Cox proportional hazards | Sharma 2015 (VA, n=83,010, HR=0.44) |
| C2 | TRT → reduced all-cause mortality | Kaplan-Meier + log-rank | Shores 2012 (n=1,031), Muraleedharan 2013 |
| C3 | TRT → LDL reduction, HDL increase | Linear dose-response over time | Saad 2016 interrupted/continuous (published curves) |
| C4 | TRT → CRP reduction (inflammation) | Exponential decay | Saad 2016 (CRP: 1.4 → 0.9 mg/dL) |
| C5 | TRT → blood pressure normalization | SBP/DBP time series | Saad 2016 (SBP: 135 → 125 mmHg) |

### Category D: Cross-Track Hypotheses (Novel)

| # | Hypothesis | Tracks Involved | Model |
|---|-----------|-----------------|-------|
| D1 | Gut microbiome diversity predicts TRT response | Track 2 × Track 4 | Pielou evenness → treatment response stratification |
| D2 | Anderson gut confinement correlates with metabolic syndrome | Track 2 × Track 4 | ξ(gut) → HOMA-IR, BMI |
| D3 | HRV improvement tracks TRT cardiovascular benefit | Track 3 × Track 4 | SDNN/RMSSD → cardiac risk reduction |
| D4 | Population TRT Monte Carlo with IIV + microbiome covariate | Tracks 1+2+4 | 10K virtual patients, Monte Carlo |

---

## Compute & Data Profile

### Data Requirements

| Source | Size | Access | NestGate Route |
|--------|------|--------|----------------|
| PubMed abstracts (TRT, testosterone) | ~5,000 abstracts | `data.ncbi_search` | Free, E-utilities |
| Harman 2001 BLSA serum T tables | Published figures (digitize) | Manual extraction | Stored in NestGate |
| Saad 2013/2016 registry curves | Published figures (digitize) | Manual extraction | Stored in NestGate |
| Sharma 2015 VA cohort summary stats | Published tables | Extracted | Stored in NestGate |
| FDA BRUDAC testimony 2014 | Public record | FDA.gov | Stored in NestGate |
| Testopel/Nebido PK label data | FDA labels | Public (DailyMed) | Stored in NestGate |
| GEO androgen receptor expression | ~50GB raw, ~500MB processed | `data.ncbi_fetch` (GEO) | SRA accession |

### Compute Requirements

| Experiment | CPU (Tier 1) | GPU (Tier 2) | Memory | Time |
|------------|:------------:|:------------:|:------:|:----:|
| A1: Age T decline (curve fitting) | Trivial | Not needed | <1MB | <1s |
| A2-A5: Testosterone PK formulations | Moderate (ODE solver) | ODE batch GPU | <10MB | <10s |
| A6: Population PK (10K patients) | Heavy (10K ODE solves) | **Ideal** (embarrassingly parallel) | ~100MB | ~1min CPU, ~1s GPU |
| B1-B5: Metabolic time series | Moderate (mixed-effects) | Batch GPU for MCMC | ~50MB | ~30s CPU |
| C1-C2: Survival/hazard models | Moderate | Not needed (analytical) | <10MB | <5s |
| D4: Population Monte Carlo (10K × microbiome) | **Heavy** | **Required** (Monte Carlo × ODE × Anderson) | ~500MB | ~10min CPU, ~10s GPU |

### Hardware Mapping (Basement HPC)

| Gate | Role | Why |
|------|------|-----|
| Eastgate (i9-12900, RTX 4070, 32GB) | Development + Tier 1 testing | Current workstation |
| Northgate (i9-14900K, RTX 5090, 192GB) | GPU Tier 2 (population Monte Carlo) | Largest VRAM, fastest GPU |
| Strandgate (Dual EPYC, 256GB) | NCBI data + batch processing | Most cores, most RAM |
| biomeGate (TR 3970X, Titan V, 256GB) | f64-native GPU validation | Titan V has native f64 |
| Westgate (76TB ZFS) | Cold storage for NCBI data | Storage gateway |

---

## Implementation Plan

### Phase 1: PK Foundations (Exp030-035) — COMPLETE

All 6 experiments validated at Tier 0 (Python) + Tier 1 (Rust CPU):

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 030 | Testosterone PK: IM injection steady-state | 12 | 11 |
| 031 | Testosterone PK: pellet depot (5-month) | 10 | 10 |
| 032 | Testosterone decline: age-related hypogonadism | 10 | 8 |
| 033 | TRT metabolic response: weight/BMI/waist | 10 | 7 |
| 034 | TRT cardiovascular: lipids + CRP + BP | 10 | 10 |
| 035 | TRT diabetes: HbA1c + insulin sensitivity | 10 | 10 |

### Phase 2: Population Modeling + Cross-Track (Exp036-038) — COMPLETE

| Exp | Title | Python | Rust Binary |
|-----|-------|:------:|:-----------:|
| 036 | Population TRT Monte Carlo (10K virtual patients, IIV + age) | 12 | 10 |
| 037 | Testosterone–gut axis: microbiome stratification (cross-track) | 12 | 10 |
| 038 | HRV × TRT cardiovascular (cross-track D3, Kleiger 1987) | 10 | 10 |

Exp037 validates cross-track hypotheses D1/D2: Pielou evenness → Anderson disorder → gut metabolic response, stratified by high/low microbiome diversity.

Exp038 validates cross-track hypothesis D3: SDNN improvement correlates with TRT cardiovascular benefit. Composite cardiac risk score combines HRV and testosterone level factors (Kleiger 1987, Mok Ch. 6).

### Phase 3: GPU + LAN HPC (Tier 2+3) — LIVE

GPU pipeline validated (Exp053-055). CPU vs GPU parity matrix (Exp060, 27/27). Mixed hardware dispatch via NUCLEUS topology (Exp061, 22/22). PCIe P2P transfer planning (Exp062, 26/26).

1. Population PK on barraCuda GPU (10K patients) — **validated** (Exp053, PopPK shader)
2. metalForge: dispatch planning with NUCLEUS topology — **validated** (Exp061)
3. biomeOS NUCLEUS deployment graph for healthSpring — **defined** (`healthspring_deploy.toml`)

### Phase 4: Per-Person Translation (V9) — COMPLETE

Patient-parameterized clinical TRT scenarios close the gap between population models and individual patient care.

1. `PatientTrtProfile` → `trt_clinical_scenario()` → 8-node graph with edges, channels, ranges — **Exp063**
2. 5 patient archetypes (young athletic, middle metabolic, older cardiovascular, diabetic, post-FMT) — **validated**
3. petalTongue clinical mode preset (sidebars hidden, awakening skipped, graph fitted) — **Exp063**
4. IPC push to running petalTongue via Unix socket JSON-RPC — **Exp064**
5. SAME DAVE motor command channel bridges scenario config → UI state — **petalTongue V9**

### Phase 5: Live Clinical Dashboard (V12) — COMPLETE

Streaming clinical visualization of TRT treatment over time.

1. Live PK trough streaming via `push_timeseries` (12-week simulation, weekly updates) — **Exp073**
2. HRV improvement gauge via `push_gauge` (tracks SDNN improvement across treatment) — **Exp073**
3. HbA1c trajectory and cardiac risk comparison via `push_replace_binding` (Bar channel swap at weeks 4/8/12) — **Exp073**
4. Domain theming: `domain=clinical` with `UiConfig` passthrough for clinical mode rendering — **Exp073**
5. Capability querying and interaction subscription validated end-to-end — **Exp074** (mock petalTongue, 12/12)
6. 5 TRT archetypes wired into `dump_scenarios` pipeline (13 total scenarios) — **Exp056**

### Phase 6: NLME Population PK (V14) — COMPLETE

Sovereign replacement for NONMEM (FOCE), Monolix (SAEM), and WinNonlin (NCA). Applied to testosterone cypionate population PK using Mok 2018 / Shoskes 2016 parameters.

1. FOCE estimation (150 iterations, 30 subjects) — theta, omega, sigma recovery validated (Exp075)
2. SAEM estimation (200 iterations, 30 subjects) — stochastic E-step, wider tolerance than FOCE (Exp075)
3. NCA metrics: lambda-z terminal slope, AUC∞ extrapolation, MRT, clearance, Vss (Exp075)
4. NLME diagnostics: CWRES (mean <2.0), VPC (50 simulations, 5th/50th/95th percentile bands), GOF (R²≥0) (Exp075)
5. Full petalTongue pipeline: 5 new NLME nodes (population, NCA, CWRES, VPC, GOF) wired into full study (28 nodes, 121 channels) (Exp076)
6. Deterministic reproducibility: same seed → identical FOCE/SAEM objective function values (Exp075)

---

## Connection to gen3/baseCamp Paper 13

This sub-thesis is the primary driver for baseCamp Paper 13's Track 4 (Endocrinology). The "claim verification pipeline" is the novel contribution — if we can systematically extract and validate claims from clinical books, the same pipeline applies to any medical reference, review article, or clinical guideline. The computational models we build (testosterone PK, metabolic time series, survival analysis) are validated against the same published registry data that Mok cites, creating a closed validation loop.

The cross-track hypotheses (D1-D4) connect this work to the Anderson localization framework that underlies the entire ecoPrimals thesis: gut microbiome diversity → Anderson disorder → colonization resistance → metabolic health → TRT response.
