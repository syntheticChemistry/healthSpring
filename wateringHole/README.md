# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V61 — **UniBin** architecture (`healthspring_unibin`: `certify`, `validate`, `serve`, `status`, `version`) + IPC-first library defaults (`default = []`, **`barracuda-lib`** opt-in); primalSpring **v0.9.25**. guideStone Level 5 (57/57). **999 tests**, 95 experiments. ecoBin 0.9.0. barraCuda v0.3.13. Zero clippy, zero unsafe.
**Last Updated**: May 9, 2026

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
| **V61** | [Interstadial Eukaryotic Evolution](handoffs/HEALTHSPRING_V61_INTERSTADIAL_EUKARYOTIC_HANDOFF_MAY09_2026.md) | May 9 | UniBin (`certify`/`validate`/…); **`certification/`** organelle; **`composition/`** + **`validation/scenarios/`**; IPC-first defaults + **`barracuda-lib`** opt-in; **`fossilRecord/`**; primalSpring v0.9.25 pinned |
| **V60** | [Deep Debt Evolution](handoffs/HEALTHSPRING_V60_DEEP_DEBT_EVOLUTION_HANDOFF_MAY08_2026.md) | May 8 | Optional barracuda-lib + IPC fallbacks, exp123 NUCLEUS parity, 53 paired notebooks, validate_pk_models, gpu_parity benchmarks, dataset fetch scripts, timeout centralization, capability-first discovery, records/viz splits, exp119–122 CI. barraCuda v0.3.13. |
| | *V1–V59 → `handoffs/archive/`* | | Fossil record (includes superseded **V54** upstream evolution and **V59** deep-debt composition handoffs) |

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
