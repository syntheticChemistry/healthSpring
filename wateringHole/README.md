# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V64k — Deep Debt Reconfirmation: all 7 audit categories at zero debt after V64j wire fixes. **902+ tests**, 18 scenarios, 88 capabilities. primalSpring **v0.9.25**. ecoBin 0.9.0. barraCuda v0.3.13. Zero clippy pedantic+nursery, zero unsafe, zero TODO.
**Last Updated**: May 13, 2026

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
| **V64k** | [Deep Debt Reconfirmation](handoffs/HEALTHSPRING_V64K_UPSTREAM_HANDOFF_MAY13_2026.md) | May 13 | All 7 audit categories at zero debt, refreshed audit answers, no regressions from V64j |
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
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.13 | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. |
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
