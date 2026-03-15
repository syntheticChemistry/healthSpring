# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V23 — deep debt remediation + production hardening. 435 tests, 61 experiments, 55+ wired JSON-RPC capabilities. AGPL-3.0-or-later across all files. `clippy::nursery` enforced, zero `#[allow()]`, zero TODO/FIXME, `UniBin`-compliant primal binary. barraCuda v0.3.5 pinned.
**Last Updated**: March 15, 2026

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
| **V23** | [V23 Deep Remediation Handoff](handoffs/HEALTHSPRING_V23_DEEP_REMEDIATION_BARRACUDA_TOADSTOOL_HANDOFF_MAR15_2026.md) | Mar 15 | Deep debt remediation, Tier A/B GPU rewire plan, NLME absorption candidates, quality standards. Supersedes V22 + V15 absorption. |
| **V20** | [V20 petalTongue V16 Visualization Handoff](handoffs/HEALTHSPRING_V20_PETALTONGUE_V16_VIZ_BARRACUDA_TOADSTOOL_HANDOFF_MAR10_2026.md) | Mar 10 | petalTongue V16 scenarios, compute pipeline viz, unified dashboard, patient explorer. |
| **V19** | [V19 Full-Stack Portability Handoff](handoffs/HEALTHSPRING_V19_FULLSTACK_BARRACUDA_TOADSTOOL_HANDOFF_MAR10_2026.md) | Mar 10 | GPU scaling, toadStool V16 dispatch, mixed NUCLEUS, PCIe P2P bypass. |
| | *V1–V22 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.5 (`a60819c`) | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. Tier A rewire ready (Hill, PopPK, Diversity → upstream ops). |
| **toadStool** | Compute pipeline dispatch (CPU/GPU/NPU routing, streaming, callbacks) | S142+ | **Live** — V16 StageOps dispatched via `execute_cpu`, `execute_streaming`, `execute_auto`. |
| **metalForge** | NUCLEUS topology, substrate capabilities, PCIe P2P transfer planning, `plan_dispatch` | local | **Live** — 9 Workload variants, mixed Tower/Node/Nest dispatch, PCIe Gen4 P2P (31.5 GB/s). |
| **wetSpring** | 16S pipeline, Anderson lattice, Gonzales immunology | V107 | Validated (9,060+ checks) |
| **neuralSpring** | Hill dose-response, PK decay, tissue lattice, MATRIX | V90 | Validated |
| **groundSpring** | Uncertainty propagation, spectral methods | V100 | Validated |
| **airSpring** | CytokineBrain, sensor fusion patterns | v0.7.5 | Validated |
| **hotSpring** | Lattice methods, Anderson spectral theory | v0.6.25+ | Validated |
| **petalTongue** | UI/visualization platform + SAME DAVE neuroanatomy | v1.3.0+ | **Absorbed** + local wiring |

---

## Convention

Following wetSpring/hotSpring naming pattern:

`HEALTHSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: healthSpring → barraCuda (math), healthSpring → toadStool (hardware), healthSpring → petalTongue (visualization). No reverse dependencies.
