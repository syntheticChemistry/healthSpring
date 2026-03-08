# healthSpring Validation Methodology

**Version**: 0.1
**Date**: March 8, 2026

---

## Overview

healthSpring validates human health applications using the same constrained evolution protocol as all ecoPrimals springs: Python control → Rust CPU → Rust GPU → metalForge dispatch. Each tier demonstrates that the Pure Rust implementation faithfully reproduces the reference algorithm, then promotes it to faster substrates with documented math parity.

The key difference from other springs: healthSpring validates **applications**, not just algorithms. A PK model must not only reproduce published curves — it must produce clinically interpretable outputs (AUC, Cmax, trough levels) with documented uncertainty bounds. A colonization resistance score must correlate with clinical outcomes, not just match a lattice simulation.

---

## Tier 0: Python Control

Reference implementations from published papers, implemented in standard scientific Python (NumPy, SciPy, scikit-learn). These establish the ground truth.

**Acceptance criteria**:
- Reproduces published figures, tables, or numerical results
- Documented random seeds where applicable
- Pinned dependency versions in `requirements.txt`
- All control scripts in `control/` with `run_all.sh`

## Tier 1: Rust CPU

Pure Rust reimplementation using `healthspring-barracuda` crate. f64 throughout (no f32 shortcuts). All tolerances named and documented.

**Acceptance criteria**:
- Matches Python control within named tolerance
- `#![forbid(unsafe_code)]` in all modules
- `clippy::pedantic` clean
- Every tolerance has a constant name and rationale

## Tier 2: Rust GPU

BarraCUDA WGSL shader promotion. Same algorithm, GPU-parallel execution.

**Acceptance criteria**:
- Matches Rust CPU within documented GPU tolerance (DF64 precision bounds)
- Zero local WGSL shaders (all consumed from standalone `barraCuda`)
- Feature-gated (`--features gpu`)

## Tier 3: metalForge Dispatch

ToadStool cross-substrate routing. CPU ↔ GPU ↔ NPU dispatch validated.

**Acceptance criteria**:
- Matches Tier 2 results via ToadStool `ComputeDispatch`
- Routing decisions logged and reproducible
- Feature-gated (`--features metalforge`)

---

## Tolerance Documentation

Every numerical comparison uses a named tolerance constant:

```rust
const HILL_IC50_TOL: f64 = 1e-6;       // Hill equation IC50 fit
const PK_CONC_TOL: f64 = 1e-4;         // Concentration-time curve
const PK_AUC_TOL: f64 = 0.01;          // AUC (1% relative)
const DIVERSITY_SHANNON_TOL: f64 = 1e-8; // Shannon diversity index
const ANDERSON_XI_TOL: f64 = 0.05;     // Localization length (5% relative)
const ECG_RPEAK_TOL_MS: f64 = 5.0;     // R-peak detection (5ms)
const SPO2_TOL_PCT: f64 = 2.0;         // SpO2 estimation (2% clinical)
```

---

## Experiment Numbering

Experiments are numbered sequentially: `Exp001`, `Exp002`, etc. Each experiment:
- Has a Python control in `control/<domain>/`
- Has a Rust validation binary in `barracuda/`
- Has results in `experiments/results/`
- Is documented in `whitePaper/experiments/NNN_<name>.md`

---

## Clinical Validation Note

healthSpring performs **computational validation** — proving that algorithms reproduce published results and scale to GPU. Clinical validation (does this tool improve patient outcomes?) requires prospective studies, IRB approval, and institutional partnerships. healthSpring provides the computational foundation; clinical validation is a separate, future phase that requires wet-lab and clinical collaborators.
