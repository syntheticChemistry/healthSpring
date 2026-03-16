# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V30 — Cross-Spring Absorption + Zero-Panic Evolution. 611 tests, 73 experiments, 42 baselines with provenance, 113/113 cross-validation. Dual-format capability parsing, zero-panic validation, compute_dispatch client, barracuda::health delegation, deny.toml.
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
| **V30** | [V30 Cross-Spring Absorption Handoff](handoffs/HEALTHSPRING_V30_CROSS_SPRING_ABSORPTION_ZERO_PANIC_HANDOFF_MAR16_2026.md) | Mar 16 | Dual-format capability parsing, zero-panic validation, compute_dispatch client, barracuda::health delegation, deny.toml. Supersedes V29. |
| | *V1–V29 → `handoffs/archive/`* | | Fossil record |

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
