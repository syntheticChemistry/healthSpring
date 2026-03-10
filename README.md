# healthSpring — Human Health Applications of Sovereign Scientific Computing

**An ecoPrimals Spring** — human health applications validating PK/PD, microbiome, biosignal, and endocrine pipelines against Python baselines via Pure Rust + barraCuda GPU. Follows the **Write → Absorb → Lean** cycle adopted from wetSpring/hotSpring.

**Date:** March 10, 2026
**License:** AGPL-3.0-or-later
**MSRV:** 1.87
**Status:** V19 — 395 tests (329 barraCuda + 33 forge + 30 toadStool + 3 doc-tests), 59 experiments, 194 Python cross-validation checks, full-stack portability validated (barraCuda CPU → GPU → toadStool dispatch → metalForge NUCLEUS routing). V19: GPU scaling bench (Exp085, 47/47), toadStool V16 streaming dispatch (Exp086, 24/24), mixed NUCLEUS V16 dispatch with PCIe P2P bypass (Exp087, 35/35). V18: CPU parity benchmarks (Exp084, Rust 84× faster than Python). V17: GPU portability (3 new WGSL shaders, Exp083 25/25). V16: 6 new domain primitives (Exp077-082) + paper queue complete (30/30). Zero unsafe code, zero clippy warnings (`#![deny(clippy::pedantic)]`), `cargo fmt` clean.

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
| Version | **V19** (full-stack portability: CPU → GPU → toadStool → NUCLEUS) |
| Rust lib tests | 329 (barraCuda) |
| Rust forge tests | 33 (metalForge) |
| Rust toadStool tests | 30 |
| Doc-tests | 3 (`shannon_index`, `hill_dose_response`, `auc_trapezoidal`) |
| **Total tests** | **395** |
| Paper queue | **30/30 complete** |
| Python control checks | 194 (cross-validation) |
| Experiments complete | 59 (Tier 0+1+2+3 + diagnostics + GPU + clinical + NLME + V16 primitives + GPU scaling + dispatch + NUCLEUS) |
| GPU validation (Tier 2) | **Live** — 6 WGSL shaders, fused pipeline, 42/42 parity checks |
| GPU V16 shaders | `michaelis_menten_batch_f64.wgsl`, `scfa_batch_f64.wgsl`, `beat_classify_batch_f64.wgsl` |
| GPU scaling | Hill crossover 100K, PK crossover 5M, peak 207 M elements/s; V16 linear scaling confirmed |
| CPU parity | Rust 84× faster than Python across V16 primitives (SCFA 160×, antibiotic 233×, beat 149×) |
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

### V16 Primitives (Exp077-082)

Six new domain experiments closing the paper queue (30/30):

- Michaelis-Menten nonlinear PK (capacity-limited elimination) — Exp077
- Antibiotic perturbation (diversity decline/recovery dynamics) — Exp078
- SCFA production (Michaelis-Menten kinetics: acetate, propionate, butyrate) — Exp079
- Gut-brain serotonin (tryptophan metabolism pathway) — Exp080
- EDA stress detection (SCL, phasic decomposition, SCR) — Exp081
- Arrhythmia beat classification (template correlation: Normal, PVC, PAC) — Exp082

### GPU V16 Parity (Exp083)

- GPU parity for V16 primitives: 3 new WGSL compute shaders (MM batch, SCFA batch, Beat classify) — Exp083

### CPU Parity Benchmarks (Exp084)

- V16 CPU parity bench: Rust 84× faster than Python across 6 primitives (33 Rust checks, 17 Python checks) — Exp084

### GPU Scaling + toadStool Dispatch + NUCLEUS Routing (Exp085-087)

- barraCuda GPU vs CPU V16 scaling bench (4 scales × 3 ops, fused pipeline, metalForge routing) — Exp085
- toadStool V16 streaming dispatch (execute_cpu + streaming callbacks, GPU-mappability) — Exp086
- metalForge mixed NUCLEUS V16 dispatch (Tower/Node/Nest topology, PCIe P2P bypass, plan_dispatch) — Exp087

---

## Validation Protocol

```
Tier 0: Python control (published algorithm, reference implementation)
Tier 1: Rust CPU (Pure Rust, f64-canonical, tolerance-documented)
Tier 2: Rust GPU (barraCuda WGSL shaders, math parity with CPU)
Tier 3: metalForge (toadStool dispatch, cross-substrate routing)
```

**Current state**: Tier 0+1 complete for 30 experiments (paper queue 30/30). **Tier 2 live**: 6 WGSL shaders (3 original + 3 V16), fused pipeline, CPU vs GPU parity matrix. **Tier 3 live**: metalForge NUCLEUS routing for all Workload variants, toadStool streaming dispatch, PCIe P2P bypass. **V19**: GPU scaling bench (linear scaling confirmed at 4 scales), toadStool V16 dispatch (streaming + callbacks), mixed NUCLEUS V16 dispatch (Tower/Node/Nest + PCIe P2P GPU↔NPU). **V18**: CPU parity — Rust 84× faster than Python across V16 primitives.

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
│       ├── diversity_f64.wgsl
│       ├── michaelis_menten_batch_f64.wgsl
│       ├── scfa_batch_f64.wgsl
│       └── beat_classify_batch_f64.wgsl
├── control/             # Python baselines (Tier 0) — 194 cross-validation checks
│   ├── pkpd/            # exp001–exp006, exp077 + cross_validate.py
│   ├── microbiome/      # exp010–exp013, exp078–exp080
│   ├── biosignal/       # exp020–exp023, exp081–exp082
│   ├── endocrine/       # exp030–exp038
│   ├── validation/      # Exp040 CPU parity
│   └── scripts/         # Benchmark scripts + timing JSON results
├── experiments/         # 59 validation binaries
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
│   ├── exp073–exp074/   # petalTongue evolution
│   ├── exp075–exp076/   # NLME + full pipeline
│   ├── exp077–exp082/   # V16 primitives (MM PK, antibiotic, SCFA, serotonin, EDA, arrhythmia)
│   ├── exp083/          # GPU V16 parity (25/25)
│   ├── exp084/          # CPU parity bench (Rust 84× faster)
│   └── exp085–exp087/   # GPU scaling + toadStool dispatch + NUCLEUS routing
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
cargo test --workspace                  # 395 tests (329 barraCuda + 33 forge + 30 toadStool + 3 doc-tests)
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

# V16 primitives
cargo run --release --bin exp077_michaelis_menten_pk      # Nonlinear PK
cargo run --release --bin exp084_v16_cpu_parity_bench     # CPU parity: Rust 84× faster

# GPU scaling + dispatch + NUCLEUS
cargo run --release --bin exp085_gpu_vs_cpu_v16_bench     # 47 checks — GPU scaling
cargo run --release --bin exp086_toadstool_v16_dispatch   # 24 checks — toadStool dispatch
cargo run --release --bin exp087_mixed_nucleus_v16        # 35 checks — NUCLEUS routing

# Python controls
python3 control/scripts/bench_v16_cpu_vs_python.py       # V16 Python timing baseline
python3 control/scripts/compare_v16_benchmarks.py        # Rust vs Python comparison
python3 control/scripts/control_exp085_gpu_scaling.py    # GPU scaling validation
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
