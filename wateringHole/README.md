# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V60 — Deep debt evolution complete (optional barracuda-lib, NUCLEUS exp123 parity, notebooks, validate_pk_models, gpu benchmarks). guideStone Level 5 (57/57). 1,002 tests, 95 experiments. ecoBin 0.9.0. barraCuda v0.3.13. Zero clippy, zero unsafe.
**Last Updated**: May 8, 2026

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
| **V60** | [Deep Debt Evolution](handoffs/HEALTHSPRING_V60_DEEP_DEBT_EVOLUTION_HANDOFF_MAY08_2026.md) | May 8 | Optional barracuda-lib + IPC fallbacks, exp123 NUCLEUS parity, 53 paired notebooks, validate_pk_models, gpu_parity benchmarks, dataset fetch scripts, timeout centralization, capability-first discovery, records/viz splits, exp119–122 CI. barraCuda v0.3.13. |
| **V59** | [Deep Debt + Composition Evolution](handoffs/HEALTHSPRING_V59_DEEP_DEBT_COMPOSITION_HANDOFF_APR27_2026.md) | Apr 27 | Deep debt resolved: typed enums, clone reduction, capability-first routing. Phase 46 NUCLEUS composition (18/24). Primal evolution patterns for upstream absorption. 27 gaps documented. |
| **V54** | [Upstream Evolution](handoffs/HEALTHSPRING_V54_UPSTREAM_EVOLUTION_HANDOFF_APR18_2026.md) | Apr 18 | Handoff for primalSpring, barraCuda, toadStool, metalForge, biomeOS, all springs. guideStone integration report, corrected IPC framing, dual-tower ionic bridge pattern, composition patterns for NUCLEUS deployment. |
| | *V1–V52, V53–V58 → `handoffs/archive/`* | | Fossil record |

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
