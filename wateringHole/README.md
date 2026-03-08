# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V3 — 17 experiments, 4 tracks complete (Tier 0+1)
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
| **V3** | [HEALTHSPRING_V3_BARRACUDA_TOADSTOOL_EVOLUTION_HANDOFF_MAR08_2026.md](handoffs/HEALTHSPRING_V3_BARRACUDA_TOADSTOOL_EVOLUTION_HANDOFF_MAR08_2026.md) | Mar 8 | Full V3 handoff: 17 experiments across 4 tracks (192 Python, 179 binary, 103 lib). GPU workload candidates, absorption candidates, cross-track testosterone-gut axis discovery, evolution guidance for barraCuda + toadStool. |
| | *V1 → `handoffs/archive/`* | | Fossil record: initial 5-experiment PK/PD handoff |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math (exp, log, pow), ODE solvers, fused ops | v0.3.3 | Available — not yet consumed (Tier 2 pending) |
| **wetSpring** | 16S pipeline, Anderson lattice, Gonzales immunology | V99 | Validated (8,886 checks) |
| **neuralSpring** | Hill dose-response, PK decay, tissue lattice, MATRIX | V90 | Validated (2,279 checks) |
| **groundSpring** | Uncertainty propagation, spectral methods | V100 | Validated |
| **airSpring** | CytokineBrain, sensor fusion patterns | v0.7.5 | Validated |
| **hotSpring** | Lattice methods, Anderson spectral theory | v0.6.17+ | Validated |

---

## Convention

Following wetSpring/hotSpring naming pattern:

`HEALTHSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: healthSpring → barraCuda (math) and healthSpring → toadStool (hardware). No reverse dependencies.
