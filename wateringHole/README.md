# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V9 — 37 experiments, 4 tracks + diagnostics + GPU + dispatch + clinical TRT + SAME DAVE integration (Tier 0+1+2+3)
**Last Updated**: March 9, 2026

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
| **V9** | [V9 Clinical Translation + SAME DAVE Handoff](handoffs/HEALTHSPRING_V9_CLINICAL_SAMEDAVE_HANDOFF_MAR09_2026.md) | Mar 9 | Patient-parameterized TRT scenarios, SAME DAVE motor command channel, IPC bridge, clinical mode presets. Absorption tables for barraCuda, toadStool, petalTongue. |
| V8 | [V8 Mixed Dispatch Handoff](handoffs/HEALTHSPRING_V8_MIXED_DISPATCH_HANDOFF_MAR09_2026.md) | Mar 9 | CPU vs GPU parity, mixed NUCLEUS dispatch, PCIe P2P transfers, DispatchPlan. |
| V7.1 | [V7 Visualization Handoff](handoffs/HEALTHSPRING_V7_FULL_VISUALIZATION_BARRACUDA_TOADSTOOL_HANDOFF_MAR09_2026.md) | Mar 9 | petalTongue visualization, scenario builders, chart rendering |
| | *V1, V3, V4, V5, V6, V6.1 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math (exp, log, pow), ODE solvers, fused ops | v0.3.3 | **Live** — Tier 2+3 GPU via local shaders + GpuContext |
| **wetSpring** | 16S pipeline, Anderson lattice, Gonzales immunology | V99 | Validated (8,886 checks) |
| **neuralSpring** | Hill dose-response, PK decay, tissue lattice, MATRIX | V90 | Validated (2,279 checks) |
| **groundSpring** | Uncertainty propagation, spectral methods | V100 | Validated |
| **airSpring** | CytokineBrain, sensor fusion patterns | v0.7.5 | Validated |
| **hotSpring** | Lattice methods, Anderson spectral theory | v0.6.17+ | Validated |
| **petalTongue** | UI/visualization platform + SAME DAVE neuroanatomy | v1.3.0+ | **Absorbed** + local wiring: DataChannel, ClinicalRange, renderers, theme, clinical mode. Motor command channel + IPC bridge for runtime UI control. |

---

## Convention

Following wetSpring/hotSpring naming pattern:

`HEALTHSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: healthSpring → barraCuda (math), healthSpring → toadStool (hardware), healthSpring → petalTongue (visualization). No reverse dependencies.
