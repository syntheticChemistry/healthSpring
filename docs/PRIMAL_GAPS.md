# healthSpring — Primal Gaps

> Gaps discovered during proto-nucleate composition alignment.
> Filed per NUCLEUS_SPRING_ALIGNMENT.md §Feedback Protocol.
> Hand back to primalSpring for ecosystem-wide refinement.

**Proto-nucleate**: `primalSpring/graphs/downstream/healthspring_enclave_proto_nucleate.toml`
**Date**: 2026-05-29
**healthSpring version**: V65a (ecoBin 0.9.0, guideStone Level 5 via **`healthspring_unibin certify`**, primalSpring **v0.9.27**, barraCuda **v0.4.0**, V65a: Wave 60 Eukaryotic Gate — BLAKE3 provenance backfill (62 files), sporePrint sovereign publish (SP-4), `bonding.*` protocol wired, registry 470+, 59 validation scenarios, 1,052 tests, all 7 deep debt categories zero, clippy zero)

---

## V61 resolutions (May 9, 2026)

The following items from earlier gap narratives are **closed or superseded in-tree** for V61:

- **CompositionContext migration** — legacy primal-discovery helpers deprecated toward primalSpring `CompositionContext`; health-facing wrapper in **`composition/`** (`HealthCompositionContext`).
- **Certification organelle** — standalone guidestone logic absorbed into **`certification/`**; **`healthspring_unibin certify`** is the supported entrypoint (fossil **`healthspring_guidestone`** retained under **`fossilRecord/`**).
- **IPC-first defaults** — **`healthspring-barracuda`** uses **`default = []`**; opt into **`barracuda-lib`** for direct barraCuda linkage / GPU paths.
- **Scenario absorption** — sixteen experiment mains moved to **`validation/scenarios/`** with sources archived under **`fossilRecord/experiments_prokaryotic_may2026/`**.
- **UniBin surface** — **`healthspring_unibin`** exposes **`certify`**, **`validate`**, **`serve`**, **`status`**, **`version`** alongside **`healthspring_primal`**.

Remaining ecosystem gaps below (ionic bridge, BTSP server, provenance trio behaviors, etc.) are unchanged unless a row explicitly says fixed.

---

## 1. Capability Vocabulary Mismatch

**Gap**: The proto-nucleate declares `health.pharmacology`, `health.genomics`,
`health.clinical`, `health.de_identify`, and `health.aggregate` as capabilities
for the healthSpring node. healthSpring's actual capability surface uses
`science.{domain}.{operation}` (58 science methods) plus 30 infrastructure
capabilities (88 total). There is no `health.*` science namespace — only `health.liveness`
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

## 2. Ionic Bridge / Bonding Policy — Substantially Resolved (V64z)

**Gap**: The proto-nucleate declares a dual-tower architecture with an ionic
bridge between Tower A (patient data) and Tower B (analytics). The bonding
policy specifies covalent intra-tower, ionic cross-family, and encryption
tiers.

**Resolution history**:
- BearDog **Wave 97** (May 8, 2026) shipped `crypto.contract.*` signing layer.
- healthSpring **V64e** wired `TowerAtomic::ionic_propose/countersign/verify`
  for the Ed25519 signing layer.
- primalSpring **Wave 38** (May 22, 2026) shipped `IonicContractRegistry` with
  full state machine (`bonding::ionic_runtime`): `bonding.propose` →
  `bonding.accept` → `record_call` → `bonding.modify_scope` →
  `bonding.terminate` → `bonding.seal` with TTL expiry and usage metering.
- healthSpring **V64z** wired `bonding.*` protocol layer into `TowerAtomic`
  (`bonding_propose`, `bonding_accept`, `bonding_terminate`, `bonding_status`)
  routing through the coordination socket to primalSpring's
  `IonicContractRegistry`. Added `bonding` capability domain to routing table,
  niche CONSUMED_CAPABILITIES, and capability registry.

**Remaining**:
- **NestGate egress fence**: `storage.egress_fence` was a healthSpring-invented
  wire name with no upstream NestGate counterpart. Removed from
  CONSUMED_CAPABILITIES and capability registry in V64z. Real egress policy
  (time-series fence, family-scoped encryption at rest) awaits NestGate
  `content.egress` or equivalent when NestGate ships enclave enforcement.

**Status**: Ionic bridge **WIRED** — both Ed25519 signing layer
(`crypto.contract.*`) and contract state machine (`bonding.*`) available.
NestGate egress fence reconciled (phantom wire name removed). Live E2E
validation pending deployed NUCLEUS with multi-tower topology.

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
tooling per `GUIDESTONE_COMPOSITION_STANDARD`). **`healthspring_unibin certify`** (and the absorbed **`certification/`** organelle) use `primalspring::composition::validate_parity` for generic IPC
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
`RHIZOCRYPT_FAMILY_SEED` before running **`healthspring_unibin certify`** / fossil **`healthspring_guidestone`** or any
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

## 23. Provenance Trio Wire Dispatch Failures (Phase 46 Composition)

**Gap**: rhizoCrypt, loamSpine, and sweetGrass returned `-32601 MethodNotFound`
(originally misdiagnosed as "empty UDS responses") because healthSpring sent
non-canonical method names that fell through dispatch.

**Root cause** (confirmed May 13, 2026 via GAP-36 reconciliation):
- healthSpring sent `commit.create` → loamSpine canonical is `spine.create`
- healthSpring sent `ledger.append` → loamSpine canonical is `entry.append`
- healthSpring sent `provenance.session.create` → rhizoCrypt canonical is `dag.session.create`
- healthSpring sent `braid.attribution.create` → sweetGrass canonical is `braid.create`

**Resolution**: Two-sided fix:
1. **Upstream**: All three primals shipped `normalize_method()` alias tables
   (rhizoCrypt S68: 21 aliases, loamSpine v0.9.16: 6 aliases, sweetGrass v0.7.35: 10 aliases)
2. **Local (V64j)**: healthSpring `loamspine.rs` fixed `commit.create`→`spine.create`,
   `ledger.append`→`entry.append`. `data/provenance.rs` fixed `commit.create`→`spine.create`.
   All other method names were already canonical (`dag.*`, `braid.create`, `storage.*`).

**Status**: **RESOLVED V64j** — both sides (upstream aliases + local canonical names) fixed.

---

## 24. Songbird Crypto Provider Discovery Failure

**Gap**: Songbird fails to start with `Error: Failed to discover crypto
provider: No Crypto provider available` even when beardog's socket is
active and `--beardog-socket` is passed.

**Evidence**: `gate_nucleus.sh start` → songbird exits immediately
after beardog is running and its socket is confirmed. Log shows songbird's
internal discovery doesn't find beardog via the provided socket path or
env vars.

**Impact**: Discovery service offline. Composition falls back to
`{cap}-{family}.sock` naming convention (which works), but no dynamic
discovery.

**Proposed resolution**: Songbird's internal crypto provider discovery
may need a different env var name or startup sequence. This should be
documented in songbird's CLI help or a wateringHole spec.

**Status**: Documented. Composition works without songbird via the symlink
alias pattern.

---

## 25. petalTongue Proprioception Unavailable in Server Mode

**Gap**: `proprioception.get` returns no `frame_rate` field when petalTongue
runs in `server` mode (headless, no GUI window). In `live` mode the
proprioception endpoint reports fps, active scenes, and user interactivity.

**Evidence**: healthSpring composition (headless) — petalTongue accepted
scene pushes and interaction subscriptions, but proprioception.get returned
no frame_rate data.

**Impact**: Headless/CI compositions cannot monitor petalTongue health via
proprioception. Scene push and interaction subscribe/poll still work.

**Proposed resolution**: petalTongue server mode should return synthetic
proprioception data (e.g., `frame_rate: 0.0`, `active_scenes: N`,
`user_interactivity: "none"`) for monitoring even without a render loop.

**Status**: Documented. Non-blocking.

---

## 26. NestGate Not in Default PRIMAL_LIST

**Gap**: `gate_nucleus.sh (was composition_nucleus.sh, now archived)` default `PRIMAL_LIST` does not include
`nestgate`. Storage capability is offline unless explicitly added:
`PRIMAL_LIST="beardog songbird toadstool barracuda rhizocrypt loamspine sweetgrass nestgate petaltongue"`.

**Impact**: Storage round-trip tests skip. Springs that need storage must
override `PRIMAL_LIST`.

**Proposed resolution**: Add `nestgate` to the default `PRIMAL_LIST` in
`gate_nucleus.sh (was composition_nucleus.sh, now archived)`, or document the override pattern.

**Status**: Documented. Workaround: set `PRIMAL_LIST` explicitly.

---

## 27. socat Dependency Not Documented

**Gap**: `nucleus_composition_lib.sh` uses `socat` for all JSON-RPC
transport (`send_rpc`), but `socat` is not universally installed and is
not listed as a dependency.

**Impact**: Compositions fail on systems without socat. healthSpring created
a `nc -U` shim (`tools/socat`) as a workaround.

**Proposed resolution**: Document socat as a required dependency in the
DOWNSTREAM_COMPOSITION_EXPLORER_GUIDE, or make the lib detect and use
`nc -U` / `ncat` as fallback.

**Status**: Documented. healthSpring provides `tools/socat` shim using
`nc -q 1 -U`.

---

## Deep Debt Evolution — May 8, 2026 *(historical V60 notes)*

### New capabilities delivered this session

- **capability_registry.toml** created with TOML + sync test against primalSpring
- **exp123_nucleus_parity** validates full NUCLEUS pipeline for health niche
- **barraCuda optional** (`barracuda-lib` feature) — IPC-first sovereign deployment path
- **BarraCudaClient** evolved to capability-based discovery (`stats` capability first)
- **Timeout centralization** — all IPC/server timeouts in `tolerances.rs`
- **exp119–122** now have `[[bin]]` entries and are in CI composition loop

### Gaps handed back to primalSpring

- Gap #2 (ionic bridge): still blocked on BearDog + NestGate evolution
- Gap #9 (Squirrel): blocked on Squirrel maturity
- Gap #10 (BTSP server): client ready, blocked on BearDog BTSP server
- Gaps #20–27: documented workarounds stable, await upstream primal evolution

---

## Tier 2 Convergence Wave — May 13, 2026 (V64f)

### Resolved during convergence wiring

- **`toadstool.validate`** — wired in `ipc::compute_dispatch::validate_workload()`. Accepts workload TOML path, returns validity, GPU availability, precision tier, estimated dispatch time.
- **`toadstool.list_workloads`** — wired in `ipc::compute_dispatch::list_workloads()`. Queries available workloads via IPC with optional filter.
- **`barracuda.precision.route`** — wired in `ipc::barracuda_client::BarraCudaClient::precision_route()`. Aligned response fields to canonical wire contract: `recommended_tier`, `fma_safe`, `requires_compiler`, `hardware_hint`.
- **Ionic bridge stubs** — `TowerAtomic::ionic_propose/countersign/verify` wired in V64e. BearDog contract methods callable when primals are live.
- **`--format json`** — wired in unibin, `validate_pk_models`, `validate_ltee_b5`.
- **`--list`** — added to unibin `validate` subcommand for plasmidBin compatibility.
- **plasmidBin cell.toml** — updated to include compute trio nodes (toadStool, barraCuda, coralReef) and validation targets.
- **plasmidBin niche** — promoted from `nest` to `full` composition, now includes all 12 NUCLEUS primals.
- **PRIMAL_PROOF_IPC_MAPPING.md** — created, documenting all 17 domain operation → precision route mappings.
- **LTEE B5 lithoSpore packaging** — `tolerances.toml` + `LITHO_MODULE_README.md` added, documenting exact reproduction commands, tolerance envelopes, and BLAKE3 provenance chain.

### Gaps found during convergence wiring

| # | Gap | Source | Upstream Action |
|---|-----|--------|-----------------|
| 28 | plasmidBin cell TOML was stale — missing compute trio nodes | Convergence wiring | **Fixed locally**: compute trio added to `healthspring_cell.toml` |
| 29 | plasmidBin niche was under-specced (`nest` composition without toadStool/barraCuda/coralReef) | Convergence wiring | **Fixed locally**: promoted to `full` composition |
| 30 | `precision.route` blurb contract (`viable`/`capabilities`/`reason`) diverges from `LIVE_SCIENCE_API.md` (`recommended_tier`/`fma_safe`/`requires_compiler`). healthSpring wires to `LIVE_SCIENCE_API.md` as canonical. | Wire contract review | primalSpring: reconcile blurb → `LIVE_SCIENCE_API.md` |
| 31 | lithoSpore module ingestion for B5 blocked on lithoSpore team accepting `ltee-symbiont-pk` module candidate | LTEE handoff | lithoSpore: ingest `control/ltee_symbiont_pkpd/` with BLAKE3 |

### Provenance elevation (V64g)

- **Dual wire shape eliminated** (Gap #23 partial) — `data/provenance.rs` now uses canonical `dag.session.create`, `dag.event.append` etc. instead of `capability.call` envelope with divergent operation names. All 8 provenance tests pass.
- **`NestComposition` facade wired** — full Nest Atomic chain (NestGate + rhizoCrypt + loamSpine + sweetGrass + BearDog) orchestrated as a single composed unit. Graceful degradation at each step (trio still returns empty UDS responses per Gap #23).
- **Python baseline provenance strengthened** — `expected_values.json` + `tolerances.toml` for 7 science tracks. 30+ DOIs added to scripts and `records_science.rs`.

| # | Gap | Source | Upstream Action |
|---|-----|--------|-----------------|
| 32 | `NestComposition` end-to-end testing blocked by trio empty UDS responses | Provenance elevation | rhizoCrypt/loamSpine/sweetGrass: return non-empty JSON-RPC results |
| 33 | Dataset SHA256 checksums still empty in `data/manifest.toml` | Provenance audit | healthSpring: populate checksums when datasets are fetched |

### Nest Atomic Validation Sprint (V64h)

Upstream audit: **ecoPrimals — Atomic Specialist Validation Sprint (May 13, 2026)**. healthSpring owns the Nest Atomic (neutron) — proves clinical data can be stored, provenanced, ledgered, and attributed through Nest alone.

**Delivered:**

- **`s_nest_atomic` validation scenario** — 9-phase validation exercising all 7 Nest primals through clinical data:
  - Phase 1: Structural routing (7 capability → primal mappings verified)
  - Phase 2: Liveness probes for all 7 primals (bearDog, songbird, skunkBat, nestGate, rhizoCrypt, loamSpine, sweetGrass)
  - Phase 3: NestGate `storage.store` / `storage.retrieve` / `storage.exists` / `storage.list`
  - Phase 4: rhizoCrypt `dag.session.create` / `dag.event.append` (3-event chain: ingest → validate → transform)
  - Phase 5: BearDog `crypto.sign` (Merkle root Ed25519 signature)
  - Phase 6: loamSpine `entry.append` (immutable ledger commit)
  - Phase 7: sweetGrass `braid.create` / `braid.commit` (attribution braid with PROV-O agents)
  - Phase 8: Tower auxiliary — songbird `discovery.peers` + skunkBat `defense.audit`
  - Phase 9: Full chain recoverability audit (content → session → merkle → signature → ledger → braid)
- **`healthspring_nest_atomic.toml` deploy graph** — 7-node Nest Atomic graph with ionic bonding policy and MethodGate trust model
- **Niche manifest updated** — `healthspring_niche.toml` includes `nest_atomic` graph entry
- **`--format json` validated** — `validate --scenario nest-atomic --format json` produces structured CI output
- **Shared checklist status:**
  - [x] Deploy graph loads and resolves correctly
  - [x] Primals start via composition (CompositionContext)
  - [x] health.liveness probed for every primal in the atomic
  - [x] capabilities.list returns expected capabilities
  - [x] Each capability exercised with real clinical data (not mocks)
  - [x] Pass/fail per capability — honest skip when primal not running
  - [x] `--format json` output works for CI consumption
  - [x] Gaps documented below

**Wire name mapping (canonical + aliases per GAP-36 reconciliation):**

| Domain | Canonical Wire Name | Accepted Aliases | Primal |
|--------|-------------------|-----------------|--------|
| CAS (immutable) | `content.put` / `content.get` | — | nestGate |
| Blob (keyed) | `storage.store` / `storage.retrieve` | `storage.put`→`storage.store`, `storage.get`→`storage.retrieve` | nestGate |
| DAG session | `dag.session.create` | `provenance.session.create` (21 total aliases) | rhizoCrypt |
| DAG events | `dag.event.append` | `provenance.event.append` | rhizoCrypt |
| Ledger lifecycle | `spine.create` / `spine.get` | `session.create`, `ledger.create`, `session.state` (6 aliases) | loamSpine |
| Ledger entries | `entry.append` / `entry.get` | — (native, no alias needed) | loamSpine |
| Attribution | `braid.create` / `braid.commit` | `braid.attribution.create`, `attribution.create_braid` (10 aliases) | sweetGrass |

| # | Gap | Source | Upstream Action |
|---|-----|--------|-----------------|
| 34 | `content.*` (CAS) vs `storage.*` (blob) — **BY DESIGN** per biomeOS `capability_registry.toml`. Both route to nestGate, different semantics (immutable BLAKE3 vs keyed mutable). | Wire contract review | **RESOLVED V64j**: confirmed by biomeOS registry + LIVE_SCIENCE_API |
| 35 | `entry.append` is canonical loamSpine. `ledger.entry.append` was never a loamSpine method — it's `entry.append` (native) or `session.create`→`spine.create` (aliased). | Wire contract review | **RESOLVED V64j**: loamSpine v0.9.16 alias table documents canonical names |
| 36 | Nest Atomic exercises were blocked by Gap #23 (trio `-32601 MethodNotFound`) — now unblocked by upstream alias shipping + local canonical name fixes | Validation sprint | **RESOLVED V64j**: upstream aliases + local wire name fixes |
| 37 | `NestComposition` facade used `"data"` as capability domain for NestGate `storage.store` — should align to `"storage"` per routing table | Internal wire review | **Fixed V64h**: refactored `nest.rs` `record_event` to use `"storage"` domain |

### Deep Debt Resolution Sprint (V64i)

**Full audit — all 7 priority categories at zero debt:**

| Category | Status | Detail |
|----------|--------|--------|
| TODO/FIXME/HACK | **0** | Zero markers in entire codebase |
| `unsafe` code | **0** | `#![forbid(unsafe_code)]` enforced at lib + workspace |
| Production mocks | **0** | All mocks in `#[cfg(test)]` |
| `unimplemented!`/`todo!`/`panic!` (non-test) | **0** | Zero incomplete implementations |
| Files > 800 LOC | **0** | Largest file: 597 lines |
| Clippy pedantic+nursery | **0 warnings, 0 errors** | Full pass on `--all-targets` |
| External C deps (default build) | **0** | `ring`/`wgpu` gated behind features |
| Hardcoded primal routing | **0** | All via `primal_names::*` + capability discovery |

**Fixed in V64i:**

- Hardcoded `"healthSpring"` → `crate::PRIMAL_NAME` in provenance session JSON
- `match` → `if let` in `NestComposition` (3 sites)
- `unwrap()` → `f64::total_cmp` in eigenvalue sorts (3 sites)
- `i32 as f64` → `f64::from` in benchmarks + tests (7 sites)
- `s_nest_atomic` decomposed into 9 phase functions (pedantic `too_many_lines`)

**Audit questions answered:**

| Question | Answer |
|----------|--------|
| Python baselines for barraCuda CPU parity? | **Yes**: `control/validation/exp040_barracuda_cpu.py` (stats, Hill, Shannon, Simpson, Chao1, Anderson, Bray-Curtis, PPG) — **now matched by `s_barracuda_cpu_parity` scenario (10 checks, PASS)**. |
| Kokkos/Galaxy/SciPy/LAMMPS GPU benchmarks? | **No**: GPU parity depends on `wgpu` feature + live GPU. No Kokkos/LAMMPS benchmarks — barraCuda's WGSL shaders are sovereign, not porting external frameworks. CPU benchmarks exist in `benches/cpu_parity.rs`. |
| What's not implemented/validated/tested? | **Coverage substantially closed (Wave 60 stabilization)**: 59 scenarios now cover all 10 tracks + LTEE B5 (8 checks) + barraCuda CPU parity (10 checks). Remaining ~5 experiment binaries (exp084/085 GPU scaling, exp112-113 dispatch) lack dedicated scenarios but are exercised by CI composition jobs. Modules without inline unit tests: `certification/`, `composition/`, `gpu/sovereign.rs`, `gpu/cpu_fallback.rs`, `microbiome/anderson.rs` (covered by integration tests). |
| Unreviewed papers from queue? | **2**: LTEE E2 (Mardikoraem & Woldring 2025 "HOLIgraph") and E4 (Woldring Lab 2024 macrocyclic peptides). 45/45 main-track papers complete. |
| Datasets to examine? | **5 datasets** in `data/manifest.toml`, all lacking SHA256 checksums. `qs_gene_matrix` has no fetch script. MitBIH, ChEMBL, HMP 16S, GEO AR ready for fetch but unverified. |

| # | Gap | Source | Upstream Action |
|---|-----|--------|-----------------|
| 38 | ~~\~30 Python baselines without Rust scenarios~~ **SUBSTANTIALLY CLOSED**: 59 scenarios; only ~5 experiment binaries without dedicated scenarios (GPU scaling, dispatch parity — CI-covered) | Deep debt audit → **Wave 60 stabilization** | DONE: exp040 + B5 absorbed |
| 39 | LTEE E2 + E4 papers queued, not reviewed | Paper queue audit | healthSpring: review when relevant to provenance work |
| 40 | Dataset SHA256 checksums empty + `qs_gene_matrix` fetch unimplemented | Data provenance audit | healthSpring: populate post-fetch, implement fetch script |
| 41 | No GPU parity benchmarks (Kokkos/LAMMPS/SciPy) | Benchmark audit | Not applicable — sovereign WGSL shaders, not framework ports |

### Delta Spring Evolution — Upstream Clear (V64j)

**GAP-36 RESOLVED.** All three provenance trio primals shipped `normalize_method()` alias tables:
- **rhizoCrypt S68**: 21 `provenance.*` → `dag.*` aliases + `SEMANTIC_ALIASES`
- **loamSpine v0.9.16**: 6 aliases (`session.create`→`spine.create`, `ledger.create`→`spine.create`, etc.)
- **sweetGrass v0.7.35**: 10 aliases (`braid.attribution.create`→`braid.create`, etc.)

**Local fixes:**
- `loamspine.rs`: `commit.create`→`spine.create`, `ledger.append`→`entry.append` (new canonical fns + backward-compat aliases)
- `data/provenance.rs`: `commit.create`→`spine.create`

**Gaps resolved in V64j:**
- Gap #23 — root cause identified as `-32601 MethodNotFound` (not empty responses); fixed both sides
- Gap #32 — NestComposition testing unblocked (transitive from #23)
- Gap #34 — `content.*` vs `storage.*` confirmed by-design per biomeOS registry (CAS vs blob)
- Gap #35 — `entry.append` is canonical loamSpine; aliases document the full vocabulary
- Gap #36 — Trio wire aliases shipped; Nest Atomic live-ready

**Remaining upstream blockers:**

- **NestGate egress fence** (Gap #2) — ionic bridge partially resolved, NestGate side still needed
- **BTSP server** (Gap #10) — client ready, BearDog BTSP server pending
- **Socket discovery standardization** (Gap #22) — rhizocrypt, sweetgrass, squirrel capability sockets
- **Songbird crypto provider** (Gap #24) — startup discovery failure

### Deep Debt Reconfirmation Sprint (V64k)

Re-audit after V64j wire name changes. **All 7 categories confirmed at zero debt.**

| Category | Status | Detail |
|----------|--------|--------|
| TODO/FIXME/HACK | **0** | Zero markers in entire codebase |
| `unsafe` code | **0** | `#![forbid(unsafe_code)]` enforced at lib + all 5 binary crates |
| Production mocks | **0** | All mocks in `#[cfg(test)] mod tests` (2 mock fns in `visualization/ipc_push/mod.rs`) |
| `unimplemented!`/`todo!`/`panic!` (non-test) | **0** | All 20 `panic!` calls are inside `#[cfg(test)]` test blocks |
| Files > 800 LOC | **0** | Largest file: 597 lines (`ipc/proptest_ipc.rs`) |
| Clippy pedantic+nursery | **0 warnings, 0 errors** | Full pass on `--all-targets`, including V64j additions |
| External C deps (default build) | **0 runtime** | `blake3` uses `cc` at build-time for SIMD assembly; no C runtime deps. `ring` gated behind `nestgate` feature. |
| Hardcoded primal routing | **0** | All via `primal_names::*` constants + capability discovery. Self-knowledge constants (`PRIMAL_NAME`, `PRIMAL_ID`, `TOOL_NAME`) are legitimate self-identification. |

**Audit questions (refreshed):**

| Question | Answer |
|----------|--------|
| Python baselines for barraCuda CPU parity? | **Yes**: `control/validation/exp040_barracuda_cpu.py` (stats, Hill, Shannon, Simpson, Chao1, Anderson) + `control/scripts/bench_barracuda_cpu_vs_python.py` (Hill, oral PK, Shannon/Simpson/Pielou, AUC, pop MC). Rust parity: full for exp040; partial for bench suite (oral PK, Pielou, trapezoidal AUC gaps). |
| GPU benchmarks? | `gpu_parity.rs` (Hill/diversity/popPK/MM batch via wgpu), `kokkos_parity.rs` (Kokkos-modeled CPU patterns). No SciPy/LAMMPS/Galaxy direct comparisons — sovereign WGSL shaders, not framework ports. |
| What's not implemented? | **Substantially closed**: 59 scenarios cover all 10 tracks including V16 primitives (exp078-082 all have scenarios). Remaining: exp084/085 (GPU scaling) not in scenario registry, exp112-113 (dispatch) CI-only. |
| Unreviewed papers? | **2**: LTEE E2 (HOLIgraph) and E4 (macrocyclic peptides). 45/45 main-track complete. |
| Datasets? | **5** in `data/manifest.toml`, all SHA256 empty. `qs_gene_matrix` lacks fetch script. Other 4 have scripts but unverified. |

**No new gaps.** All findings unchanged from V64i. V64j wire name changes introduced zero debt.

### Wire Hygiene Sprint (V64l)

ludoSpring Tower atomic (first live validation) discovered two wire contract mismatches absorbed by healthSpring:

**bearDog `crypto.sign`**: Expects base64-encoded `"message"` param, not raw `"payload"`. healthSpring sent `{"payload": ..., "algorithm": "ed25519"}` — bearDog would reject with `"Missing required parameter: message"`.

**Fix**: `s_nest_atomic.rs` Phase 5 and `NestComposition.sign()` in `nest.rs` now send `{"message": base64_encode(data), "purpose": "..."}`. Added `base64 = "0.22"` as direct dependency.

**skunkBat `security.audit_log`**: Canonical wire method is `security.audit_log`, not `defense.audit`. healthSpring's Phase 8 and deploy graph used the wrong name.

**Fix**: `s_nest_atomic.rs` Phase 8 → `security.audit_log`. Deploy graph (`healthspring_niche_deploy.toml`) skunkBat capabilities updated. `niche.rs` stale `defense.audit` removed. Routing table updated (`defense.audit` → `security.audit`).

**plasmidBin cell.toml**: Created `graphs/healthspring_cell.toml` for cellular deployment (biomeos deploy format). Follows ludoSpring's `[[nodes]]` pattern with full Nest Atomic + Tower Atomic + compute trio.

| # | Gap | Source | Upstream Action |
|---|-----|--------|-----------------|
| 42 | Foundation Thread 10 (Provenance) is empty — healthSpring domain | Upstream directive | healthSpring: seed expression when sporeGarden structure is confirmed |

### Upstream Audit Absorption Sprint (V64n — May 14, 2026)

primalSpring ecosystem status update (May 14): plasmidBin deployment evolution complete. Tower atomic mandated as bearDog + songBird + skunkBat. barraCuda v0.4.0 released. sourDough internalization planned (v0.3.0–v0.6.0). healthSpring status: "composing" per manifest.toml.

**Findings from upstream audit:**

1. **Deploy graph Tower comments stale** — all 4 deploy-style graphs said "Tower = bearDog + songBird"; skunkBat was already present as a node but not acknowledged in Tower comments or healthspring `depends_on`.
2. **`healthspring_nest_atomic.toml` skunkBat capabilities still `defense.audit`** — V64l fix missed this graph.
3. **`healthspring_niche_deploy.toml` wire names non-canonical** — rhizoCrypt used `dag.create_session`/`dag.append_event`; loamSpine used `commit.session`/`commit.entry`; sweetGrass used `provenance.create_braid`.
4. **`routing.rs` missing `content` domain** — NestGate's CAS surface (`content.*`) had no routing entry; `ALL_CAPS` also omitted `stats`.
5. **`niche.rs` CONSUMED_CAPABILITIES stale** — legacy wire names (`dag.create_session`, `commit.session`, `provenance.create_braid`, `audit.log`, `crypto.ionic_bond`), missing `crypto.contract.*` and `content.*`.
6. **`capability_registry.toml`** — `crypto.ionic_bond` stale (now `crypto.contract.*`), DAG/braid not canonical-first, skunkBat missing `baseline.*`/`metadata.*`/`response.*`.
7. **Cargo.toml version comment** — "v0.3.13" stale (upstream barraCuda is v0.4.0).
8. **infra/plasmidBin manifest.toml** — healthSpring entry says `tests = 1014`, `V64e`, `evolution = "composing"` (stale, upstream-owned).
9. **infra/plasmidBin ports.env** — `NICHE_HEALTHSPRING` under-validates vs manifest niche primals (upstream-owned).

**Fixes applied locally:**

- All deploy graphs: Tower comments updated to include skunkBat, healthspring `depends_on` includes skunkBat
- `healthspring_nest_atomic.toml`: `defense.audit`/`defense.recon`/`defense.threat` → `security.audit_log`/`baseline.observe`/`baseline.anomaly`
- `healthspring_niche_deploy.toml`: rhizoCrypt → `dag.session.create`/`dag.event.append`/`dag.merkle.root`/`dag.merkle.verify`; loamSpine → `spine.create`/`entry.append`; sweetGrass → `braid.create`/`braid.commit`/`braid.get`; skunkBat `by_capability` → `audit`
- `healthspring_cell.toml`: skunkBat moved to Tower Atomic section, `by_capability` → `audit`
- `routing.rs`: added `"content"` → NestGate, `"stats"` to `ALL_CAPS`
- `niche.rs`: CONSUMED_CAPABILITIES updated to canonical wire names + `crypto.contract.*` + `content.*`
- `capability_registry.toml`: `crypto.contract.*` replaces `crypto.ionic_bond`; DAG/braid/audit canonical-first; content section added; skunkBat full capability surface
- `Cargo.toml`: version comment → "v0.4.0"

**Upstream items (not local debt — hand back to primalSpring):**

| # | Gap | Owner | Action |
|---|-----|-------|--------|
| 43 | plasmidBin `manifest.toml` healthSpring stale (tests=1014, V64e) | infra/plasmidBin | Update to tests=1021, V65a |
| 44 | `ports.env` NICHE_HEALTHSPRING under-validates (missing toadstool, barracuda, coralreef, petaltongue) | infra/plasmidBin | Sync with `[niches.healthspring]` in manifest.toml |
| 45 | sourDough deployment internalization: 15 healthSpring shell scripts are candidates | primals/sourDough | Map to sourdough subcommands as v0.4.0–v0.6.0 ships |

**Composing → composed blockers (all upstream/coordination):**
- Ionic bridge — **WIRED V64z**: `bonding.*` protocol (IonicContractRegistry) + `crypto.contract.*` signing layer; WS-1 substantially resolved
- BTSP transport negotiation — `FAMILY_SEED` breaks mixed deploys (Gap #20)
- Foundation Thread 10 — provenance expression pending sporeGarden (Gap #42)
- Nest live deploy — needs running primals for `s_nest_atomic` against live NUCLEUS

### Wave 17 Signal Adoption Sprint (V64o — May 16, 2026)

primalSpring Wave 17 (451 methods, 41 scenarios): Neural API Signal Elevation shipped. `ctx.dispatch()` and `ctx.announce()` on `CompositionContext` provide single-call signal dispatch and registration. 14 atomic signals defined in `graphs/signals/`. GAP-GS-015 fixed (ALL_CAPS/BTSP_EXTRA_CAPS re-exported from `composition/mod.rs`).

**Fixes applied locally:**

1. **`primal.announce` registration** — `server/registration.rs` now tries `primal.announce` (single-call, Wave 17) before falling back to legacy `lifecycle.register` + N × `capability.register`. Wire format: `{ primal_id, transport, methods, lifecycle: { state: "running" } }`.

2. **Signal dispatch in NestComposition** — `full_lifecycle()` now tries `signal.dispatch("nest.store", ...)` + `signal.dispatch("nest.commit", ...)` before falling back to the manual 5-step chain. The signal path collapses `storage.store → dag.event.append → crypto.sign → spine.create → braid.*` into two biomeOS-managed graph executions.

3. **Signal dispatch in data/provenance** — `complete_data_session()` now tries `signal.dispatch("nest.commit", ...)` via orchestrator before falling back to manual `dag.dehydrate → spine.create → braid.create` chain.

4. **451-method registry sync** — `capability_registry.toml` updated with Wave 17 entries: `[fido2]` (beardog.fido2.authenticate/discover/register), `[genetic]` (ceremony_init/finalize, derive_key, entropy_contribute), `[certificate]` (certificate.verify), `[primal_registry]` (primal.announce, primal.info), `[signals]` (all 14 atomic signals + signal.dispatch).

5. **Routing domain expansion** — `routing.rs` `ALL_CAPS` expanded with `signal`, `certificate`, `genetic`, `fido2`, `primal` domains. `capability_to_primal` maps `signal` → biomeOS, `fido2` → bearDog, `primal` → primalSpring, `certificate`/`genetic` → ecosystem.

6. **Niche CONSUMED_CAPABILITIES** — `niche.rs` updated with `signal.dispatch`, `primal.announce`, `primal.info`, `certificate.verify`.

7. **GAP-GS-015 confirmed** — `cargo check --workspace` passes clean against primalSpring HEAD.

**Foundation Threads 3+8 assessment:**
- Threads 3 (Immunology / `IMMUNO_DRUG_DISCOVERY.md`) and 8 (Human Health / `SOVEREIGN_HEALTH.md`) are documented as "active" in CHANGELOG and handoffs
- Expression artifacts (`sporeGarden/foundation/` tree, `THREAD_INDEX.toml`, expression docs) are **not present** in this workspace — they live upstream in primalSpring/sporeGarden
- `PRIMAL_GAPS.md` only tracks Thread 10 (Gap #42); Threads 3+8 are external expression responsibilities
- healthSpring's B5 (symbiont PK/PD) is the lithoSpore module candidate for Thread 3+8 content

| # | Gap | Source | Upstream Action |
|---|-----|--------|-----------------|
| 46 | Foundation Threads 3+8 expressions not in healthSpring workspace | Wave 17 directive | primalSpring: confirm sporeGarden Thread 3+8 structure; healthSpring contributes B5 lithoSpore module |
| 47 | Signal dispatch live validation pending | Wave 17 adoption | healthSpring: run `s_nest_atomic` with biomeOS signal.dispatch to validate nest.store/nest.commit signal path |

### Deep Debt Re-Audit (V64p — May 16, 2026)

Post-Wave 17 comprehensive re-audit. **All 7 categories confirmed at zero debt.**

| Category | Status | Detail |
|----------|--------|--------|
| TODO/FIXME/HACK | **0** | Zero markers in entire codebase (214 .rs files audited) |
| `unsafe` code | **0** | `#![forbid(unsafe_code)]` enforced on lib.rs + all 6 binary crate roots |
| Production mocks | **0** | All mocks in `#[cfg(test)]`. One doc-comment "Stub" label on cfg-gated GPU API (not runtime) |
| `unimplemented!`/`todo!`/`panic!` (non-test) | **0** | All 20 `panic!` calls inside `#[cfg(test)]` blocks |
| `.unwrap()`/`.expect()` (non-test) | **0** | All in `#[cfg(test)]` or doc comments |
| Files > 800 LOC | **0** | Largest file: 597 lines (`ipc/proptest_ipc.rs`) |
| Clippy pedantic+nursery | **0 warnings** | Previous 3 warnings fixed: unused param, identical match arms, missing `#[must_use]` |

**Additional fixes in V64p:**
- `tolerances.rs`: `IPC_RETRY_MAX_ATTEMPTS` constant extracted (was bare `3` in two places in `rpc.rs`)
- `routing.rs`: `"fido2"` merged into bearDog arm (clippy identical-bodies)
- `data/provenance.rs`: unused `socket` param removed from `try_signal_commit`, `#[must_use]` added to `complete_data_session`

**Audit answers (refreshed V64p):**

| Question | Answer |
|----------|--------|
| Python benchmarks for barraCuda CPU parity? | **Yes**: `control/scripts/bench_barracuda_cpu_vs_python.py` (Hill, oral PK, Shannon/Simpson/Pielou, AUC, population MC) + `control/validation/exp040_barracuda_cpu.py` (analytical: Hill, IV bolus, two-compartment, Shannon/Simpson/Pielou/Chao1, Bray-Curtis, PPG). Rust mirror: `ecoPrimal/benches/cpu_parity.rs`. Gap: timing bench does not cover Chao1, Bray-Curtis, IV bolus, PPG (only analytical baseline does). |
| GPU benchmarks? | `gpu_parity.rs` (Hill/diversity/popPK/MM batch via wgpu); `kokkos_parity.rs` (Kokkos-modeled CPU patterns — no Kokkos dependency). No Galaxy/SciPy-GPU/LAMMPS — sovereign WGSL shaders, not framework ports. |
| What's not implemented? | **Substantially closed (Wave 60)**: 59 scenarios cover all 10 tracks. All V16 primitives (exp078-082) have scenarios. Only exp084/085 (GPU scaling) and exp112-113 (dispatch parity) lack dedicated scenarios (CI-covered). Modules without inline unit tests: `certification/`, `composition/`, `gpu/sovereign.rs`, `gpu/cpu_fallback.rs`, `microbiome/anderson.rs` (covered by integration/experiment tests). |
| Unreviewed papers? | **2**: LTEE E2 (Mardikoraem & Woldring 2025 "HOLIgraph" OATP PK/PD) and E4 (Woldring Lab 2024 macrocyclic peptides). 45/45 main-track complete. |
| Datasets? | **5** in `data/manifest.toml`, all SHA256 empty. `qs_gene_matrix` lacks fetch script. Others (`mitbih`, `chembl_hill_panel`, `hmp_16s`, `geo_androgen_receptor`) scripted but unverified. |
| External deps? | All appropriate: `serde`/`serde_json` (ecosystem standard), `tracing` (structured logging), `clap` (CLI), `base64` (crypto payloads), `thiserror` (error derives). Optional: `wgpu`/`tokio`/`bytemuck` (GPU feature), `ureq` (nestgate HTTP). No Rust replacement opportunities with favorable cost/benefit. |

### Deep Debt Re-Audit #2 (V64s — May 16, 2026)

Post-Wave 20 comprehensive re-audit. **All 7 categories confirmed at zero debt.**

| Category | Status | Detail |
|----------|--------|--------|
| TODO/FIXME/HACK | **0** | Zero markers. 3 "workaround" hits in gpu/mod.rs + gpu/sovereign.rs doc comments describing f32 transcendental workarounds — not debt markers, they document the sovereign pipeline replacement path |
| `unsafe` code | **0** | Only the word "unsafe" in a doc comment in server/signal.rs ("Since we forbid unsafe...") |
| Production mocks | **0** | All mocks in `#[cfg(test)]`. Feature-gated GPU codegen stub returns `None` when `barracuda-lib` disabled — compile-time shim, not runtime mock |
| `unimplemented!`/`todo!`/`panic!` (non-test) | **0** | All 21 `panic!` calls inside `#[cfg(test)]` blocks |
| `.unwrap()`/`.expect()` (non-test) | **0** | 99 `.unwrap()` + 65 `.expect()` — all in `#[cfg(test)]` or doc comments |
| Files > 800 LOC | **0** | Largest: 597 lines (`ipc/proptest_ipc.rs`, test-only) |
| Clippy pedantic+nursery | **0 warnings** | Clean |

**No changes required.** V64r Wave 20 changes (`capability_domains()` helper, `primal.list` registry addition) introduced no debt.

---

## Summary Matrix

| # | Gap | Blocked On | healthSpring Action | primalSpring Action |
|---|-----|------------|--------------------|--------------------|
| 1 | Capability namespace | — | **Fixed V49**: aliases added | Confirm alignment |
| 2 | Ionic bridge | NestGate egress policy | **V64z**: `bonding.*` protocol wired + `crypto.contract.*` signing; `storage.egress_fence` phantom removed | NestGate: ship `content.egress` or equivalent |
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
| 17 | barraCuda lib→IPC (Level 5) | — | **V54**: reframed — 9 methods are local domain compositions, not wire gaps. **`healthspring_unibin certify`** + **`certification/`** use `CompositionContext` for generic IPC | None (V53 ask withdrawn) |
| 18 | guideStone P3 (CHECKSUMS) | — | **Fixed V55**: BLAKE3 via `primalspring::checksums::verify_manifest()`. SKIP when no manifest (honest scaffolding). | — |
| 19 | barraCuda: `stats.variance`, `stats.correlation` | — | **RESOLVED V57**: Sprint 44 added both; guideStone validates in Tier 2+3 | — |
| 20 | BTSP production mode breaks IPC | primalSpring transport | **V57**: documented, `FAMILY_SEED` workaround | Negotiate BTSP capability |
| 21 | Crypto probe schema mismatch | BearDog method spec | **V57**: documented, SKIPped in guideStone | Publish method signatures |
| 22 | Missing socket discovery (DAG/AI/commit) | Ecosystem socket std | **V57**: documented, SKIPped in guideStone | Standardize capability sockets |
| 23 | Provenance trio wire dispatch (`-32601`) | — | **RESOLVED V64j**: root cause was non-canonical method names; upstream aliases + local fixes | — |
| 24 | Songbird crypto provider discovery | Songbird startup docs | **V58**: documented, symlink workaround | Document songbird startup deps |
| 25 | petalTongue proprioception in server mode | petalTongue server | **V58**: documented, non-blocking | Add synthetic proprioception in server mode |
| 26 | NestGate not in default PRIMAL_LIST | gate_nucleus.sh (was composition_nucleus.sh, now archived) | **V58**: documented, PRIMAL_LIST override | Add nestgate to defaults |
| 27 | socat dependency undocumented | Lib docs | **V58**: `nc -U` shim provided | Document dep or add fallback |
| 28 | plasmidBin cell TOML stale | Convergence wiring | **Fixed V64f**: compute trio added | — |
| 29 | plasmidBin niche under-specced | Convergence wiring | **Fixed V64f**: promoted to `full` | — |
| 30 | precision.route blurb/API divergence | Wire contract review | **V64f**: wired to LIVE_SCIENCE_API | Reconcile blurb |
| 31 | lithoSpore B5 module ingestion | LTEE handoff | **V64f**: packaged | lithoSpore: ingest module |
| 32 | NestComposition testing blocked by trio dispatch | Provenance elevation | **RESOLVED V64j**: root cause was Gap #23 (non-canonical method names); fixed by alias shipping + local canonical names | — |
| 33 | Dataset SHA256 checksums empty | Provenance audit | **V64g**: documented | Populate on dataset fetch |
| 34 | `content.*` vs `storage.*` — by design | — | **RESOLVED V64j**: CAS vs blob, both nestGate, confirmed by biomeOS registry | — |
| 35 | loamSpine method naming clarified | — | **RESOLVED V64j**: `entry.append` canonical, aliases shipped | — |
| 36 | Nest Atomic live exercises unblocked | — | **RESOLVED V64j**: upstream aliases + local canonical names | — |
| 37 | NestComposition `"data"` domain misroute | Internal wire review | **Fixed V64h**: `"storage"` domain | — |
| 38 | ~30 Python baselines w/o scenarios | — | **CLOSED Wave 60**: 59 scenarios, exp040+B5 absorbed | — |
| 39 | LTEE E2+E4 papers queued | Paper queue audit | **V64i**: documented | — |
| 40 | Dataset SHA256 + fetch gaps | Data audit | **V64i**: documented | — |
| 41 | No GPU parity benchmarks | Benchmark audit | **V64i**: N/A (sovereign WGSL) | — |
| 42 | Foundation Thread 10 (Provenance) empty | Upstream directive | **V64l**: documented, seed when sporeGarden confirmed | — |
| 43 | plasmidBin manifest.toml healthSpring stale | infra/plasmidBin | **V64n**: documented | Update tests=1021, V65a |
| 44 | ports.env NICHE_HEALTHSPRING under-validates | infra/plasmidBin | **V64n**: documented | Sync with manifest niche |
| 45 | sourDough shell script internalization | primals/sourDough | **V64n**: 15 scripts mapped | Map to sourdough v0.4.0+ |
| 46 | Foundation Threads 3+8 expressions missing | Wave 17 directive | **V64o**: B5 lithoSpore candidate ready | Confirm sporeGarden structure |
| 47 | Signal dispatch live validation | Wave 17 adoption | **V64o**: signal paths wired, pending live test | Run with biomeOS signal.dispatch |
