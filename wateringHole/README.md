# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V6 — 30 experiments, 4 tracks + diagnostics + petalTongue + GPU pipeline (Tier 0+1+2)
**Last Updated**: March 8, 2026

---

## Purpose

The wateringHole is where springs coordinate. Handoff documents record:
- What healthSpring needs from upstream primals (barraCuda primitives, toadStool dispatch)
- What healthSpring contributes back (health-specific primitives for absorption)
- Cross-spring shader evolution status
- Evolution guidance for the barraCuda/toadStool team

---

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V6.1** | [V6.1 petalTongue Absorption Complete](handoffs/HEALTHSPRING_V6.1_PETALTONGUE_LEAN_HANDOFF_MAR09_2026.md) | Mar 9 | petalTongue absorbed DataChannel, ClinicalRange, renderers, clinical theme. healthSpring petaltongue-health removed (lean phase). ClinicalRange.status aligned to String. |
| **V6** | [V6 GPU Pipeline](handoffs/HEALTHSPRING_V6_GPU_PIPELINE_BARRACUDA_TOADSTOOL_HANDOFF_MAR08_2026.md) | Mar 8 | GPU Tier 2 live: 3 WGSL shaders, GpuContext, fused pipeline, toadStool GPU dispatch, scaling to 10M. |
| | *V1, V3, V4, V5 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math (exp, log, pow), ODE solvers, fused ops | v0.3.3 | **Live** — Tier 2 GPU via local shaders + GpuContext |
| **wetSpring** | 16S pipeline, Anderson lattice, Gonzales immunology | V99 | Validated (8,886 checks) |
| **neuralSpring** | Hill dose-response, PK decay, tissue lattice, MATRIX | V90 | Validated (2,279 checks) |
| **groundSpring** | Uncertainty propagation, spectral methods | V100 | Validated |
| **airSpring** | CytokineBrain, sensor fusion patterns | v0.7.5 | Validated |
| **hotSpring** | Lattice methods, Anderson spectral theory | v0.6.17+ | Validated |
| **petalTongue** | UI/visualization platform | v1.3.0 | **Absorbed**: DataChannel, ClinicalRange, renderers, theme. healthSpring leaned. |

---

## Convention

Following wetSpring/hotSpring naming pattern:

`HEALTHSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: healthSpring → barraCuda (math) and healthSpring → toadStool (hardware). No reverse dependencies.
