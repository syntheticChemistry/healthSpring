# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V13 — 47 experiments, 4 tracks + diagnostics + GPU + dispatch + clinical TRT + streaming + petalTongue evolution + interaction (Tier 0+1+2+3). Deep audit: Anderson eigensolver, smart refactor, math deduplication, centralized RNG, capability-based discovery, 4 doc-tests, flaky test fix.
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
| **V13** | [V13 Deep Audit Evolution Handoff](handoffs/HEALTHSPRING_V13_DEEP_AUDIT_EVOLUTION_HANDOFF_MAR09_2026.md) | Mar 9 | Anderson eigensolver (QL algorithm), smart clinical.rs refactor, LCG PRNG centralization, math deduplication, capability-based Songbird discovery, flaky IPC test fix, 4 doc-tests, tolerance registry update. Absorption tables for barraCuda (eigensolver, RNG), toadStool (RNG, streaming), petalTongue (capabilities, interaction). Evolution readiness assessment. |
| V12 | [V12 petalTongue Evolution Handoff](handoffs/HEALTHSPRING_V12_PETALTONGUE_EVOLUTION_HANDOFF_MAR09_2026.md) | Mar 9 | Full stream ops (replace), domain theming, UiConfig passthrough, capabilities query, interaction subscription, 5 TRT archetypes, PBPK tissue profiles, Pan-Tompkins intermediates, Anderson spectra. |
| | *V1, V3, V4, V5, V6, V6.1, V7, V8, V9 → `handoffs/archive/`* | | Fossil record |

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
