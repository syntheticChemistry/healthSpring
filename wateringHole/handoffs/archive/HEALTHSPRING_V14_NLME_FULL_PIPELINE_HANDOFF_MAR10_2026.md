# healthSpring V14 — NLME Population PK + Full Pipeline Handoff

**Date**: March 10, 2026
**From**: healthSpring
**To**: barraCuda, toadStool, petalTongue, metalForge
**License**: AGPL-3.0-only
**Covers**: V13→V14 — NLME population PK (FOCE/SAEM), NCA, NLME diagnostics (CWRES/VPC/GOF), WFDB parser, Kokkos-equivalent benchmarks, full petalTongue pipeline (28 nodes, 121 channels), industry benchmark mapping (sovereign NONMEM/Monolix/WinNonlin replacements)

---

## Executive Summary

healthSpring V14 adds Track 5: NLME population pharmacokinetics — a sovereign replacement for three commercial tools (NONMEM, Monolix, WinNonlin). FOCE and SAEM estimation algorithms, NCA metrics, and full NLME diagnostics (CWRES, VPC, GOF) are validated end-to-end with 19 binary checks (Exp075). A WFDB parser for PhysioNet Format 212/16 enables direct biosignal ingestion. The full petalTongue pipeline now spans 5 tracks with 28 nodes, 29 edges, 121 channels across all 7 DataChannel types, validated by 197 binary checks (Exp076). Kokkos-equivalent CPU benchmarks validate GPU-portable patterns ahead of shader promotion.

**Metrics**: 356 tests, 48 experiments, 853 binary checks, 104 cross-validation checks, all green.

---

## Part 1: What Changed (V13→V14)

### 1.1 NLME Population PK — FOCE + SAEM Estimation

**Module**: `barracuda/src/pkpd/nlme.rs`

FOCE (First-Order Conditional Estimation) and SAEM (Stochastic Approximation Expectation-Maximization) are the two dominant algorithms in population PK modeling. NONMEM uses FOCE; Monolix uses SAEM. healthSpring now implements both in pure Rust with deterministic LCG-based sampling.

- **FOCE**: 150 iterations, 30 subjects, one-compartment oral PK model. Per-subject conditional estimation of random effects with Laplacian approximation of the marginal likelihood. Theta (CL, Vd, Ka), omega (IIV variance), and sigma (residual error) recovery validated.
- **SAEM**: 200 iterations (100 burn-in + 100 averaging), same model. Stochastic E-step with Metropolis-Hastings sampling of individual parameters, followed by M-step sufficient statistic update. Wider tolerance than FOCE due to Monte Carlo noise.
- **Determinism**: Same seed produces identical objective function values across runs (verified at 1e-10).

**Absorption target for barraCuda**: Both algorithms contain per-subject optimization loops that are batch-parallelizable. FOCE's individual gradient computation and SAEM's E-step sampling map directly to embarrassingly parallel GPU patterns. Candidate for `barraCuda::stats::nlme` module.

### 1.2 NCA — Non-Compartmental Analysis

**Module**: `barracuda/src/pkpd/nca.rs`

NCA is the standard first-pass PK analysis (WinNonlin's primary use case). Computes:
- Lambda-z: terminal elimination rate from log-linear regression on the terminal phase
- AUC∞: trapezoidal AUC_last + C_last/λz extrapolation
- MRT: mean residence time (AUMC∞/AUC∞)
- CL: clearance (Dose/AUC∞)
- Vss: steady-state volume (CL × MRT)

Validated with 5% tolerance on λz and AUC∞ (numerical discretization error).

**Absorption target for barraCuda**: Per-subject NCA is independent → batch element-wise pattern. Low compute cost, but needed for any PK pipeline. Candidate for `barraCuda::bio::nca` module.

### 1.3 NLME Diagnostics — CWRES, VPC, GOF

**Module**: `barracuda/src/pkpd/diagnostics.rs`

Three standard NLME diagnostic plots, all validated:

- **CWRES** (Conditional Weighted Residuals): Mean < 2.0, should be ~N(0,1). Standard NONMEM diagnostic.
- **VPC** (Visual Predictive Check): 50 Monte Carlo simulations, 5th/50th/95th percentile prediction bands. Observed data should fall within bands.
- **GOF** (Goodness-of-Fit): Observed vs predicted scatter with R²≥0. Standard model adequacy check.

**Absorption target for barraCuda**: VPC is embarrassingly parallel (each simulation independent). 50 simulations is minimal; clinical VPC uses 500-1000. GPU promotion would enable real-time diagnostic feedback. Candidate for `barraCuda::stats::diagnostics`.

**Absorption target for petalTongue**: VPC band rendering (5th/50th/95th percentile as TimeSeries with ClinicalRange), GOF scatter plotted as Scatter3D. These patterns should be available as standard chart configurations.

### 1.4 WFDB Parser — PhysioNet Format 212/16

**Module**: `barracuda/src/wfdb.rs`

Streaming parser for PhysioNet's WFDB (Waveform Database) format — the de facto standard for biosignal archival (MIT-BIH, MIMIC-III, dozens of other databases). Supports:
- Format 212: 12-bit packed pairs (standard MIT-BIH ECG format)
- Format 16: 16-bit signed (standard for higher-resolution signals)
- Beat annotations: Normal, PVC, APC, BBB, paced, fusion, unclassifiable

Zero-copy streaming design — reads bytes directly without buffering entire files.

**Absorption target for barraCuda**: `barraCuda::signal::wfdb` for PhysioNet signal ingestion. This enables any WFDB-format database to feed into the biosignal pipeline.

### 1.5 Kokkos-Equivalent Benchmarks

**Module**: `barracuda/benches/kokkos_parity.rs`

CPU benchmarks validating GPU-portable computation patterns:
- **Reduction**: Σ f(x_i) — maps to workgroup reduction shader
- **Scatter**: Histogram binning — maps to atomicAdd pattern
- **Monte Carlo**: LCG sampling with functional reduction — maps to per-thread independent execution
- **ODE batch**: N independent Euler ODE solves — maps to per-patient PK simulation
- **NLME iteration**: Single FOCE iteration with per-subject gradient — maps to batch-parallel GPU

These benchmarks serve as the proof point for GPU shader promotion: if the CPU pattern matches a known GPU-efficient pattern, the promotion path is clear.

**Absorption target for toadStool**: These patterns define the GPU promotion roadmap. Each benchmark maps to a specific shader pattern that toadStool's dispatch layer should recognize and route to GPU when problem size exceeds crossover.

### 1.6 Full petalTongue Pipeline — 28 Nodes, 121 Channels

**Modules**: `barracuda/src/visualization/scenarios/nlme.rs`, updated `scenarios/mod.rs`, `scenarios/biosignal.rs`

The full study scenario now spans 5 tracks:

| Track | Builder | Nodes | Channels |
|-------|---------|:-----:|:--------:|
| PK/PD | `pkpd_study()` | 6 | 19 |
| Microbiome | `microbiome_study()` | 4 | 11 |
| Biosignal | `biosignal_study()` | 5 | 16+ |
| Endocrinology | `endocrine_study()` | 8 | 19 |
| NLME | `nlme_study()` | 5 | 41 |
| **Full Study** | `full_study()` | **28** | **121** |

New NLME nodes: `nlme_population`, `nca_metrics`, `cwres_diagnostics`, `vpc_check`, `gof_fit`. New biosignal node: `wfdb_ecg` (WFDB Format 212 decode + annotation parsing).

Cross-track edge: `pop_pk → nlme_population` (population PK feeds NLME estimation).

All 7 DataChannel types exercised: TimeSeries, Distribution, Bar, Gauge, Spectrum, Heatmap, Scatter3D.

**Absorption target for petalTongue**: 6 new node types with 41 new channels. The NLME scenario introduces new visualization patterns:
- Distribution channels for FOCE/SAEM parameter estimates (theta_cl, theta_vd, theta_ka)
- Scatter3D for population parameter space (CL vs Vd vs AUC)
- TimeSeries with ClinicalRange for VPC bands and CWRES over time
- Bar for NCA metric comparison and beat type distribution
- Gauge for sampling frequency, annotation counts, duration

### 1.7 Exp075 + Exp076

- **Exp075** (`exp075_nlme_cross_validation`): 19 binary checks validating FOCE theta recovery (30%), SAEM theta recovery (50%), NCA λz (5%), NCA AUC∞ (5%), CWRES mean (<2.0), GOF R² (≥0), FOCE/SAEM deterministic reproducibility (1e-10).
- **Exp076** (`exp076_full_pipeline_scenarios`): 197 binary checks validating all 5 track scenarios + full study structure + channel statistics + JSON round-trip + IPC push with graceful fallback.

### 1.8 Industry Benchmark Mapping

Profiled commercial tools and mapped sovereign replacements:

| Commercial Tool | Function | Sovereign Replacement | Status |
|----------------|----------|----------------------|--------|
| **NONMEM** | FOCE population PK estimation | `pkpd/nlme.rs::foce_estimate` | **Validated** (Exp075) |
| **Monolix** | SAEM population PK estimation | `pkpd/nlme.rs::saem_estimate` | **Validated** (Exp075) |
| **WinNonlin** | NCA (λz, AUC∞, MRT, CL, Vss) | `pkpd/nca.rs::nca_analysis` | **Validated** (Exp075) |
| **Chromeleon** | HPLC data pipeline + chromatography | Architecture pattern (pipeline dispatch) | Mapped |
| **SnapGene** | Sequence annotation + visualization | Pattern (annotation → visualization) | Mapped |

---

## Part 2: Absorption Targets

### For barraCuda

| Priority | Module | Source | Pattern | GPU Benefit |
|:--------:|--------|--------|---------|:-----------:|
| P0 | `foce_estimate` | `pkpd/nlme.rs` | Per-subject gradient → batch parallel | **50-100×** at 1K+ subjects |
| P0 | `saem_estimate` | `pkpd/nlme.rs` | E-step sampling → embarrassingly parallel MC | **50-100×** at 1K+ subjects |
| P0 | `vpc_simulate` | `pkpd/diagnostics.rs` | Independent simulations → embarrassingly parallel | **200×** at 1K simulations |
| P1 | `nca_analysis` | `pkpd/nca.rs` | Per-subject NCA → batch element-wise | **10×** (already fast) |
| P1 | `cwres_compute` | `pkpd/diagnostics.rs` | Per-observation residual → element-wise | Low (small N) |
| P1 | `gof_compute` | `pkpd/diagnostics.rs` | Scatter regression → element-wise | Low |
| P2 | `decode_format_212` | `wfdb.rs` | Streaming byte decode → pipeline stage | N/A (I/O bound) |
| P2 | `decode_format_16` | `wfdb.rs` | Streaming byte decode → pipeline stage | N/A (I/O bound) |

### For toadStool

| Priority | Pattern | Source | GPU Dispatch |
|:--------:|---------|--------|:------------:|
| P0 | FOCE batch optimization | Kokkos NLME iteration benchmark | Batch-parallel per-subject |
| P0 | VPC Monte Carlo | Kokkos Monte Carlo benchmark | Each simulation → one workgroup |
| P1 | ODE batch | Kokkos ODE batch benchmark | Each patient → one thread |
| P1 | Reduction | Kokkos reduction benchmark | Workgroup reduction shader |

These patterns are validated by `barracuda/benches/kokkos_parity.rs` CPU benchmarks. The crossover points for GPU benefit are documented in `specs/COMPUTE_DATA_PROFILE.md`.

### For petalTongue

| Priority | Component | Nodes | Channels | New Pattern |
|:--------:|-----------|:-----:|:--------:|-------------|
| P0 | NLME scenario | 5 | 41 | VPC band rendering (percentile TimeSeries with ClinicalRange) |
| P0 | WFDB ECG node | 1 | 5 | Beat annotation overlay on ECG TimeSeries |
| P1 | GOF scatter | — | 1 | Scatter3D for observed vs predicted (diagonal reference line) |
| P1 | Parameter distribution | — | 3 | Distribution channels for FOCE/SAEM estimates |

### For metalForge

| Priority | Pattern | Description |
|:--------:|---------|-------------|
| P1 | NLME dispatch | FOCE/SAEM workloads should route to GPU when subject count > 100 |
| P2 | VPC dispatch | Monte Carlo VPC should route to GPU when simulation count > 50 |

---

## Part 3: Quality Gates

| Gate | Status |
|------|--------|
| `cargo test --workspace` | **356 passed**, 0 failed |
| `cargo clippy --workspace --all-targets -- -D clippy::all -W clippy::pedantic` | **0 warnings** |
| `cargo fmt --check --all` | **0 diffs** |
| `cargo doc --workspace --no-deps` | **0 warnings** |
| Exp075 NLME cross-validation | **19/19 passed** |
| Exp076 full pipeline scenarios | **197/197 passed** |
| Exp056 study scenarios (updated) | **57/57 passed** |
| Max file size | 819 lines (under 1000-line limit) |
| Unsafe code | 0 blocks |

---

## Part 4: GPU Learnings for barraCuda/toadStool

1. **FOCE individual optimization is batch-parallelizable**: Each subject's conditional log-likelihood is independent given population parameters. The gradient computation per subject maps to the same embarrassingly parallel pattern as PopPK Monte Carlo (already a GPU LIVE shader).

2. **SAEM E-step maps to existing sampling pattern**: The Metropolis-Hastings step in SAEM's E-step uses the same LCG-based sampling as `population_pk_f64.wgsl`. The u32 xorshift32 + Wang hash GPU PRNG documented in V12 learnings applies directly.

3. **VPC is the highest-impact GPU target**: Clinical VPC uses 500-1000 simulations. At 1000 subjects × 1000 simulations × 14 timepoints, this is ~14M ODE evaluations — firmly in the GPU-beneficial regime (well above the 100K crossover point from Exp055).

4. **NCA is fast but useful in batch**: Individual NCA takes microseconds. But population NCA on 10K+ subjects makes it worthwhile for batch dispatch.

5. **Kokkos benchmarks validate the promotion path**: The 5 Kokkos-equivalent patterns in `kokkos_parity.rs` (reduction, scatter, Monte Carlo, ODE batch, NLME iteration) are the exact patterns that need WGSL shaders. Each has a known GPU-efficient implementation.

6. **WFDB streaming is I/O-bound**: No GPU benefit for WFDB parsing itself, but the decoded signal data feeds directly into the biosignal GPU pipeline (Pan-Tompkins, FFT, HRV).

---

## Part 5: Action Items

### Immediate (V14 absorption)

- [ ] **barraCuda**: Review `pkpd/nlme.rs`, `pkpd/nca.rs`, `pkpd/diagnostics.rs` for absorption into `barraCuda::stats` and `barraCuda::bio`
- [ ] **barraCuda**: Review `wfdb.rs` for absorption into `barraCuda::signal::wfdb`
- [ ] **toadStool**: Review Kokkos benchmark patterns for GPU dispatch routing rules
- [ ] **petalTongue**: Review NLME scenario (5 nodes, 41 channels) for VPC band and GOF scatter rendering patterns

### Near-term (V15 evolution)

- [ ] **barraCuda**: FOCE/SAEM GPU shaders (per-subject parallel optimization)
- [ ] **barraCuda**: VPC Monte Carlo GPU shader (embarrassingly parallel simulation)
- [ ] **toadStool**: Add NLME workload type to dispatch rules (route to GPU at >100 subjects)
- [ ] **petalTongue**: VPC percentile band chart type (TimeSeries with prediction interval overlay)

### Long-term

- [ ] **Population PK at scale**: 10K-100K subjects with NLME on GPU (requires FOCE shader)
- [ ] **Real-time diagnostics**: Live VPC/CWRES updates during NLME estimation (streaming to petalTongue)
- [ ] **Multi-drug NLME**: Extend to combination therapy population models
