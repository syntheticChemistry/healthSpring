# healthSpring: Compute & Data Hunger Profile

**Last Updated**: March 8, 2026

---

## Executive Summary

healthSpring spans four domains with dramatically different compute/data profiles:

| Track | Data Hunger | Compute Hunger | GPU Benefit | LAN HPC Need |
|-------|:-----------:|:--------------:|:-----------:|:------------:|
| 1 — PK/PD | Low (published params) | **Medium → High** (population Monte Carlo) | **10-100×** for pop PK | Moderate |
| 2 — Microbiome | **Medium** (NCBI 16S) | **Medium → High** (eigendecomposition) | **50-500×** for Anderson spectral | High for L>500 |
| 3 — Biosignal | Low (synthetic or PhysioNet) | Low (streaming) | **10×** for FFT-based filters | Low |
| 4 — Endocrinology | **Medium** (literature extraction + NCBI) | **Medium → Very High** (cross-track Monte Carlo) | **100-1000×** for D4 | **High** |

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

### Track 2: Microbiome

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 010 Diversity indices | 5 community profiles | <0.01s | None | — | Already complete |
| 011 Anderson gut (L=200) | 50 realizations × L² Hamiltonian | 1s CPU | **20×** for eigh GPU | 32MB | Already complete |
| 011+ Anderson gut (L=1000) | 50 × 1M matrix | ~5min CPU | **200×** → 1.5s Lanczos GPU | 1GB | Needs Lanczos GPU |
| 011++ Anderson gut 3D (20³) | 8000-site Hamiltonian | ~1hr CPU | **500×** → 7s GPU eigh | 500MB | hotSpring `BatchedEighGpu` |
| 012 C. diff resistance | 30 realizations × L² | 0.4s CPU | Same as 011 | 8MB | Already complete |
| 037 Gut-TRT axis | NCBI 16S + clinical covariates | — | GPU diversity + Anderson | — | NestGate data pipeline |

### Track 3: Biosignal

| Exp | Data In | CPU Time | GPU Benefit | VRAM | Notes |
|-----|---------|----------|-------------|------|-------|
| 020 Pan-Tompkins QRS | 3600 samples (10s ECG) | 0.2s CPU | **5×** for FFT GPU | 1MB | Already complete |
| 020+ MIT-BIH (PhysioNet) | 48 records × 30min × 360Hz | ~30s CPU | **10×** for streaming GPU | 50MB | Open data (PhysioNet) |
| 021 HRV analysis | Same as 020+ | ~10s CPU | Low benefit | 10MB | Planned |
| 022 SpO2 (PPG) | Synthetic or open | ~5s CPU | Low benefit | 5MB | Planned |

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
| Exp001-020, 030-035 | NCBI bulk data (50+ GB) |
| Basic population PK (1K) | Cross-track Monte Carlo (D4) |
| | f64-native GPU validation (Titan V) |
| | Full biomeOS NUCLEUS pipeline |
