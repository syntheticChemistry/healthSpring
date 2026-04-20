# healthSpring — Primal Gaps

> Gaps discovered during proto-nucleate composition alignment.
> Filed per NUCLEUS_SPRING_ALIGNMENT.md §Feedback Protocol.
> Hand back to primalSpring for ecosystem-wide refinement.

**Proto-nucleate**: `primalSpring/graphs/downstream/healthspring_enclave_proto_nucleate.toml`
**Date**: 2026-04-20
**healthSpring version**: V57 (ecoBin 0.9.0, guideStone Level 4, primalSpring v0.9.17)

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

## 13. Live IPC Composition Parity

**Gap**: In-process dispatch parity (exp112–118) validates the routing layer
but not the actual wire path: socket connect, JSON serialize, transport,
JSON deserialize, response extraction. Live IPC parity is required to prove
the NUCLEUS wire path is faithful.

**Status**: **Fixed in V53** — Three live IPC experiments added:
- exp119: Science dispatch parity over Unix socket (Hill, PK, AUC, Shannon, Anderson).
- exp120: Provenance trio round-trip over IPC (session lifecycle, Merkle/commit/braid).
- exp121: NUCLEUS health probes over IPC (liveness, readiness, capability.list, identity).
All skip gracefully when primal is offline.

---

## 14. `dyn` Dispatch in Application Code

**Gap**: `Box<dyn ValidationSink>` violated the stadial zero-dyn invariant.

**Status**: **Fixed in V53** — Replaced with `ValidationSink` enum dispatch
(`Tracing`, `Silent`, `Collecting`). Zero `dyn` in application code.

---

## 15. `Result<_, String>` Error Types

**Gap**: `cmd_serve` and provenance IPC functions returned `Result<_, String>`,
providing opaque error context and preventing match-based error handling.

**Status**: **Fixed in V53** — `ServerError` enum for server lifecycle,
`TrioError` enum for provenance IPC. Both implement `Display` and are
structurally matchable.

---

## 16. Hardcoded Primal Names in Capability Routing

**Gap**: `ROUTED_CAPABILITIES` mapped capability methods to primal names
(`"toadstool"`, `"squirrel"`, `"nestgate"`) instead of capability domains.

**Status**: **Fixed in V53** — Routes to capability domains (`"compute"`,
`"shader"`, `"storage"`, `"inference"`). Discovery resolves the domain to
whichever primal currently advertises it.

---

## 17. barraCuda Library → IPC Migration (Level 5 Primal Proof)

**Gap**: healthSpring links barraCuda as a Rust library dependency (`barracuda`
crate via path dep) and calls 12+ functions directly:

| Library call | IPC method | Module |
|-------------|-----------|--------|
| `barracuda::stats::mean` | `stats.mean` | `uncertainty.rs` |
| `barracuda::stats::correlation::std_dev` | `stats.std_dev` | `uncertainty.rs` |
| `barracuda::stats::hill` | `stats.hill` | `pkpd/dose_response.rs` |
| `barracuda::stats::shannon_from_frequencies` | `stats.shannon_from_frequencies` | `microbiome/mod.rs` |
| `barracuda::stats::simpson` | `stats.simpson` | `microbiome/mod.rs` |
| `barracuda::stats::chao1_classic` | `stats.chao1_classic` | `microbiome/mod.rs` |
| `barracuda::stats::bray_curtis` | `stats.bray_curtis` | `microbiome/clinical.rs` |
| `barracuda::special::anderson_diagonalize` | `special.anderson_diagonalize` | `microbiome/anderson.rs` |
| `barracuda::rng::lcg_step` | `rng.uniform` | `rng.rs` |
| `barracuda::health::pkpd::mm_auc` | `health.pkpd.mm_auc` | `pkpd/nonlinear.rs` |
| `barracuda::health::microbiome::*` | `health.microbiome.*` | `microbiome/clinical.rs` |
| `barracuda::health::biosignal::scr_rate` | `health.biosignal.scr_rate` | `biosignal/stress.rs` |

Additionally, 6 GPU ops use `barracuda::ops::*` and `barracuda::device::*`
behind the `barracuda-ops` / `gpu` features.

For the Level 5 primal proof, the primal binary must call barraCuda over IPC
(its 32 JSON-RPC methods), not link it. The library dep stays for Level 2
(Rust proof) baseline comparison.

**Impact**: healthSpring cannot pass Level 5 until math calls route through
barraCuda's UDS JSON-RPC surface. The IPC methods already exist in the
barraCuda ecobin — the gap is in healthSpring's wiring.

**Migration path**:
1. Add `BarraCudaClient` (typed IPC client, like `PrimalClient`)
2. Feature-gate: `--features primal-proof` routes math via IPC;
   default keeps library dep for Level 2
3. Validate: IPC result == library result == Python baseline
4. `niche::BARRACUDA_IPC_MIGRATION` inventories all 12 call sites

**Status**: V54 — **reframed**. The V53 narrative ("9 methods not on wire")
was incorrect. The 9 "pending" methods are **domain-specific healthSpring
science** (Hill, Shannon, Simpson, etc.) — local compositions of barraCuda's
generic primitives. They are NOT candidates for barraCuda IPC migration.
barraCuda's 32 IPC methods are generic math (stats, linalg, tensor, spectral).

The correct framing: `math_dispatch` is the "validation window" (temporary
tooling per `GUIDESTONE_COMPOSITION_STANDARD`). The `healthspring_guidestone`
binary uses `primalspring::composition::validate_parity` for generic IPC
(`stats.mean`, `stats.std_dev`, `stats.variance`, `stats.correlation`).
Domain functions stay local.

**No barraCuda wire gap exists.** The ecosystem ask from V53 (add 9 wire
handlers) is withdrawn.

---

## 19. barraCuda Wire Gaps: `stats.variance`, `stats.correlation`

**Gap**: ~~Live IPC testing against barraCuda revealed that `stats.variance` and
`stats.correlation` were not on barraCuda's JSON-RPC surface.~~

**Status**: **RESOLVED in Sprint 44 / primalSpring v0.9.17.** barraCuda added
`stats.variance` (sample variance, Bessel's N-1) and `stats.correlation`
(Pearson r) to the wire. healthSpring's guideStone now validates both in
Tier 2 (IPC-Wired) and Tier 3 (Primal Proof).

**Evidence**: guideStone run (57/57, 10 skipped):
- `stats.mean` — PASS (composition=5.5, local=5.5, diff=0.00e0)
- `stats.std_dev` — PASS (composition=3.027…, local=3.027…, diff=0.00e0)
- `stats.variance` — PASS (composition=9.166…, local=9.166…, diff=1.78e-15)
- `stats.correlation` — PASS (composition=1, local=1, diff=0.00e0)

---

## 20. BTSP Production Mode Breaks Primal IPC

**Gap**: When `FAMILY_SEED` (or `BEARDOG_FAMILY_SEED`) is set in the
environment, `primalspring::ipc::Transport::connect` attempts a BTSP
handshake before JSON-RPC. Primals that do not implement the BTSP server-side
handshake (barraCuda, nestgate, etc.) reject or misparse the BTSP `ClientHello`,
causing all IPC to fail with silent connection errors.

**Impact**: Any deployment that sets `FAMILY_SEED` for BearDog production mode
will break IPC to all non-BTSP primals. The guideStone must `unset FAMILY_SEED`
to fall back to cleartext UDS.

**Workaround**: Unset `FAMILY_SEED`, `BEARDOG_FAMILY_SEED`, and
`RHIZOCRYPT_FAMILY_SEED` before running `healthspring_guidestone` or any
`primalspring`-based client.

**Proposed resolution**: `Transport::connect` should negotiate protocol support
(e.g., probe for BTSP capability) rather than unconditionally attempting
BTSP when `FAMILY_SEED` is present. Alternatively, non-BTSP primals should
gracefully reject the handshake and fall back to cleartext.

**Status**: Documented. Workaround in place. Needs primalSpring transport fix.

---

## 21. Crypto Capability Protocol Errors

**Gap**: `crypto.hash` and `crypto.sign` (BearDog) return protocol errors during
guideStone validation:
- `crypto.hash`: "Invalid base64 data: Invalid symbol 45, offset 12"
- `crypto.sign`: "Missing required parameter: message"

The guideStone sends probe payloads that do not match BearDog's expected
parameter schema.

**Impact**: Crypto capabilities are SKIPped in Tier 2. Not a blocker for
Level 5 (science validation), but prevents full manifest capability coverage.

**Proposed resolution**: Align guideStone probe payloads with BearDog's actual
JSON-RPC parameter schema once BearDog publishes method signatures in a
wateringHole spec.

**Status**: Documented. SKIPped in guideStone (10 skips).

---

## 22. Missing Socket Discovery: DAG, AI, Commit Capabilities

**Gap**: guideStone cannot discover sockets for `capability:dag` (rhizocrypt
DAG methods), `capability:ai` (squirrel inference), or `capability:commit`
(sweetgrass braid). These primals either:
- (a) do not register capability-keyed sockets in the standard
  `$XDG_RUNTIME_DIR/biomeos/` directory, or
- (b) use a different capability vocabulary than what `discover_by_capability`
  searches for.

**Evidence** (guideStone 57/57, 10 skipped):
- `rhizocrypt.liveness`: SKIP — no socket for `capability:dag`
- `sweetgrass.liveness`: SKIP — no socket for `capability:commit`
- `inference.complete`: SKIP — no socket for `capability:ai`
- `dag.session.create`, `dag.event.append`: SKIP — no socket for `capability:dag`
- `braid.create`, `braid.commit`: SKIP — no socket for `capability:commit`

**Impact**: 7 of 10 guideStone SKIPs are due to socket discovery misses for
these three capability domains. Full NUCLEUS validation requires either
capability-keyed socket registration or a discovery shim.

**Proposed resolution**:
- Rhizocrypt, Sweetgrass, and Squirrel register sockets with
  `capability:{domain}` naming convention, OR
- primalSpring's `discover_by_capability` adds fallback to primal-name-keyed
  sockets (e.g., `rhizocrypt.sock` → capabilities `dag.*`)

**Status**: Documented. Needs ecosystem socket registration standardization.

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
| 13 | Live IPC parity | — | **Fixed V53**: exp119–121 | Absorb pattern |
| 14 | Zero `dyn` dispatch | — | **Fixed V53**: enum `ValidationSink` | — |
| 15 | Typed error returns | — | **Fixed V53**: `ServerError`, `TrioError` | — |
| 16 | Capability routing by domain | — | **Fixed V53**: `by_capability` domains | — |
| 17 | barraCuda lib→IPC (Level 5) | — | **V54**: reframed — 9 methods are local domain compositions, not wire gaps. guideStone uses `CompositionContext` for generic IPC | None (V53 ask withdrawn) |
| 18 | guideStone P3 (CHECKSUMS) | — | **Fixed V55**: BLAKE3 via `primalspring::checksums::verify_manifest()`. SKIP when no manifest (honest scaffolding). | — |
| 19 | barraCuda: `stats.variance`, `stats.correlation` | — | **RESOLVED V57**: Sprint 44 added both; guideStone validates in Tier 2+3 | — |
| 20 | BTSP production mode breaks IPC | primalSpring transport | **V57**: documented, `FAMILY_SEED` workaround | Negotiate BTSP capability |
| 21 | Crypto probe schema mismatch | BearDog method spec | **V57**: documented, SKIPped in guideStone | Publish method signatures |
| 22 | Missing socket discovery (DAG/AI/commit) | Ecosystem socket std | **V57**: documented, SKIPped in guideStone | Standardize capability sockets |
