# healthSpring V61 — Upstream Primal Teams Handoff

**Date:** May 9, 2026  
**healthSpring:** V61 (eukaryotic / UniBin)  
**primalSpring:** v0.9.25 (pinned in `ecoPrimal/Cargo.toml`)  
**Architecture:** IPC-first default (`barracuda-lib` optional); barraCuda math primitives validated via JSON-RPC parity where wired

## Purpose

This document communicates healthSpring’s primal dependencies, wire-ready surface expectations, capability probe patterns, and evolution asks to each upstream primal team. It also describes composition patterns for NUCLEUS deployment and neuralAPI-style serving via biomeOS.

Canonical self-knowledge lives in `ecoPrimal/src/niche.rs` (dependencies, advertised/consumed capabilities, proto-nucleate validation list, barraCuda IPC migration inventory, cost estimates). Socket-prefix hints are in `ecoPrimal/src/primal_names.rs`. Capability-to-provider naming for routing tables is in `ecoPrimal/src/composition/routing.rs`.

## Per-Primal Usage & Asks

### barraCuda (tensor, stats)

**Code paths:** `ecoPrimal/src/ipc/barracuda_client.rs` (typed JSON-RPC client), `ecoPrimal/src/math_dispatch.rs` (mean/std_dev IPC vs library vs pure fallback), `ecoPrimal/src/composition/context.rs` (`HealthCompositionContext` → `tensor` capability), `ecoPrimal/src/certification/composition.rs` (parity validation).

**Discovery:** `BarraCudaClient::discover()` tries capability `stats` first (`discover_by_capability_public("stats")`), then name fallback `barracuda`. Primal client identity string is `"barracuda"`.

**Wire-ready methods consumed**

| Method | Request schema (healthSpring) | Response extraction |
|--------|------------------------------|---------------------|
| `stats.mean` | `{"data": [f64, …]}` | JSON-RPC `result` as `f64` (flex keys also accepted in `HealthCompositionContext`: `mean`, `result`) |
| `stats.std_dev` | `{"data": [f64, …]}` | same pattern (`std_dev`, `result`) |
| `stats.variance` | `{"data": [f64, …]}` | sample variance parity vs local \( \sigma^2 \) from std_dev (`variance`, `result`) |
| `stats.correlation` | `{"x": [f64, …], "y": [f64, …]}` | Pearson \(r\); guideStone uses self-correlation \(r = 1.0\) (`correlation`, `result`) |
| `rng.uniform` | `{"n", "min", "max", "seed"}` | array of `f64` at top-level `result` or nested |
| `rng.normal` | `{"n", "mean", "std_dev", "seed"}` | same array extraction as uniform |

**Parity / certification:** `validate_barracuda_math_ipc` and `validate_primal_proof` call `primalspring::composition::validate_parity` for `stats.mean`, `stats.std_dev`, `stats.variance`, `stats.correlation` against `math_dispatch` local values and `IPC_ROUND_TRIP_TOL`. With default features (no `barracuda-lib`), `mean`/`std_dev` route through IPC when an ecobin is discoverable; domain statistics (Hill, Shannon, Simpson, etc.) stay **local compositions** and are not asserted as barraCuda IPC methods.

**RNG:** `BarraCudaClient` implements `rng_uniform` and `rng_normal`. Stochastic science in `rng.rs` still prefers the deterministic LCG path (`barracuda::rng` when `barracuda-lib`, inlined when not)—not automatically switched to IPC.

**Asks**

- **Maintain** `stats.variance` (sample variance, Bessel \(N-1\)) and `stats.correlation` (Pearson) on the dispatch table; healthSpring treats Sprint 44 / primalSpring v0.9.17+ as the baseline and regression-tests both.
- **`rng.normal`:** Client is implemented; please confirm stable JSON-RPC behavior and batch result shape matches `rng.uniform` (top-level array vs `{ "result": [...] }`) so healthSpring can add optional IPC parity tests when desired.

---

### coralReef (shader)

**Used for:** WGSL compilation on the sovereign GPU path and generic shader IPC.

**Code paths:** `ecoPrimal/src/ipc/shader_dispatch.rs` — `shader.compile` (arbitrary params blob), `shader.validate` with `{"source": "<wgsl>"}`, `capability.list` filtered for `shader.*`. Discovery via `HEALTHSPRING_SHADER_SOCKET` / `HEALTHSPRING_SHADER_PRIMAL` / capability probe `shader`. `ecoPrimal/src/gpu/sovereign.rs` documents pipeline order: barraCuda WGSL → **coralReef compile** → **toadStool dispatch** → native binary (`sovereign-dispatch` + `gpu` + `barracuda-lib`).

**Feature-gated:** `gpu` / `sovereign-dispatch` features chain coralReef-related behavior; shader IPC helpers compile without GPU when used from dispatch module.

**Ask:** Expose `shader.compile` (and related `shader.*`) consistently in primal manifests / capability registry so `connect_by_capability("shader")` and directory probes succeed in headless NUCLEUS compositions.

---

### toadStool (compute)

**Used for:** GPU/compute job orchestration over IPC — `compute.dispatch.submit` / `compute.dispatch.result` / `compute.dispatch.capabilities` (`ecoPrimal/src/ipc/compute_dispatch.rs`). Sovereign GPU path targets toadStool after coralReef (`gpu/sovereign.rs`). Tower Atomic patterns reference `compute.dispatch.submit` (`ipc/tower_atomic.rs`).

**Local crate:** Workspace member `toadstool/` wraps deployment-facing compute patterns; niche lists optional dependency `toadstool` with capability `compute`.

**Ask:** None blocking — continue stabilizing `compute.dispatch.*` response shapes (`job_id`, error propagation).

---

### BearDog (security, crypto)

**Used for:** `crypto.hash` (probe uses `{"data": "<utf8 string>", "algorithm": "blake3"}`), `crypto.sign` (probe uses `{"data": "guidestone-attestation"}`). Niche consumes `crypto.hash`, `crypto.sign`, `crypto.ionic_bond`.

**Validated in:** `certification/composition.rs` → `probe_capability` on capability domain **`security`** (not `crypto` key).

**Ask:** Publish authoritative JSON-RPC parameter schemas for `crypto.hash` / `crypto.sign` (guideStone currently hits protocol/schema mismatches per `docs/PRIMAL_GAPS.md` §21 — probes SKIP). Longer term: `crypto.ionic_bond` / family verification for proto-nucleate ionic bridge (gap §2).

---

### Songbird (discovery, net.discovery)

**Used for:** Primal discovery in full `CompositionContext::discover()` (primalSpring **tier 1**): Songbird client on capability `discovery`, then `ipc.resolve` with `{"primal_id": <canonical provider id>}` per capability. healthSpring’s certification path uses **`CompositionContext::from_live_discovery_with_fallback()`**, which **skips tier 1** and uses tiers **2–5** only (local capability connect + TCP fallback)—so Songbird is optional for guideStone unless callers switch to `discover()`.

**Also:** `ipc/tower_atomic.rs` patterns for capability lookup; dual-method fallback `discovery.find_by_capability` vs `net.discovery.find_by_capability` (see gaps §3).

**Ask:** Standardize canonical discovery method names with primalSpring; document crypto-provider startup requirements (gaps §24).

---

### NestGate (storage)

**Used for:** `storage.store` with `{"key", "value}` and `storage.retrieve` with `{"key"}`; round-trip checks accept `value`, `data`, or `result` in the payload (`certification/composition.rs`). Niche: optional dependency, capability `storage`; consumed capabilities include `storage.egress_fence`.

**Cargo:** `nestgate` feature on `healthspring-barracuda` enables optional `ureq` dependency (HTTP/data-tier paths).

**Ask:** Consider ecosystem default inclusion of NestGate in composition scripts (`PRIMAL_LIST`) so storage probes don’t SKIP (gaps §26). Implement/document `storage.egress_fence` for ionic bridge alignment (gaps §2).

---

### rhizoCrypt (dag)

**Used for:** DAG provenance — `dag.session.create`, `dag.event.append`.

**IPC client:** `ecoPrimal/src/ipc/provenance/rhizocrypt.rs`

| Method | Params (healthSpring) |
|--------|------------------------|
| `dag.session.create` | `{"experiment": "<string>"}` |
| `dag.event.append` | `{"session_id": "<string>", "event": "<string>", "data": <JSON value>}` |

**certification note:** `validate_manifest_capabilities` probes `dag.event.append` with **`session_id` omitted** (`{"event","data"}` only) as a structural probe when live session creation is unavailable.

**Routing:** capability domain **`dag`** → primal name prefix `rhizocrypt` (`composition/routing.rs`).

**Ask:** Confirm production servers accept the **`session_id` + `event` + `data`** shape for `dag.event.append`. Register capability-keyed sockets so `connect_by_capability("dag")` succeeds (gaps §22–23).

---

### loamSpine (commit, ledger, merkle)

**Used for:** Ledger lines — `commit.create`, `ledger.append`.

**IPC client:** `ecoPrimal/src/ipc/provenance/loamspine.rs`

| Method | Params (healthSpring) |
|--------|------------------------|
| `commit.create` | `{"experiment": "<string>", "data": <JSON value>}` |
| `ledger.append` | `{"commit_id": "<string>", "entry": <JSON value>}` |

**Routing:** domains `commit`, `ledger`, `spine`, `merkle` → `loamspine`.

**Ask:** Confirm `commit.create` responses surface a **`commit_id`** (string) consumable by `ledger.append` in the JSON-RPC result object healthSpring expects downstream. Align empty-response behavior on UDS with JSON-RPC framing (gaps §23).

---

### sweetGrass (braid, attribution)

**Used for:** `braid.create`, `braid.commit`.

**IPC client:** `ecoPrimal/src/ipc/provenance/sweetgrass.rs`

| Method | Params (healthSpring) |
|--------|------------------------|
| `braid.create` | `{"experiment": "<string>"}` |
| `braid.commit` | `{"braid_id": "<string>", "data": <JSON value>}` |

**certification note:** `CompositionContext::call` uses capability key **`commit`** for both `braid.create` and `braid.commit` (see `certification/composition.rs`)—routing table maps `braid` / `attribution` → sweetGrass for name resolution, but live probes follow the `commit` capability bucket as wired in certification.

**Ask:** Confirm `braid.create` returns a **`braid_id`** field usable by `braid.commit`. Capability socket registration for `commit` / `braid` discovery (gaps §22–23).

---

### Squirrel (inference, model)

**Used for:** `inference.complete` — `{"prompt": "…", "max_tokens": 1}`; `inference.embed` — `{"text": "…"}`. Probes use capability domain **`ai`** in certification (matches primalSpring routing where inference endpoints are grouped under the `ai` client bucket).

**Discovery:** `ipc/discover.rs` tries `model` capability first, then **`inference`** if not found.

**Ask:** None blocking for structural probes; canonical `model.*` vs `inference.*` naming remains an ecosystem coordination item (gaps §4). Optional Squirrel node in deploy graph (`healthspring_niche_deploy.toml`, gaps §9).

---

### petalTongue (visualization)

**Used for:** Visualization push API — DataChannel scenarios, IPC push client under `ecoPrimal/src/visualization/`. Capability announcements via `visualization::capabilities::announce_all`.

**Feature-gated:** Behind visualization paths tied to optional ecosystem composition (see visualization modules); not required for core certification tiers.

**Ask:** Headless `server` mode proprioception payload (`proprioception.get`) for CI monitoring (gaps §25).

---

### biomeOS (orchestration, lifecycle)

**Used for:** Orchestrator registration and lifecycle heartbeat from `healthspring_primal`.

**Code paths:** `ecoPrimal/src/bin/healthspring_primal/server/registration.rs`

- Resolves orchestrator via `ipc::socket::orchestrator_socket()` → `resolve_socket_dir()` joined with `biomeOS.sock` unless `BIOMEOS_ORCHESTRATOR_SOCKET` overrides.
- **`lifecycle.register`:** `{"name", "socket_path", "pid"}` (primal name + UDS path).
- **`capability.register`:** domain + per-capability registrations with optional `semantic_mappings`, `canonical_provider` for routed caps.
- **`lifecycle.status`:** heartbeat every 30s with `requests_served` counter.
- **Songbird announcement:** `announce_to_songbird` delegates to `visualization::capabilities::announce_all` when discovery is reachable.

**Ask:** Document the **neuralAPI gateway** pattern for HTTP → JSON-RPC translation and how `capability.discover` / Neural API tier interacts with springs that only expose UDS — downstream springs rely on primalSpring’s tier 2–5 behavior when Tower is absent.

---

## NUCLEUS Composition Patterns

**Composition entrypoints**

- **`HealthCompositionContext::from_live_discovery_with_fallback()`** wraps `primalspring::composition::CompositionContext::from_live_discovery_with_fallback()` — builds capability-keyed clients using **tiers 2–5** only: `connect_by_capability` for each capability in primalSpring’s fallback table, then **TCP** fallback using `{PRIMAL}_PORT` env overrides and defaults (`PRIMALSPRING_HOST` host string). Does **not** call Songbird tier 1; use **`CompositionContext::discover()`** for full tier 1–5 escalation when Tower/Songbird is available.

**healthSpring bind-side socket directory (four tiers)** — independent from primalSpring discovery tiers; used for **this primal’s listening socket** (`ecoPrimal/src/ipc/socket.rs` → `resolve_socket_dir`):

1. `BIOMEOS_SOCKET_DIR`
2. `$XDG_RUNTIME_DIR/biomeos/`
3. `$HOME/.cache/biomeos/` (Linux) or `~/Library/Caches/biomeos/` (macOS)
4. `/tmp/biomeos` (warn logged)

Instance socket: `HEALTHSPRING_SOCKET` override, else `{resolve_socket_dir}/{PRIMAL_NAME}-{family}.sock` with `HEALTHSPRING_FAMILY_ID` (default `default`).

**Capability-keyed `call()`**

- Inner context: `CompositionContext::call(capability_domain, method, params)` routes to the discovered client for that domain (e.g. `"tensor"` + `"stats.mean"`).

**Skip vs fail on IPC errors**

- `certification/composition.rs` — `skip_or_fail`: `IpcError::is_connection_error()` or `is_protocol_error()` → **SKIP**; else **FAIL**.

**Structured validation**

- **`ValidationResult`** from primalSpring (`section`, `check_bool`, `check_skip`, `finish`, `exit_code` / `exit_code_skip_aware`).
- **`ScenarioMeta` / `Scenario` registry** — `ecoPrimal/src/validation/scenarios/`: tracks **`Track`** (`PkPd`, `Microbiome`, `Biosignal`, `Endocrine`, `Comparative`, `Discovery`, `Composition`, `Toxicology`) and **`Tier`** (`Rust`, `Live`, `Both`).

**Certification organelle**

- **`healthspring_barracuda::certification::certify(max_tier)`** (`ecoPrimal/src/certification/mod.rs`): **`MAX_TIER = 3`**.
  - Tier 1: bare properties + local domain science.
  - Tier 2: liveness for `tensor`, `security`, `storage`, `dag`, `commit`; barraCuda math IPC parity; manifest capability probes.
  - Tier 3: primal proof (repeat generic stats parity).
  - If **no** primals alive at Tier 2 entry (`validate_liveness` → 0), returns early after Tier 1 (bare summary).
- **Exit semantics (UniBin):** `healthspring_unibin certify` uses `ValidationResult::exit_code()` from primalSpring — **0** if `all_passed()`, **1** otherwise (skip-only runs may still exit 1 when `passed == 0`). Use `exit_code_skip_aware()` where CI must distinguish all-skipped (**2**) vs failed (**1**).

---

## neuralAPI / biomeOS Deployment

**`healthspring_primal` binary** (`ecoPrimal/src/bin/healthspring_primal/main.rs`)

- **JSON-RPC 2.0** over **Unix domain socket** (primary path documented in module docs).
- **Optional TCP:** `--port` or `HEALTHSPRING_PORT` for newline-delimited JSON-RPC.
- **UniBin alignment:** subcommands `serve`/`server`, `version`, `capabilities`.
- On serve: registers with biomeOS orchestrator (`registration.rs`), announces capabilities, heartbeat thread.

**`healthspring_unibin`** (`ecoPrimal/src/bin/healthspring/main.rs`)

- Consolidates `certify`, `validate`, `serve`, `status`, `version`.
- **`serve`:** Currently prints guidance to run **`healthspring_primal serve`** — **full server module extraction is planned** (explicit note in source).

Exposes health-domain and infrastructure capabilities for NUCLEUS composition (see primal binary `capabilities` module and niche `CAPABILITIES`).

---

## For Downstream Springs

- **Certification organelle:** Library `certification::certify(max_tier)` for self-validating absorption (UniBin + legacy `healthspring_guidestone` delegate).
- **`validation/scenarios/`:** Registry with **`Track`** taxonomy + **`Tier`** classification; each scenario runs against `CompositionContext`.
- **`composition/context.rs`:** `HealthCompositionContext` wraps primalSpring with typed **`tensor`** accessors (`stats_*`) and discovery constructors.
- **`ipc/provenance/{rhizocrypt,loamspine,sweetgrass}.rs`:** Thin JSON-RPC wrappers for the provenance trio.
- **`fossilRecord/`:** Experiments and archived handoffs retained per interstadial policy (see `wateringHole/handoffs/archive/`).
- **IPC-first default features:** Default build omits `barracuda-lib`; optional library enables upstream barraCuda types for GPU/sovereign paths.
- **`Cow<'static, str>`:** Used in visualization topology structs (`visualization/scenarios/topology.rs`) to reduce allocations for static scenario labels.
- **Four-tier bind socket resolution** plus **`HEALTHSPRING_SOCKET`** explicit override for sovereign deployments.

---

## Gaps Found During Evolution

Source: `docs/PRIMAL_GAPS.md` (May 8, 2026 baseline; matrix updated through V58). Items **still open or awaiting upstream** include:

| Area | Status |
|------|--------|
| **Ionic bridge / egress policy** | Blocked on BearDog (`crypto.ionic_bond`, family verification) + NestGate (`storage.egress_fence`) |
| **Songbird discovery naming** | Dual fallback in healthSpring; canonical `discovery.*` vs `net.discovery.*` pending ecosystem agreement |
| **Inference namespace** | healthSpring accepts `model.*` + `inference.*`; canonical namespace pending primalSpring/Squirrel |
| **BTSP** | Client module present; production `FAMILY_SEED` breaks non-BTSP primals — transport negotiation needed (gap §20); BTSP server endpoint on BearDog pending (§10) |
| **BearDog probe schemas** | Align JSON params with live server — reduces SKIP noise (§21) |
| **Capability-keyed sockets** | DAG / AI / commit domains often undiscoverable via standard dirs — registration convention needed (§22) |
| **Provenance trio UDS responses** | Empty/malformed JSON-RPC responses under some composition launches (§23) |
| **Songbird crypto provider discovery** | Startup/docs — workaround via socket naming (§24) |
| **petalTongue proprioception** | Server mode lacks fps-like fields (§25) |
| **NestGate in default PRIMAL_LIST** | Composition scripts may omit NestGate unless overridden (§26) |
| **socat vs `nc -U`** | Document dependency or standardize fallback (§27) |

**Recently resolved / reframed (for context):** Capability vocabulary aliases (§1), readiness gating (§5), resilience wiring (§6), YAML manifest breadth (§7), deploy graph metadata (§8), typed IPC + deploy validation experiments (§11–12), live IPC parity experiments (§13), barraCuda domain-method IPC migration narrative (§17), `stats.variance` / `stats.correlation` wire availability (§19).

---

*Generated from repository sources under `/home/irongate/Development/ecoPrimals/springs/healthSpring` — correlate line-level behavior with the cited paths when integrating.*
