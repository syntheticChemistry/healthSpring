# healthSpring wateringHole

Cross-spring handoff documents and evolution coordination.

**Status**: V20 — 61 experiments, full-stack portability validated (barraCuda CPU → GPU → toadStool streaming dispatch → metalForge NUCLEUS routing with PCIe P2P bypass). 395 tests, 194 Python cross-validation checks. 6 WGSL shaders. Rust 84× faster than Python (V18). GPU scaling linear, toadStool V16 dispatch, mixed NUCLEUS V16 with PCIe P2P GPU↔NPU bypass (V19). V20: petalTongue V16 visualization (34-node full study, 16 scenarios, patient explorer).
**Last Updated**: March 10, 2026

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
| **V20** | [V20 petalTongue V16 Visualization Handoff](handoffs/HEALTHSPRING_V20_PETALTONGUE_V16_VIZ_BARRACUDA_TOADSTOOL_HANDOFF_MAR10_2026.md) | Mar 10 | petalTongue V16 scenarios (34 nodes, 38 edges), compute pipeline viz, unified dashboard (Exp088, 326/326), patient explorer (Exp089, 14/14), 16 scenario JSONs. |
| **V19** | [V19 Full-Stack Portability Handoff](handoffs/HEALTHSPRING_V19_FULLSTACK_BARRACUDA_TOADSTOOL_HANDOFF_MAR10_2026.md) | Mar 10 | GPU scaling (Exp085, 47/47), toadStool V16 dispatch (Exp086, 24/24), mixed NUCLEUS (Exp087, 35/35), PCIe P2P bypass, barraCuda absorption guidance, EDA SIMD optimization target. |
| **V15** | [V15 Upstream Rewire Handoff](handoffs/HEALTHSPRING_V15_UPSTREAM_REWIRE_HANDOFF_MAR10_2026.md) | Mar 10 | Upstream rewire: precision routing, upstream `barracuda` dependency, WGSL shader provenance, ecosystem catch-up. |
| | *V1–V14.1 → `handoffs/archive/`* | | Fossil record |

## Archive

Superseded handoffs in `handoffs/archive/` — preserved as fossil record.

---

## Cross-Spring Dependencies

| From | What | Version | Status |
|------|------|---------|--------|
| **barraCuda** | Core math, PK/PD ops, diversity, LCG PRNG, eigensolver, `Fp64Strategy`, V16 GPU ops (MM batch, SCFA batch, Beat classify) | v0.3.3+ | **Live** — Tier 2+3 GPU via 6 WGSL shaders + `GpuContext` + `execute_fused`. V16 scaling validated. |
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
