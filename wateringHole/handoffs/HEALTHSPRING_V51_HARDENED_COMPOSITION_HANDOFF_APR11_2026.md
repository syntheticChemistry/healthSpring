<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V51 — Hardened Composition Patterns Handoff

**Date**: 2026-04-11
**Version**: V51 (0.10.0 → ecoBin 0.8.0)
**Phase**: Composition hardening — upstream IPC patterns adopted from primalSpring
**From**: healthSpring
**To**: barraCuda, toadStool, primalSpring, biomeOS, neuralSpring, coralReef, ecosystem

---

## Context

V50 established the composition evolution path (Python → Rust → IPC dispatch →
NUCLEUS composition). V51 hardens the IPC and deployment surface by adopting
upstream patterns from `primalSpring`, `plasmidBin`, and `infra/wateringHole/`
standards (PRIMAL_IPC_PROTOCOL v3.1, CAPABILITY_WIRE_STANDARD, DEPLOYMENT_VALIDATION).

The key shift: healthSpring now has a production-grade primal binary with TCP
transport, domain symlink discovery, BTSP handshake readiness, typed IPC clients,
and structured discovery — all aligned with the hardened composition patterns
that primalSpring defined for the ecosystem.

---

## V51 Changes

### IPC Transport Hardening

- **TCP JSON-RPC listener**: `healthspring_primal serve --port 9860` binds a
  newline-delimited JSON-RPC 2.0 listener on `0.0.0.0:{port}`. Also reads
  `HEALTHSPRING_PORT` env var. Aligned with `PRIMAL_IPC_PROTOCOL.md` v3.1
  TCP fallback transport and `plasmidBin/healthspring/metadata.toml`
  `tcp_port_default = 9860`.
- **`server` subcommand alias**: `healthspring_primal server` = `serve` per
  UniBin standard.
- **Domain symlink**: On bind, creates `health.sock` → `healthspring-{family}.sock`
  in the socket directory. Cleaned on shutdown. Enables capability-domain
  discovery per IPC protocol v3.1 §Domain Symlinks.
- **Generic connection handler**: `handle_lines<R,W>` supports both `UnixStream`
  and `TcpStream` transparently.

### Capability Wire Standard Compliance

- **`methods: [string]`**: `capabilities.list` response now includes a top-level
  flat `methods` array containing all capability method names, per
  `PRIMAL_CAPABILITY_WIRE_STANDARD_APR08_2026.md`.
- **`identity.get`**: New JSON-RPC method returning primal metadata: name, version,
  domain, license, architecture (`ecoBin`), composition model (`nucleated`),
  particle profile (`neutron_heavy`), proto-nucleate reference.
- **`health.check`**: Lightweight probe returning status, primal name, version,
  domain, uptime — distinct from `health.readiness` (which gates on science
  dispatch) and `health.liveness` (unconditional alive check).
- **`status` field in `health.readiness`**: Returns `"healthy"` or `"degraded"`
  alongside the existing `ready` boolean.

### Capability Registration Hardening

- **LOCAL vs ROUTED split**: `LOCAL_CAPABILITIES` (served in-process) and
  `ROUTED_CAPABILITIES` (proxied to canonical providers like `toadstool`,
  `squirrel`, `nestgate`) are separated with explicit `served_locally` and
  `canonical_provider` metadata in `capability.register` payloads.
- **`provided_capabilities()`**: Structured output in `capabilities.list`
  distinguishing local from routed capabilities with provider attribution.

### IPC Client Infrastructure

- **`ipc/btsp.rs`**: BTSP (BearDog Transport Security Protocol) client handshake
  module. `BtspMessage` enum for 4-step handshake, `family_seed_from_env()`,
  `client_hello()`, pure-Rust base64 decoder. Ready for when BearDog exposes
  BTSP endpoints.
- **`ipc/client.rs`**: Typed `PrimalClient` (health probes with method fallback
  chains, capability queries, typed RPC calls) and `InferenceClient` (Squirrel
  discovery, `inference.complete`, `inference.embed`, `inference.models`).
  Replaces raw `rpc::send` for cross-primal communication.
- **`ipc/discover.rs`**: Structured `DiscoveryResult` (socket path + source) and
  `DiscoverySource` enum (EnvOverride, CapabilityProbe, WellKnownPath,
  XdgSocket, NotFound). Typed `discover_compute()`, `discover_data()`,
  `discover_inference()`, `discover_orchestrator()` functions.

### Bug Fixes

- **`CoralReefDevice` → `SovereignDevice`**: Fixed 5 occurrences in
  `gpu/sovereign.rs` (upstream barraCuda API rename).
- **`plasmidBin/manifest.lock`**: healthspring version 0.7.0 → 0.8.0 (resolves
  deployment version drift).
- **Provenance doc link**: Fixed broken intra-doc link in `provenance/mod.rs`.

---

## Validation State

- **976 tests**: 845 lib + 33 forge + 51 toadstool + 12 integration + 20 IPC +
  5 WFDB + 1 exp + 9 doc-tests. All passing.
- **89 experiment binaries**: 83 science + 6 composition Tier 4. All
  `ValidationHarness` → `h.exit()`.
- **Zero**: `unsafe`, `#[allow()]`, `TODO`, `FIXME`, clippy warnings, fmt diffs.

---

## For primalSpring

1. **Patterns adopted**: healthSpring now mirrors primalSpring's `niche.rs`
   LOCAL/ROUTED capability registration, `ipc/client.rs` PrimalClient pattern,
   `ipc/btsp_handshake.rs` BTSP module, and `ipc/discover.rs` structured
   discovery. These patterns are validated and working in healthSpring's
   composition experiments.

2. **Composition validation as Tier 4**: exp112–117 validates that IPC dispatch
   reproduces direct Rust with zero divergence. This pattern should become a
   `composition_validation` crate in primalSpring that all springs can depend on.

3. **`identity.get` implemented**: healthSpring returns composition metadata
   (nucleated, neutron_heavy, proto-nucleate reference). Recommend standardizing
   the response shape across all primals.

4. **Discovery method naming**: healthSpring still uses dual fallback
   (`discovery.find_by_capability` + `net.discovery.find_by_capability`).
   Please confirm canonical Songbird method name.

---

## For biomeOS

1. **TCP transport ready**: healthSpring can now serve on TCP for environments
   where UDS is not available (containers, cross-host). Port configurable via
   `--port` or `HEALTHSPRING_PORT`.

2. **Domain symlink**: `health.sock` enables capability-domain discovery without
   probing every socket file.

3. **Deploy graph unchanged**: `graphs/healthspring_niche_deploy.toml` is still
   the canonical deploy definition. The binary now supports all transports
   biomeOS might use.

---

## For barraCuda / coralReef

1. **`SovereignDevice` API confirmed**: healthSpring's `gpu/sovereign.rs` now
   references `barraCuda::device::SovereignDevice` (previously `CoralReefDevice`).
   The sovereign-dispatch feature flag is ready for coralReef device availability.

2. **TensorSession still blocking**: Local WGSL shaders retained until
   `TensorSession` API ships. Same request as V50.

---

## Composition Patterns for Ecosystem Adoption

### 1. TCP + UDS Dual Transport

Every primal should support both Unix domain sockets (primary, low-latency) and
newline-delimited TCP (fallback, containers, cross-host). healthSpring's pattern:
spawn TCP listener thread if `--port` is provided, share state via `Arc`.

### 2. Domain Symlink Discovery

On bind, create `{domain}.sock` → `{primal}-{family}.sock`. biomeOS and other
primals can discover capabilities by domain without probing all sockets.

### 3. LOCAL vs ROUTED Capability Registration

Primals should distinguish capabilities they serve in-process from those they
proxy to canonical providers. This enables biomeOS to make informed routing
decisions (local = fast, routed = may involve network hop).

### 4. Typed IPC Clients

Replace raw `rpc::send` with typed `PrimalClient` wrappers that handle method
fallback chains, health probe ordering, and structured error classification.

### 5. Structured Discovery

Use `DiscoveryResult` + `DiscoverySource` to track *how* a primal was found.
This aids debugging and enables observability of the discovery path.

---

## What Springs Can Harvest

| Pattern | Location | Status |
|---------|----------|--------|
| TCP listener | `server.rs` `accept_tcp()` | Production |
| Domain symlink | `server.rs` `create_domain_symlink()` | Production |
| `identity.get` | `capabilities.rs` `handle_identity_get()` | Production |
| `health.check` | `routing.rs` `handle_health_check()` | Production |
| LOCAL/ROUTED split | `capabilities.rs` constants | Production |
| BTSP handshake | `ipc/btsp.rs` | Ready (BearDog pending) |
| PrimalClient | `ipc/client.rs` | Production |
| InferenceClient | `ipc/client.rs` | Ready (Squirrel pending) |
| DiscoveryResult | `ipc/discover.rs` | Production |
