# healthSpring Experiments

**Last Updated**: March 10, 2026
**Status**: V16 — 54 experiments, all 30 paper queue entries complete. V16: Michaelis-Menten PK (Exp077), antibiotic perturbation (Exp078), SCFA production (Exp079), gut-brain serotonin (Exp080), EDA stress (Exp081), arrhythmia classification (Exp082). NLME (FOCE/SAEM), NCA, diagnostics (CWRES/VPC/GOF), WFDB, Kokkos benchmarks, full petalTongue pipeline (28 nodes, 121 channels).

Each experiment is a standalone Rust binary that validates a specific scientific claim or system capability. Experiments follow the four-tier pipeline: Python control (Tier 0) → Rust CPU (Tier 1) → GPU (Tier 2) → metalForge dispatch (Tier 3).

---

## Experiment Inventory

### Track 1: PK/PD (Exp001-006) — Tier 0+1

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 001 | `exp001_hill_dose_response` | Hill dose-response (4 JAK inhibitors + canine reference) | 18 |
| 002 | `exp002_one_compartment_pk` | One-compartment PK (IV + oral + multiple dosing + AUC) | 18 |
| 003 | `exp003_two_compartment_pk` | Two-compartment PK (biexponential α/β) | 11 |
| 004 | `exp004_mab_pk_transfer` | mAb cross-species transfer (lokivetmab → human) | 7 |
| 005 | `exp005_population_pk` | Population PK Monte Carlo (1,000 patients) | 12 |
| 006 | `exp006_pbpk_compartments` | PBPK 5-tissue model (liver, kidney, muscle, fat, rest) | 13 |

### Track 2: Microbiome (Exp010-013) — Tier 0+1

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 010 | `exp010_diversity_indices` | Shannon/Simpson/Pielou/Chao1 diversity | 12 |
| 011 | `exp011_anderson_gut_lattice` | Anderson localization in gut (ξ, IPR, CR) | 14 |
| 012 | `exp012_cdiff_resistance` | C. difficile colonization resistance score | 10 |
| 013 | `exp013_fmt_rcdi` | FMT microbiota transplant for rCDI | 12 |

### Track 3: Biosignal (Exp020-023) — Tier 0+1

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 020 | `exp020_pan_tompkins_qrs` | Pan-Tompkins QRS detection (5-stage intermediates) | 12 |
| 021 | `exp021_hrv_metrics` | HRV metrics (SDNN, RMSSD, pNN50, power spectrum) | 10 |
| 022 | `exp022_ppg_spo2` | PPG SpO2 R-value calibration | 11 |
| 023 | `exp023_biosignal_fusion` | Multi-channel fusion (ECG + PPG + EDA) | 11 |

### Track 4: Endocrinology (Exp030-038) — Tier 0+1

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 030 | `exp030_testosterone_im_pk` | Testosterone PK: IM injection steady-state | 11 |
| 031 | `exp031_testosterone_pellet_pk` | Testosterone PK: pellet depot (5-month) | 10 |
| 032 | `exp032_age_testosterone_decline` | Age-related testosterone decline (BLSA model) | 8 |
| 033 | `exp033_trt_weight_trajectory` | TRT metabolic response (Saad 2013 registry) | 7 |
| 034 | `exp034_trt_cardiovascular` | TRT cardiovascular (lipids + CRP + BP) | 10 |
| 035 | `exp035_trt_diabetes` | TRT diabetes (HbA1c + insulin sensitivity) | 10 |
| 036 | `exp036_population_trt_montecarlo` | Population TRT Monte Carlo (10K patients) | 10 |
| 037 | `exp037_testosterone_gut_axis` | Testosterone–gut axis cross-track | 10 |
| 038 | `exp038_hrv_trt_cardiovascular` | HRV × TRT cardiovascular cross-track | 10 |

### Validation (Exp040) — Tier 0+1

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 040 | `exp040_barracuda_cpu_parity` | barraCuda CPU parity (analytical contracts) | 15 |

### Integrated Diagnostics (Exp050-052)

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 050 | `exp050_diagnostic_pipeline` | Integrated 4-track patient diagnostic | structural |
| 050 | `exp050_scenario_dump` | Scenario JSON dump | structural |
| 051 | `exp051_population_diagnostic` | Population diagnostic Monte Carlo | structural |
| 052 | `exp052_petaltongue_render` | petalTongue scenario schema validation | structural |

### GPU Pipeline (Exp053-055) — Tier 2

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 053 | `exp053_gpu_parity` | WGSL shader output vs CPU baseline | 17 |
| 054 | `exp054_gpu_pipeline` | Fused pipeline + toadStool GPU dispatch | structural |
| 055 | `exp055_gpu_scaling` | 1K→10M scaling, crossover analysis | structural |

### Visualization (Exp056) — petalTongue

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 056 | `exp056_study_scenarios` | Full 5-track scenario generation (7 channel types, 14 scenarios) | 57 |
| 056 | `dump_scenarios` | Write 14 scenarios to disk or push via IPC | 14 files |

### CPU vs GPU + Mixed Dispatch (Exp060-062) — Tier 2+3

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 060 | `exp060_cpu_vs_gpu_pipeline` | CPU vs GPU parity matrix (3 kernels × 3 scales) | 27 |
| 061 | `exp061_mixed_hardware_dispatch` | NUCLEUS topology dispatch routing | 22 |
| 062 | `exp062_pcie_transfer_validation` | PCIe P2P DMA transfer planning | 26 |

### Clinical TRT + petalTongue IPC (Exp063-065)

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 063 | `exp063_clinical_trt_scenarios` | Patient-parameterized TRT (5 archetypes) | structural |
| 064 | `exp064_ipc_push` | IPC push to petalTongue (JSON-RPC) | structural |
| 065 | `exp065_live_dashboard` | Live streaming dashboard (ECG, HRV, PK) | structural |

### Compute & Benchmark (Exp066-072)

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 066 | `exp066_barracuda_cpu_bench` | barraCuda CPU benchmark timing | structural |
| 067 | `exp067_gpu_parity_extended` | Extended GPU kernel validation | structural |
| 068 | `exp068_gpu_benchmark` | GPU throughput at scale | structural |
| 069 | `exp069_toadstool_dispatch_matrix` | toadStool stage assignment validation | structural |
| 070 | `exp070_pcie_p2p_bypass` | PCIe P2P bypass (NPU→GPU direct) | structural |
| 071 | `exp071_mixed_system_pipeline` | CPU+GPU+NPU coordinated execution | structural |
| 072 | `exp072_compute_dashboard` | toadStool streaming → petalTongue live gauges | 8 |

### petalTongue Evolution (Exp073-074)

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 073 | `exp073_clinical_trt_dashboard` | Live TRT dashboard (PK streaming, cardiac risk replace) | 7 |
| 074 | `exp074_interaction_roundtrip` | Mock petalTongue interaction roundtrip | 12 |

### NLME + Full Pipeline (Exp075-076) — V14

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 075 | `exp075_nlme_cross_validation` | NLME cross-validation (FOCE/SAEM, NCA, CWRES, GOF) | 19 |
| 076 | `exp076_full_pipeline_scenarios` | Full pipeline petalTongue scenario validation (5 tracks) | 197 |

---

## Running Experiments

```bash
# Build all experiments
cargo build --workspace --release

# Run all experiments
for bin in target/release/exp0*; do
    [ -x "$bin" ] && "$bin"
done

# Run a specific experiment
cargo run --release --bin exp073_clinical_trt_dashboard

# Run compute validation suite
./scripts/compute_dashboard.sh

# Run petalTongue scenario generation
cargo run --release --bin dump_scenarios
```

---

## Open Data

All experiments use publicly accessible data or published model parameters. No proprietary data dependencies. See `specs/PAPER_REVIEW_QUEUE.md` for the per-experiment open data audit.
