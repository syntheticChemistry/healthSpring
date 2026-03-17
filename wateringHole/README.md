# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V33 — Protocol Evolution + Centralized Cast Algebra. 635 tests, 73 experiments, 42 baselines with provenance, 113/113 cross-validation. IpcError::is_recoverable(), DispatchOutcome enum, generic discovery helpers, centralized cast module.
**Last Updated**: March 16, 2026

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
| **V33** | [V33 Protocol Evolution + Cast Algebra](handoffs/HEALTHSPRING_V33_PROTOCOL_EVOLUTION_CAST_ALGEBRA_HANDOFF_MAR16_2026.md) | Mar 16 | IpcError::is_recoverable(), DispatchOutcome enum, generic discovery helpers, centralized cast module. Supersedes V32. |
| **V33** | [V33 toadStool/barraCuda Absorption](handoffs/HEALTHSPRING_V33_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR16_2026.md) | Mar 16 | 3 Tier B shaders for barraCuda, 12+ module consumption map, GPU learnings, IPC patterns, action items for barraCuda/toadStool/coralReef. |
| | *V1–V32 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.5 (`a60819c`) | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. Tier A rewire ready (Hill, PopPK, Diversity → upstream ops). |
| **toadStool** | Compute pipeline dispatch (CPU/GPU/NPU routing, streaming, callbacks) | S142+ | **Live** — V16 StageOps dispatched via `execute_cpu`, `execute_streaming`, `execute_auto`. |
| **metalForge** | NUCLEUS topology, substrate capabilities, PCIe P2P transfer planning, `plan_dispatch` | local | **Live** — 9 Workload variants, mixed Tower/Node/Nest dispatch, PCIe Gen4 P2P (31.5 GB/s). |
| **wetSpring** | 16S pipeline, Anderson lattice, Gonzales immunology, OrExit pattern | V123 | Validated (1,703 tests, 376 experiments) |
| **neuralSpring** | Hill dose-response, PK decay, tissue lattice, MATRIX, dual-format caps | S157 | Validated (1,115+ tests) |
| **groundSpring** | Uncertainty propagation, spectral methods, zero-panic pattern | V109 | Validated (912+ tests) |
| **airSpring** | CytokineBrain, sensor fusion patterns, deny.toml | v0.8.4 | Validated |
| **hotSpring** | Lattice methods, Anderson spectral theory, GlowPlug boot | v0.6.31 | Validated (848 tests) |
| **ludoSpring** | Session decomposition, typed transitions, dispatch client | V22 | Validated (394 tests) |
| **petalTongue** | UI/visualization platform + SAME DAVE neuroanatomy | v1.3.0+ | **Absorbed** + local wiring |

---

## Convention

Following wetSpring/hotSpring naming pattern:

`HEALTHSPRING_{VERSION}_{TOPIC}_HANDOFF_{DATE}.md`

Handoffs flow: healthSpring → barraCuda (math), healthSpring → toadStool (hardware), healthSpring → petalTongue (visualization). No reverse dependencies.
