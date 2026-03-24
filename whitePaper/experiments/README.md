# healthSpring Experiments

Validation experiments documenting the four-tier pipeline (Python → Rust CPU → GPU → metalForge) for each health application domain.

**Status**: V44 — Deep Debt Resolution & Modern Idiomatic Evolution. 83 experiments, 928 Rust tests, 59 JSON-RPC capabilities (46 science + 13 infra). 54 baselines with verified check counts and DOI baseline sources. `primal_names` module centralized. `gpu/mod.rs` smart-refactored (696→413). Tolerance migration across 8 experiments. toadStool 51 tests. WFDB annotations 11 tests.
**Last Updated**: March 24, 2026

---

## Completed Experiments

### Track 1: PK/PD Modeling

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 001 | Hill dose-response (4 human JAK inhibitors) | nS-601 extension | 0,1,2 | 19 | 18 |
| 002 | One-compartment PK (IV bolus + oral + multi-dose) | Rowland & Tozer Ch. 3 | 0,1 | 12 | 18 |
| 003 | Two-compartment PK (biexponential α/β) | Rowland & Tozer Ch. 19 | 0,1 | 15 | 11 |
| 004 | mAb PK cross-species transfer (lokivetmab → nemolizumab) | nS-603 extension | 0,1 | 12 | 7 |
| 005 | Population PK Monte Carlo (1,000 patients) | Mould & Upton 2013 | 0,1,2 | 15 | 12 |
| 006 | PBPK 5-tissue physiological compartments | Gabrielsson & Weiner | 0,1 | 13 | 13 |
| 096 | Niclosamide delivery (formulation PK) | Drug delivery | 0,1 | control | binary |

### Track 2: Gut Microbiome

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 010 | Shannon/Simpson/Pielou/Chao1 diversity | wetSpring Track 1 | 0,1,2 | 14 | 12 |
| 011 | Anderson localization in gut lattice | wetSpring Exp107 extension | 0,1 | 12 | 14 |
| 012 | C. diff colonization resistance score | Jenior 2021 / Anderson ξ | 0,1 | 10 | 10 |
| 013 | FMT microbiota transplant for rCDI | van Nood 2013 / Bray-Curtis | 0,1 | 12 | 12 |
| 107 | QS-augmented Anderson (quorum-sensing disorder) | Phase 3 QS gene profiling | 0,1 | control | binary |
| 108 | Real 16S Anderson (HMP/NCBI SRA) | HMP pipeline | 0,1 | control | binary |

### Track 3: Biosignal Processing

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 020 | Pan-Tompkins QRS detection (5-stage intermediates) | Pan & Tompkins 1985 | 0,1 | 12 | 12 |
| 021 | HRV metrics (SDNN, RMSSD, pNN50) | Task Force 1996 | 0,1 | 10 | 10 |
| 022 | PPG SpO2 R-value calibration | Beer-Lambert / Tremper 1989 | 0,1 | 11 | 11 |
| 023 | Multi-channel fusion (ECG + PPG + EDA) | Composite weighted index | 0,1 | 11 | 11 |
| 109 | MIT-BIH arrhythmia (full database validation) | PhysioNet MIT-BIH | 0,1 | control | binary |

### Track 4: Endocrinology (Testosterone PK / TRT Outcomes)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 030 | Testosterone PK: IM injection steady-state | Shoskes 2016, Ross 2004 | 0,1 | 12 | 11 |
| 031 | Testosterone PK: pellet depot (5-month) | Testopel label, Cavender 2009 | 0,1 | 10 | 10 |
| 032 | Age-related testosterone decline | Harman 2001 (BLSA, n=890) | 0,1 | 10 | 8 |
| 033 | TRT metabolic response: weight/BMI/waist | Saad 2013 (n=411, 5yr) | 0,1 | 10 | 7 |
| 034 | TRT cardiovascular: lipids + CRP + BP | Sharma 2015 (VA, n=83,010) | 0,1 | 10 | 10 |
| 035 | TRT diabetes: HbA1c + insulin sensitivity | Kapoor 2006 (RCT) | 0,1 | 10 | 10 |
| 036 | Population TRT Monte Carlo (10K patients) | Lognormal IIV, age-adjusted | 0,1 | 12 | 10 |
| 037 | Testosterone–gut axis: microbiome stratification | Cross-track 2×4 hypothesis | 0,1 | 12 | 10 |
| 038 | HRV × TRT cardiovascular (cross-track D3) | Kleiger 1987 / Mok Ch. 6 | 0,1 | 10 | 10 |

### Validation

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 040 | barraCuda CPU parity (15 analytical contracts) | Analytical identities | 0,1 | 15 | 15 |

### Integrated Diagnostics

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 050 | Integrated diagnostic pipeline (4 tracks + cross-track) | Physiological ranges | 1 | — | 35 |
| 051 | Population diagnostic Monte Carlo (1000 patients) | Statistical validation | 1 | — | 21 |
| 052 | petalTongue scenario schema validation | JSON round-trip | 1 | — | 31 |

### GPU Pipeline (Tier 2)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 053 | GPU parity: WGSL shaders vs CPU (Hill, PopPK, Diversity) | CPU reference | 2 | — | 17 |
| 054 | Fused pipeline: single encoder + toadStool GPU dispatch | Individual dispatch | 2,3 | — | 11 |
| 055 | GPU scaling: 1K→10M sweep, crossover, field deployment | CPU baseline | 2 | — | Benchmark |

### Visualization (petalTongue Scenarios)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 056 | Full petalTongue visualization (5 tracks, 7 channel types, 14 scenarios) | Schema validation | 1 | — | 57 |

### Dispatch and Transfer (V8)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 060 | CPU vs GPU parity matrix — 3 kernels × 3 scales | CPU reference | 2,3 | — | 27 |
| 061 | Mixed hardware dispatch — NUCLEUS topology, PCIe P2P | CPU fallback | 3 | — | 22 |
| 062 | PCIe P2P transfer validation — Gen3/4/5 bandwidth | Analytical | 3 | — | 26 |

### Clinical TRT Scenarios & petalTongue IPC (V9)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 063 | Patient-parameterized TRT (5 archetypes, clinical mode) | Schema validation | 1 | — | Structural |
| 064 | IPC push to petalTongue (Unix socket, JSON-RPC, fallback) | E2E integration | 1 | — | Structural |
| 065 | Live streaming dashboard (ECG, HRV, PK via StreamSession) | Backpressure validation | 1 | — | Structural |

### Compute & Benchmark (V10-V11)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 066 | barraCuda CPU benchmark (Hill, PopPK, Diversity timing) | Timing reference | 1 | — | Structural |
| 067 | GPU parity extended (additional kernel validation) | CPU reference | 2 | — | Structural |
| 068 | GPU benchmark (throughput at scale) | CPU timing | 2 | — | Structural |
| 069 | toadStool dispatch matrix (stage assignment) | Manual dispatch | 3 | — | Structural |
| 070 | PCIe P2P bypass (NPU→GPU direct transfer) | CPU roundtrip | 3 | — | Structural |
| 071 | Mixed system pipeline (CPU+GPU+NPU coordinated) | Sequential fallback | 3 | — | Structural |
| 072 | Compute dashboard (toadStool → petalTongue live gauges) | Schema validation | 1 | — | 8 |

### petalTongue Evolution (V12)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 073 | Clinical TRT live dashboard (PK trough, HRV, cardiac risk) | Schema + stream validation | 1 | — | 7 |
| 074 | Interaction roundtrip (mock petalTongue: caps + subscribe) | Mock server E2E | 1 | — | 12 |

### NLME + Full Pipeline (V14)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 075 | NLME cross-validation (FOCE/SAEM, NCA, CWRES, GOF) | Beal & Sheiner, Kuhn & Lavielle | 0,1 | — | 19 |
| 076 | Full pipeline petalTongue validation (5 tracks, 28 nodes) | Structural + schema | 1 | — | 197 |

### Track 6: Comparative Medicine (V25)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 100 | Canine IL-31 serum kinetics (Gonzales 2013) | CM-001 | 0,1 | control | binary |
| 101 | Canine oclacitinib JAK1 selectivity (Gonzales 2014) | CM-002 | 0,1 | control | binary |
| 102 | IL-31 pruritus time-course recovery (Gonzales 2016) | CM-003 | 0,1 | control | binary |
| 103 | Lokivetmab dose-duration (Fleck/Gonzales 2021) | CM-004 | 0,1 | control | binary |
| 104 | Cross-species PK allometric scaling | CM-005 | 0,1 | control | binary |
| 105 | Canine gut microbiome Anderson lattice | CM-006 | 0,1 | control | binary |
| 106 | Feline hyperthyroidism methimazole PK (Trepanier 2006) | CM-007 | 0,1 | control | binary |
| 110 | Equine laminitis inflammatory cascade (hoof lamellae) | CM-008 | 0,1 | control | binary |

### Track 7: Drug Discovery (V25)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 090 | Anderson-augmented MATRIX scoring | DD-001 | 0,1 | control | binary |
| 091 | ADDRC HTS analysis (Z'-factor, SSMD) | DD-002 | 0,1 | control | binary |
| 092 | Compound IC50 profiling (library sweep) | DD-003 | 0,1 | control | binary |
| 093 | ChEMBL JAK panel bioactivity | DD-004 | 0,1 | control | binary |
| 094 | Rho/MRTF/SRF fibrosis pathway scoring (Neubig) | DD-005 | 0,1 | control | binary |
| 095 | iPSC skin model readout (Gonzales) | DD-006 | 0,1 | control | binary |

### Track 9: Low-Affinity Binding / Toxicology / Simulation (V39)

| Exp | Name | Control | Tiers | Python | Rust Binary |
|-----|------|---------|:-----:|:------:|:-----------:|
| 097 | Low-affinity binding landscape (composite selectivity, Gini) | Hill 1910, Lorenz/Gini | 0,1 | control | 18 |
| 098 | Toxicity landscape (Anderson IPR, clearance, delocalization) | Anderson 1958, Rowland & Tozer | 0,1 | control | 22 |
| 099 | Hormesis (biphasic curve, mithridatism, caloric restriction, hygiene) | Calabrese & Baldwin 2003 | 0,1 | control | 18 |
| 111 | Causal terrarium (mechanistic fitness, ecosystem reshaping) | Lotka-Volterra, HSP/SOD/p53/mTOR | 0,1 | control | 18 |

### Cross-Validation

| Test | Scope | Matches | Status |
|------|-------|:-------:|--------|
| cross_validate.py | All 7 tracks (Tracks 1–7) | 113/113 | **Complete** (V28) |

---

## Directory Layout

```
experiments/
├── exp001_hill_dose_response/
├── exp002_one_compartment_pk/
├── exp003_two_compartment_pk/
├── exp004_mab_pk_transfer/
├── exp005_population_pk/
├── exp006_pbpk_compartments/
├── exp010_diversity_indices/
├── exp011_anderson_gut_lattice/
├── exp012_cdiff_resistance/
├── exp013_fmt_rcdi/
├── exp020_pan_tompkins_qrs/
├── exp021_hrv_metrics/
├── exp022_ppg_spo2/
├── exp023_biosignal_fusion/
├── exp030_testosterone_im_pk/
├── exp031_testosterone_pellet_pk/
├── exp032_age_testosterone_decline/
├── exp033_trt_weight_trajectory/
├── exp034_trt_cardiovascular/
├── exp035_trt_diabetes/
├── exp036_population_trt_montecarlo/
├── exp037_testosterone_gut_axis/
├── exp038_hrv_trt_cardiovascular/
├── exp040_barracuda_cpu_parity/
├── exp050_diagnostic_pipeline/
├── exp051_population_diagnostic/
├── exp052_petaltongue_render/
├── exp053_gpu_parity/
├── exp054_gpu_pipeline/
├── exp055_gpu_scaling/
├── exp056_study_scenarios/       # V7: Full petalTongue visualization for all 4 tracks
├── exp060_cpu_vs_gpu_pipeline/   # V8: CPU vs GPU parity matrix
├── exp061_mixed_hardware_dispatch/ # V8: NUCLEUS topology + DispatchPlan
├── exp062_pcie_transfer_validation/ # V8: PCIe P2P Gen3/4/5 bandwidth
├── exp063_clinical_trt_scenarios/  # V9: Patient-parameterized TRT (5 archetypes)
├── exp064_ipc_push/               # V9: IPC push to petalTongue (JSON-RPC)
├── exp065_live_dashboard/         # V10: Live streaming dashboard
├── exp066_barracuda_cpu_bench/    # V10: CPU benchmarks
├── exp067_gpu_parity_extended/    # V10: Extended GPU validation
├── exp068_gpu_benchmark/          # V10: GPU throughput benchmark
├── exp069_toadstool_dispatch_matrix/ # V11: toadStool dispatch validation
├── exp070_pcie_p2p_bypass/        # V11: PCIe P2P bypass (NPU→GPU)
├── exp071_mixed_system_pipeline/  # V11: Mixed CPU+GPU+NPU pipeline
├── exp072_compute_dashboard/      # V11: Compute dashboard → petalTongue
├── exp073_clinical_trt_dashboard/ # V12: Live TRT clinical dashboard
├── exp074_interaction_roundtrip/  # V12: Interaction roundtrip validation
├── exp075_nlme_cross_validation/  # V14: NLME (FOCE/SAEM, NCA, diagnostics)
├── exp076_full_pipeline_scenarios/ # V14: Full pipeline petalTongue validation
├── exp077_michaelis_menten_pk/    # V16: MM nonlinear PK
├── exp078_antibiotic_perturbation/ # V16: Antibiotic perturbation
├── exp079_scfa_production/       # V16: SCFA production
├── exp080_gut_brain_serotonin/   # V16: Gut-brain serotonin axis
├── exp081_eda_stress_detection/  # V16: EDA stress detection
├── exp082_arrhythmia_classification/ # V16: Arrhythmia beat classification
├── exp083_gpu_v16_parity/        # V16: GPU parity
├── exp084_v16_cpu_parity_bench/   # V18: CPU vs Python bench
├── exp085_gpu_vs_cpu_v16_bench/  # V19: GPU scaling bench
├── exp086_toadstool_v16_dispatch/ # V19: toadStool dispatch
├── exp087_mixed_nucleus_v16/     # V19: NUCLEUS routing
├── exp088_unified_dashboard/     # V20: petalTongue V16 dashboard
├── exp089_patient_explorer/      # V20: Patient explorer
├── exp090_matrix_scoring/        # V25: Track 7 — MATRIX scoring
├── exp091_addrc_hts/             # V25: Track 7 — ADDRC HTS
├── exp092_compound_library/      # V25: Track 7 — compound IC50
├── exp093_chembl_jak_panel/      # V25: Track 7 — ChEMBL JAK panel
├── exp094_rho_mrtf_fibrosis/     # V25: Track 7 — fibrosis scoring
├── exp095_ipsc_skin_model/       # V36: Track 7 — iPSC skin model
├── exp096_niclosamide_delivery/  # V36: Track 1 — niclosamide delivery
├── exp100_canine_il31/           # V25: Track 6 — canine IL-31
├── exp101_canine_jak1/           # V25: Track 6 — JAK1 selectivity
├── exp102_il31_pruritus_timecourse/ # V25: Track 6 — pruritus
├── exp103_lokivetmab_duration/   # V25: Track 6 — lokivetmab
├── exp104_cross_species_pk/      # V25: Track 6 — cross-species PK
├── exp105_canine_gut_anderson/   # V25: Track 6 — canine gut Anderson
├── exp106_feline_hyperthyroid/   # V25: Track 6 — feline MM PK
├── exp107_qs_augmented_anderson/ # V36: Track 2 — QS-augmented Anderson
├── exp108_real_16s_anderson/     # V36: Track 2 — real 16S Anderson
├── exp109_mitbih_arrhythmia/     # V36: Track 3 — MIT-BIH arrhythmia
└── exp110_equine_laminitis/      # V36: Track 6 — equine laminitis
```

Controls live in `control/`:
```
control/
├── pkpd/
│   ├── exp001_hill_dose_response.py
│   ├── exp002_one_compartment_pk.py
│   ├── exp003_two_compartment_pk.py
│   ├── exp004_mab_pk_transfer.py
│   ├── exp005_population_pk.py
│   ├── exp006_pbpk_compartments.py
│   └── cross_validate.py              # 113 checks, all 7 tracks
├── microbiome/
│   ├── exp010_diversity_indices.py
│   ├── exp011_anderson_gut_lattice.py
│   ├── exp012_cdiff_resistance.py
│   └── exp013_fmt_rcdi.py
├── biosignal/
│   ├── exp020_pan_tompkins_qrs.py
│   ├── exp021_hrv_metrics.py
│   ├── exp022_ppg_spo2.py
│   └── exp023_fusion.py
├── endocrine/
│   ├── exp030_testosterone_im_pk.py
│   ├── exp031_testosterone_pellet_pk.py
│   ├── exp032_age_testosterone_decline.py
│   ├── exp033_trt_weight_trajectory.py
│   ├── exp034_trt_cardiovascular.py
│   ├── exp035_trt_diabetes.py
│   ├── exp036_population_trt_montecarlo.py
│   ├── exp037_testosterone_gut_axis.py
│   └── exp038_hrv_trt_cardiovascular.py
└── validation/
    └── exp040_barracuda_cpu.py
```

---

## Numbering Convention

- **001–009**: Track 1 (PK/PD)
- **010–019**: Track 2 (Microbiome)
- **020–029**: Track 3 (Biosignal)
- **030–039**: Track 4 (Endocrinology)
- **040–049**: Validation and parity
- **050–052**: Integrated diagnostics and visualization schema
- **053–055**: GPU pipeline (Tier 2) and scaling
- **056–059**: Visualization and petalTongue scenario generation
- **060–062**: CPU vs GPU parity, mixed hardware dispatch, PCIe transfer
- **063–065**: Clinical TRT patient scenarios, petalTongue IPC, live streaming
- **066–068**: Compute benchmarks (CPU and GPU)
- **069–072**: toadStool dispatch, PCIe bypass, mixed systems, compute dashboard
- **073–074**: petalTongue evolution (interaction, streaming, capabilities)
- **075–076**: NLME population PK + full pipeline validation
- **077–082**: V16 primitives (MM PK, antibiotic, SCFA, serotonin, EDA, arrhythmia)
- **083–089**: GPU V16 parity, benchmarks, toadStool dispatch, petalTongue V16
- **090–094**: Track 7 (Drug Discovery / ADDRC)
- **100–106**: Track 6 (Comparative Medicine / One Health)
- **097–099**: Track 9 (Low-Affinity Binding / Toxicology / Hormesis)
- **107–110**: V36 extensions (QS Anderson, real 16S, MIT-BIH, equine laminitis)
- **111**: Track 9 (Causal Terrarium — multi-scale simulation)
- **112+**: Future extensions

---

## How to Add a New Experiment

1. Write Python control in `control/{track}/exp{NNN}_{name}.py` — runs checks inline
2. Add Rust implementations to `ecoPrimal/src/{module}.rs` with `#[cfg(test)]` unit tests
3. Create validation binary in `experiments/exp{NNN}_{name}/` (workspace member)
4. Run Python control → Rust unit tests → validation binary
5. Update this README, `experiments/README.md`, `specs/PAPER_REVIEW_QUEUE.md`, and `whitePaper/baseCamp/`
