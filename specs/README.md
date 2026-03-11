<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Specifications

**Last Updated**: March 10, 2026
**Status**: V20 — 61 experiments, 395 Rust tests (329 barraCuda + 33 forge + 30 toadStool + 3 doc-tests), 194 Python cross-validation checks. Full-stack portability: barraCuda CPU → GPU (6 WGSL shaders) → toadStool streaming dispatch → metalForge NUCLEUS routing (PCIe P2P bypass). Paper queue 30/30 complete. V20: petalTongue V16 visualization (Exp088) + patient explorer (Exp089). V19: GPU scaling bench + toadStool dispatch + mixed NUCLEUS (106 new checks). V18: Rust 84× faster than Python (Exp084). V16-V17: 6 new domain primitives + 3 new WGSL shaders.
**Domain**: Human health applications — PK/PD, gut microbiome, biosignal, endocrinology

---

## Quick Status

| Metric | Value |
|--------|-------|
| Rust lib tests | 395 (329 barraCuda + 33 forge + 30 toadStool + 3 doc-tests) |
| Python control checks | 194 (cross-validation) |
| Experiments | 61 (30 Tier 0+1 + 3 diagnostic + 3 GPU + 1 viz + 3 dispatch + 3 clinical + 7 compute + 2 interaction + 2 NLME + 6 V16 primitives + 1 GPU V16 + 1 CPU bench + 3 V19 full-stack + 2 V20 visualization) |
| GPU validation (Tier 2) | **Live** — 6 WGSL shaders, fused pipeline, 42/42 parity, GPU scaling confirmed |
| metalForge validation (Tier 3) | 33 tests + Exp087 (35/35) — NUCLEUS dispatch with PCIe P2P bypass |
| toadStool validation | 30 tests + Exp086 (24/24) — V16 streaming dispatch |
| CPU parity | Rust 84× faster than Python (Exp084, 33+17 checks) |
| NLME population PK | FOCE + SAEM estimation, NCA, CWRES/VPC/GOF diagnostics |
| Paper queue | **30/30 complete** |
| Faculty | Gonzales (MSU), Lisabeth (ADDRC), Neubig (Drug Discovery), Mok (Allure Medical) |

---

## Paper Review Queue

Papers queued for reproduction and extension. Organized by track. See [PAPER_REVIEW_QUEUE.md](PAPER_REVIEW_QUEUE.md) for the detailed queue with per-paper status.

### Track Summary

| Track | Domain | Experiments | Python checks | Rust binary checks | Rust lib tests | Status |
|-------|--------|:-----------:|:------------:|:------------------:|:--------------:|--------|
| 1 | PK/PD | 6 (Exp001-006) | 86 | 79 | 39 | **Complete** |
| 2 | Microbiome | 4 (Exp010-013) | 48 | 48 | 12 | **Complete** |
| 3 | Biosignal | 4 (Exp020-023) | 44 | 44 | 5 | **Complete** |
| 4 | Endocrinology | 9 (Exp030-038) | 96 | 86 | 47 | **Complete** |
| Validation | CPU parity | 1 (Exp040) | 15 | 15 | — | **Complete** |
| Diagnostics | Integrated pipeline | 3 (Exp050-052) | — | 87 | — | **Complete** |
| GPU | Tier 2 pipeline | 3 (Exp053-055) | — | GPU live | — | **Complete** |
| Visualization | petalTongue scenarios | 1 (Exp056) | — | 50 | — | **Complete** |
| Dispatch | CPU vs GPU + mixed HW | 3 (Exp060-062) | — | 75 | — | **Complete** |
| Clinical | TRT + IPC + streaming | 3 (Exp063-065) | — | structural | — | **Complete** |
| Compute | Benchmarks + dashboard | 7 (Exp066-072) | — | structural | — | **Complete** |
| petalTongue | Evolution + interaction | 2 (Exp073-074) | — | 19 | — | **Complete** |
| NLME | Pop PK + full pipeline | 2 (Exp075-076) | — | 216 | — | **Complete** |
| V16 Primitives | MM PK, antibiotic, SCFA, serotonin, EDA, arrhythmia | 6 (Exp077-082) | 63 | structural | 27 | **Complete** |
| GPU V16 | GPU parity for V16 ops | 1 (Exp083) | — | 25 | — | **Complete** |
| CPU Parity | Rust vs Python bench | 1 (Exp084) | 17 | 33 | — | **Complete** |
| V19 Full-Stack | GPU scaling + dispatch + NUCLEUS | 3 (Exp085-087) | 10 | 106 | — | **Complete** |
| V20 Visualization | petalTongue V16 + patient explorer | 2 (Exp088-089) | — | 340 | — | **Complete** |
| **Total** | | **61** | **379** | | **395** | **All green** |

---

## Open Data Provenance

All healthSpring experiments use **publicly accessible data or published model parameters**. No proprietary data dependencies.

| Source | Tracks | Access |
|--------|--------|--------|
| Published pharmacokinetic parameters (FDA labels, Phase III papers) | Track 1 | Open (peer-reviewed literature) |
| Gonzales 2014, Fleck/Gonzales 2021 (published IC50, dose-duration) | Track 1 | Open (J Vet Pharmacol Ther, Vet Dermatol) |
| Kabashima 2020, Silverberg 2021 (nemolizumab Phase III) | Track 1 | Open (NEJM, JAMA Derm) |
| NCBI GEO / SRA (RNA-seq, 16S amplicon) | Track 2 | Open (NCBI public repository) |
| MIT-BIH Arrhythmia Database (PhysioNet) | Track 3 | Open (physionet.org, ODC-By license) |
| Harman et al. 2001 (BLSA longitudinal testosterone) | Track 4 | Open (JCEM) |
| Saad et al. 2013, 2016 (Moscow/Bremerhaven TRT registries) | Track 4 | Open (Obesity, Int J Obes) |
| Kapoor et al. 2006 (TRT RCT, Sheffield) | Track 4 | Open (Diabetes Care) |
| Sharma et al. 2015 (TRT cardiovascular meta-analysis) | Track 4 | Open (JAMA IM) |

### Open Data Controls for Paper Validation

Every paper in the review queue is validated against open data at three tiers:

1. **Tier 0 (Python control)**: Reference implementation using published equations and parameters from peer-reviewed literature. All source data is cited with DOI or accession number.
2. **Tier 1 (Rust CPU)**: Pure Rust implementation validated against the Python baseline within documented tolerances (`specs/TOLERANCE_REGISTRY.md`). Same open data sources.
3. **Tier 2 (Rust GPU)**: WGSL shader output validated against CPU baseline for math parity. No additional data — proves compute substrate independence.
4. **Tier 3 (metalForge)**: Cross-substrate dispatch validated for routing correctness. toadStool pipeline produces identical results regardless of CPU/GPU/NPU assignment.

No experiment requires proprietary data, commercial software licenses, or restricted datasets.

---

## Tier Pipeline

```
Tier 0: Python control
  └─ Published algorithm, reference NumPy implementation
  └─ Generates baseline checks (ground truth)
  └─ Every experiment starts here

Tier 1: Rust CPU
  └─ Pure Rust, f64-canonical, #![forbid(unsafe_code)]
  └─ Unit tests in barracuda/src/{module}.rs
  └─ Validation binary in experiments/exp{NNN}/
  └─ Cross-validated against Python baseline (< 1e-6 tolerance)

Tier 2: Rust GPU (barraCuda WGSL) ← LIVE
  └─ 6 WGSL shaders: Hill, PopPK, Diversity, MM batch, SCFA batch, Beat classify
  └─ GpuContext: persistent device, fused unidirectional pipeline
  └─ Math parity: 42/42 checks across all ops
  └─ Scaling: GPU crossover at 100K elements, V16 linear scaling confirmed (Exp085)
  └─ Validated: Exp053-055 (original), Exp083 (V16 parity), Exp085 (V16 scaling)

Tier 3: metalForge + toadStool dispatch ← LIVE
  └─ toadStool Pipeline::execute_cpu/gpu/streaming/auto dispatch for all StageOps
  └─ metalForge select_substrate() routes 9 Workload variants by element count
  └─ Cross-substrate: CPU ↔ GPU ↔ NPU routing
  └─ NUCLEUS atomics: Tower/Node/Nest hierarchy for mixed hardware
  └─ PCIe P2P: GPU↔NPU direct DMA, bypassing CPU roundtrip (31.5 GB/s Gen4)
  └─ plan_dispatch: 5-stage mixed pipeline (GPU→GPU→GPU→NPU→CPU) validated
  └─ biomeOS atomic graphs for node and tower deployments
  └─ Validated: Exp060-062 (original), Exp086 (V16 dispatch), Exp087 (V16 NUCLEUS)

Visualization: petalTongue V16 + compute
  └─ V16 scenario builder: 6 nodes (MM PK, antibiotic, SCFA, serotonin, EDA, arrhythmia)
  └─ Compute pipeline: GPU scaling, NUCLEUS topology, mixed dispatch
  └─ Full study: 34 nodes, 38 edges (all 7 DataChannel types)
  └─ Unified dashboard: 326 validation checks (Exp088)
  └─ Patient explorer: CLI-parameterized diagnostic + V16 + streaming (Exp089)
```

---

## barraCuda Requirements

See [BARRACUDA_REQUIREMENTS.md](BARRACUDA_REQUIREMENTS.md) for detailed primitive inventory.

---

## Cross-Spring Dependencies

| Dependency | What healthSpring uses |
|------------|----------------------|
| **wetSpring** | 16S pipeline (Track 2), Anderson lattice (Papers 01/06), Gonzales immunology (Exp273–286) |
| **neuralSpring** | Hill dose-response (nS-601), PK decay (nS-603), tissue lattice (nS-604), MATRIX (nS-605) |
| **groundSpring** | Error propagation, uncertainty quantification, spectral methods |
| **airSpring** | CytokineBrain visualization, sensor fusion patterns |
| **hotSpring** | Lattice methods (SU(3) → tissue lattice), Anderson spectral theory |
| **barraCuda** | All GPU primitives (standalone math library, v0.3.3) |
| **toadStool** | Compute pipeline dispatch (CPU/GPU/NPU routing, streaming) |
| **metalForge** | NUCLEUS topology, substrate capabilities, PCIe transfer planning |
| **petalTongue** | Universal UI — 7 DataChannel types, streaming, interaction, domain theming |
| **biomeOS** | Deployment graphs, atomic orchestration (healthspring_deploy.toml) |
