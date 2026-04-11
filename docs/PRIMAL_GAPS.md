# healthSpring — Primal Gaps

> Gaps discovered during proto-nucleate composition alignment.
> Filed per NUCLEUS_SPRING_ALIGNMENT.md §Feedback Protocol.
> Hand back to primalSpring for ecosystem-wide refinement.

**Proto-nucleate**: `primalSpring/graphs/downstream/healthspring_enclave_proto_nucleate.toml`
**Date**: 2026-04-10
**healthSpring version**: V48 (0.8.0)

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

**Status**: healthSpring V48 adds `inference.*` aliases; `health.*` science
aliases deferred pending primalSpring decision on namespace.

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

**Status**: Pending Songbird/primalSpring semantic naming alignment.

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

## Summary Matrix

| # | Gap | Blocked On | healthSpring Action | primalSpring Action |
|---|-----|------------|--------------------|--------------------|
| 1 | Capability namespace | Naming decision | Add aliases if (a) | Update proto if (b) |
| 2 | Ionic bridge | BearDog + NestGate | Wire when available | Evolve primals |
| 3 | Discovery naming | Songbird alignment | Update tower_atomic | Standardize names |
| 4 | Inference namespace | Squirrel alignment | `inference.*` added | Pick canonical ns |
| 5 | Readiness semantics | — | Fixed | — |
| 6 | Resilience wiring | — | `resilient_send` added | — |
| 7 | YAML manifest | — | Fixed | — |
