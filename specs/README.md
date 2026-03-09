# healthSpring Specifications

**Last Updated**: March 9, 2026
**Status**: V7 — 31 experiments, 201 Rust tests, 418 binary checks, 104 cross-validation checks. GPU Tier 2 live. Full petalTongue visualization: 4 per-track scenario builders, 22 nodes, 62 data channels.
**Domain**: Human health applications — PK/PD, gut microbiome, biosignal, endocrinology

---

## Quick Status

| Metric | Value |
|--------|-------|
| Rust lib tests | 201 (161 barraCuda + 27 forge + 13 toadStool) |
| Python control checks | 104 (cross-validation) |
| Rust binary checks | 418 (371 + 47 scenario) |
| Experiments | 31 (24 Tier 0+1 + 3 diagnostic + 3 GPU + 1 visualization) |
| GPU validation (Tier 2) | **Live** — 3 WGSL shaders, fused pipeline, 17/17 parity |
| metalForge validation (Tier 3) | 27 tests (substrate routing) |
| Paper queue | 24/30 complete |
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
| **Total** | | **30** | **289** | **346+** | **200** | **All green** |

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
  └─ 3 WGSL shaders: hill_dose_response_f64, population_pk_f64, diversity_f64
  └─ GpuContext: persistent device, fused unidirectional pipeline
  └─ Math parity: max_rel < 1e-4 (f32 transcendental path), 17/17 checks
  └─ Scaling: GPU crossover at 100K elements, peak 207 M/s (RTX 4070)
  └─ Validated: Exp053 (parity), Exp054 (fused pipeline), Exp055 (scaling)

Tier 3: metalForge + toadStool dispatch ← IN PROGRESS
  └─ toadStool Pipeline::execute_gpu() dispatches via GpuContext
  └─ metalForge select_substrate() routes by element count
  └─ Cross-substrate: CPU ↔ GPU ↔ NPU routing
  └─ Target: single GPU in the field (Pi + eGPU), future TPU/NPU
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
| **barraCuda** | All GPU primitives (standalone math library) |
