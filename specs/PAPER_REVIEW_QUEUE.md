# healthSpring Paper Review Queue

**Last Updated**: March 9, 2026
**Status**: 47 experiments complete — 630 Rust binary checks, 317 tests (250 barracuda + 33 forge + 30 toadStool + 4 doc-tests), 104 cross-validation checks. GPU Tier 2+3 live. V13 deep audit: Anderson eigensolver, smart refactoring, math deduplication.

---

## Queue Summary

| Track | Papers Queued | Started | Complete |
|-------|:------------:|:-------:|:--------:|
| Track 1: PK/PD | 7 | 6 | 6 |
| Track 2: Microbiome | 7 | 4 | 4 |
| Track 3: Biosignal | 6 | 4 | 4 |
| Track 4: Endocrinology | 9 | 9 | 9 |
| Validation | 1 | 1 | 1 |
| **Total** | **30** | **24** | **24** |

---

## Completed

| ID | Paper | Experiment | Python | Rust Binary | Tier |
|----|-------|-----------|:------:|:-----------:|:----:|
| PK-007 | Gonzales oclacitinib → human JAK inhibitor PK | Exp001 | 19 | 18 | 0,1,2 |
| PK-001 | Rowland & Tozer Ch. 3 — one-compartment PK | Exp002 | 12 | 18 | 0,1 |
| PK-001 | Rowland & Tozer Ch. 19 — two-compartment PK | Exp003 | 15 | 11 | 0,1 |
| PK-006 | Gonzales lokivetmab → human mAb PK | Exp004 | 12 | 7 | 0,1 |
| PK-002 | Mould & Upton 2013 — Population PK Monte Carlo | Exp005 | 15 | 12 | 0,1,2 |
| PK-004 | Gabrielsson & Weiner PBPK compartments | Exp006 | 13 | 13 | 0,1 |
| MB-007 | wetSpring 16S → diversity indices | Exp010 | 14 | 12 | 0,1,2 |
| MB-006 | wetSpring Anderson soil → gut colonization | Exp011 | 12 | 14 | 0,1 |
| MB-001 | Jenior et al. C. diff metabolic modeling | Exp012 | 10 | 10 | 0,1 |
| MB-002 | McGill synthetic microbiota for rCDI | Exp013 | 12 | 12 | 0,1 |
| BS-001 | Pan & Tompkins QRS detection | Exp020 | 12 | 12 | 0,1 |
| BS-002 | MIT-BIH HRV metrics (SDNN, RMSSD, pNN50) | Exp021 | 10 | 10 | 0,1 |
| BS-003 | Wearable PPG → SpO2 via R-value calibration | Exp022 | 11 | 11 | 0,1 |
| BS-004 | Multi-channel biosignal fusion (ECG + PPG + EDA) | Exp023 | 11 | 11 | 0,1 |
| EN-001 | Mok Ch.11 + Shoskes 2016 — IM testosterone PK | Exp030 | 12 | 11 | 0,1 |
| EN-002 | Testopel label + Cavender 2009 — Pellet PK | Exp031 | 10 | 10 | 0,1 |
| EN-003 | Harman 2001 (BLSA) — Age testosterone decline | Exp032 | 10 | 8 | 0,1 |
| EN-004 | Saad 2013 registry — TRT weight trajectory | Exp033 | 10 | 7 | 0,1 |
| EN-005 | Saad 2016 + Sharma 2015 — TRT cardiovascular | Exp034 | 10 | 10 | 0,1 |
| EN-006 | Kapoor 2006 + Dhindsa 2016 — TRT diabetes | Exp035 | 10 | 10 | 0,1 |
| EN-007 | Composite registries — Population TRT Monte Carlo | Exp036 | 12 | 10 | 0,1 |
| EN-008 | Cross-track D1/D2 — Testosterone-gut axis | Exp037 | 12 | 10 | 0,1 |
| EN-D3 | Mok D3: HRV × TRT cardiovascular cross-track | Exp038 | 10 | 10 | 0,1 |
| VAL-001 | barraCuda CPU parity — analytical contracts | Exp040 | 15 | 15 | 0,1 |

---

## GPU Tier 2 Validation (V8-V12)

Papers PK-007, PK-002, and MB-007 are validated at Tier 2 via WGSL shaders:

| Paper | GPU Shader | Parity | Experiment |
|-------|-----------|--------|-----------|
| PK-007 (Hill dose-response) | `hill_dose_response_f64.wgsl` | < 1e-4 | Exp053, Exp060 |
| PK-002 (Population PK) | `population_pk_f64.wgsl` | < 1e-4 | Exp053, Exp060 |
| MB-007 (Diversity indices) | `diversity_f64.wgsl` | < 1e-4 | Exp053, Exp060 |

CPU vs GPU parity matrix (Exp060): 27/27 checks across 3 kernels × 3 scales.

---

## Mixed Hardware Dispatch (V8-V12)

metalForge NUCLEUS topology validation:

| Validation | Experiment | Checks |
|-----------|-----------|--------|
| NUCLEUS dispatch routing | Exp061 | 22/22 |
| PCIe P2P transfer planning | Exp062 | 26/26 |
| PCIe P2P bypass (NPU→GPU) | Exp070 | structural |
| Mixed system pipeline (CPU+GPU+NPU) | Exp071 | structural |
| toadStool dispatch matrix | Exp069 | structural |

---

## Next Papers (Remaining Queue)

All 24 queued papers complete at Tier 0+1. Three have Tier 2 GPU validation. Next evolution targets:

1. **GPU Tier 2**: Anderson eigensolve (Exp011) → `anderson_lyapunov_f64.wgsl`
2. **GPU Tier 2**: Biosignal FFT (Exp020-023) → GPU radix-2 FFT for real-time ECG/PPG
3. **metalForge Tier 3**: Full pipeline through NUCLEUS topology with biomeOS atomic deployment graphs
4. **NPU Tier 3**: Pan-Tompkins streaming via Akida NPU (toadStool NPU dispatch path)

---

## Open Data Audit

All experiments use publicly accessible data. No proprietary dependencies.

| Experiment | Open Data Source | Access Method |
|-----------|-----------------|---------------|
| Exp001 | Published IC50 values (Gonzales 2014, FDA labels) | Literature |
| Exp002 | Textbook equations (Rowland & Tozer) | Published reference |
| Exp003 | Textbook equations (Rowland & Tozer Ch. 19) | Published reference |
| Exp004 | Fleck/Gonzales 2021 + Kabashima 2020 Phase III | Peer-reviewed papers |
| Exp005 | Published population PK parameters | Literature + simulation |
| Exp006 | Gabrielsson & Weiner PBPK model | Published tissue parameters |
| Exp010 | Synthetic communities (defined abundances) | Self-generated from literature |
| Exp011 | wetSpring validated lattice code (V99) | ecoPrimals internal |
| Exp012 | Published CDI clinical parameters | Literature |
| Exp013 | McGill FMT parameters (literature) | Peer-reviewed transplant studies |
| Exp020 | MIT-BIH Arrhythmia Database | PhysioNet (open) |
| Exp021 | Synthetic ECG (internal) | Generated from Gaussian P-QRS-T model |
| Exp022 | Beer-Lambert PPG calibration | Published SpO2 calibration curves |
| Exp023 | Synthetic multi-channel (ECG+PPG+EDA) | Internal generated signals |
| Exp030 | Shoskes 2016, Ross 2004 (IM PK) | Peer-reviewed papers |
| Exp031 | Testopel PI, Cavender 2009 | FDA label + literature |
| Exp032 | Harman 2001 (BLSA, n=890), Feldman 2002 | Peer-reviewed longitudinal |
| Exp033 | Saad 2013 registry (n=411) | Published figures |
| Exp034 | Saad 2016, Sharma 2015 (VA, n=83,010) | Published registry/cohort |
| Exp035 | Kapoor 2006 (RCT), Dhindsa 2016, Hackett 2014 | Peer-reviewed RCTs |
| Exp036 | Composite registries (simulation) | Literature parameters |
| Exp037 | Cross-track (ecoPrimals thesis) | Internal + Ridaura 2013, Tremellen 2012 |
| Exp038 | Kleiger 1987 HRV mortality + Mok TRT claims | Peer-reviewed + clinical |
| Exp040 | Analytical identities (textbook) | Mathematical definitions |
| Exp050-052 | Composite diagnostic (all tracks) | Same open sources as Tracks 1-4 |
| Exp053-055 | GPU parity (no new data) | CPU baseline as ground truth |
| Exp056 | Visualization (no new data) | Scenario builders use existing data |
| Exp060-062 | Hardware dispatch (no new data) | CPU/GPU parity as ground truth |
| Exp063-065 | Clinical TRT (published registries) | Same open sources as Track 4 |
| Exp066-072 | Compute benchmarks (no new data) | Existing pipelines, timing only |
| Exp073-074 | petalTongue evolution (no new data) | Existing scenarios + mock server |

See `specs/README.md` for full provenance table.
