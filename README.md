# healthSpring — Human Health Applications of Sovereign Scientific Computing

**An ecoPrimals Spring** — human health applications validating PK/PD, microbiome, biosignal, and endocrine pipelines against Python baselines via Pure Rust + barraCuda GPU. Follows the **Write → Absorb → Lean** cycle adopted from wetSpring/hotSpring.

**Date:** March 8, 2026
**License:** AGPL-3.0-or-later
**MSRV:** 1.87
**Status:** V3 — 103 lib tests, 17 experiments, 192 Python checks, 179 Rust binary checks. Tier 0+1 complete across all 4 tracks. Zero unsafe code, `cargo clippy --workspace -- -D warnings` **ZERO WARNINGS**. Ecosystem: barraCuda `v0.3.3`, wetSpring V99, neuralSpring V90.

---

## What This Is

healthSpring is the sixth ecoPrimals spring. Where the other five springs validate published science — reproducing papers to prove the pipeline — healthSpring builds **usable applications** of that validated science for human health.

The other springs do the chemistry. healthSpring makes the drug.

| Spring | Role | healthSpring relationship |
|--------|------|--------------------------|
| **wetSpring** | Life science validation (16S, LC-MS, immunology) | Gut microbiome analytics, Anderson colonization resistance, Exp037 cross-track |
| **neuralSpring** | ML primitives, PK/PD surrogates | Hill dose-response, population PK, clinical prediction |
| **hotSpring** | Plasma physics, lattice methods | Lattice tissue modeling, Anderson spectral theory |
| **airSpring** | Agricultural IoT, evapotranspiration | CytokineBrain → clinical cytokine network visualization |
| **groundSpring** | Uncertainty, spectral theory | Error propagation, confidence intervals for clinical tools |

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Version | V3 |
| Rust lib tests | 103 |
| Rust binary checks | 179 |
| Python control checks | 192 |
| Experiments complete | 17 (Tier 0+1) |
| GPU validation | — (Write phase next) |
| metalForge validation | — |
| Paper queue | 17/28 complete |
| Faculty | Gonzales (MSU Pharm/Tox), Lisabeth (ADDRC), Neubig (Drug Discovery), Mok (Allure Medical) |
| Unsafe blocks | 0 |
| Clippy warnings | 0 |

---

## Domains

### Track 1: Pharmacokinetic / Pharmacodynamic Modeling (Exp001-005)

Pure Rust PK/PD tools replacing Python/NONMEM dependency chains. Extends neuralSpring nS-601–605 (veterinary) to human therapeutics.

- Hill dose-response (4 human JAK inhibitors + canine reference) — Exp001
- One-compartment PK (IV bolus + oral Bateman + multiple dosing + AUC) — Exp002
- Two-compartment PK (biexponential α/β phases, peripheral compartment) — Exp003
- mAb PK cross-species transfer (lokivetmab → nemolizumab/dupilumab) — Exp004
- Population PK Monte Carlo (1,000 virtual patients, lognormal IIV) — Exp005

### Track 2: Gut Microbiome and Colonization Resistance (Exp010-012)

Extends wetSpring's Anderson localization framework from soil to gut.

- Shannon/Simpson/Pielou diversity indices + Chao1 richness — Exp010
- Anderson localization in gut lattice (1D localization length ξ) — Exp011
- C. difficile colonization resistance score — Exp012

### Track 3: Biosignal Processing (Exp020)

Real-time physiological signal analysis on sovereign hardware.

- Pan-Tompkins QRS detection (ECG R-peak) — Exp020

### Track 4: Endocrinology — Testosterone PK and TRT Outcomes (Exp030-037)

Clinical claim verification pipeline: extracting quantifiable claims from Dr. Charles Mok's clinical reference and validating against published registry data.

- Testosterone PK: IM injection steady-state (weekly vs biweekly) — Exp030
- Testosterone PK: pellet depot (5-month, zero-order release) — Exp031
- Age-related testosterone decline (Harman 2001 BLSA model) — Exp032
- TRT metabolic response: weight/BMI/waist (Saad 2013 registry) — Exp033
- TRT cardiovascular: lipids + CRP + BP (Saad 2016, Sharma 2015) — Exp034
- TRT diabetes: HbA1c + insulin sensitivity (Kapoor 2006 RCT) — Exp035
- Population TRT Monte Carlo (10K virtual patients, IIV + age-adjustment) — Exp036
- Testosterone–gut axis: microbiome stratification (cross-track 2×4) — Exp037

---

## Validation Protocol

```
Tier 0: Python control (published algorithm, reference implementation)
Tier 1: Rust CPU (Pure Rust, f64-canonical, tolerance-documented)
Tier 2: Rust GPU (barraCuda WGSL shaders, math parity with CPU)
Tier 3: metalForge (toadStool dispatch, cross-substrate routing)
```

**Current state**: Tier 0+1 validated for all 17 experiments. Next: barraCuda GPU shaders.

---

## Directory Structure

```
healthSpring/
├── barracuda/           # Rust library — PK/PD, microbiome, biosignal, endocrine
│   └── src/
│       ├── lib.rs       # 103 tests, #![forbid(unsafe_code)]
│       ├── pkpd.rs      # Track 1: Hill, 1/2-compartment, allometric, pop PK
│       ├── microbiome.rs # Track 2: Shannon, Simpson, Pielou, Chao1, Anderson W
│       ├── biosignal.rs  # Track 3: Pan-Tompkins, IIR bandpass, HRV
│       └── endocrine.rs  # Track 4: testosterone PK, decline, TRT outcomes, gut axis
├── control/             # Python baselines (Tier 0) — 192 checks
│   ├── pkpd/            # exp001–exp005 + cross_validate.py
│   ├── microbiome/      # exp010–exp012
│   ├── biosignal/       # exp020
│   └── endocrine/       # exp030–exp037
├── experiments/         # Validation binaries (Tier 1) — 179 checks
│   ├── exp001_hill_dose_response/
│   ├── exp002_one_compartment_pk/
│   ├── exp005_population_pk/
│   ├── exp011_anderson_gut_lattice/
│   ├── exp012_cdiff_resistance/
│   ├── exp020_pan_tompkins_qrs/
│   ├── exp030_testosterone_im_pk/
│   ├── exp031_testosterone_pellet_pk/
│   ├── exp032_age_testosterone_decline/
│   ├── exp033_trt_weight_trajectory/
│   ├── exp034_trt_cardiovascular/
│   ├── exp035_trt_diabetes/
│   ├── exp036_population_trt_montecarlo/
│   └── exp037_testosterone_gut_axis/
├── metalForge/          # Cross-substrate dispatch (Tier 3, scaffold)
│   └── forge/
├── specs/               # Paper queue, compute profile, integration plan
├── whitePaper/          # Scientific documentation
│   ├── baseCamp/        # Faculty-linked sub-theses
│   └── experiments/     # Experiment plan and status
├── wateringHole/        # Cross-spring handoffs
│   └── handoffs/        # → barraCuda, toadStool, biomeOS
├── Cargo.toml           # Workspace (16 crates)
└── README.md            # This file
```

---

## Build

```bash
cargo test --workspace                  # 103 lib tests
cargo clippy --workspace -- -D warnings # Lint — zero warnings

# Run individual validation binaries
cargo run --bin exp030_testosterone_im_pk
cargo run --bin exp036_population_trt_montecarlo
cargo run --bin exp037_testosterone_gut_axis

# Python controls
python3 control/endocrine/exp030_testosterone_im_pk.py
python3 control/endocrine/exp036_population_trt_montecarlo.py
```

---

## Relationship to ecoPrimals

healthSpring is a public scientific validation repository in the ecoPrimals ecosystem. It consumes `barraCuda` (vendor-agnostic GPU math library) and validates health application pipelines using the same constrained evolution methodology as the other five springs.

The springs validate science. healthSpring applies it.
