<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V62 — Upstream Primal & Spring Handoff

**Date**: May 10, 2026
**Version**: V62
**primalSpring**: v0.9.25 (pinned)
**Tests**: 999 (868 lib + 131 integration/workspace)
**Capabilities**: 87 JSON-RPC methods
**Architecture**: Eukaryotic UniBin + IPC-first defaults + CI cross-sync

---

## What Changed in V62

### CI Cross-Sync (canonical 403 alignment)

healthSpring now implements all 5 methods from primalSpring's `[health]` domain:

| Method | Status |
|--------|--------|
| `health.check` | Live (V59+) |
| `health.liveness` | Live (V59+) |
| `health.readiness` | Live (V59+) |
| `health.monitor` | **New V62** — uptime, request count, capability surface metrics |
| `health.probe` | **New V62** — deep health probe with science dispatch readiness |

Registry sync integration tests: 3/3 pass, zero collisions, zero drift vs canonical 403.

### skunkBat Audit Wiring

- New `ipc/audit.rs` with `audit_log()` and `audit_certification()` via `HealthCompositionContext`
- `SKUNKBAT` added to `primal_names.rs`
- `"audit"` / `"audit.log"` routed in `composition/routing.rs`
- skunkBat registered as optional `NicheDependency` (capability: `"audit"`)
- `"audit.log"` added to `CONSUMED_CAPABILITIES` in `niche.rs`
- Local `config/capability_registry.toml` has `[audit]` section (owner: skunkbat, consumed)

**Ask for skunkBat team**: When Phase 3 audit forwarding ships, healthSpring is ready to emit certification and validation events to `rhizoCrypt` DAG + `sweetGrass` braid.

### biomeOS v3.51 Absorption

- `composition.status` handler returns `{ primal, domain, primal_health, resource_pressure }`
- `method.register` handler acknowledges dynamic method registration with `tracing::info` logging
- Both registered in `ALL_CAPABILITIES`, `LOCAL_CAPABILITIES`, and local registry

**Ask for biomeOS team**: healthSpring is ready for live `composition.status` polling and `method.register` calls from biomeOS orchestration.

### Sovereignty Push

- NCBI E-utilities and SRA base URLs now env-configurable:
  - `HEALTHSPRING_NCBI_EUTILS_BASE` (default: `https://eutils.ncbi.nlm.nih.gov/entrez/eutils`)
  - `HEALTHSPRING_NCBI_SRA_BASE` (default: `https://trace.ncbi.nlm.nih.gov/Traces/sra/sra.cgi`)
- Air-gapped and proxy deployments can override without code changes

### Deep Debt Resolved

- All `NicheDependency` entries in `niche.rs` now reference `primal_names::*` constants (was string literals)
- `BarraCudaClient::discover()` uses `primal_names::BARRACUDA` (was hardcoded `"barracuda"`)
- Zero `TODO`/`FIXME`/`unimplemented!` in all Rust code
- Zero unsafe code (`#![forbid(unsafe_code)]` crate-wide)
- No files >800 lines; no mocks in production code; all dependencies Rust-native

---

## Primal Usage and Asks

### barraCuda (tensor / stats)

**Usage**: 14 IPC migration entries (`stats.mean`, `stats.std_dev`, `stats.variance`, `stats.correlation`, `stats.hill`, `stats.shannon_from_frequencies`, `stats.simpson`, `stats.chao1_classic`, `stats.bray_curtis`, `special.anderson_diagonalize`, `rng.uniform`, `health.pkpd.mm_auc`, `health.microbiome.antibiotic_perturbation`, `health.biosignal.scr_rate`). Feature-gated `barracuda-lib` for library calls; IPC-first by default.

**Ask**: Continue expanding health-domain compositions (`health.pkpd.*`, `health.microbiome.*`, `health.biosignal.*`) on barraCuda's JSON-RPC surface. healthSpring validates parity between library and IPC for all 14 entries.

### coralReef (shader compile)

**Usage**: 6 WGSL shaders (Hill, PopPK, Diversity, MM, SCFA, Beat) compiled via coralReef pipeline.

**Ask**: No new shaders needed. Existing pipeline stable.

### toadStool (compute dispatch)

**Usage**: V16 StageOps dispatched via `execute_cpu`, `execute_streaming`, `execute_auto`. metalForge dispatch for NUCLEUS topology.

**Ask**: Continue dispatch contract stability. healthSpring validates 24/24 toadStool dispatch checks.

### NestGate (storage)

**Usage**: `storage.store`, `storage.retrieve`, `storage.egress_fence` consumed.

**Ask**: No changes needed.

### rhizoCrypt (DAG)

**Usage**: `dag.create_session`, `dag.append_event`, `dag.dehydrate` consumed for provenance.

**Ask**: Stable. Will receive audit events via skunkBat forwarding when Phase 3 ships.

### loamSpine (ledger / spine)

**Usage**: `commit.session` consumed for provenance trio.

**Ask**: No changes needed.

### sweetGrass (commit / braid)

**Usage**: `provenance.create_braid` consumed for provenance trio.

**Ask**: Will receive audit analytics via skunkBat forwarding when Phase 3 ships.

### BearDog (crypto)

**Usage**: `crypto.hash`, `crypto.sign`, `crypto.ionic_bond` consumed.

**Ask**: No changes needed. Ionic bridge pattern validated.

### Songbird (discovery)

**Usage**: `discovery.find_by_capability` consumed.

**Ask**: No changes needed.

### Squirrel (inference)

**Usage**: `inference.complete`, `inference.embed`, `inference.models` consumed.

**Ask**: No changes needed.

### petalTongue (visualization)

**Usage**: `DataChannel` push for scenario visualization. `PetalTongueDataChannel` types.

**Ask**: No changes needed.

### skunkBat (audit) — NEW

**Usage**: `audit.log` consumed via `ipc/audit.rs`.

**Ask**: Emit certification tier results and validation events. Ready for Phase 3 forwarding.

### biomeOS (orchestration)

**Usage**: `composition.status` served, `method.register` served, lifecycle/health probes served.

**Ask**: Poll `composition.status` for primal health monitoring. Use `method.register` for dynamic capability surface evolution.

---

## NUCLEUS Composition Patterns

### Deploy Graph Convention

healthSpring uses 7+ deploy TOMLs following the `[graph]` metadata + ordered nodes pattern from projectNUCLEUS. Each node specifies `depends_on`, `by_capability`, and `health_method` (Wire Standard L3).

### neuralAPI Deployment

healthSpring's `healthspring_primal` binary exposes 87 JSON-RPC methods over Unix sockets. biomeOS deploys it as a niche node in NUCLEUS compositions. The `composition.status` endpoint enables health monitoring without polling individual science methods.

### Certification as Gate

`healthspring_unibin certify` runs 57/57 checks across Tiers 1-3. This is a pre-deployment gate: if certification fails, the primal should not be deployed into a NUCLEUS graph. The `audit_certification()` function forwards results to skunkBat for ecosystem-wide visibility.

### Tier 4 IPC-First Architecture

All `barracuda::` library usage is behind `#[cfg(feature = "barracuda-lib")]`. Default builds (`default = []`) route all math through IPC. This means:
- healthSpring can be deployed without barraCuda co-located
- barraCuda can be swapped for any primal advertising `stats.*` capabilities
- Composition is truly capability-based, not identity-based

---

## Patterns for Other Springs to Absorb

1. **`primal_names.rs` centralization**: Single source of truth for primal name constants. All niche dependencies and IPC clients reference these constants instead of string literals.

2. **Env-configurable external URLs**: Any external API base URL should have an `ENV_VAR` override with a sensible default. Enables air-gapped and proxy deployments without code changes.

3. **`HealthCompositionContext` wrapper pattern**: Domain-specific typed accessors over `CompositionContext` (e.g., `stats_mean()`, `stats_std_dev()`). Keeps domain code clean while delegating IPC to primalSpring.

4. **Audit event forwarding**: `audit_log(ctx, event_type, data)` and `audit_certification(ctx, tier, passed, failed, skipped)` — ready for Phase 3.

5. **`composition.status` / `method.register`**: biomeOS v3.51 integration. Every spring should serve these for orchestration visibility.

6. **Registry sync integration tests**: Validate local capabilities against primalSpring canonical 403. Catches drift early.

---

## Remaining Gaps (for upstream teams)

| Gap | Owner | Description |
|-----|-------|-------------|
| skunkBat Phase 3 | skunkBat team | Audit forwarding to rhizoCrypt DAG + sweetGrass braid — healthSpring is wired and waiting |
| guideStone L6 | primalSpring | NUCLEUS deployment validation — healthSpring has L5 (57/57) and composition infrastructure for L6 |
| GPU Tier 2 expansion | barraCuda | Anderson eigensolve, biosignal FFT, MM population on GPU |
| NPU Tier 3 | toadStool/metalForge | Neural processing unit dispatch for ML-accelerated paths |
| NLME on GPU | barraCuda | FOCE/SAEM population PK estimation with GPU acceleration |
| MIMIC-IV / openFDA | ecosystem | Large clinical dataset integration when provenance pipeline matures |

---

## Downstream References

- **projectNUCLEUS**: `workloads/healthspring/healthspring-pk-validation.toml` (workload stub)
- **foundation**: Thread 8 (Human Health/Clinical); `data/sources/thread08_health.toml`; Papers 13/22
- **foundation gap**: `expressions/SOVEREIGN_HEALTH.md` referenced but not yet created

---

*Supersedes: V61 interstadial eukaryotic + upstream primal handoffs (archived)*
