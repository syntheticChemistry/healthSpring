<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# healthSpring V60 — Deep Debt Evolution Handoff

**Date:** May 8, 2026
**From:** healthSpring (V60)
**To:** primalSpring, upstream primal teams, downstream springs
**Status:** V60 — Deep debt evolution complete. 95 experiments, 1,002 tests, 83 capabilities, 53 Python baselines + 53 notebooks. barraCuda v0.3.13. guideStone Level 5. ecoBin 0.9.0. Zero clippy, zero unsafe.

---

## What Changed (V59 → V60)

| Item | Description |
|------|-------------|
| **barracuda-lib feature** | barraCuda/barracuda-core now optional deps. `default = ["barracuda-lib"]`. IPC-first sovereign path when disabled. `math_dispatch.rs` pure-Rust fallbacks for Hill, Shannon, Simpson, Chao1, Bray-Curtis, MM AUC, SCR rate. |
| **exp123_nucleus_parity** | Full NUCLEUS pipeline parity for health niche: Tower (crypto+discovery) + Node (barraCuda stats IPC) + Nest (storage round-trip) + cross-atomic + niche science. Replicates primalSpring exp094. |
| **53 .ipynb notebooks** | All Python control scripts converted to paired notebooks with paper-linkage cells via `tools/py_to_notebook.py`. |
| **validate_pk_models** | New binary for projectNUCLEUS workload (Hill, 1-compartment PK, PopPK, Michaelis-Menten). 16 checks, exit 0/1. |
| **gpu_parity.rs** | Criterion GPU benchmarks: Hill 10K, Diversity 1K, PopPK 5K, MM 512. Feature-gated `gpu`. |
| **Dataset fetch scripts** | `fetch_mitbih.sh`, `fetch_chembl.sh`, `fetch_hmp_16s.sh`, `fetch_geo_ar.sh` with BLAKE3 provenance. |
| **Timeout centralization** | All IPC/server/viz timeouts moved to `tolerances.rs` (10 new constants). |
| **Capability-based discovery** | `BarraCudaClient::discover()` uses `stats` capability first, `barracuda` name fallback. |
| **Tolerance migration** | Inline `1e-15`/`1e-10` in exp122 + guidestone replaced with `tolerances::MACHINE_EPSILON_STRICT` / `MACHINE_EPSILON`. |
| **records_infra.rs split** | 777 LOC monolith split into `records_discovery.rs`, `records_gpu.rs`, `records_composition.rs`, `records_infra.rs`. |
| **viz tests split** | `scenarios/tests.rs` (732 LOC) split into `tests_biosignal.rs`, `tests_pkpd.rs`, `tests_endocrine.rs`, `tests_microbiome.rs`. |
| **exp119-122 CI** | Added `[[bin]]` entries + CI composition job coverage for all 5 new experiments. |
| **Clippy clean** | Fixed `map_unwrap_or`, `doc_markdown`, `format_collect`, `useless_conversion`. |
| **Version bump** | barraCuda v0.3.12 → v0.3.13 across 17 active docs + CI verification. |
| **Paper queue fix** | CM-003/CM-004 inconsistency resolved (moved to Completed table). |

---

## Patterns for Upstream Absorption

These patterns were pioneered or refined by healthSpring and are ready for ecosystem-wide adoption:

1. **`capability_registry.toml` + CI sync test** — Single TOML source of truth for all JSON-RPC methods, cross-synced against primalSpring's canonical 389-method registry in CI. Every spring should have one.

2. **Capability-first discovery** — `discover_by_capability("stats")` before `discover_primal("barracuda")`. No hardcoded primal IDs in dispatch modules. Only self-knowledge constants in `primal_names.rs`.

3. **`PrimalClient` health/capability fallback chains** — Structured probe path: `health.liveness` → capability filter → name-based → env override. Handles heterogeneous primals gracefully.

4. **Dual-namespace RPC** — Songbird discovery (`discovery.find_by_capability` + `net.discovery.find_by_capability`), inference (`inference.*` + `model.*`). Springs should wire both and let ecosystem converge.

5. **`math_dispatch` + `BARRACUDA_IPC_MIGRATION` inventory** — Explicit separation of generic IPC methods (2: mean, std_dev) vs spring-local domain compositions (9: Hill, Shannon, etc.). WIRE_READY_COUNT / TOTAL_COUNT for progress tracking.

6. **Composition experiment ladder (Tier 3→5)** — Structured validation from dispatch parity (T3) through live IPC (T4) to full NUCLEUS composition (T5). `exp095_proto_nucleate_template` as scaffold.

7. **Sovereign GPU pipeline** — barraCuda WGSL → coralReef compile → toadStool dispatch, with `wgpu` fallback. `sovereign-dispatch` feature gate.

8. **`barracuda-lib` optional feature** — Default-on library link, IPC-only when off. Pattern ready for all springs to adopt.

---

## Per-Primal Asks (Upstream)

### barraCuda
- **TensorSession API**: healthSpring has `execute_fused_local` ready; need barraCuda session-style API for fused multi-op GPU pipelines.
- **Variance/correlation on wire**: `stats.variance` and `stats.correlation` should join `stats.mean`/`stats.std_dev` on the JSON-RPC surface for full IPC parity.

### BearDog
- **BTSP server endpoint**: Client exists (`ipc/btsp.rs`); server-side transport not yet available.
- **Crypto probe schema**: `health.liveness` response payloads don't match healthSpring's probe expectations; guideStone SKIPs crypto tiers.
- **FAMILY_SEED transport negotiation**: Current `FAMILY_SEED` env breaks non-BTSP primals; need transport-level negotiation.

### Songbird
- **Canonical discovery method names**: `discovery.find_by_capability` vs `net.discovery.find_by_capability` — healthSpring wires both as fallback; ecosystem should converge.
- **Crypto provider startup**: Songbird fails to start when BearDog crypto socket unavailable.

### NestGate
- **Default in PRIMAL_LIST**: NestGate not in default composition scripts; must be manually added.
- **`storage.egress_fence`**: Ionic bridge blocked on egress fence implementation.

### Squirrel
- **ecoBin stability**: Squirrel binary maturity for composition testing.
- **Canonical namespace**: `inference.*` vs `model.*` — healthSpring supports both; ecosystem should pick one.

### petalTongue
- **Server-mode proprioception**: `proprioception.get` weak in headless/server deployments.

### biomeOS
- **Orchestrator socket conventions**: Standardize discovery for `lifecycle.*` endpoints.
- **socat/shim replacement**: `tools/socat` workaround for UDS composition testing should become unnecessary.

### coralReef
- **Compiler cache**: Sovereign dispatch depends on cache strategy.
- **df64 preamble standardization**: `strip_f64_enable()` workaround still needed.

### Provenance Trio (rhizoCrypt / loamSpine / sweetGrass)
- **Capability-keyed sockets**: `dag.*`, `commit.*`, `braid.*` domains need socket discovery (Gap #22).
- **UDS JSON-RPC response conformance**: Empty responses on Unix socket compositions (Gap #23).

---

## For Downstream Springs (River Delta)

Patterns ready for adoption by hotSpring, wetSpring, neuralSpring, ludoSpring, groundSpring, airSpring:

| Pattern | How to adopt |
|---------|--------------|
| `barracuda-lib` optional | Copy feature gate from healthSpring `ecoPrimal/Cargo.toml`; add pure-Rust fallbacks in your `math_dispatch`. |
| `capability_registry.toml` | Create `config/capability_registry.toml` per your niche; add `integration_registry_sync.rs` test. |
| NUCLEUS parity experiment | Use `exp095_proto_nucleate_template` as scaffold; replicate exp123 for your niche. |
| projectNUCLEUS workload | Create a `validate_*` binary matching your workload TOML in `gardens/projectNUCLEUS/workloads/`. |
| Python→notebook conversion | Run `tools/py_to_notebook.py` (or copy it) against your `control/` scripts. |
| GPU Criterion benchmarks | Add `benches/gpu_parity.rs` with your domain ops, feature-gated behind `gpu`. |

---

## Test Matrix (V60)

| Metric | Value |
|--------|-------|
| Library tests (`cargo test --lib`) | 868 |
| Integration + doc tests | 134 |
| Workspace total (`cargo test --workspace`) | 1,002 |
| Clippy (pedantic+nursery) | 0 errors |
| Unsafe blocks | 0 |
| `#[allow()]` in production | 0 |
| Experiments | 95 (83 science + 12 composition) |
| Composition experiments | 12 (exp112-123) |
| guideStone | Level 5 (57/57 properties) |
| Python baselines | 53 scripts + 53 notebooks |
| Provenance coverage | 95+ records (100%) |
| Capabilities | 83 JSON-RPC methods |
| Deploy graphs | 7 |
| Criterion benchmarks | 4 (cpu_parity, kokkos_parity, upstream_parity, gpu_parity) |
| ecoBin | 0.9.0 (static PIE, x86_64+aarch64 musl) |

---

## Known Gaps (Active)

| Gap | Owner | Status | Priority |
|-----|-------|--------|----------|
| #2 Ionic bridge | BearDog + NestGate | Blocked — egress fence + crypto.ionic_bond | P2 |
| #9 Squirrel maturity | Squirrel | Blocked — ecoBin stability | P3 |
| #10 BTSP server | BearDog | Blocked — server endpoint | P2 |
| #20 FAMILY_SEED | BearDog | Workaround — unset for non-BTSP | P2 |
| #21 Crypto probe schema | BearDog | SKIP in guideStone | P3 |
| #22 Capability sockets | Ecosystem | dag/ai/commit domain sockets | P2 |
| #23 Provenance trio UDS | rhizoCrypt/loamSpine/sweetGrass | Empty JSON-RPC responses | P2 |
| #24 Songbird crypto provider | Songbird | Startup failure without BearDog | P3 |
| #25 petalTongue proprioception | petalTongue | Weak in server mode | P3 |
| #26 NestGate default | Composition scripts | Manual addition required | P3 |
| #27 socat shim | biomeOS / tools | Workaround for UDS testing | P3 |

---

## Fossil Record

V1–V52 archived in `wateringHole/handoffs/archive/`. V53–V58 archived in this release. At V60 publication, V54 upstream evolution and V59 deep-debt composition handoffs still sat beside this file at top level; **May 9, 2026:** both were moved into `handoffs/archive/` (superseded by V60/V61 — same filenames).
