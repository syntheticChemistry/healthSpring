# Changelog

All notable changes to healthSpring are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses internal versioning (V-series) for development milestones.

## V53 — 2026-04-17 — Composition Parity (Live IPC)

### Added
- **Tier 4 live IPC experiments**: Three new composition experiments that exercise the
  full Unix socket JSON-RPC wire path against a live healthSpring primal server:
  - `exp119_composition_live_parity` — science dispatch via IPC vs direct Rust
    (Hill, compartment PK, AUC, Shannon, Anderson). Graceful skip when primal offline.
  - `exp120_composition_live_provenance` — provenance trio round-trip over IPC
    (session lifecycle, Merkle root, commit, braid).
  - `exp121_composition_live_health` — NUCLEUS health probes over IPC
    (liveness, readiness, capability.list, identity.get, niche science dispatch).
- `niche::COMPOSITION_EXPERIMENTS` — centralized registry mapping all 10 composition
  experiments to their validation tier (tier3/tier4).
- `niche::PROTO_NUCLEATE_VALIDATION_CAPABILITIES` — 10 IPC methods mirrored from
  `healthspring_enclave_proto_nucleate.toml` manifest (Level 5 readiness).
- `niche::BARRACUDA_IPC_MIGRATION` — 12 library→IPC call site mappings for Level 5
  primal proof (barraCuda library deps must become IPC calls).
- **`math_dispatch` module**: Centralizes all 11 non-RNG `barracuda::` call sites.
  Behind `--features primal-proof`, wire-ready methods (`stats.mean`, `stats.std_dev`)
  route through barraCuda ecobin IPC. Falls back to library when offline.
- **`BarraCudaClient`** (`ipc/barracuda_client.rs`): Typed IPC client for barraCuda's
  JSON-RPC surface (`stats.mean`, `stats.std_dev`, `rng.uniform`, `health.liveness`).
- **`primal-proof` feature**: Feature flag for Level 5 IPC routing in `math_dispatch`.
- **`exp122_primal_proof_barracuda_parity`**: Level 5 validation — `math_dispatch`
  known-values, `BarraCudaClient` IPC vs local, wire-pending inventory check.
- PRIMAL_GAPS.md §17 — barraCuda lib→IPC gap documented with migration plan.
  9/11 methods pending barraCuda wire handlers.
- `#![forbid(unsafe_code)]` applied directly to `ecoPrimal/src/lib.rs` crate root.
- Provenance records for exp119–122 in `records_infra.rs` (track: composition).

### Changed
- **`ValidationSink` refactored**: `Box<dyn ValidationSink>` replaced with enum dispatch
  (`ValidationSink::{Tracing, Silent, Collecting}`) — stadial zero-dyn compliance.
- **`ServerError` typed enum**: `cmd_serve` returns `ServerError` instead of `String`.
- **`TrioError` typed enum**: `capability_call`/`resilient_capability_call` in provenance
  return `TrioError` instead of `String`.
- **Capability routing by domain**: `ROUTED_CAPABILITIES` maps to capability domains
  (`compute`, `shader`, `storage`, `inference`) instead of hardcoded primal names.
- CWRES bound in exp075 sourced from `tolerances::CWRES_MEAN` (was hardcoded `3.0`).
- barraCuda version comment updated to v0.3.12 (workspace current).
- ecoBin 0.9.0 harvested to `infra/plasmidBin/healthspring/` (3.2 MB static-PIE x86_64-musl).

### Fixed
- Removed `TracingSink`, `SilentSink` type exports (replaced by `ValidationSink` enum).
- Redundant `CollectingSink` import in harness tests.

## V52 — 2026-04-11 — Composition Validation

### Added
- **Tier 5 deploy graph validation**: `exp118_composition_deploy_graph_validation` (99 checks)
  validates fragment metadata, node presence, bonding policy, capability surface coverage,
  and Squirrel optionality against proto-nucleate expectations.
- `bench` CI job — compiles all benchmarks on every PR (regression gate).
- `barracuda-ops` feature tests run on every PR (GPU code coverage without full GPU hardware).

### Changed
- `PrimalClient.call()` upgraded from `rpc::try_send` to `rpc::resilient_send` (retry + backoff
  by default). New `try_call()` for single-attempt scenarios.
- `handle_primal_forward` in routing.rs migrated from raw `rpc::resilient_send` to typed
  `PrimalClient` (resilient default, structured error reporting).
- `cargo llvm-cov` expanded from `--lib` to full workspace (lib + integration tests),
  `--fail-under-lines 90`.
- `control/tolerances.py` docstring updated: documented as intentional subset of `tolerances.rs`
  (Rust-only constants deliberately omitted — no Python consumer).

### Fixed
- `clippy::match_wildcard_for_single_variants` in `gpu/sovereign.rs` (`_ => "unknown"` →
  explicit `GpuOp::HillSweep { .. } => "HillSweep"`).

## V51 — 2026-04-11 — Hardened Composition Patterns

### Added
- TCP JSON-RPC listener via `--port` flag or `HEALTHSPRING_PORT` env var (newline-delimited
  JSON-RPC 2.0 over TCP, aligned with `PRIMAL_IPC_PROTOCOL.md` v3.1).
- `server` subcommand alias for `serve` (UniBin standard).
- `identity.get` JSON-RPC method returning primal metadata (name, version, domain, license,
  architecture, composition model, particle profile, proto-nucleate reference).
- `health.check` JSON-RPC method for lightweight health probe (status, primal, version,
  domain, uptime).
- `methods: [string]` top-level array in `capabilities.list` response per
  `PRIMAL_CAPABILITY_WIRE_STANDARD_APR08_2026.md`.
- `LOCAL_CAPABILITIES` and `ROUTED_CAPABILITIES` constants with `served_locally` and
  `canonical_provider` metadata in capability registration payloads.
- `provided_capabilities()` structured output in `capabilities.list` (local vs routed).
- Domain symlink (`health.sock`) created on bind, cleaned on shutdown (capability-domain
  discovery per `PRIMAL_IPC_PROTOCOL.md` v3.1).
- `ipc/btsp.rs` — BTSP (BearDog Transport Security Protocol) client handshake module:
  `BtspMessage` enum, `family_seed_from_env()`, `client_hello()`, pure-Rust base64 decoder.
- `ipc/client.rs` — Typed `PrimalClient` (health/capabilities fallback chains, typed calls)
  and `InferenceClient` (discover, complete, embed, models) wrappers.
- `ipc/discover.rs` — Structured `DiscoveryResult` and `DiscoverySource` for traceable
  primal discovery (env override, capability probe, well-known path, not found).
- `status` field (`"healthy"` / `"degraded"`) in `health.readiness` response.
- V51 handoff: `wateringHole/handoffs/HEALTHSPRING_V51_HARDENED_COMPOSITION_HANDOFF_APR11_2026.md`.

### Changed
- `CoralReefDevice` → `SovereignDevice` in `gpu/sovereign.rs` (upstream API rename).
- `handle_connection` refactored to generic `handle_lines<R,W>` supporting both Unix and TCP.
- `cmd_serve` accepts `tcp_port: Option<u16>` and spawns TCP listener thread when provided.
- `register_with_biomeos` iterates `LOCAL_CAPABILITIES` and `ROUTED_CAPABILITIES` separately
  with `served_locally`/`canonical_provider` metadata per primalSpring niche pattern.
- `plasmidBin/manifest.lock` healthspring version updated 0.7.0 → 0.8.0 (resolves drift).

### Fixed
- `CoralReefDevice` compile error in `gpu/sovereign.rs` (5 occurrences).
- Broken intra-doc link in `provenance/mod.rs`.
- `clippy::needless_pass_by_value` in `accept_tcp`, `handle_unix_connection`,
  `handle_tcp_connection` (justified `#[expect]` with reasons).
- `clippy::map_unwrap_or` in TCP port logging.

## V50 — 2026-04-11 — Composition Evolution

### Added
- Optional Squirrel node in `healthspring_niche_deploy.toml` (`required = false`) for
  `inference.*` capabilities when available.
- Dual-method discovery fallback in `tower_atomic.rs`: tries `discovery.find_by_capability`
  first, falls back to legacy `net.discovery.find_by_capability`.
- Provenance registry split: `registry.rs` (80 LOC logic) + `records_science.rs` (460 LOC,
  Tracks 1–5) + `records_infra.rs` (720 LOC, Tracks 6–10+). All under 1000 LOC.
- V50 handoff: `wateringHole/handoffs/HEALTHSPRING_V50_COMPOSITION_EVOLUTION_HANDOFF_APR11_2026.md`.
- Cross-team primal evolution handoff for barraCuda, toadStool, primalSpring, biomeOS.

### Changed
- `primal.forward` routing: capability-based discovery first, name-based fallback.
- exp112/exp113 refactored: `main()` extracted into domain-coherent helper functions.
- exp114: `if_same_then_else` and `unnecessary_map_or` clippy errors fixed.
- exp116: `overly_complex_bool_expr` tautologies fixed.
- Inline tolerance values in toadstool/metalForge tests migrated to `tolerances::*` constants.
- Doc backticks added for `BearDog`, `PopPK` references in module-level docs.
- `PRIMAL_GAPS.md` updated to V50: §3 (dual discovery fallback), §9 (Squirrel optional node).

### Fixed
- 4 clippy errors in composition experiments (exp112, exp114, exp116).
- 7 clippy warnings in library code (doc formatting, hex literals, paragraph breaks).
- 579-line `cargo fmt` drift resolved.
- `provenance/registry.rs` split from 1224 LOC to 3 files (all under 1000 LOC standard).

## V49 — 2026-04-10 — Composition Audit Remediation

### Added
- `health.genomics` capability + proto-nucleate alias → `science.microbiome.qs_gene_profile`.
  All five `health.*` proto-nucleate aliases are now wired.
- `exp117_composition_ipc_roundtrip` — Tier 4 IPC wire protocol validation (71 checks):
  round-trip serialization, proto-nucleate alias dispatch, health probe routing, full
  capability surface completeness.
- Deploy graph fragment metadata: `fragments`, `particle_profile`, `proto_nucleate`,
  `[graph.bonding]` with bond type, trust model, encryption tiers per atomic boundary.
- Bonding policy matrix documented in `tower_atomic.rs` module docs.
- WGSL shader removal plan documented in `gpu/mod.rs` (all 6 absorbed upstream).
- 36 new provenance registry entries for non-Python experiments → 89 total (100% coverage).
- `PRIMAL_GAPS.md` gaps §8 (deploy fragment metadata — fixed) and §9 (Squirrel in deploy).
- V49 handoff: `wateringHole/handoffs/HEALTHSPRING_V49_COMPOSITION_AUDIT_HANDOFF_APR10_2026.md`.
- V49 classification table for 21 tolerance-exempt experiments in `TOLERANCE_REGISTRY.md`.

### Changed
- barraCuda pin updated from v0.3.7 (`c04d848`) to v0.3.11 (`7f6649f`).
- `uncertainty::std_dev` delegated to `barracuda::stats::correlation::std_dev`.
- Sovereign dispatch error messages now name each unsupported op + reference EVOLUTION_MAP.
- `provenance::tests::registry_complete` split into `registry_covers_all_python_scripts`
  and `registry_covers_all_experiments` (decoupled from 1:1 Python file assumption).
- All spec docs (`BARRACUDA_REQUIREMENTS`, `EVOLUTION_MAP`, `TOLERANCE_REGISTRY`) updated to V49.

### Fixed
- `cross_validate.py`: `TOL_AUC` aliased to `LEVEL_SPACING_RATIO` (0.02) instead of
  `AUC_TRAPEZOIDAL` (0.01). Fixed to use correct registry constant.
- CI: added `pip install -r control/requirements.txt` before Python cross-validation.
- `PRIMAL_GAPS.md` gap §1 resolved — `health.*` science aliases implemented (option a).

## V48 — 2026-04-10

### Added
- `docs/PRIMAL_GAPS.md` — primal composition gap registry per NUCLEUS alignment protocol.
- `CHANGELOG.md` — presentation standard compliance.
- `inference.*` capability aliases alongside `model.*` for Squirrel/neuralSpring alignment.
- `health.*` proto-nucleate aliases (`health.pharmacology`, `health.clinical`,
  `health.de_identify`, `health.aggregate`) registered in `ALL_CAPABILITIES`.
- `resilient_send` in `ipc/rpc.rs` — retry with exponential backoff for retriable IPC errors.
- Infrastructure capabilities in YAML niche manifest (`niches/healthspring-health.yaml`).
- Cross-compile CI targets (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`).
- GPU CI job enabled as weekly scheduled run.
- **Tier 4 composition validation** — 5 new experiments (exp112–exp116, 73 checks total).
- ecoBin static-PIE binary (x86_64-musl, 2.5 MB) harvested to `infra/plasmidBin/healthspring/`.
- CI cross-compile job with artifact upload and static linkage verification.

### Changed
- `health.readiness` gates on science dispatch status.
- `build_semantic_mappings()` refactored to programmatic map construction.

### Fixed
- YAML niche manifest now includes all capabilities the binary serves.

## V47 — 2026-04-07

### Added
- `HEALTHSPRING_V47_UPSTREAM_ABSORPTION_HANDOFF_APR07_2026.md` — upstream
  absorption, health triad, deploy graph references.
- `HEALTHSPRING_V46_COMPOSITION_CONVERGENCE_HANDOFF_APR07_2026.md`.
- `HEALTHSPRING_V45_CAPABILITY_SYNC_IPC_FUZZ_HANDOFF_APR07_2026.md`.

## V44 — 2026-03-24

### Added
- 83 validation experiments with `ValidationHarness` (hotSpring pattern).
- JSON-RPC primal server (`healthspring_primal`) with 75+ capabilities.
- 6 WGSL shaders for GPU dispatch (Hill, PopPK, Diversity, MM, SCFA, BeatClassify).
- barraCuda integration: 6 GPU ops + CPU primitives (stats, rng, health modules).
- Provenance trio session API (begin/record/complete/status).
- Tower Atomic integration (BearDog + Songbird discovery).
- petalTongue visualization IPC push with clinical scenario nodes.
- `metalForge/forge` — NUCLEUS dispatch and composition.
- `toadstool` — compute pipeline staging.
- Proptest IPC fuzzing (protocol, transport, dispatch).
- `cargo-deny` configuration banning C/native dependencies.
- 90%+ library line coverage enforced in CI.
- SPDX `AGPL-3.0-or-later` headers on all source files.
