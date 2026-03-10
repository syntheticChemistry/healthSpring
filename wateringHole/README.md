# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V14.1 — 48 experiments, 5 tracks + diagnostics + GPU + dispatch + clinical TRT + streaming + petalTongue evolution + interaction + NLME (Tier 0+1+2+3). NLME population PK (FOCE + SAEM), NCA, diagnostics, WFDB, Kokkos benchmarks, full pipeline (28 nodes, 121 channels). V14.1 deep debt: biosignal modular refactor, `#![deny(clippy::pedantic)]`, DFT deduplication.
**Last Updated**: March 10, 2026

---

## Purpose

The wateringHole is where springs coordinate. Handoff documents record:
- What healthSpring needs from upstream primals (barraCuda primitives, toadStool dispatch)
- What healthSpring contributes back (health-specific primitives for absorption)
- Cross-spring shader evolution status
- Evolution guidance for the barraCuda/toadStool team
- Per-person translation pipeline: how validated science becomes individual patient insight

---

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V14.1** | [V14.1 Deep Debt + Absorption Handoff](handoffs/HEALTHSPRING_V14.1_DEEP_DEBT_ABSORPTION_HANDOFF_MAR10_2026.md) | Mar 10 | biosignal modular refactor (953→6 submodules), `#![deny(clippy::pedantic)]` in all lib crates, DFT deduplication, idiomatic Rust patterns, provenance fixes. Full absorption guidance for barraCuda/toadStool. |
| V14 | [V14 NLME + Full Pipeline Handoff](handoffs/HEALTHSPRING_V14_NLME_FULL_PIPELINE_HANDOFF_MAR10_2026.md) | Mar 10 | NLME population PK (FOCE + SAEM), NCA, CWRES/VPC/GOF diagnostics, WFDB parser, Kokkos benchmarks, full petalTongue pipeline (28 nodes, 121 channels), industry benchmark mapping (NONMEM, Monolix, WinNonlin). |
| | *V1–V13 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math (exp, log, pow), ODE solvers, fused ops | v0.3.3 | **Live** — Tier 2+3 GPU via local shaders + GpuContext |
| **wetSpring** | 16S pipeline, Anderson lattice, Gonzales immunology | V101 | Validated (9,060+ checks) |
| **neuralSpring** | Hill dose-response, PK decay, tissue lattice, MATRIX | V90 | Validated |
| **groundSpring** | Uncertainty propagation, spectral methods | V100 | Validated |
| **airSpring** | CytokineBrain, sensor fusion patterns | v0.7.5 | Validated |
| **hotSpring** | Lattice methods, Anderson spectral theory | v0.6.17+ | Validated |
| **petalTongue** | UI/visualization platform + SAME DAVE neuroanatomy | v1.3.0+ | **Absorbed** + local wiring: DataChannel, ClinicalRange, renderers, theme, clinical mode. Motor command channel + IPC bridge for runtime UI control. |

---

## Convention

Following wetSpring/hotSpring naming pattern:

`HEALTHSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: healthSpring → barraCuda (math), healthSpring → toadStool (hardware), healthSpring → petalTongue (visualization). No reverse dependencies.
