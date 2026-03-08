# healthSpring Specifications

**Last Updated**: March 8, 2026
**Status**: V3 — 17 experiments complete, 103 Rust tests, 192 Python checks, 179 binary checks
**Domain**: Human health applications — PK/PD, gut microbiome, biosignal, endocrinology

---

## Quick Status

| Metric | Value |
|--------|-------|
| Rust lib tests | 103 |
| Python control checks | 192 |
| Rust binary checks | 179 |
| Experiments (Tier 0+1) | 17 complete |
| GPU validation (Tier 2) | — (Write phase next) |
| metalForge validation (Tier 3) | — |
| Paper queue | 17/28 complete |
| Faculty | Gonzales (MSU), Lisabeth (ADDRC), Neubig (Drug Discovery), Mok (Allure Medical) |

---

## Paper Review Queue

Papers queued for reproduction and extension. Organized by track. See [PAPER_REVIEW_QUEUE.md](PAPER_REVIEW_QUEUE.md) for the detailed queue with per-paper status.

### Track Summary

| Track | Domain | Experiments | Python checks | Rust binary checks | Rust lib tests | Status |
|-------|--------|:-----------:|:------------:|:------------------:|:--------------:|--------|
| 1 | PK/PD | 5 (Exp001-005) | 55 | 43 | 39 | **Complete** |
| 2 | Microbiome | 3 (Exp010-012) | 33 | 27 | 12 | **Complete** |
| 3 | Biosignal | 1 (Exp020) | 11 | 10 | 5 | **Complete** |
| 4 | Endocrinology | 8 (Exp030-037) | 93 | 99 | 47 | **Complete** |
| **Total** | | **17** | **192** | **179** | **103** | **All green** |

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

Tier 2: Rust GPU (barraCuda WGSL)
  └─ Vectorized shaders for parallel workloads
  └─ Math parity with Tier 1 (documented tolerance per primitive)
  └─ First target: Exp005/036 (population PK Monte Carlo)

Tier 3: metalForge (toadStool dispatch)
  └─ Cross-substrate: CPU ↔ GPU ↔ NPU routing
  └─ Same model, different hardware
  └─ Target: clinical PK on desktop/server/wearable
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
