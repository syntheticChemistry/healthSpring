# healthSpring Experiments

**Last Updated**: April 20, 2026
**Status**: V55 — guideStone Level 3. 94 experiments, 948+ tests. Eleven composition experiments (exp112–122). `healthspring_guidestone` binary (three-tier primal proof harness per v1.1.0): Tier 1 local, Tier 2 IPC-wired, Tier 3 full NUCLEUS primal proof. P3 BLAKE3 checksums. primalSpring v0.9.16.

Each experiment is a standalone Rust binary that validates a specific scientific claim or system capability. Experiments follow a six-tier pipeline:

- **Tier 0**: Python control baseline (analytical known-values)
- **Tier 1**: Rust CPU validation (direct Rust vs Python, exit 0/1)
- **Tier 2**: GPU parity (WGSL shader vs Rust CPU)
- **Tier 3**: metalForge dispatch (NUCLEUS routing, PCIe P2P)
- **Tier 4**: Composition validation (IPC dispatch vs direct Rust — the primal composition surface)
- **Tier 5**: Deploy graph validation (TOML graph ↔ proto-nucleate ↔ capability surface consistency)

**All 94 experiments** use the standardized `ValidationHarness` pattern (zero ad-hoc validation).

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
| 050 | `exp050_diagnostic_pipeline` | Integrated 4-track patient diagnostic | ValidationHarness |
| 050 | `exp050_scenario_dump` | Scenario JSON dump | ValidationHarness |
| 051 | `exp051_population_diagnostic` | Population diagnostic Monte Carlo | ValidationHarness |
| 052 | `exp052_petaltongue_render` | petalTongue scenario schema validation | ValidationHarness |

### GPU Pipeline (Exp053-055) — Tier 2

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 053 | `exp053_gpu_parity` | WGSL shader output vs CPU baseline | 17 |
| 054 | `exp054_gpu_pipeline` | Fused pipeline + toadStool GPU dispatch | ValidationHarness |
| 055 | `exp055_gpu_scaling` | 1K→10M scaling, crossover analysis | ValidationHarness |

### Visualization (Exp056) — petalTongue

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 056 | `exp056_study_scenarios` | Full 5-track scenario generation (7 channel types, 14 scenarios) | 57 |
| 056 | `dump_scenarios` | Write 16 scenarios to disk or push via IPC | 14 files |

### CPU vs GPU + Mixed Dispatch (Exp060-062) — Tier 2+3

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 060 | `exp060_cpu_vs_gpu_pipeline` | CPU vs GPU parity matrix (3 kernels × 3 scales) | 27 |
| 061 | `exp061_mixed_hardware_dispatch` | NUCLEUS topology dispatch routing | 22 |
| 062 | `exp062_pcie_transfer_validation` | PCIe P2P DMA transfer planning | 26 |

### Clinical TRT + petalTongue IPC (Exp063-065)

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 063 | `exp063_clinical_trt_scenarios` | Patient-parameterized TRT (5 archetypes) | ValidationHarness |
| 064 | `exp064_ipc_push` | IPC push to petalTongue (JSON-RPC) | ValidationHarness |
| 065 | `exp065_live_dashboard` | Live streaming dashboard (ECG, HRV, PK) | ValidationHarness |

### Compute & Benchmark (Exp066-072)

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 066 | `exp066_barracuda_cpu_bench` | barraCuda CPU benchmark timing | ValidationHarness |
| 067 | `exp067_gpu_parity_extended` | Extended GPU kernel validation | ValidationHarness |
| 068 | `exp068_gpu_benchmark` | GPU throughput at scale | ValidationHarness |
| 069 | `exp069_toadstool_dispatch_matrix` | toadStool stage assignment validation | ValidationHarness |
| 070 | `exp070_pcie_p2p_bypass` | PCIe P2P bypass (NPU→GPU direct) | ValidationHarness |
| 071 | `exp071_mixed_system_pipeline` | CPU+GPU+NPU coordinated execution | ValidationHarness |
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

### V16 Paper Queue Complete (Exp077-082) — V16

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 077 | `exp077_michaelis_menten_pk` | Michaelis-Menten nonlinear PK (phenytoin) | 10 |
| 078 | `exp078_antibiotic_perturbation` | Antibiotic perturbation recovery model | 10 |
| 079 | `exp079_scfa_production` | SCFA metabolic production model | 11 |
| 080 | `exp080_gut_brain_serotonin` | Gut-brain serotonin pathway | 10 |
| 081 | `exp081_eda_stress_detection` | EDA autonomic stress detection | 11 |
| 082 | `exp082_arrhythmia_classification` | Arrhythmia beat classification | 11 |

### GPU V16 Portability (Exp083) — V17

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 083 | `exp083_gpu_v16_parity` | GPU parity for V16 primitives + metalForge routing | 25 |

### CPU Parity Benchmarks (Exp084) — V18

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 084 | `exp084_v16_cpu_parity_bench` | Rust vs Python timing: 84× faster (33 Rust + 17 Python) | 33 |

### GPU Scaling + toadStool Dispatch + NUCLEUS Routing (Exp085-087) — V19

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 085 | `exp085_gpu_vs_cpu_v16_bench` | GPU scaling (4 scales × 3 ops) + fused pipeline + metalForge routing | 47 |
| 086 | `exp086_toadstool_v16_dispatch` | toadStool streaming dispatch + callbacks + GPU-mappability | 24 |
| 087 | `exp087_mixed_nucleus_v16` | NUCLEUS Tower/Node/Nest + PCIe P2P bypass + plan_dispatch | 35 |

### petalTongue V16 Visualization + Patient Explorer (Exp088-089) — V20

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 088 | `exp088_unified_dashboard` | Unified dashboard — all scenarios (original + V16 + compute) validated, pushed, or dumped | 326 |
| 089 | `exp089_patient_explorer` | Patient explorer — diagnostic + V16 analysis, CLI params, streaming to petalTongue | 14 |

### Track 7: Drug Discovery (Exp090-096) — V25+

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 090 | `exp090_matrix_scoring` | Anderson-augmented MATRIX scoring (pathway × geometry × disorder) | 14 |
| 091 | `exp091_addrc_hts` | ADDRC high-throughput screening analysis (Z'-factor, SSMD, %inhibition) | 14 |
| 092 | `exp092_compound_library` | Batch IC50 profiling + selectivity ranking | 14 |
| 093 | `exp093_chembl_jak_panel` | ChEMBL JAK kinase selectivity panel | 14 |
| 094 | `exp094_rho_mrtf_fibrosis` | Rho/MRTF/SRF fibrosis pathway scoring (Neubig) | 14 |
| 095 | `exp095_ipsc_skin_model` | iPSC skin model (Gonzales/Ellsworth) | ValidationHarness |
| 096 | `exp096_niclosamide_delivery` | Niclosamide delivery (Ellsworth med chem) | ValidationHarness |

### Track 6: Comparative Medicine (Exp100-110) — V25+

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 100 | `exp100_canine_il31` | Canine IL-31 serum kinetics (Gonzales 2013) | 14 |
| 101 | `exp101_canine_jak1` | Canine JAK1 selectivity panel (Gonzales 2014) | 15 |
| 102 | `exp102_il31_pruritus_timecourse` | IL-31 pruritus time-course recovery (Gonzales 2016) | 13 |
| 103 | `exp103_lokivetmab_duration` | Lokivetmab dose-duration relationship (Fleck 2021) | 15 |
| 104 | `exp104_cross_species_pk` | Cross-species allometric PK scaling | 12 |
| 105 | `exp105_canine_gut_anderson` | Canine gut microbiome Anderson lattice | 13 |
| 106 | `exp106_feline_hyperthyroid` | Feline hyperthyroidism methimazole PK | 14 |
| 107 | `exp107_qs_augmented_anderson` | QS-augmented Anderson localization | ValidationHarness |
| 108 | `exp108_real_16s_anderson` | Real 16S Anderson (wetSpring integration) | ValidationHarness |
| 109 | `exp109_mitbih_arrhythmia` | MIT-BIH arrhythmia classification | ValidationHarness |
| 110 | `exp110_equine_laminitis` | Equine laminitis model | ValidationHarness |

### Tier 4: Composition Validation (Exp112-117) — V48

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 112 | `exp112_composition_pkpd` | IPC dispatch vs direct Rust for PK/PD (Hill, IV, AUC, MM) | 12 |
| 113 | `exp113_composition_microbiome` | IPC dispatch vs direct Rust for microbiome (Shannon, Simpson, Anderson) | 10 |
| 114 | `exp114_composition_health_triad` | Capability surface coverage (58+ methods, 10 domains, structured errors) | 17 |
| 115 | `exp115_composition_proto_nucleate` | Proto-nucleate alignment (socket resolution, discovery, constants) | 20 |
| 116 | `exp116_composition_provenance` | Provenance session lifecycle (registry, data sessions, trio probe) | 14 |
| 117 | `exp117_composition_ipc_roundtrip` | IPC wire protocol round-trip + proto-nucleate aliases + capability surface | 71 |

### Tier 3: Deploy Graph Validation (Exp118) — V52

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 118 | `exp118_composition_deploy_graph_validation` | Deploy graph ↔ proto-nucleate structural alignment (fragments, nodes, bonding, capabilities, Squirrel optionality) | 99 |

### Tier 4: Live IPC Composition Parity (Exp119-121) — V53

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 119 | `exp119_composition_live_parity` | Live IPC science dispatch vs direct Rust (Hill, 1-comp PK, AUC, Shannon, Anderson) | 6+ |
| 120 | `exp120_composition_live_provenance` | Live provenance trio round-trip (session lifecycle, Merkle, commit, braid) | 5+ |
| 121 | `exp121_composition_live_health` | Live NUCLEUS health probes (liveness, readiness, capability.list, identity.get, niche dispatch) | 6+ |

### Tier 5: Level 5 Primal Proof — barraCuda IPC (Exp122) — V53

| Exp | Binary | Domain | Checks |
|-----|--------|--------|:------:|
| 122 | `exp122_primal_proof_barracuda_parity` | `math_dispatch` known-values, `BarraCudaClient` IPC vs local (stats.mean, stats.std_dev, rng.uniform), wire-pending inventory | 10+ |

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

# petalTongue V16 visualization + patient explorer
cargo run --release --bin exp088_unified_dashboard              # 326 checks
cargo run --release --bin exp089_patient_explorer               # 14 checks
cargo run --release --bin exp089_patient_explorer -- --age 55 --weight 220 --baseline-t 280

# Track 6+7: Comparative Medicine + Drug Discovery
cargo run --release --bin exp090_matrix_scoring
cargo run --release --bin exp100_canine_il31
cargo run --release --bin exp106_feline_hyperthyroid

# Tier 4: Composition validation (IPC dispatch parity)
cargo run --release --bin exp112_composition_pkpd
cargo run --release --bin exp113_composition_microbiome
cargo run --release --bin exp114_composition_health_triad
cargo run --release --bin exp115_composition_proto_nucleate
cargo run --release --bin exp116_composition_provenance

# Tier 3: Deploy graph validation
cargo run --release --bin exp118_composition_deploy_graph_validation  # 99 checks

# Tier 4: Live IPC composition parity (requires running healthspring_primal)
cargo run --release --bin healthspring_primal -- serve &  # start primal server
cargo run --release --bin exp119_composition_live_parity              # science parity
cargo run --release --bin exp120_composition_live_provenance          # provenance trio
cargo run --release --bin exp121_composition_live_health              # health probes
```

---

## Open Data

All experiments use publicly accessible data or published model parameters. No proprietary data dependencies. See `specs/PAPER_REVIEW_QUEUE.md` for the per-experiment open data audit.
