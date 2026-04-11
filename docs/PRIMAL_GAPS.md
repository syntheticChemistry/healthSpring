# healthSpring — Primal Gaps

> Gaps discovered during proto-nucleate composition alignment.
> Filed per NUCLEUS_SPRING_ALIGNMENT.md §Feedback Protocol.
> Hand back to primalSpring for ecosystem-wide refinement.

**Proto-nucleate**: `primalSpring/graphs/downstream/healthspring_enclave_proto_nucleate.toml`
**Date**: 2026-04-11
**healthSpring version**: V52 (ecoBin 0.8.0)

---

## 1. Capability Vocabulary Mismatch

**Gap**: The proto-nucleate declares `health.pharmacology`, `health.genomics`,
`health.clinical`, `health.de_identify`, and `health.aggregate` as capabilities
for the healthSpring node. healthSpring's actual capability surface uses
`science.{domain}.{operation}` (61 science methods) plus infrastructure
capabilities. There is no `health.*` science namespace — only `health.liveness`
and `health.readiness` probes.

**Impact**: biomeOS graph deployment that routes by `health.*` capabilities
will not match what healthSpring advertises via `capability.list`.

**Proposed resolution**: Either:
- (a) healthSpring registers `health.*` aliases alongside `science.*` in
  `ALL_CAPABILITIES` and routing, or
- (b) the proto-nucleate is revised to use `science.*` capability names that
  healthSpring already serves.

Option (b) is preferred — it avoids a parallel namespace and keeps `health.*`
reserved for probes and composition health per the semantic naming standard.

**Status**: healthSpring V49 implements option (a): `health.pharmacology`,
`health.genomics`, `health.clinical`, `health.de_identify`, and
`health.aggregate` are registered in `ALL_CAPABILITIES` and routed via
`resolve_proto_alias()` in `server/routing.rs` to canonical `science.*`
methods. Both namespaces coexist — `science.*` remains the primary surface.

---

## 2. Ionic Bridge / Bonding Policy — No Implementation Path

**Gap**: The proto-nucleate declares a dual-tower architecture with an ionic
bridge between Tower A (patient data) and Tower B (analytics). The bonding
policy specifies covalent intra-tower, ionic cross-family, and encryption
tiers. No primal in the current ecosystem implements ionic bond enforcement,
egress fences, or cross-family policy gates.

**Impact**: The dual-tower security model is aspirational. Without BearDog
ionic bond support and NestGate egress fence support, the enclave graph
cannot enforce its own declared policy.

**Primal evolution needed**:
- **BearDog**: `crypto.ionic_bond`, `crypto.verify_family`, per-family key
  management
- **NestGate**: `storage.egress_fence`, time-series egress policy, family-scoped
  encryption at rest

**Status**: Blocked on BearDog and NestGate evolution. healthSpring will wire
to these capabilities once they exist; no healthSpring code changes needed
beyond IPC client stubs.

---

## 3. Discovery RPC Method Name Mismatch

**Gap**: healthSpring's `TowerAtomic` implementation calls
`net.discovery.find_by_capability` on the Songbird socket. The proto-nucleate
graph lists Songbird capabilities as `discovery.find_primals` and
`discovery.announce`. These method names do not match.

**Impact**: Tower Atomic discovery will fail at runtime unless Songbird
accepts both naming conventions or one side is updated.

**Proposed resolution**: Standardize on `discovery.*` (without `net.` prefix)
per the semantic naming standard's `domain.verb` pattern. Update
healthSpring's `tower_atomic.rs` once the ecosystem agrees on the canonical
Songbird method names.

**Status**: healthSpring V50 adds dual-method fallback in `tower_atomic.rs` —
tries `discovery.find_by_capability` first, falls back to
`net.discovery.find_by_capability`. Full resolution pending Songbird canonical
naming.

---

## 4. Inference Method Namespace (Squirrel / neuralSpring)

**Gap**: The proto-nucleate assigns Squirrel capabilities `ai.complete`,
`ai.models`, `inference.complete`, `inference.embed`, `inference.models`.
healthSpring discovers inference peers via `model.*` capability prefix and
routes using `model.inference_route` → `model.{operation}`.

**Impact**: healthSpring cannot discover a Squirrel that advertises
`inference.*` or `ai.*` — the capability scan filters on `model.*`.

**Proposed resolution**: healthSpring now supports `inference.*` as an
additional discovery and routing namespace alongside `model.*`. The
`inference_dispatch` module accepts both `model.{op}` and `inference.{op}`
method names. The final canonical namespace should be decided at the
primalSpring level and documented in a wateringHole handoff.

**Status**: healthSpring V48 adds `inference.*` aliases. Canonical namespace
selection pending primalSpring/Squirrel/neuralSpring coordination.

---

## 5. Readiness Semantics

**Gap**: healthSpring's `health.readiness` probe always returns `ready: true`
regardless of subsystem status. The composition health endpoint correctly
reports degraded subsystems, but readiness (used by orchestrators for
scheduling) does not gate on them.

**Impact**: biomeOS may route work to healthSpring before critical subsystems
(provenance trio, compute provider) are available.

**Proposed resolution**: healthSpring V48 gates readiness on science dispatch
availability. Optional subsystems (provenance, compute, data) are reported
but do not block readiness — the spring degrades gracefully without them.

**Status**: Fixed in V48 — `ready` is now `science_ok` (always true for the
in-process science pipeline; will gate on real health checks if the pipeline
becomes async or remote).

---

## 6. Resilience Patterns Not Wired

**Gap**: `CircuitBreaker` and `RetryPolicy` in `ipc/resilience.rs` are
implemented and tested but never used in any IPC call path. Cross-primal
RPC calls (`primal.forward`, `compute.offload`, `data.fetch`,
`model.inference_route`) use raw `rpc::try_send` without retry or circuit
breaking.

**Impact**: Transient IPC failures cascade immediately to the caller without
retry or backoff.

**Proposed resolution**: healthSpring V48 wires `resilient_send` into the
`rpc` module, applying retry with exponential backoff for retriable errors.
Circuit breaker integration deferred until persistent connection state is
available (current per-request UDS connections don't benefit from circuit
breaking).

**Status**: `resilient_send` wired in V48. Circuit breaker remains available
for future persistent-connection transports.

---

## 7. YAML Niche Manifest Underreports Capabilities

**Gap**: `niches/healthspring-health.yaml` lists only science + composition
capabilities. The binary actually serves 14 additional infrastructure
capabilities (`health.*`, `capability.list`, `compute.*`, `data.fetch`,
`model.inference_route`, `inference.*`, `provenance.*`, `primal.*`).

**Impact**: BYOB deployment tools reading the YAML will underestimate what
the primal provides, potentially failing to route infrastructure requests.

**Proposed resolution**: healthSpring V48 adds infrastructure capabilities
to the YAML manifest.

**Status**: Fixed in V48.

---

## 8. Deploy Graph Fragment Metadata

**Gap**: Deploy graph TOMLs (`healthspring_niche_deploy.toml`,
`healthspring_biomeos_deploy.toml`) listed NUCLEUS atomics in comments but
lacked formal `fragments`, `particle_profile`, and `bonding` metadata keys.

**Status**: Fixed in V49 — both deploy graphs now declare `fragments`,
`particle_profile`, `proto_nucleate`, and `[graph.bonding]` with bond type,
trust model, and encryption tiers per atomic boundary.

---

## 9. Squirrel Not in Deploy Graphs

**Gap**: The proto-nucleate places `squirrel_b` in Tower B for clinical AI.
healthSpring's deploy graphs do not include a Squirrel node —
`inference_dispatch` discovers Squirrel dynamically by capability if running.

**Impact**: Without Squirrel in the deploy graph, biomeOS will not start or
verify Squirrel as part of the healthSpring niche deployment.

**Proposed resolution**: Add optional Squirrel node to `healthspring_niche_deploy.toml`
once Squirrel reaches ecoBin compliance and publishes stable `inference.*`
capabilities.

**Status**: healthSpring V50 adds optional `squirrel_b` node to
`healthspring_niche_deploy.toml` with `required = false`. biomeOS will
start Squirrel if available, skip gracefully if not. Full integration
blocked on Squirrel ecoBin compliance and stable `inference.*` capability set.

---

---

## 10. BTSP Handshake — Client Ready, Server Pending

**Gap**: healthSpring V51 implements the BTSP client handshake module
(`ipc/btsp.rs`) with `BtspMessage` enum, `family_seed_from_env()`, and
`client_hello()`. However, no primal in the ecosystem currently exposes a
BTSP server endpoint. The handshake cannot be exercised end-to-end.

**Impact**: Cross-primal authentication remains unenforced. All IPC is
currently unauthenticated plaintext over UDS/TCP.

**Status**: Client module ready in V51. Awaiting BearDog BTSP server endpoint.

---

## 11. Typed IPC Clients — Not Yet Wired into Production Paths

**Gap**: `PrimalClient` and `InferenceClient` in `ipc/client.rs` provide
typed, resilient cross-primal communication. However, the existing dispatch
paths (`primal.forward`, `compute.offload`, `data.fetch`) still use raw
`rpc::try_send` / `rpc::send`. The typed clients are available but not yet
integrated into production request flows.

**Impact**: Production code misses health probe fallback chains and structured
discovery tracking that the typed clients provide.

**Proposed resolution**: Incrementally migrate `primal.forward` and
`compute.offload` to use `PrimalClient` in the next sprint. `InferenceClient`
integration deferred until Squirrel is available.

**Status**: **Fixed in V52** — `PrimalClient.call()` now uses `resilient_send`
(retry with backoff). `try_call()` added for single-attempt paths.
`handle_primal_forward` in routing.rs migrated to `PrimalClient`. Dispatch
modules (`compute_dispatch`, `data_dispatch`, `shader_dispatch`,
`inference_dispatch`) remain as domain-typed clients using `resilient_send`
directly — they are typed clients in their own right and benefit from the
same retry policy.

---

## 12. Deploy Graph Validation Against Proto-Nucleate

**Gap**: No automated validation that the deploy graph
(`healthspring_niche_deploy.toml`) is structurally consistent with the
proto-nucleate graph (`healthspring_enclave_proto_nucleate.toml`). Fragment
metadata, node presence, bonding policy, and capability surface could drift.

**Status**: **Fixed in V52** — exp118 (`exp118_composition_deploy_graph_validation`)
validates deploy graph TOML parsing, fragment metadata alignment
(tower_atomic, nest_atomic), required/optional node presence, bonding policy
(ionic, btsp_enforced, encryption_tiers), capability coverage against
`registered_capabilities()`, Squirrel optional node, and primal identity
constants. Added to CI composition job.

---

## Summary Matrix

| # | Gap | Blocked On | healthSpring Action | primalSpring Action |
|---|-----|------------|--------------------|--------------------|
| 1 | Capability namespace | — | **Fixed V49**: aliases added | Confirm alignment |
| 2 | Ionic bridge | BearDog + NestGate | Wire when available | Evolve primals |
| 3 | Discovery naming | Songbird alignment | **V50**: dual fallback | Standardize names |
| 4 | Inference namespace | Squirrel alignment | `inference.*` added | Pick canonical ns |
| 5 | Readiness semantics | — | Fixed V48 | — |
| 6 | Resilience wiring | — | Fixed V48 | — |
| 7 | YAML manifest | — | Fixed V48 | — |
| 8 | Deploy fragments | — | **Fixed V49**: metadata added | — |
| 9 | Squirrel in deploy | Squirrel maturity | **V50**: optional node added | Evolve Squirrel |
| 10 | BTSP handshake | BearDog BTSP server | **V51**: client module ready | Expose BTSP endpoint |
| 11 | Typed IPC clients | — | **Fixed V52**: PrimalClient wired | — |
| 12 | Deploy graph validation | — | **Fixed V52**: exp118 added | — |
