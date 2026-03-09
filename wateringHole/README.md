# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V7 — 31 experiments, 4 tracks + diagnostics + GPU pipeline + visualization (Tier 0+1+2)
**Last Updated**: March 9, 2026

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
| **V7** | [V7 Full Visualization + barraCuda Handoff](handoffs/HEALTHSPRING_V7_FULL_VISUALIZATION_BARRACUDA_TOADSTOOL_HANDOFF_MAR09_2026.md) | Mar 9 | Per-track scenario builders (22 nodes, 62 data channels, 13 clinical ranges). Exp056 validates 47 checks. barraCuda/toadStool evolution handoff. |
| | *V1, V3, V4, V5, V6, V6.1 → `handoffs/archive/`* | | Fossil record |

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
