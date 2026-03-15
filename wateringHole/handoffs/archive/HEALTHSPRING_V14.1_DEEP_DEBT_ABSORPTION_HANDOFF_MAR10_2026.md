# healthSpring V14.1 — Deep Debt Evolution + Absorption Handoff

**Date**: March 10, 2026
**From**: healthSpring V14.1
**To**: barraCuda, toadStool, petalTongue, metalForge
**License**: AGPL-3.0-or-later
**Covers**: V14→V14.1 — Deep debt resolution: biosignal modular refactor, `#![deny(clippy::pedantic)]` enforcement across all lib crates, DFT deduplication, idiomatic Rust evolution, provenance fixes. Complete absorption guidance for the barraCuda/toadStool team.

---

## Executive Summary

healthSpring V14.1 is a code quality and structural evolution. No new experiments — instead, the entire codebase was audited for debt, hardcoding, non-idiomatic patterns, and lint compliance. The biosignal module (953 lines) was smart-refactored into 6 domain-coherent submodules. `clippy::pedantic` was promoted from warn to deny in all three lib crates. All DFT implementations were centralized. Dead code was removed. Provenance paths were corrected.

**Metrics**: 356 tests, 48 experiments, 853 binary checks, 104 cross-validation checks. Zero clippy warnings under `#![deny(clippy::pedantic)]`. Zero formatting diffs. Zero unsafe code. All files under 1000-line limit.

This handoff also consolidates the full absorption roadmap from V1–V14.1 for the barraCuda/toadStool team, documenting what healthSpring has validated, what patterns are ready for absorption, and what we learned along the way.

---

## Part 1: What Changed (V14→V14.1)

### 1.1 biosignal.rs → biosignal/ Modular Refactor

The monolithic `biosignal.rs` (953 lines) was split into 6 domain-coherent submodules:

| Module | Responsibility | Key Exports |
|--------|---------------|-------------|
| `biosignal/ecg.rs` | Pan-Tompkins QRS detection pipeline | `pan_tompkins`, `PanTompkinsResult`, `DetectionMetrics` |
| `biosignal/hrv.rs` | Heart rate variability metrics | `sdnn_ms`, `rmssd_ms`, `pnn50`, `heart_rate_from_peaks` |
| `biosignal/ppg.rs` | Photoplethysmography SpO2 | `ppg_r_value`, `spo2_from_r`, `SyntheticPpg` |
| `biosignal/eda.rs` | Electrodermal activity | `eda_scl`, `eda_phasic`, `eda_detect_scr` |
| `biosignal/fusion.rs` | Multi-channel fusion | `FusedHealthAssessment`, `fuse_channels` |
| `biosignal/fft.rs` | DFT/IDFT utilities | `rfft`, `irfft` |

`biosignal/mod.rs` re-exports all public items at the module level, maintaining full API compatibility (`healthspring_barracuda::biosignal::*` unchanged).

**Why this matters for barraCuda**: Each submodule maps cleanly to a GPU shader domain. ECG/PPG/EDA are streaming signal pipelines (NPU candidates). HRV is batch-parallel (GPU). FFT should be replaced by upstream `barraCuda::signal::fft` when available. Fusion is a reduction pattern.

### 1.2 `#![deny(clippy::pedantic)]` Promotion

All three lib crates (`barracuda/src/lib.rs`, `toadstool/src/lib.rs`, `metalForge/forge/src/lib.rs`) promoted from `#![warn(clippy::pedantic)]` to `#![deny(clippy::pedantic)]`. All warnings resolved:

| Lint | Resolution |
|------|-----------|
| `clippy::must_use_candidate` | Added `#[must_use]` to pure functions (`rfft`, `irfft`, etc.) |
| `clippy::suboptimal_flops` | Applied `mul_add()` for fused multiply-add operations |
| `clippy::redundant_clone` | Removed unnecessary `.clone()` calls |
| `clippy::branches_sharing_code` | Hoisted shared code before/after if-else blocks |
| `clippy::option_if_let_else` | Replaced `if let Some(x) = opt { ... } else { ... }` with `map_or_else` |
| `clippy::significant_drop_tightening` | Extracted fields before dropping mutex guards |
| `clippy::while_float` | Converted float loop bounds to integer loops |
| `clippy::too_long_first_doc_paragraph` | Split long doc paragraphs |

### 1.3 DFT Deduplication

`visualization/scenarios/biosignal.rs` contained a local DFT implementation for HRV power spectrum calculation. This was replaced with a call to `biosignal::fft::rfft`, centralizing all DFT operations. The O(n²) naive DFT is documented as a known limitation — upstream `barraCuda::signal::fft` (radix-2) should replace it.

### 1.4 Idiomatic Rust Patterns

| Pattern | Before | After |
|---------|--------|-------|
| Option chain | `if let Some(prev) = prev_nest { if prev == id { None } else { Some(plan_transfer(...)) } } else { None }` | `prev_nest.filter(\|&prev\| prev != id).map(\|prev\| plan_transfer(...))` |
| Dead code | `let cpu_stages: Vec<_> = ...` (unused) | Removed |
| Unfulfilled lint | `#[expect(clippy::tuple_array_conversions)]` on wrong function | Moved to correct function |

### 1.5 Provenance Fixes

| File | Fix |
|------|-----|
| `control/biosignal/exp023_baseline.json` | Script path `exp023_biosignal_fusion.py` → `exp023_fusion.py` |
| `control/update_provenance.py` | Same path correction |
| `control/validation/exp040_baseline.json` | Script path `exp040_barracuda_cpu_parity.py` → `exp040_barracuda_cpu.py` |

---

## Part 2: Full Absorption Roadmap (V1–V14.1 Consolidated)

This section consolidates all absorption targets across healthSpring's history for the barraCuda/toadStool team.

### 2.1 For barraCuda — Primitives Ready for Absorption

#### P0: High-Impact GPU Targets

| Module | Source | Pattern | GPU Benefit | Status |
|--------|--------|---------|:-----------:|--------|
| FOCE estimation | `pkpd/nlme.rs` | Per-subject gradient → batch parallel | 50-100× at 1K+ subjects | Validated (Exp075) |
| SAEM estimation | `pkpd/nlme.rs` | E-step sampling → embarrassingly parallel MC | 50-100× at 1K+ subjects | Validated (Exp075) |
| VPC simulation | `pkpd/diagnostics.rs` | Independent simulations → embarrassingly parallel | 200× at 1K simulations | Validated (Exp075) |
| Population PK MC | `pkpd/mod.rs` | Per-patient independent ODE → embarrassingly parallel | Live (Exp053) | GPU shader validated |
| Hill dose-response | `pkpd/hill.rs` | Element-wise, f32 exp/log workaround | Live (Exp053) | GPU shader validated |
| Diversity indices | `microbiome.rs` | Workgroup reduction | Live (Exp053) | GPU shader validated |

#### P1: Batch-Parallel Targets

| Module | Source | Pattern | Notes |
|--------|--------|---------|-------|
| NCA analysis | `pkpd/nca.rs` | Per-subject NCA → batch element-wise | Low per-unit cost, useful in batch |
| CWRES residuals | `pkpd/diagnostics.rs` | Per-observation → element-wise | Small N |
| Pan-Tompkins | `biosignal/ecg.rs` | Streaming pipeline stages | NPU candidate (Akida) |
| HRV computation | `biosignal/hrv.rs` | Batch-parallel across patients | GPU useful at population scale |
| Anderson eigensolver | `microbiome.rs` | QL tridiagonal → batch lattice | GPU Lanczos (from hotSpring lineage) |

#### P2: Signal / I/O Modules

| Module | Source | Pattern | Notes |
|--------|--------|---------|-------|
| WFDB parser | `wfdb.rs` | Streaming byte decode | I/O bound, no GPU benefit |
| FFT | `biosignal/fft.rs` | O(n²) naive DFT | Replace with radix-2 from upstream |
| SpO2 R-value | `biosignal/ppg.rs` | Element-wise | Trivial |
| EDA decomposition | `biosignal/eda.rs` | Streaming + threshold | NPU candidate |

### 2.2 For toadStool — Dispatch Patterns

healthSpring validated these Kokkos-equivalent patterns (`benches/kokkos_parity.rs`):

| Pattern | Benchmark | GPU Crossover | toadStool Routing |
|---------|-----------|:-------------:|-------------------|
| Reduction | `kokkos_reduction` | Workgroup shader | Route at N > 10K |
| Scatter (histogram) | `kokkos_scatter` | atomicAdd | Route at N > 50K |
| Monte Carlo | `kokkos_monte_carlo` | Per-thread independent | Route at N > 100K |
| ODE batch | `kokkos_ode_batch` | Per-patient thread | Route at N > 5K patients |
| NLME iteration | `kokkos_nlme_iteration` | Per-subject parallel | Route at N > 100 subjects |

toadStool should recognize these workload signatures in `StageOp` and route to GPU when problem size exceeds crossover.

### 2.3 For petalTongue — Visualization Patterns

| Pattern | Source | Channels | Notes |
|---------|--------|:--------:|-------|
| VPC band rendering | NLME scenario | TimeSeries with ClinicalRange | Percentile prediction bands (5th/50th/95th) |
| GOF scatter | NLME scenario | Scatter3D | Observed vs predicted, diagonal reference |
| Beat annotation overlay | WFDB ECG node | TimeSeries + annotations | Normal/PVC/APC/BBB markers on ECG trace |
| Parameter distribution | NLME scenario | Distribution | FOCE/SAEM theta estimates |
| Clinical mode preset | Clinical TRT | Motor commands | Bundle: hide sidebars, skip awakening, fit view |

### 2.4 For metalForge — Dispatch Rules

| Workload | Threshold | Target |
|----------|-----------|--------|
| NLME FOCE/SAEM | > 100 subjects | GPU |
| VPC Monte Carlo | > 50 simulations | GPU |
| Population PK | > 5M elements | GPU |
| Pan-Tompkins streaming | Always | NPU (Akida) if available, CPU fallback |

---

## Part 3: Quality Gates

| Gate | Status |
|------|--------|
| `cargo test --workspace` | **356 passed**, 0 failed |
| `cargo clippy --workspace --all-targets -- -W clippy::pedantic -W clippy::nursery` | **0 warnings** |
| `cargo fmt --check --all` | **0 diffs** |
| `cargo doc --workspace --no-deps` | **0 warnings** |
| `#![deny(clippy::pedantic)]` in all lib crates | **Enforced** |
| `#![forbid(unsafe_code)]` in all lib crates | **Enforced** |
| Max file size | 819 lines (under 1000-line limit) |
| All experiments | 48/48 green |

---

## Part 4: Learnings for barraCuda/toadStool Evolution

### 4.1 Code Quality Patterns That Scale

1. **`#![deny(clippy::pedantic)]` from day one**: Retroactive promotion required fixing 62+ warnings. Easier to enforce from the start. The lints that generated the most fixes: `must_use_candidate`, `suboptimal_flops`, `branches_sharing_code`.

2. **Smart modular refactoring > arbitrary splitting**: biosignal was split by domain (ECG, PPG, EDA, HRV, fusion, FFT) not by line count. Each module maps to a distinct GPU/NPU shader domain. The `mod.rs` re-export pattern preserves API compatibility.

3. **Centralize math utilities early**: DFT was implemented in two places. `mul_add()` was missed in several. A `barraCuda::math::prelude` with common operations would prevent duplication across springs.

4. **Provenance automation catches drift**: The `update_provenance.py` script caught path mismatches that manual inspection missed. Every baseline JSON should have automated provenance checks in CI.

### 4.2 GPU Shader Promotion Observations

1. **f64 `pow()` workaround is stable**: The `exp(n * log(c))` via f32 cast pattern for `pow(f64, f64)` in Hill dose-response has held across V6–V14.1 with no numerical drift. This should be a documented barraCuda pattern.

2. **u32 PRNG is sufficient for Monte Carlo**: The u32 xorshift32 + Wang hash GPU PRNG produces adequate statistical properties for PK Monte Carlo at 10M elements. A `barraCuda::gpu::prng` module would benefit all springs.

3. **Fused pipeline overhead is dominant at small N**: The 31.7× overhead reduction from fused vs individual dispatch means toadStool should default to fused for any multi-stage pipeline under 100K elements.

4. **WGSL `enable f64` directive must be stripped**: This was learned in V6 and remains true. barraCuda's shader compiler should handle this automatically.

### 4.3 Cross-Spring Shader Provenance

| Shader | healthSpring Source | GPU Pattern | Shared With |
|--------|-------------------|-------------|-------------|
| `hill_dose_response_f64.wgsl` | Exp001 (Hill equation) | Element-wise | neuralSpring (nS-601) |
| `population_pk_f64.wgsl` | Exp005 (PopPK MC) | Embarrassingly parallel | neuralSpring |
| `diversity_f64.wgsl` | Exp010 (Shannon/Simpson) | Workgroup reduction | wetSpring (16S diversity) |

These shaders should be promoted to barraCuda core and shared via the WGSL shader registry.

---

## Part 5: Action Items

### Immediate (V14.1 absorption)

- [ ] **barraCuda**: Absorb `biosignal/fft.rs` → replace O(n²) DFT with radix-2 `barraCuda::signal::fft`
- [ ] **barraCuda**: Review biosignal submodule structure as template for other signal processing domains
- [ ] **barraCuda**: Document f64 `pow()` workaround as standard pattern in shader guidelines
- [ ] **toadStool**: Review Kokkos benchmark crossover points for dispatch routing thresholds
- [ ] **All lib crates**: Adopt `#![deny(clippy::pedantic)]` as standard (healthSpring validates it's achievable)

### Near-term (V15 evolution)

- [ ] **barraCuda**: FOCE per-subject GPU shader (batch-parallel gradient computation)
- [ ] **barraCuda**: VPC Monte Carlo GPU shader (embarrassingly parallel, highest ROI)
- [ ] **barraCuda**: SAEM E-step GPU shader (Metropolis-Hastings sampling)
- [ ] **toadStool**: NLME workload type in dispatch rules (GPU at >100 subjects)
- [ ] **petalTongue**: VPC percentile band chart type
- [ ] **petalTongue**: Beat annotation overlay for WFDB ECG

### Long-term

- [ ] **Population-scale NLME**: 10K-100K subjects on GPU (requires FOCE shader + streaming)
- [ ] **Real-time VPC**: Live diagnostic updates during NLME estimation → petalTongue streaming
- [ ] **NPU biosignal pipeline**: Pan-Tompkins on Akida AKD1000 via PCIe P2P bypass
- [ ] **Cross-spring shader registry**: healthSpring + wetSpring + hotSpring WGSL shaders in shared catalog
