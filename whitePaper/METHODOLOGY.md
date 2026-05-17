# healthSpring Validation Methodology

**Version**: 0.6
**Date**: May 17, 2026 (V64x)

---

## Overview

healthSpring validates health-of-living-systems applications using a six-level constrained evolution ladder: Python baseline → Rust CPU → barraCuda CPU → barraCuda GPU → guideStone/UniBin → NUCLEUS deployment. Each level uses the previous as its validation target, producing documented math parity at every transition.

The key difference from other springs: healthSpring validates **applications**, not just algorithms. A PK model must not only reproduce published curves — it must produce clinically interpretable outputs (AUC, Cmax, trough levels) with documented uncertainty bounds. A colonization resistance score must correlate with clinical outcomes, not just match a lattice simulation.

As of V64l, healthSpring also serves as the **Nest Atomic Specialist** — proving the NUCLEUS Nest Atomic composition (data lineage and provenance) end-to-end through clinical data pipelines via a 9-phase validation scenario.

---

## Level 1: Python Baseline

Reference implementations from published papers, implemented in standard scientific Python (NumPy, SciPy, scikit-learn). These establish the ground truth.

**Acceptance criteria**:
- Reproduces published figures, tables, or numerical results
- Documented random seeds where applicable
- Pinned dependency versions in `requirements.txt`
- All control scripts in `control/` with `run_all.sh`

## Level 2: Rust CPU

Pure Rust reimplementation using `healthspring-barracuda` crate. f64 throughout (no f32 shortcuts). All tolerances named and documented.

**Acceptance criteria**:
- Matches Python control within named tolerance
- `#![forbid(unsafe_code)]` in all modules
- `clippy::pedantic` + `clippy::nursery` clean
- Every tolerance has a constant name and rationale

## Level 3: barraCuda CPU

Same algorithm dispatched through barraCuda's WGSL shader pipeline running on CPU fallback. Validates that the primal math library produces identical results.

**Acceptance criteria**:
- Matches Rust CPU within documented tolerance
- Zero local WGSL shaders (all consumed from standalone `barraCuda`)

## Level 4: barraCuda GPU

WGSL shader execution on actual GPU hardware. Vendor-agnostic (NVIDIA, AMD, Intel, Apple via wgpu).

**Acceptance criteria**:
- Matches barraCuda CPU within documented GPU tolerance (DF64 precision bounds)
- Feature-gated (`--features gpu`)

## Level 5: guideStone / UniBin

Self-validating binary (`healthspring_unibin certify`) proves bare properties 1–5 (Deterministic, Traceable, Self-Verifying via BLAKE3, Env-Agnostic, Tolerance-Documented). When NUCLEUS is deployed, IPC parity is validated via `primalspring::composition`. The `healthspring_primal` binary exposes 88 capabilities via JSON-RPC 2.0 over Unix sockets.

**Acceptance criteria**:
- `healthspring_unibin certify` passes all self-checks
- IPC dispatch (`PrimalClient.call()`) matches direct Rust call within `DETERMINISM` tolerance
- `--format json` output for CI/projectNUCLEUS ingestion
- 17 validation scenarios across 8 tracks pass

## Level 6: NUCLEUS Deployment

plasmidBin cellular deployment via `healthspring_cell.toml`. Full Nest Atomic provenance pipeline: NestGate CAS → rhizoCrypt DAG → loamSpine ledger → sweetGrass attribution. Deploy graphs compose Tower, Node, Nest, and Meta primals.

**Acceptance criteria**:
- `s_nest_atomic` 9-phase scenario passes against live primals
- `healthspring_cell.toml` accepted by plasmidBin
- Wire contracts match upstream canonical names (BearDog: base64 `message`; skunkBat: `security.audit_log`; loamSpine: `spine.create`/`entry.append`)
- Provenance trio round-trip produces valid Merkle root, ledger entry, and braid attribution

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
- Has a Rust validation binary in `experiments/expNNN_<name>/`
- Produces output to `sandbox/` (gitignored)
- Is documented in `whitePaper/experiments/README.md` and `whitePaper/baseCamp/`

---

## Nest Atomic Provenance Pipeline (V64l)

The Nest Atomic validation scenario (`s_nest_atomic`) exercises the full provenance chain in 9 phases:

```
Phase 1: NestGate CAS — store content, get content hash
Phase 2: rhizoCrypt DAG — create DAG node referencing content hash
Phase 3: loamSpine ledger — spine.create + entry.append with Merkle root
Phase 4: sweetGrass attribution — braid.create with semantic metadata
Phase 5: BearDog crypto — crypto.sign with base64-encoded Merkle root
Phase 6: Cross-atomic references — verify chain integrity across primals
Phase 7: sweetGrass analytics — query attribution for experiment metadata
Phase 8: Tower auxiliary — security.audit_log via skunkBat
Phase 9: End-to-end summary — verify all phases connected
```

The `NestComposition` facade (`ipc/provenance/nest.rs`) orchestrates this pipeline for production use. The validation scenario runs against live primals when available and skips gracefully when offline.

---

## Clinical Validation Note

healthSpring performs **computational validation** — proving that algorithms reproduce published results and scale to GPU. Clinical validation (does this tool improve patient outcomes?) requires prospective studies, IRB approval, and institutional partnerships. healthSpring provides the computational foundation; clinical validation is a separate, future phase that requires wet-lab and clinical collaborators.
