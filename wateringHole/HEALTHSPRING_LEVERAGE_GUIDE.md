<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Leverage Guide — Standalone, Trio, and Full Niche

**Date**: March 18, 2026
**Primal**: healthSpring V36 (`healthspring-barracuda` 0.1.0)
**Audience**: All springs, all primals, biomeOS integrators
**Status**: Active

---

## Purpose

This document describes how healthSpring can be leveraged — alone and in composition with other primals — by springs and ecosystem consumers. Each primal in the ecosystem produces an equivalent guide. Together, these guides form a combinatorial recipe book for emergent behaviors.

healthSpring provides **human health science computation** — PK/PD modeling, microbiome analytics, biosignal processing, endocrine models, and diagnostic pipelines. Pure Rust, zero unsafe, zero `#[allow()]`, zero clippy warnings. 6 GPU ops rewired to barraCuda upstream. 617 tests, 79 experiments, 79 capabilities.

**Philosophy**: Health science is sovereign. Hill dose-response, Shannon diversity, Pan-Tompkins QRS — these are universal primitives. healthSpring owns the biology; other primals own the hardware, the network, the storage, the identity. Discover at runtime, compose at will.

---

## 1. Standalone Usage

These patterns use healthSpring alone — no other primals required.

### 1.1 Direct Library Dependency

**For**: Any spring needing PK/PD, microbiome, biosignal, or endocrine models.

```toml
[dependencies]
healthspring-barracuda = { path = "../healthSpring/ecoPrimal" }
# Optional: GPU acceleration
healthspring-barracuda = { path = "../healthSpring/ecoPrimal", features = ["barracuda-ops"] }
# Optional: NestGate data fetch (NCBI, SRA)
healthspring-barracuda = { path = "../healthSpring/ecoPrimal", features = ["nestgate"] }
```

**Key modules and entry points**:

| Module | Entry Points | Domain |
|--------|--------------|--------|
| `pkpd` | `hill_dose_response`, `auc_trapezoidal`, `nca_iv`, `population_pk` | Dose-response, NCA, compartmental PK |
| `microbiome` | `shannon_index`, `simpson_index`, `pielou_evenness`, `chao1`, `anderson_gut` | Diversity, Anderson lattice |
| `biosignal` | `pan_tompkins`, `sdnn_ms`, `rmssd_ms`, `ppg_r_value`, `spo2_from_r`, `classify_all_beats` | ECG, HRV, PPG, beat classification |
| `endocrine` | `testosterone_pk`, `trt_outcomes`, `cardiac_risk` | TRT, testosterone PK |
| `discovery` | `estimate_ic50`, `matrix_score`, `score_compound` | HTS, MATRIX scoring |
| `comparative` | `lokivetmab_pk`, `methimazole_simulate`, `il31_serum_kinetics` | Canine, feline, allometric |
| `qs` | `qs_profile`, `effective_disorder` | QS gene profiling |
| `uncertainty` | `bootstrap_mean`, `jackknife_mean_variance`, `monte_carlo_propagate` | Bootstrap, jackknife, MC |

### 1.2 Quick Examples

**Hill dose-response**:
```rust
use healthspring_barracuda::pkpd::hill_dose_response;
let response = hill_dose_response(10.0, 10.0, 1.0, 1.0);
// response ≈ 0.5 at C = IC50
```

**Diversity indices**:
```rust
use healthspring_barracuda::microbiome;
let abundances = [0.25, 0.25, 0.25, 0.25];
let shannon = microbiome::shannon_index(&abundances);
let simpson = microbiome::simpson_index(&abundances);
```

**PK simulation**:
```rust
use healthspring_barracuda::pkpd::{pk_oral_one_compartment, pk_multiple_dose};
// One-compartment oral, 5 doses q12h
let concs = pk_multiple_dose(
    |t| pk_oral_one_compartment(100.0, 0.8, 50.0, 1.0, 0.1, t),
    12.0, 5, &times,
);
```

**QRS detection**:
```rust
use healthspring_barracuda::biosignal::pan_tompkins;
let result = pan_tompkins(&ecg_signal, 250.0);
// result.peaks = QRS peak sample indices
```

### 1.3 IPC Science Primal

**For**: Any primal wanting health computation without a compile-time dependency.

```
→ healthspring.sock: {"jsonrpc":"2.0","method":"science.pkpd.hill_dose_response","params":{"concentration":10,"ic50":10,"hill_n":1,"e_max":1},"id":1}
← {"jsonrpc":"2.0","result":{"response":0.5},"id":1}
```

**Discovery**: `discover::discover_socket(&socket_env_var("healthspring"), "healthspring")`

### 1.4 Validation Harness Pattern

**For**: Any spring needing structured validation with provenance.

healthSpring's `Validator` + `OrExit<T>` pattern (absorbed from hotSpring):
- Named checks with pass/fail/exit-code
- Provenance headers (script, commit, date, hardware, command)
- Zero-panic: `OrExit` replaces all `unwrap()`/`expect()` in binaries

---

## 2. Trio Usage (Spring + barraCuda + toadStool)

### 2.1 GPU Acceleration — 6 Ops

| healthSpring Op | barraCuda Op | Feature Gate | Tier |
|-----------------|-------------|--------------|------|
| `HillSweep` | `barracuda::ops::HillFunctionF64` | `barracuda-ops` | A |
| `PopulationPkBatch` | `barracuda::ops::PopulationPkF64` | `barracuda-ops` | A |
| `DiversityBatch` | `barracuda::ops::bio::DiversityFusionGpu` | `barracuda-ops` | A |
| `MichaelisMentenBatch` | `barracuda::ops::health::MichaelisMentenBatchGpu` | `barracuda-ops` | B |
| `ScfaBatch` | `barracuda::ops::health::ScfaBatchGpu` | `barracuda-ops` | B |
| `BeatClassifyBatch` | `barracuda::ops::health::BeatClassifyGpu` | `barracuda-ops` | B |

All 6 ops delegate to barraCuda upstream. Local WGSL shaders retained as validation targets.

### 2.2 Feature Flags

| Flag | Purpose |
|------|---------|
| `barracuda-ops` | Enable GPU dispatch via barraCuda primitives |
| `nestgate` | NCBI/SRA fetch via NestGate (adds `ureq`, content-addressed cache) |
| `sovereign-dispatch` | WGSL → coralReef compile → native binary (no wgpu) |

### 2.3 Pipeline Dispatch via toadStool

healthSpring submits `compute.offload` with workload type. toadStool routes to CPU/GPU/NPU via metalForge. Progress callbacks stream to petalTongue for live visualization.

**Novel pattern**: **Substrate-aware health** — population PK Monte Carlo on GPU (8.2× at 100K patients), biosignal fusion on NPU for real-time wearables. toadStool routes by workload characteristics discovered at runtime.

---

## 3. Composition Usage (Full Niche)

### 3.1 biomeOS Niche Architecture

healthSpring is a **niche**, not a node. The primal provides capabilities; graphs define composition.

| Component | Count |
|-----------|-------|
| Primal binary | `healthspring_primal` |
| Capabilities | 79 |
| Domain dispatchers | 6 (`pkpd`, `microbiome`, `biosignal`, `endocrine`, `diagnostic`, `clinical`) |
| Workflow graphs | 5 |

### 3.2 Workflow Graphs

| Graph | Coordination | Description |
|-------|--------------|-------------|
| `patient_assessment` | ConditionalDag | 4-track diagnostic: PK/PD ∥ Microbiome ∥ Biosignal ∥ Endocrine → Composite |
| `trt_scenario` | Sequential | TRT clinical: endocrine PK → outcomes → HRV response → cardiac risk → visualize |
| `microbiome_analysis` | Sequential | Diversity → Anderson → colonization → QS profiling → SCFA |
| `biosignal_monitor` | Continuous | ECG → QRS → HRV → fusion → render @ 250 Hz |
| `deploy` | Sequential | Bring up niche: Tower → Trio → NestGate → ToadStool → healthSpring |

### 3.3 petalTongue Integration

healthSpring exposes `DataChannel`, `HealthScenario`, `ClinicalRange` schemas. Scenario builders: `pkpd_study`, `microbiome_study`, `biosignal_study`, `endocrine_study`, `nlme_study`, `full_study`. Live push via `PetalTonguePushClient` for streaming gauges and time series.

---

## 4. Capability Reference

Top capabilities by domain (46 science + 13 infra):

| Domain | Key Capabilities |
|--------|------------------|
| **PK/PD** | `hill_dose_response`, `one_compartment_pk`, `population_pk`, `nlme_foce`, `nlme_saem`, `nca_analysis`, `michaelis_menten_nonlinear`, `vpc_simulate`, `gof_compute` |
| **Microbiome** | `shannon_index`, `simpson_index`, `pielou_evenness`, `chao1`, `anderson_gut`, `colonization_resistance`, `scfa_production`, `qs_gene_profile`, `qs_effective_disorder` |
| **Biosignal** | `pan_tompkins`, `hrv_metrics`, `ppg_spo2`, `arrhythmia_classification`, `fuse_channels`, `wfdb_decode` |
| **Endocrine** | `testosterone_pk`, `trt_outcomes`, `population_trt`, `hrv_trt_response`, `cardiac_risk` |
| **Diagnostic** | `assess_patient`, `composite_risk`, `population_montecarlo` |
| **Clinical** | `trt_scenario`, `patient_parameterize`, `risk_annotate` |
| **Infra** | `provenance.begin/record/complete`, `primal.forward`, `compute.offload`, `data.fetch`, `health.liveness`, `health.readiness` |

---

## 5. Data Provenance

- **All datasets from public sources**: NCBI, SRA, PhysioNet (MIT-BIH), ChEMBL, published PK parameters.
- **49 Python control scripts** with provenance: `control/` subdirs (pkpd, microbiome, biosignal, endocrine, discovery, comparative, validation). Each script emits baseline JSON; Rust experiments validate against it.
- **113/113 cross-validation checks**: Every experiment with a Python baseline passes parity. Provenance chain: `begin_data_session` → `record_fetch_step` → `complete_data_session` (rhizoCrypt + loamSpine + sweetGrass when trio available).

---

## 6. GPU Pipeline

| Component | Status |
|-----------|--------|
| WGSL shaders | 6 (`hill_dose_response_f64`, `population_pk_f64`, `diversity_f64`, `michaelis_menten_batch_f64`, `scfa_batch_f64`, `beat_classify_batch_f64`) |
| barraCuda rewire | All 6 ops delegate to `barracuda::ops::*` |
| Tier A + B | Complete |
| Fused pipeline | `gpu/fused.rs` — upload → N compute → readback |
| metalForge routing | `select_substrate()` → CPU/GPU/NPU by workload; NUCLEUS topology (Tower/Node/Nest) |

---

## 7. Evolution Status

| Metric | V36 |
|--------|-----|
| Tests | 617 |
| Experiments | 79 |
| Python baselines | 42 |
| Cross-validation | 113/113 |
| Capabilities | 79 |
| GPU ops (barraCuda) | 6/6 |
| Unsafe blocks | 0 |
| `#[allow()]` | 0 |
| Clippy warnings | 0 |
| cargo-deny | Enforced |
| C-dep ban list | 14 crates (openssl-sys, libz-sys, etc.) |
| WGSL license | AGPL-3.0-or-later |

---

## 8. Cross-Spring Patterns

### What healthSpring Absorbed

| Source | Absorption |
|--------|------------|
| **barraCuda** | `HillFunctionF64`, `PopulationPkF64`, `DiversityFusionGpu`, `MichaelisMentenBatchGpu`, `ScfaBatchGpu`, `BeatClassifyGpu`; `OdeSystem`, `BatchedOdeRK4`; `lcg_step`, `mean`, `uniform_f64_sequence` |
| **toadStool** | Compute dispatch, streaming pipeline, capability discovery |
| **metalForge** | NUCLEUS topology, `select_substrate()`, PCIe P2P transfer planning |
| **hotSpring** | `Validator`, `OrExit<T>`, tolerance registry pattern |
| **groundSpring** | `cargo-deny` in CI, dependency hygiene |
| **wetSpring** | Diversity indices (Shannon, Simpson, Chao1), Anderson lattice concepts |

### What healthSpring Contributes Back

| Contribution | Consumer |
|--------------|----------|
| **6 WGSL shaders** | barraCuda `ops::health` (MM, SCFA, BeatClassify absorbed upstream) |
| **Health domain primitives** | barraCuda `ops::health::*` |
| **Validation harness** | All springs (OrExit, named tolerances) |
| **Niche deployment pattern** | biomeOS (first health niche: 5 graphs, 6 domains) |
| **petalTongue schema** | `HealthScenario`, `DataChannel`, `ClinicalRange` |
| **Comparative medicine** | Canine IL-31/JAK1, feline methimazole, allometric scaling |
| **Discovery pipeline** | MATRIX scoring, HTS, fibrosis, compound IC50 |

---

## Versioning

This guide tracks healthSpring's evolution. As capabilities are added, compositions are updated.

| Version | Date | Changes |
|---------|------|---------|
| V36 | March 18, 2026 | Initial guide: 79 capabilities, 6 GPU ops rewired, 5 workflow graphs, 6 domain dispatchers |
