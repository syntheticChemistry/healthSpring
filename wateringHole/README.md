# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V64o — Wave 17 Signal Adoption: `primal.announce` registration, `nest.store`/`nest.commit` dispatch, 451-method registry sync. **1,018 tests** (workspace), 17 scenarios, 88 capabilities. primalSpring **v0.9.25**. ecoBin 0.9.0. barraCuda v0.4.0. Zero clippy, zero unsafe, zero TODO.
**Last Updated**: May 16, 2026

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
| **V64r** | [Wave 20 Schema Standard](handoffs/HEALTHSPRING_V64R_WAVE20_SCHEMA_STANDARD_MAY16_2026.md) | May 16 | `capability.list` canonical envelope (capabilities + count), 452-method registry sync (primal.list), nest.commit signal-path confirmed |
| V64o | [Wave 17 Signal Adoption](handoffs/HEALTHSPRING_V64O_WAVE17_SIGNAL_ADOPTION_MAY16_2026.md) | May 16 | `primal.announce` registration, `nest.store`/`nest.commit` signal dispatch in NestComposition + data/provenance, 451-method registry sync, routing/niche domain expansion, GAP-GS-015 confirmed, Foundation Threads 3+8, GAPs 46-47 |
| V64n | [Upstream Audit Absorption](handoffs/HEALTHSPRING_V64N_UPSTREAM_HANDOFF_MAY14_2026.md) | May 14 | Tower = bearDog + songBird + skunkBat in all graphs, deploy graph canonicalization, routing `content` domain, capability registry sync, barraCuda v0.4.0, GAPs 43-45 |
| V64m | [Comprehensive Handoff](handoffs/HEALTHSPRING_V64M_COMPREHENSIVE_HANDOFF_MAY13_2026.md) | May 13 | All wire contract learnings, Nest Atomic composition pattern, provenance pipeline, plasmidBin cell.toml, Foundation Thread 10, recommendations for upstream teams |
| V64l | [Wire Hygiene](handoffs/HEALTHSPRING_V64L_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | bearDog base64 `message`, skunkBat `security.audit_log`, plasmidBin cell.toml, Foundation Thread 10 gap |
| V64k | [Deep Debt Reconfirmation](handoffs/HEALTHSPRING_V64K_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | All 7 audit categories at zero debt, refreshed audit answers, no regressions from V64j |
| V64j | [Delta Spring Evolution](handoffs/HEALTHSPRING_V64J_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | GAP-36 resolved, provenance trio wire names canonical, 5 gaps closed (#23, #32, #34, #35, #36), Nest Atomic live-ready |
| V64i | [Deep Debt Resolution](handoffs/HEALTHSPRING_V64I_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | Full 7-category audit at zero debt, clippy pedantic+nursery clean, hardcoding eliminated, audit answers, gaps #38-41 |
| V64h | [Nest Atomic Validation Sprint](handoffs/HEALTHSPRING_V64H_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | Nest Atomic (neutron) 9-phase validation, 7-node deploy graph, NestComposition domain fix, gaps #34-37 |
| V64g | [Provenance Elevation Handoff](handoffs/HEALTHSPRING_V64G_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | Auditable data chains: expected_values.json + tolerances.toml for 7 tracks, unified IPC wire shape, NestComposition facade, 30+ DOIs, gaps #32-33 |
| V64f | [Tier 2 Convergence Handoff](handoffs/HEALTHSPRING_V64F_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | Tier 2 wired, precision mapping, --list, LTEE B5 lithoSpore module, plasmidBin |
| | *V1–V64 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.4.0 | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. |
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

Handoffs flow: healthSpring → barraCuda (math), healthSpring → toadStool (hardware), healthSpring → petalTongue (visualization), healthSpring → primalSpring (composition), healthSpring → all springs (patterns). No reverse dependencies.
