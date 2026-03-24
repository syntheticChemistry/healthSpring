# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V44 — Cross-Spring Absorption. 928 tests, 83 experiments, 83/83 ValidationHarness, 87+ named tolerances, 54 provenance records with verified check counts + DOI baseline sources. 59 capabilities (46 science + 13 infra). 9 science tracks. Self-knowledge compliance; simulation/validation refactoring. barraCuda v0.3.7 (CI-gated). Zero clippy (pedantic+nursery), zero unsafe, zero `#[allow]`, zero `#[expect]` in production library code.
**Last Updated**: March 24, 2026

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
| **V44** | [toadStool/barraCuda Absorption](handoffs/HEALTHSPRING_V44_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR24_2026.md) | Mar 24 | Cross-spring absorption, self-knowledge compliance, simulation/validation refactoring; 928 tests. |
| **V44** | [Primal & Spring Evolution](handoffs/HEALTHSPRING_V44_PRIMAL_SPRING_EVOLUTION_HANDOFF_MAR24_2026.md) | Mar 24 | Cast module absorption request, upstream contract tolerances, 7-primal discovery, patterns for all teams. |
| **V42** | [Comprehensive Audit](handoffs/archive/HEALTHSPRING_V42_COMPREHENSIVE_AUDIT_HANDOFF_MAR24_2026.md) | Mar 24 | Full-stack audit + 13 remediation actions. Supersedes all prior V42 handoffs. |
| **V42** | [toadStool/barraCuda Absorption](handoffs/archive/HEALTHSPRING_V42_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR24_2026.md) | Mar 24 | 6 shader absorption requests, TensorSession P0, ODE codegen registry, sovereign dispatch roadmap. |
| | *V1–V42 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.7 (`c04d848`) | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. All 6 ops rewired (Hill, PopPK, Diversity, MM, SCFA, Beat). |
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
