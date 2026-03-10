# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V15 — 48 experiments, 5 tracks + diagnostics + GPU + dispatch + clinical TRT + streaming + petalTongue evolution + interaction + NLME (Tier 0+1+2+3). V15: upstream rewire — precision routing (metalForge `PrecisionRouting` mirroring toadStool S128), upstream barraCuda dependency wired (`upstream-ops` feature), WGSL shader provenance documented (local → barraCuda canonical), handoff catch-up to ecosystem state (toadStool S142, barraCuda absorption sprint, coralReef Phase 10).
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
| **V15** | [V15 Upstream Rewire Handoff](handoffs/HEALTHSPRING_V15_UPSTREAM_REWIRE_HANDOFF_MAR10_2026.md) | Mar 10 | Upstream rewire: precision routing (`PrecisionRouting` enum in metalForge), upstream `barracuda` dependency (`upstream-ops` feature), WGSL shader provenance mapping, ecosystem catch-up (toadStool S142, barraCuda absorption sprint, coralReef Phase 10). |
| | *V1–V14.1 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy` | v0.3.3+ | **Live** — Tier 2+3 GPU via local shaders + `GpuContext`. Upstream has absorbed Hill, PopPK, diversity, PRNG, eigensolver. `upstream-ops` feature wired for canonical consumption. |
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
