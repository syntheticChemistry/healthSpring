# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V52 — Composition Validation. 985+ tests, 90 experiments (84 science + 7 composition Tier 4/5), 90 provenance entries (100% coverage). Typed IPC clients wired (PrimalClient resilient default), Tier 5 deploy graph validation (exp118, 99 checks), GPU tests on every PR, expanded coverage scope. barraCuda v0.3.11 (7f6649f). ecoBin 0.8.0. Zero clippy, zero unsafe.
**Last Updated**: April 11, 2026

---

## Purpose

The wateringHole is where springs coordinate. Handoff documents record:
- What healthSpring needs from upstream primals (barraCuda primitives, toadStool dispatch)
- What healthSpring contributes back (health-specific primitives for absorption)
- Cross-spring shader evolution status
- Evolution guidance for the barraCuda/toadStool team
- Composition validation patterns for NUCLEUS deployment via biomeOS

---

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| **V52** | [Composition Validation](handoffs/HEALTHSPRING_V52_COMPOSITION_VALIDATION_HANDOFF_APR11_2026.md) | Apr 11 | Tier 5 deploy graph validation (exp118), typed IPC clients wired, GPU CI on every PR, expanded coverage, tolerances.py policy fix. |
| **V52** | [Primal Evolution](handoffs/HEALTHSPRING_V52_PRIMAL_EVOLUTION_HANDOFF_APR11_2026.md) | Apr 11 | Primal usage inventory, composition learnings, NUCLEUS deployment patterns, evolution opportunities for all teams. |
| **V51** | [Hardened Composition](handoffs/HEALTHSPRING_V51_HARDENED_COMPOSITION_HANDOFF_APR11_2026.md) | Apr 11 | TCP listener, BTSP, typed clients, structured discovery, `identity.get`, `health.check`, LOCAL/ROUTED split, domain symlink. Cross-ecosystem handoff. |
| **V50** | [Composition Evolution](handoffs/HEALTHSPRING_V50_COMPOSITION_EVOLUTION_HANDOFF_APR11_2026.md) | Apr 11 | Capability-first routing, Squirrel optional node, discovery dual fallback, provenance split. |
| | *V1–V49 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.11 | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. |
| **toadStool** | Compute pipeline dispatch (CPU/GPU/NPU routing, streaming, callbacks) | S142+ | **Live** — V16 StageOps dispatched via `execute_cpu`, `execute_streaming`, `execute_auto`. |
| **metalForge** | NUCLEUS topology, substrate capabilities, PCIe P2P transfer planning, `plan_dispatch` | local | **Live** — 9 Workload variants, mixed Tower/Node/Nest dispatch. |
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

Handoffs flow: healthSpring → barraCuda (math), healthSpring → toadStool (hardware), healthSpring → petalTongue (visualization), healthSpring → primalSpring (composition). No reverse dependencies.
