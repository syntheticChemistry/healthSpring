<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring: Compute & Data Hunger Profile

**Last Updated**: March 10, 2026

---

## Executive Summary

healthSpring spans five domains with dramatically different compute/data profiles:

| Track | Data Hunger | Compute Hunger | GPU Benefit | LAN HPC Need |
|-------|:-----------:|:--------------:|:-----------:|:------------:|
| 1 — PK/PD | Low (published params) | **Medium → High** (population Monte Carlo) | **10-100×** for pop PK | Moderate |
| 2 — Microbiome | **Medium** (NCBI 16S) | **Medium → High** (eigendecomposition) | **50-500×** for Anderson spectral | High for L>500 |
| 3 — Biosignal | Low (synthetic or PhysioNet) | Low → Medium (streaming + batch classify) | **10×** FFT, **100×** batch classify | Low → Medium |
| 4 — Endocrinology | **Medium** (literature extraction + NCBI) | **Medium → Very High** (cross-track Monte Carlo) | **100-1000×** for D4 | **High** |
| 5 — NLME | Low (simulated) → Medium (real trial data) | **Medium → High** (FOCE gradient, VPC MC) | **50-500×** for batch FOCE | Medium |

### V17 GPU Shader Inventory

| Shader | GpuOp | Track | Pattern | Validated |
|--------|-------|-------|---------|-----------|
| `hill_dose_response_f64.wgsl` | HillSweep | 1 | Element-wise | Exp053 |
| `population_pk_f64.wgsl` | PopulationPkBatch | 1 | Embarrassingly parallel MC | Exp053 |
| `diversity_f64.wgsl` | DiversityBatch | 2 | Workgroup reduction | Exp053 |
| `michaelis_menten_batch_f64.wgsl` | MichaelisMentenBatch | 1 | Per-patient Euler ODE + PRNG | Exp083 |
| `scfa_batch_f64.wgsl` | ScfaBatch | 2 | Element-wise MM kinetics (3-output) | Exp083 |
| `beat_classify_batch_f64.wgsl` | BeatClassifyBatch | 3 | Per-beat cross-correlation | Exp083 |

### metalForge Workload Routing (V17)

| Workload | GPU Threshold | Track |
|----------|:------------:|-------|
| PopulationPk | 5M patients | 1 |
| DoseResponse | 100K concentrations | 1 |
| DiversityIndex | 10K samples | 2 |
| BiosignalDetect | 100K Hz × channels | 3 |
| BiosignalFusion | 8+ channels | 3 |
| EndocrinePk | 10K timepoints | 4 |
| MichaelisMentenBatch | 5M patients | 1 |
| ScfaBatch | 100K elements | 2 |
| BeatClassifyBatch | 10K beats | 3 |

---

## Per-Experiment Profiles

### Track 1: PK/PD

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 001 Hill dose-response | 4 drug params (published) | <0.1s | None (too small) | — | Already complete |
| 002 One-compartment | 5 drug params | <0.1s | None | — | Already complete |
| 003 Two-compartment | 6 micro-constants | <0.1s | None | — | Already complete |
| 004 mAb transfer | 4 allometric params | <0.1s | None | — | Already complete |
| 005 Population PK | 1K patients × 500 timepoints | 0.2s CPU | **50×** (each patient = independent ODE) | 4MB | GPU ideal |
| 005+ Pop PK 100K | 100K patients | ~20s CPU | **100×** → 0.2s GPU | 400MB | **Northgate RTX 5090** |
| 005++ Pop PK 1M | 1M patients | ~200s CPU | **500×** → 0.4s GPU | 4GB | Strandgate RTX 3090 |
| 077 MM nonlinear PK | Phenytoin params + N patients | <0.1s (1 patient) | **100×** (batch Euler ODE) | 10MB | GPU shader ready (V17) |
| 077+ MM Pop PK 100K | 100K patients × ODE steps | ~30s CPU | **200×** via `michaelis_menten_batch_f64.wgsl` | 200MB | **Northgate RTX 5090** |

### Track 2: Microbiome

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 010 Diversity indices | 5 community profiles | <0.01s | None | — | Already complete |
| 011 Anderson gut (L=200) | 50 realizations × L² Hamiltonian | 1s CPU | **20×** for eigh GPU | 32MB | Already complete |
| 011+ Anderson gut (L=1000) | 50 × 1M matrix | ~5min CPU | **200×** → 1.5s Lanczos GPU | 1GB | Needs Lanczos GPU |
| 011++ Anderson gut 3D (20³) | 8000-site Hamiltonian | ~1hr CPU | **500×** → 7s GPU eigh | 500MB | hotSpring `BatchedEighGpu` |
| 012 C. diff resistance | 30 realizations × L² | 0.4s CPU | Same as 011 | 8MB | Already complete |
| 037 Gut-TRT axis | NCBI 16S + clinical covariates | — | GPU diversity + Anderson | — | NestGate data pipeline |
| 078 Antibiotic perturbation | Synthetic (Dethlefsen params) | <0.1s | Low | — | Validate vs real 16S later |
| 079 SCFA production | Fiber concentrations | <0.1s | **50×** via `scfa_batch_f64.wgsl` | 5MB | GPU shader ready (V17) |
| 079+ SCFA 100K fibers | 100K fiber inputs × 3 SCFAs | ~5s CPU | **100×** → 0.05s GPU | 50MB | Embarrassingly parallel |
| 080 Gut-brain serotonin | Tryptophan concentrations | <0.1s | Low (single pathway) | — | Cross-track D5 |
| QS gene profiling | NCBI Gene ~5GB | 1hr download | Medium (matrix construction) | 500MB | NestGate caching |

### Track 3: Biosignal

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 020 Pan-Tompkins QRS | 3600 samples (10s ECG) | 0.2s CPU | **5×** for FFT GPU | 1MB | Already complete |
| 020+ MIT-BIH (PhysioNet) | 48 records × 30min × 360Hz | ~30s CPU | **10×** for streaming GPU | 50MB | Open data (PhysioNet) |
| 021 HRV analysis | Same as 020+ | ~10s CPU | Low benefit | 10MB | Planned |
| 022 SpO2 (PPG) | Synthetic or open | ~5s CPU | Low benefit | 5MB | Planned |
| 081 EDA stress | Synthetic EDA signal | <0.1s | Low (streaming) | — | Tonic/phasic decomposition |
| 082 Arrhythmia classify | Beat windows + templates | <0.1s | **100×** via `beat_classify_batch_f64.wgsl` | 10MB | GPU shader ready (V17) |
| 082+ MIT-BIH full | 48 records × 360Hz × 30min | ~30s CPU | **50×** batch classify GPU | 100MB | WFDB parser ready |

### Track 4: Endocrinology (Mok)

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 030 T IM PK | Published PK params | <0.1s | None | — | Straightforward |
| 031 T pellet PK | Label data | <0.1s | None | — | Straightforward |
| 032 Age decline | BLSA/MMAS tables (digitized) | <1s | None | — | Curve fitting |
| 033 Weight trajectory | Saad 2013 curves (digitized) | ~5s | **10×** for MCMC | 10MB | Mixed-effects |
| 034 CV risk time series | Saad 2016 curves | ~5s | **10×** for MCMC | 10MB | Mixed-effects |
| 035 HbA1c trajectory | Kapoor 2006, Hackett 2014 | ~2s | Low | 5MB | Small RCTs |
| 036 Pop TRT 10K | 10K patients × T PK × metabolic | ~60s CPU | **100×** → <1s GPU | 100MB | **Key GPU target** |
| D4 Cross-track 10K | 10K × (PK + Anderson + metabolic) | ~600s CPU | **500×** → 1.2s GPU | 500MB | **Requires Northgate** |

---

### Track 5: NLME Population PK

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 075 NLME cross-validation | 30 subjects × 14 timepoints | ~0.5s CPU (FOCE 150 iter) | **50×** (per-subject gradient) | 1MB | FOCE + SAEM validated |
| 075+ NLME 1K subjects | 1K subjects × 14 timepoints | ~15s CPU | **100×** → 0.15s GPU | 10MB | Batch-parallel per-subject |
| 075++ NLME 10K subjects | 10K subjects | ~150s CPU | **500×** → 0.3s GPU | 100MB | **Northgate RTX 5090** |
| 075 VPC simulation | 50 simulations × 30 subjects | ~2s CPU | **20×** (embarrassingly parallel) | 5MB | Monte Carlo |
| 075+ VPC 1K sims | 1K simulations | ~40s CPU | **200×** → 0.2s GPU | 50MB | GPU ideal |
| 076 Full pipeline | 28 nodes × 121 channels | <2s CPU | N/A (scenario construction) | — | Structural validation |

---

## NCBI Data Requirements (NestGate)

| Query | Database | Est. Records | Est. Size | NestGate Method |
|-------|----------|:------------:|:---------:|-----------------|
| "testosterone replacement therapy" | PubMed | ~5,000 abstracts | ~50MB | `data.ncbi_search` |
| Androgen receptor gene expression | GEO (SRA) | ~200 datasets | ~50GB raw | `data.ncbi_fetch` |
| 16S gut microbiome + testosterone | SRA | ~20 datasets | ~10GB raw | `data.ncbi_fetch` |
| Hypogonadism GWAS summary stats | dbGaP | ~5 studies | ~1GB | Public summary stats |

**Total estimated data footprint**: ~60GB (mostly SRA raw reads, stored on Westgate ZFS)

---

## Hardware Allocation Plan

### Phase 1: Development (Current — Eastgate)
- All Tier 0 (Python) and Tier 1 (Rust CPU) experiments
- Works within 32GB RAM
- Sufficient for Exp001-020, Exp030-035

### Phase 2: GPU Validation (LAN — Northgate + biomeGate)
- Population PK Monte Carlo (Exp005+, 036, D4) → **Northgate RTX 5090** (32GB VRAM)
- f64-native validation → **biomeGate Titan V** (native f64, 12GB VRAM)
- Anderson large-lattice eigensolve → **Northgate** (L=1000 needs ~1GB)

### Phase 3: Data Pipeline (LAN — Strandgate + Westgate)
- NCBI data fetch/store → **Strandgate** (256GB RAM, dual EPYC for bulk processing)
- ZFS storage → **Westgate** (76TB cold storage)
- NestGate data RPC → biomeOS Nest atomic

### Phase 4: Full Pipeline (LAN HPC — All Gates)
- biomeOS NUCLEUS deployment:
  - **Tower**: Eastgate (development, coordination)
  - **Node**: Northgate (GPU compute), biomeGate (f64 validation)
  - **Nest**: Strandgate (NCBI data + storage), Westgate (ZFS archive)
  - **Full**: All of the above orchestrated via biomeOS

### 10GbE Network Dependency
- Population Monte Carlo dispatch: Northgate ↔ Strandgate data (~100MB/run) → **< 0.1s at 10GbE**
- NCBI bulk download: Westgate storage → **sustained 1GB/s** (60GB in 1 minute)
- Cross-gate NUCLEUS RPC: ~1ms latency at 10GbE

---

## Summary: What Needs LAN vs. What Runs Locally

| Runs Locally (Eastgate) | Needs LAN HPC |
|------------------------|---------------|
| All Tier 0 Python controls | Population Monte Carlo > 10K patients |
| All Tier 1 Rust CPU | Anderson L > 500 eigensolve |
| All 73 experiments (Exp001-106) | NCBI bulk data (50+ GB) |
| Basic population PK (1K) | Cross-track Monte Carlo (D4) |
| QS gene matrix construction (NCBI Gene API) | Real 16S HMP pipeline (50GB) |
| MIT-BIH full dataset (100MB) | PhysioNet MIMIC-III (50GB subset) |
| ChEMBL drug panel extraction (2GB) | GEO androgen receptor (50GB) |
| | f64-native GPU validation (Titan V) |
| | Full biomeOS NUCLEUS pipeline |

---

## NUCLEUS Integration Timeline

### Phase 1: Nest Atomic Locally (Eastgate) — Ready Now

- NestGate locally for NCBI data caching (NCBI Gene, PubMed)
- `biomeos nucleus start --mode nest`
- healthSpring calls `data.ncbi_search` / `data.ncbi_fetch` via Unix socket
- Content-addressed store: `ncbi:{db}:{accession}`

### Phase 2: Node Atomic Locally (Eastgate) — Ready Now

- toadStool compute dispatch via local NUCLEUS
- GPU workloads through Node Atomic pipeline (RTX 4070)
- `biomeos nucleus start --mode node`

### Phase 3: LAN Mesh (10GbE cables pending)

- Northgate: Node Atomic (GPU compute, RTX 5090) — population MC, Anderson eigensolve
- Strandgate: Nest Atomic (NCBI data, 256GB RAM) — 16S pipeline, batch processing
- Westgate: Nest Atomic (ZFS archive, 76TB) — cold storage
- Eastgate: Tower Atomic (coordination, development)
- Cross-gate RPC: ~1ms at 10GbE

### Phase 4: Full NUCLEUS

- biomeOS orchestrates the full DAG
- metalForge routes: GPU(Northgate) → NPU(Eastgate) → CPU(any)
- NestGate provenance tracking for all NCBI data
- Plasmodium collective with `.family.seed` trust
