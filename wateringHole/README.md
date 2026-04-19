# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V56 — guideStone Level 4 (NUCLEUS validated, 49/49 live). Live IPC parity against barraCuda RTX 3070. BLAKE3 CHECKSUMS (17 files). primalSpring v0.9.16. 948+ tests, 94 experiments. Three-tier harness: Tier 1 local, Tier 2 IPC-wired (barraCuda live), Tier 3 primal proof. Family-aware discovery. Protocol tolerance. barraCuda v0.3.12. ecoBin 0.9.0. Zero clippy, zero unsafe.
**Last Updated**: April 19, 2026

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
| **V56** | [NUCLEUS Validated — Upstream Handoff](handoffs/HEALTHSPRING_V56_NUCLEUS_VALIDATED_HANDOFF_APR19_2026.md) | Apr 19 | guideStone Level 4: 49/49 live against barraCuda RTX 3070. Live IPC parity. BLAKE3 CHECKSUMS. Gap 19 (stats.variance/correlation). Composition patterns for all springs. |
| **V55** | [Primal Proof Harness](handoffs/HEALTHSPRING_V55_PRIMAL_PROOF_HARNESS_HANDOFF_APR20_2026.md) | Apr 20 | Three-tier primal proof harness (Tier 1 local, Tier 2 IPC-wired, Tier 3 NUCLEUS). P3 BLAKE3 checksums. Protocol tolerance. Family-aware discovery. primalSpring v0.9.16. |
| **V54** | [guideStone Level 2](handoffs/HEALTHSPRING_V54_GUIDESTONE_HANDOFF_APR18_2026.md) | Apr 18 | `healthspring_guidestone` binary. Bare properties 1–5. NUCLEUS IPC parity via `primalspring::composition`. `math_dispatch` reframed as validation window. V53 "9 wire handlers" ask withdrawn. |
| **V54** | [Upstream Evolution](handoffs/HEALTHSPRING_V54_UPSTREAM_EVOLUTION_HANDOFF_APR18_2026.md) | Apr 18 | Handoff for primalSpring, barraCuda, toadStool, metalForge, biomeOS, all springs. guideStone integration report, corrected IPC framing, dual-tower ionic bridge pattern, composition patterns for NUCLEUS deployment. |
| **V53** | [Composition Parity](handoffs/HEALTHSPRING_V53_COMPOSITION_PARITY_HANDOFF_APR17_2026.md) | Apr 17 | Level 5 primal proof: `math_dispatch` + `BarraCudaClient` + `primal-proof` feature + exp122. Live IPC parity (exp119–121). ecoBin 0.9.0. |
| | *V1–V52 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.12 | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. |
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
