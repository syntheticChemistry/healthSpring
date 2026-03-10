# healthSpring — Human Health Applications of Sovereign Scientific Computing

**An ecoPrimals Spring** — human health applications validating PK/PD, microbiome, biosignal, and endocrine pipelines against Python baselines via Pure Rust + barraCuda GPU. Follows the **Write → Absorb → Lean** cycle adopted from wetSpring/hotSpring.

**Date:** March 10, 2026
**License:** AGPL-3.0-or-later
**MSRV:** 1.87
**Status:** V14.1 — 356 tests (289 barraCuda + 33 forge + 30 toadStool + 4 doc-tests), 48 experiments, 853 Rust binary checks, 104 cross-validation checks. NLME population PK (FOCE + SAEM — sovereign NONMEM/Monolix replacement). NCA (sovereign WinNonlin replacement). NLME diagnostics (CWRES, VPC, GOF). WFDB parser (PhysioNet Format 212/16). Kokkos-equivalent GPU benchmarks. Full petalTongue pipeline: 28 nodes, 29 edges, 121 channels, all 7 DataChannel types. Industry benchmark mapping (SnapGene, Chromeleon, NONMEM, Monolix, WinNonlin profiled). Zero unsafe code, zero clippy warnings (`#![deny(clippy::pedantic)]`), `cargo fmt` clean, `cargo doc` clean.

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
| Version | **V14.1** (NLME + full pipeline + deep debt) |
| Rust lib tests | 289 (barraCuda) |
| Rust forge tests | 33 (metalForge) |
| Rust toadStool tests | 30 |
| Doc-tests | 4 (`shannon_index`, `hill_dose_response`, `auc_trapezoidal`, `state_to_f64`) |
| **Total tests** | **356** |
| Rust binary checks | 853 |
| Python control checks | 104 (cross-validation) |
| Experiments complete | 48 (Tier 0+1+2+3 + diagnostic + visualization + clinical + streaming + interaction + NLME) |
| GPU validation (Tier 2) | **Live** — 3 WGSL shaders, fused pipeline, 17/17 parity checks |
| GPU scaling | Hill crossover 100K, PK crossover 5M, peak 207 M elements/s |
| petalTongue visualization | **Full** — 7 DataChannel types, 3 stream ops, domain theming, capabilities query, interaction subscription |
| petalTongue scenarios | 14 scenarios (6 clinical + 5 TRT archetypes + topology + dispatch + NLME) |
| petalTongue pipeline | 28 nodes, 29 edges, 121 channels across all 7 DataChannel types |
| Clinical TRT | 5 patient archetypes, live streaming dashboard (PK, HRV, HbA1c, cardiac risk) |
| NLME population PK | FOCE + SAEM estimation, NCA metrics, CWRES/VPC/GOF diagnostics |
| metalForge validation | 33 tests (NUCLEUS topology, dispatch planning, PCIe transfer) |
| toadStool validation | 30 tests + GPU dispatch + streaming + auto-dispatch |
| Faculty | Gonzales (MSU Pharm/Tox), Lisabeth (ADDRC), Neubig (Drug Discovery), Mok (Allure Medical) |
| Unsafe blocks | 0 |
| Clippy warnings | 0 (`#![deny(clippy::pedantic)]` in all lib crates, `-W clippy::nursery`) |
| Max file size | 819 lines (all files under 1000-line wateringHole limit) |

---

## V14 NLME + Full Pipeline Evolution (from V13)

V14 adds NLME population pharmacokinetics, NCA, WFDB parsing, diagnostics, Kokkos-equivalent benchmarks, full petalTongue pipeline visibility, and industry benchmark mapping.

| Change | Impact |
|--------|--------|
| **NLME population PK** | FOCE + SAEM estimation in `barracuda/src/pkpd/nlme.rs` — sovereign replacement for NONMEM/Monolix. 30 subjects, 150 FOCE iterations, 200 SAEM iterations. Theta/omega/sigma recovery validated. |
| **NCA** | Non-compartmental analysis in `barracuda/src/pkpd/nca.rs` — sovereign WinNonlin replacement. Lambda-z, AUC_inf, MRT, CL, Vss. |
| **NLME diagnostics** | CWRES, VPC (50 simulations), GOF in `barracuda/src/pkpd/diagnostics.rs`. CWRES mean <2.0, GOF R²≥0. |
| **WFDB parser** | PhysioNet Format 212/16 streaming parser in `barracuda/src/wfdb.rs`. Beat annotation parsing. |
| **Kokkos-equivalent benchmarks** | Reduction, scatter, Monte Carlo, ODE batch, NLME iteration in `barracuda/benches/kokkos_parity.rs`. GPU readiness evidence. |
| **Full petalTongue pipeline** | 28 nodes, 29 edges, 121 channels across all 7 DataChannel types. NLME scenario builder (5 nodes: population, NCA, CWRES, VPC, GOF). WFDB ECG node. |
| **Exp075** | NLME cross-validation: FOCE/SAEM parameter recovery, NCA metrics, CWRES, GOF. 19 binary checks. |
| **Exp076** | Full pipeline petalTongue scenario validation. 197 binary checks across all 5 tracks + full study. |
| **Industry benchmarks** | SnapGene, Chromeleon, NONMEM, Monolix, WinNonlin profiled. Sovereign replacements mapped to ecoPrimals stack. |

---

## V13 Deep Audit Evolution (from V12)

V13 is a code quality and correctness evolution — no new experiments, but significant structural improvements:

| Change | Impact |
|--------|--------|
| **Anderson eigensolver** | Fixed IPR bug: Hamiltonian diagonal was used instead of actual eigenvectors. Implemented tridiagonal QL algorithm in `microbiome.rs` for correct eigenvalue/eigenvector computation. Fixes `diagnostic.rs` and `scenarios/microbiome.rs`. |
| **Smart clinical.rs refactor** | 1177 → 374 lines (clinical.rs) + 819 lines (clinical_nodes.rs). Eight node-building functions extracted by domain responsibility, not arbitrary split. Both files under 1000-line limit. |
| **LCG PRNG centralization** | New `rng.rs` module (37 lines): `LCG_MULTIPLIER`, `lcg_step()`, `state_to_f64()`. Replaced hardcoded `6_364_136_223_846_793_005` in 4 files. |
| **Math deduplication** | `endocrine::evenness_to_disorder` → delegates to `microbiome::evenness_to_disorder`. `endocrine::lognormal_params` → delegates to `pkpd::LognormalParam::to_normal_params`. |
| **Capability-based discovery** | Replaced hardcoded `/tmp/songbird.sock` in `capabilities.rs` with glob-based `songbird*.sock` discovery. |
| **Flaky IPC test fix** | `AtomicU64` unique socket paths + refactored test harness eliminates `Barrier` race conditions. |
| **Doc-tests** | 4 added: `shannon_index`, `hill_dose_response`, `auc_trapezoidal`, `state_to_f64`. |
| **Tolerance registry** | Added `exp067` and `exp069` CPU parity class entries. |

---

## Domains

### Track 1: Pharmacokinetic / Pharmacodynamic Modeling (Exp001-006)

Pure Rust PK/PD tools replacing Python/NONMEM dependency chains. Extends neuralSpring nS-601–605 (veterinary) to human therapeutics.

- Hill dose-response (4 human JAK inhibitors + canine reference) — Exp001
- One-compartment PK (IV bolus + oral Bateman + multiple dosing + AUC) — Exp002
- Two-compartment PK (biexponential α/β phases, peripheral compartment) — Exp003
- mAb PK cross-species transfer (lokivetmab → nemolizumab/dupilumab) — Exp004
- Population PK Monte Carlo (1,000 virtual patients, lognormal IIV) — Exp005
- PBPK multi-compartment (5-tissue: liver, kidney, muscle, fat, rest) — Exp006

### Track 2: Gut Microbiome and Colonization Resistance (Exp010-013)

Extends wetSpring's Anderson localization framework from soil to gut.

- Shannon/Simpson/Pielou diversity indices + Chao1 richness — Exp010
- Anderson localization in gut lattice (1D localization length ξ) — Exp011
- C. difficile colonization resistance score — Exp012
- FMT RCDI (fecal microbiota transplant, recurrent C. difficile) — Exp013

### Track 3: Biosignal Processing (Exp020-023)

Real-time physiological signal analysis on sovereign hardware.

- Pan-Tompkins QRS detection (ECG R-peak, 5-stage intermediates) — Exp020
- HRV metrics (RMSSD, pNN50, LF/HF, power spectrum) — Exp021
- PPG SpO₂ (pulse oximetry, reflectance) — Exp022
- Biosignal fusion (ECG + PPG + EDA multi-modal) — Exp023

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

### Track 5: NLME Population Pharmacokinetics (Exp075-076)

Sovereign replacement for NONMEM (FOCE), Monolix (SAEM), and WinNonlin (NCA). Full population PK modeling with diagnostics.

- NLME cross-validation: FOCE + SAEM parameter recovery, NCA metrics, CWRES, GOF — Exp075
- Full pipeline petalTongue scenario validation (all 5 tracks, 28 nodes, 121 channels) — Exp076

### Integrated Diagnostics (Exp050-052)

- Integrated patient diagnostic pipeline (4 tracks + cross-track + composite risk) — Exp050
- Population diagnostic Monte Carlo (1,000 virtual patients) — Exp051
- petalTongue scenario schema validation (DataChannel, ClinicalRange) — Exp052

### GPU Pipeline (Exp053-055)

- GPU parity: WGSL shader output vs CPU baseline (Hill, PopPK, Diversity) — Exp053
- Fused pipeline: all ops in one GPU submission, toadStool dispatch — Exp054
- GPU scaling: 1K→10M sweep, crossover analysis, field deployment thesis — Exp055

### Visualization (Exp056)

- Full petalTongue 5-track scenario generation (57 checks, 7 channel types, 14 scenarios) — Exp056

### Validation Track (Exp040)

- barraCuda CPU parity (Tier 0+1 baseline for GPU migration) — Exp040

### CPU vs GPU Parity & Mixed Dispatch (Exp060-062)

- CPU vs GPU pipeline comparison (full matrix, 27 parity checks) — Exp060
- Mixed hardware dispatch via NUCLEUS topology (22 dispatch route checks) — Exp061
- PCIe P2P transfer validation (DMA planning, 26 transfer checks) — Exp062

### Clinical TRT Scenarios & petalTongue Integration (Exp063-065)

- Patient-parameterized clinical TRT scenarios (5 archetypes, 8 nodes/8 edges each) — Exp063
- IPC push to petalTongue (Unix socket discovery, JSON-RPC render push, fallback to file) — Exp064
- Live streaming dashboard (ECG, HRV, PK via StreamSession with backpressure) — Exp065

### Compute & Benchmark (Exp066-072)

- barraCuda CPU benchmark (Hill, PopPK, Diversity timing) — Exp066
- GPU parity extended (additional kernel validation) — Exp067
- GPU benchmark (throughput at scale) — Exp068
- toadStool dispatch matrix (stage assignment validation) — Exp069
- PCIe P2P bypass (NPU→GPU direct transfer) — Exp070
- Mixed system pipeline (CPU+GPU+NPU coordinated execution) — Exp071
- Compute dashboard (toadStool streaming → petalTongue live gauges) — Exp072

### petalTongue Evolution (Exp073-074)

- Clinical TRT live dashboard (PK trough streaming, HRV improvement, cardiac risk replace) — Exp073
- Interaction roundtrip (mock petalTongue: render, append, replace, gauge, capabilities, subscribe — 12/12) — Exp074

### NLME + Full Pipeline (Exp075-076)

- NLME cross-validation: FOCE/SAEM parameter recovery, NCA (λz, AUC∞), CWRES, GOF (19 checks) — Exp075
- Full pipeline petalTongue scenario validation: 5 tracks, 28 nodes, 29 edges, 121 channels, 197 checks — Exp076

---

## Validation Protocol

```
Tier 0: Python control (published algorithm, reference implementation)
Tier 1: Rust CPU (Pure Rust, f64-canonical, tolerance-documented)
Tier 2: Rust GPU (barraCuda WGSL shaders, math parity with CPU)
Tier 3: metalForge (toadStool dispatch, cross-substrate routing)
```

**Current state**: Tier 0+1 complete for 24 experiments. **Tier 2 live**: 3 WGSL shaders compiled and validated (Exp053), fused unidirectional pipeline (Exp054), scaling to 10M elements (Exp055). CPU vs GPU parity matrix (Exp060, 27/27). Mixed hardware dispatch (Exp061, 22/22). PCIe P2P transfer (Exp062, 26/26). toadStool `Pipeline::execute_gpu()` and `execute_streaming()` dispatch stages via `GpuContext`. metalForge substrate routing (Tier 3 foundation). Patient-parameterized clinical TRT scenarios (Exp063/073) with petalTongue IPC push (Exp064) and live streaming (Exp065/073). petalTongue interaction roundtrip validated (Exp074).

---

## Directory Structure

```
healthSpring/
├── barracuda/           # Rust library — PK/PD, microbiome, biosignal, endocrine
│   └── src/
│       ├── lib.rs       # 289 tests, #![forbid(unsafe_code)]
│       ├── pkpd/        # Track 1: Hill, 1/2-compartment, allometric, pop PK, PBPK, NLME (FOCE/SAEM), NCA, diagnostics
│       ├── microbiome.rs # Track 2: Shannon, Simpson, Pielou, Chao1, Anderson W, FMT, eigensolver
│       ├── biosignal/    # Track 3 (submodules after V14.1 refactor)
│       │   ├── mod.rs    # Re-exports all public items for API compatibility
│       │   ├── ecg.rs    # Pan-Tompkins QRS detection, synthetic ECG
│       │   ├── hrv.rs    # SDNN, RMSSD, pNN50, heart rate from peaks
│       │   ├── ppg.rs    # SpO2 R-value calibration, synthetic PPG
│       │   ├── eda.rs    # SCL, phasic decomposition, SCR detection
│       │   ├── fusion.rs # Multi-channel FusedHealthAssessment
│       │   └── fft.rs    # DFT/IDFT utilities (centralized)
│       ├── endocrine.rs  # Track 4: testosterone PK, decline, TRT outcomes, gut axis
│       ├── wfdb.rs      # WFDB parser (PhysioNet Format 212/16, annotations)
│       ├── rng.rs       # Deterministic LCG PRNG (centralized)
│       ├── gpu/         # Tier 2: GPU dispatch + GpuContext + fused pipeline
│       │   ├── mod.rs
│       │   ├── dispatch.rs
│       │   └── context.rs
│       └── visualization/ # petalTongue integration
│           ├── ipc_push.rs      # JSON-RPC client (render, append, replace, gauge, caps, interact)
│           ├── stream.rs        # StreamSession with backpressure
│           ├── clinical.rs      # Patient-parameterized TRT scenario builder (374 lines)
│           ├── clinical_nodes.rs # TRT node builders (819 lines)
│           ├── scenarios/       # Per-track + topology + dispatch scenario builders
│           └── capabilities.rs  # Songbird capability announcement (glob-based discovery)
│   └── shaders/health/  # WGSL compute kernels (f64)
│       ├── hill_dose_response_f64.wgsl
│       ├── population_pk_f64.wgsl
│       └── diversity_f64.wgsl
├── control/             # Python baselines (Tier 0) — 104 cross-validation checks
│   ├── pkpd/            # exp001–exp006 + cross_validate.py
│   ├── microbiome/      # exp010–exp013
│   ├── biosignal/       # exp020–exp023
│   ├── endocrine/       # exp030–exp038
│   └── validation/      # Exp040 CPU parity
├── experiments/         # 48 validation binaries (853 binary checks)
│   ├── exp001–exp006/   # Track 1: PK/PD
│   ├── exp010–exp013/   # Track 2: Microbiome
│   ├── exp020–exp023/   # Track 3: Biosignal
│   ├── exp030–exp038/   # Track 4: Endocrinology
│   ├── exp040/          # barraCuda CPU parity
│   ├── exp050–exp052/   # Integrated diagnostics
│   ├── exp053–exp056/   # GPU pipeline + visualization
│   ├── exp060–exp062/   # CPU vs GPU + mixed dispatch + PCIe
│   ├── exp063–exp065/   # Clinical TRT + IPC + live streaming
│   ├── exp066–exp072/   # Compute benchmarks + dashboard
│   ├── exp073–exp074/   # petalTongue evolution (TRT dashboard, interaction roundtrip)
│   ├── exp075/          # Track 5: NLME cross-validation (FOCE/SAEM, NCA, diagnostics)
│   └── exp076/          # Full pipeline petalTongue scenario validation (197 checks)
├── metalForge/          # Cross-substrate dispatch (Tier 3)
│   └── forge/
│       └── src/
│           ├── nucleus.rs    # NUCLEUS atomics (Tower, Node, Nest)
│           ├── dispatch.rs   # DispatchPlan, StageAssignment
│           └── transfer.rs   # PCIe P2P transfer planning
├── toadstool/           # Compute dispatch pipeline
│   └── src/
│       ├── pipeline.rs  # execute(), execute_gpu(), execute_streaming(), execute_auto()
│       └── stage.rs     # StageOp, BiosignalFusion, AucTrapezoidal, BrayCurtis
├── specs/               # Paper queue, evolution map, compute profile, integration plan
├── whitePaper/          # Scientific documentation
│   ├── baseCamp/        # Faculty-linked sub-theses
│   └── experiments/     # Experiment plan and status
├── wateringHole/        # Cross-spring handoffs
│   └── handoffs/        # → barraCuda, toadStool, petalTongue
├── scripts/             # Dashboard, visualization, sync scripts
├── Cargo.toml           # Workspace (59 members)
└── README.md            # This file
```

---

## Build

```bash
cargo test --workspace                  # 356 tests (barraCuda + forge + toadStool + doc-tests)
cargo clippy --workspace --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery  # Zero warnings (pedantic denied at crate level)
cargo fmt --check --all                 # Zero diffs
cargo doc --workspace --no-deps         # Zero warnings

# Full validation (all experiments + Python cross-checks)
cargo build --workspace --release
# Run each exp* binary, then:
python3 control/pkpd/cross_validate.py

# Run individual validation binaries
cargo run --bin exp050_diagnostic_pipeline
cargo run --bin exp051_population_diagnostic
cargo run --bin exp052_petaltongue_render

# GPU experiments (requires GPU)
cargo run --release --bin exp053_gpu_parity    # 17 parity checks
cargo run --release --bin exp054_gpu_pipeline  # Fused pipeline + toadStool
cargo run --release --bin exp055_gpu_scaling   # 1K→10M scaling benchmark

# CPU vs GPU and mixed dispatch
cargo run --release --bin exp060_cpu_vs_gpu_pipeline    # 27 parity checks
cargo run --release --bin exp061_mixed_hardware_dispatch # 22 NUCLEUS dispatch checks
cargo run --release --bin exp062_pcie_transfer_validation # 26 PCIe P2P checks

# Full petalTongue visualization — per-track scenario JSON generation
cargo run --bin exp056_study_scenarios  # 57 checks across 5 tracks
cargo run --release --bin dump_scenarios # Write 14 scenario JSON files to sandbox/scenarios/

# NLME + Full Pipeline
cargo run --bin exp075_nlme_cross_validation     # 19 checks (FOCE/SAEM/NCA/CWRES/GOF)
cargo run --bin exp076_full_pipeline_scenarios    # 197 checks (all 5 tracks + full study)

# Clinical TRT patient scenarios + live dashboard
cargo run --bin exp063_clinical_trt_scenarios      # 5 patient archetypes
cargo run --release --bin exp073_clinical_trt_dashboard  # Live streaming TRT dashboard
cargo run --release --bin exp074_interaction_roundtrip   # Interaction roundtrip validation

# Compute dashboard
./scripts/compute_dashboard.sh  # Full compute validation suite

# Load in petalTongue (topology + data channel charts)
petaltongue ui --scenario sandbox/scenarios/healthspring-full-study.json
petaltongue ui --scenario sandbox/scenarios/healthspring-trt-obese.json  # Clinical TRT mode

# Python controls
python3 control/endocrine/exp030_testosterone_im_pk.py
python3 control/endocrine/exp036_population_trt_montecarlo.py
```

---

## Relationship to ecoPrimals

healthSpring is a public scientific validation repository in the ecoPrimals ecosystem. It consumes `barraCuda` (vendor-agnostic GPU math library) and validates health application pipelines using the same constrained evolution methodology as the other five springs.

The springs validate science. healthSpring applies it.

---

## V14.1 Deep Debt Evolution (from V14)

V14.1 is a code quality evolution — zero-warning `#![deny(clippy::pedantic)]` enforcement, smart modular refactoring, and DFT deduplication.

| Change | Impact |
|--------|--------|
| **biosignal.rs → biosignal/ submodules** | 953-line monolith split into 6 domain-coherent modules (ecg, hrv, ppg, eda, fusion, fft) with `mod.rs` re-exporting all public items for API compatibility. |
| **clippy::pedantic promoted to deny** | All three lib crates (`barracuda`, `toadstool`, `metalForge/forge`) now use `#![deny(clippy::pedantic)]` instead of `#![warn(...)]`. All warnings resolved — `mul_add`, `must_use`, `const fn`, `while_float`, `branches_sharing_code`, `option_if_let_else`, `significant_drop_tightening`. |
| **DFT deduplication** | `visualization/scenarios/biosignal.rs` HRV power spectrum now delegates to `biosignal::fft::rfft` instead of local DFT reimplementation. |
| **Dead code removal** | Removed unused `cpu_stages` vector in toadStool pipeline. |
| **Idiomatic Rust** | `if let Some(prev) = prev_nest { if prev == id { ... } }` chains replaced with `prev_nest.filter().map()`. Shared code hoisted from if/else branches. |
| **exp023 provenance fix** | Corrected `exp023_biosignal_fusion.py` → `exp023_fusion.py` in baseline JSON and provenance script. |
