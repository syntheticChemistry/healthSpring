# healthSpring V65d — Wave 107 Forward Evolution

**Date**: June 10, 2026
**From**: healthSpring (ironGate)
**Wave**: 107
**Version**: V65c → V65d

---

## Context

Upstream ecosystem at Wave 107: ZERO P1, S1-S4 GRADUATED, 4-gate mesh collective LIVE, topology-aware routing SHIPPED. healthSpring absorbs Wave 74-107 upstream evolution (mesh topology, `ipc.resolve`, `DiscoveryPath`, cross-gate BTSP trust) into validation scenarios.

## Changes

### 1. `s_nest_atomic` deepened (Phases 10-11)

The existing 9-phase Nest Atomic scenario now includes two forward-evolution phases:

**Phase 10 — BTSP Posture**: Checks `ctx.btsp_authenticated()` for all four Nest-critical capabilities (storage, dag, commit, crypto). Reports per-capability BTSP state and coverage metrics. Distinguishes standalone mode (no `FAMILY_SEED`) from active BTSP deployments.

**Phase 11 — Mesh Awareness**: Calls `discovery.peers` to build mesh topology, resolves gate identity from `GATE_NAME`/`GATE_ID` env, sets `ctx.set_gate_id()`, and verifies `ipc.resolve` returns transport endpoints for the `storage` capability. Covers the full pipeline from peer discovery through topology-aware routing.

### 2. New `s_cross_gate_enclave` scenario (61st)

Five-phase scenario exercising the Wave 74-107 mesh APIs:

| Phase | Content |
|-------|---------|
| 1 | Structural routing prerequisites + gate identity from env |
| 2 | Build `MeshTopology` from `discovery.peers`, register local + remote gates |
| 3 | `ipc.resolve` transport resolution for 4 enclave-critical capabilities |
| 4 | Cross-gate readiness: reachable/unreachable capabilities, `resolve_cross_gate` |
| 5 | Mesh-aware BTSP trust posture + discovery path completeness |

Track: Composition. Tier: Both (structural without NUCLEUS, composition with).

### 3. Composition re-exports expanded

`composition/mod.rs` now re-exports from primalSpring:
- `MeshTopology`, `GateNode`, `MeshRoute` (mesh module)
- `DiscoveryPath` (context)
- `method_to_capability_domain`, `validate_parity_flex` (utilities)

### 4. Pre-existing test flake fixed

`query_fails_without_songbird` in `visualization/capabilities.rs` — assertion now matches all `CapabilityError` variants (was missing `RpcError` and `SerializationError`). Failed when live songbird on ironGate returned unexpected error shapes.

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Scenarios | 60 | **61** |
| Tests | 1,056 | **1,056** (0 failures, was 1 pre-existing flake) |
| Clippy warnings | 0 | 0 |
| TODO/FIXME/HACK | 0 | 0 |
| Largest file | 691 LOC (`s_nest_atomic.rs`) | 691 LOC |

## Upstream Dependencies

| Item | Owner | Status |
|------|-------|--------|
| NestGate `content.egress` | nestGate | Upstream — no current API. Phase deferred. |
| Live ionic E2E | bearDog + songBird | Upstream — requires multi-gate `capability.call` remote dispatch. |
| LTEE E2 (HOLIgraph) + E4 (macrocyclic peptides) | healthSpring science | Queued — data prep. |

## Ecosystem Alignment

- ironGate: 12/13 NUCLEUS, VPS relay, 4-gate mesh collective member
- Temporal sync: PARITY via `membrane temporal.sync` across wateringHole + primalSpring + healthSpring + plasmidBin
- Impulses reviewed: only `westGate` enrollment active — not healthSpring's concern
- Deep debt: all 7 categories at zero
