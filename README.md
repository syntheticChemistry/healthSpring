# healthSpring — Human Health Applications of Sovereign Scientific Computing

**An ecoPrimals Spring** — human health applications validating PK/PD, microbiome, biosignal, and endocrine pipelines against Python baselines via Pure Rust + barraCuda GPU. Follows the **Write → Absorb → Lean** cycle adopted from wetSpring/hotSpring.

**Date:** March 9, 2026
**License:** AGPL-3.0-or-later
**MSRV:** 1.87
**Status:** V6.1 — 201 unit tests (161 barraCuda + 27 forge + 13 toadStool), 30 experiments, 371 Rust binary checks, 104 cross-validation checks. **Tier 2 (GPU) live**: 3 WGSL shaders (Hill, PopPK, Diversity), `GpuContext` persistent device, fused unidirectional pipeline, toadStool GPU dispatch. **petalTongue absorption complete**: DataChannel, ClinicalRange, renderers, clinical theme absorbed upstream; `petaltongue-health` crate removed (lean phase). Zero unsafe code, `cargo clippy --workspace -- -D warnings` **ZERO WARNINGS**.

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
| Version | V6 |
| Rust lib tests | 200 (160 barraCuda + 27 forge + 13 toadStool) |
| Rust binary checks | 346 |
| Python control checks | 104 (cross-validation) |
| Experiments complete | 30 (Tier 0+1 + diagnostic + petalTongue + GPU) |
| GPU validation (Tier 2) | **Live** — 3 WGSL shaders, fused pipeline, 17/17 parity checks |
| GPU scaling | Hill crossover 100K, PK crossover 5M, peak 207 M elements/s |
| metalForge validation | 27 tests |
| toadStool validation | 13 tests + GPU dispatch (Pipeline::execute_gpu) |
| petalTongue prototype | Interactive egui dashboard |
| Paper queue | 24/30 complete |
| Faculty | Gonzales (MSU Pharm/Tox), Lisabeth (ADDRC), Neubig (Drug Discovery), Mok (Allure Medical) |
| Unsafe blocks | 0 |
| Clippy warnings | 0 |

---

## Domains

### Track 1: Pharmacokinetic / Pharmacodynamic Modeling (Exp001-006)

Pure Rust PK/PD tools replacing Python/NONMEM dependency chains. Extends neuralSpring nS-601–605 (veterinary) to human therapeutics.

- Hill dose-response (4 human JAK inhibitors + canine reference) — Exp001
- One-compartment PK (IV bolus + oral Bateman + multiple dosing + AUC) — Exp002
- Two-compartment PK (biexponential α/β phases, peripheral compartment) — Exp003
- mAb PK cross-species transfer (lokivetmab → nemolizumab/dupilumab) — Exp004
- Population PK Monte Carlo (1,000 virtual patients, lognormal IIV) — Exp005
- PBPK multi-compartment (liver, gut, systemic) — Exp006

### Track 2: Gut Microbiome and Colonization Resistance (Exp010-013)

Extends wetSpring's Anderson localization framework from soil to gut.

- Shannon/Simpson/Pielou diversity indices + Chao1 richness — Exp010
- Anderson localization in gut lattice (1D localization length ξ) — Exp011
- C. difficile colonization resistance score — Exp012
- FMT RCDI (fecal microbiota transplant, recurrent C. difficile) — Exp013

### Track 3: Biosignal Processing (Exp020-023)

Real-time physiological signal analysis on sovereign hardware.

- Pan-Tompkins QRS detection (ECG R-peak) — Exp020
- HRV metrics (RMSSD, pNN50, LF/HF) — Exp021
- PPG SpO₂ (pulse oximetry, reflectance) — Exp022
- Biosignal fusion (ECG + PPG multi-modal) — Exp023

### Track 4: Endocrinology — Testosterone PK and TRT Outcomes (Exp030-038)

Clinical claim verification pipeline: extracting quantifiable claims from Dr. Charles Mok's clinical reference and validating against published registry data.

- Testosterone PK: IM injection steady-state (weekly vs biweekly) — Exp030
- Testosterone PK: pellet depot (5-month, zero-order release) — Exp031
- Age-related testosterone decline (Harman 2001 BLSA model) — Exp032
- TRT metabolic response: weight/BMI/waist (Saad 2013 registry) — Exp033
- TRT cardiovascular: lipids + CRP + BP (Saad 2016, Sharma 2015) — Exp034
- TRT diabetes: HbA1c + insulin sensitivity (Kapoor 2006 RCT) — Exp035
- Population TRT Monte Carlo (10K virtual patients, IIV + age-adjustment) — Exp036
- Testosterone–gut axis: microbiome stratification (cross-track 2×4) — Exp037
- HRV–TRT cardiovascular (cross-track 3×4) — Exp038

### Integrated Diagnostics (Exp050-052)

- Integrated patient diagnostic pipeline (4 tracks + cross-track + composite risk) — Exp050
- Population diagnostic Monte Carlo (1,000 virtual patients) — Exp051
- petalTongue scenario schema validation (DataChannel, ClinicalRange) — Exp052

### GPU Pipeline (Exp053-055)

- GPU parity: WGSL shader output vs CPU baseline (Hill, PopPK, Diversity) — Exp053
- Fused pipeline: all ops in one GPU submission, toadStool dispatch — Exp054
- GPU scaling: 1K→10M sweep, crossover analysis, field deployment thesis — Exp055

### Validation Track (Exp040)

- barraCuda CPU parity (Tier 0+1 baseline for GPU migration) — Exp040

---

## Validation Protocol

```
Tier 0: Python control (published algorithm, reference implementation)
Tier 1: Rust CPU (Pure Rust, f64-canonical, tolerance-documented)
Tier 2: Rust GPU (barraCuda WGSL shaders, math parity with CPU)
Tier 3: metalForge (toadStool dispatch, cross-substrate routing)
```

**Current state**: Tier 0+1 complete for 24 experiments. **Tier 2 live**: 3 WGSL shaders compiled and validated (Exp053), fused unidirectional pipeline (Exp054), scaling to 10M elements (Exp055). toadStool `Pipeline::execute_gpu()` dispatches stages via `GpuContext`. metalForge substrate routing (Tier 3 foundation).

---

## Directory Structure

```
healthSpring/
├── barracuda/           # Rust library — PK/PD, microbiome, biosignal, endocrine
│   └── src/
│       ├── lib.rs       # 145 tests, #![forbid(unsafe_code)]
│       ├── pkpd/        # Track 1: Hill, 1/2-compartment, allometric, pop PK, PBPK
│       ├── microbiome.rs # Track 2: Shannon, Simpson, Pielou, Chao1, Anderson W, FMT
│       ├── biosignal.rs  # Track 3: Pan-Tompkins, IIR bandpass, HRV, PPG, fusion
│       ├── endocrine.rs  # Track 4: testosterone PK, decline, TRT outcomes, gut axis
│       ├── gpu.rs       # Tier 2: GPU dispatch + GpuContext + fused pipeline
│       └── visualization/ # petalTongue schema (DataChannel, ClinicalRange)
│   └── shaders/health/  # WGSL compute kernels (f64)
│       ├── hill_dose_response_f64.wgsl
│       ├── population_pk_f64.wgsl
│       └── diversity_f64.wgsl
├── control/             # Python baselines (Tier 0) — 104 cross-validation checks
│   ├── pkpd/            # exp001–exp006 + cross_validate.py
│   ├── microbiome/      # exp010–exp013
│   ├── biosignal/       # exp020–exp023
│   ├── endocrine/       # exp030–exp038
│   └── validation/     # Exp040 CPU parity
├── experiments/         # Validation binaries (Tier 1) — 280 checks
│   ├── exp001_hill_dose_response/
│   ├── exp002_one_compartment_pk/
│   ├── exp003_two_compartment_pk/
│   ├── exp004_mab_pk_transfer/
│   ├── exp005_population_pk/
│   ├── exp006_pbpk_compartments/
│   ├── exp010_diversity_indices/
│   ├── exp011_anderson_gut_lattice/
│   ├── exp012_cdiff_resistance/
│   ├── exp013_fmt_rcdi/
│   ├── exp020_pan_tompkins_qrs/
│   ├── exp021_hrv_metrics/
│   ├── exp022_ppg_spo2/
│   ├── exp023_biosignal_fusion/
│   ├── exp030_testosterone_im_pk/
│   ├── exp031_testosterone_pellet_pk/
│   ├── exp032_age_testosterone_decline/
│   ├── exp033_trt_weight_trajectory/
│   ├── exp034_trt_cardiovascular/
│   ├── exp035_trt_diabetes/
│   ├── exp036_population_trt_montecarlo/
│   ├── exp037_testosterone_gut_axis/
│   ├── exp038_hrv_trt_cardiovascular/
│   ├── exp040_barracuda_cpu_parity/
│   ├── exp050_diagnostic_pipeline/
│   ├── exp051_population_diagnostic/
│   ├── exp052_petaltongue_render/
│   ├── exp053_gpu_parity/         # Tier 2: WGSL vs CPU validation
│   ├── exp054_gpu_pipeline/       # Fused pipeline + toadStool GPU dispatch
│   └── exp055_gpu_scaling/        # 1K→10M scaling, crossover, field thesis
├── # petaltongue-health/  — REMOVED (V6.1): absorbed into petalTongue upstream
│   # DataChannel, ClinicalRange, renderers, clinical theme → petal-tongue-core + petal-tongue-graph
├── metalForge/          # Cross-substrate dispatch (Tier 3)
│   └── forge/
│       └── src/
│           ├── nucleus.rs
│           └── transfer.rs
├── toadstool/           # Compute dispatch pipeline
│   └── src/
│       ├── pipeline.rs
│       └── stage.rs
├── specs/               # Paper queue, compute profile, integration plan
├── whitePaper/          # Scientific documentation
│   ├── baseCamp/        # Faculty-linked sub-theses
│   └── experiments/     # Experiment plan and status
├── wateringHole/        # Cross-spring handoffs
│   └── handoffs/        # → barraCuda, toadStool, biomeOS
├── Cargo.toml           # Workspace
└── README.md            # This file
```

---

## Build

```bash
cargo test --workspace                  # 200 lib tests (barraCuda + forge + toadStool)
cargo clippy --workspace -- -D warnings # Lint — zero warnings
cargo llvm-cov report --workspace      # 96.84% coverage

# Full validation (all experiments + Python cross-checks)
cargo build --workspace --release
# Run each exp* binary (see .github/workflows/ci.yml validate job), then:
python3 control/pkpd/cross_validate.py

# Run individual validation binaries
cargo run --bin exp050_diagnostic_pipeline
cargo run --bin exp051_population_diagnostic
cargo run --bin exp052_petaltongue_render

# GPU experiments (requires GPU)
cargo run --release --bin exp053_gpu_parity    # 17 parity checks
cargo run --release --bin exp054_gpu_pipeline  # Fused pipeline + toadStool
cargo run --release --bin exp055_gpu_scaling   # 1K→10M scaling benchmark

# petaltongue-health removed in V6.1 — renderers absorbed into petalTongue upstream
# Use petalTongue with healthspring-diagnostic.json scenario instead

# Python controls
python3 control/endocrine/exp030_testosterone_im_pk.py
python3 control/endocrine/exp036_population_trt_montecarlo.py
```

---

## Relationship to ecoPrimals

healthSpring is a public scientific validation repository in the ecoPrimals ecosystem. It consumes `barraCuda` (vendor-agnostic GPU math library) and validates health application pipelines using the same constrained evolution methodology as the other five springs.

The springs validate science. healthSpring applies it.
