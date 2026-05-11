<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V62 — Upstream Primal & Spring Handoff

**Date**: May 11, 2026
**Version**: V62
**primalSpring**: v0.9.25 (pinned)
**Tests**: 999 (868 lib + 131 integration/workspace)
**Capabilities**: 87 JSON-RPC methods
**Architecture**: Eukaryotic UniBin (`healthspring_unibin` / `healthspring` alias) + IPC-first defaults + CI cross-sync + 4 NUCLEUS workloads

---

## What Changed in V62

### CI Cross-Sync (canonical 413 alignment)

healthSpring now implements all 5 methods from primalSpring's `[health]` domain:

| Method | Status |
|--------|--------|
| `health.check` | Live (V59+) |
| `health.liveness` | Live (V59+) |
| `health.readiness` | Live (V59+) |
| `health.monitor` | **New V62** — uptime, request count, capability surface metrics |
| `health.probe` | **New V62** — deep health probe with science dispatch readiness |

Registry sync integration tests: 3/3 pass, zero collisions, zero drift vs canonical 413.

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

### skunkBat in Deploy Graphs

- `graphs/healthspring_niche_deploy.toml`: skunkBat node (order 8, binary `skunkbat_primal`, provides `defense.*`, required=false)
- `graphs/healthspring_biomeos_deploy.toml`: Phase 2b verify-skunkbat node
- `graphs/healthspring_niche.toml`: `skunkbat_primal` entry (provides `defense`, `audit`, optional=true)
- Defense routing in `composition/routing.rs`: `"defense"` / `"defense.audit"` → `SKUNKBAT`
- Canonical 413 alignment: `security.audit_log` + `defense.audit` in consumed capabilities

**Status**: healthSpring is now fully wired for skunkBat in both library IPC and deploy graphs.

### `healthspring` Binary Alias

- `[[bin]]` entry in `Cargo.toml` pointing to same `src/bin/healthspring/main.rs` as `healthspring_unibin`
- NUCLEUS workloads can invoke `healthspring validate` / `healthspring certify` as expected
- Both `healthspring` and `healthspring_unibin` produce identical binaries

### 4 NUCLEUS Workloads

healthSpring now has 4 workloads in `projectNUCLEUS/workloads/healthspring/`:

| Workload | Command | Domain |
|----------|---------|--------|
| `healthspring-pk-validation` | `healthspring validate` | PK/PD — Hill, PBPK, PopPK, MM |
| `healthspring-biosignal-validation` | `healthspring validate --track biosignal` | Pan-Tompkins, HRV, PPG, EDA, arrhythmia |
| `healthspring-microbiome-validation` | `healthspring validate --track microbiome` | Shannon, Anderson, colonization, SCFA, QS |
| `healthspring-certification` | `healthspring certify` | guideStone 57/57 Tier 1-3 checks |

### Deep Debt Resolved

- All `NicheDependency` entries in `niche.rs` reference `primal_names::*` constants (was string literals)
- `BarraCudaClient::discover()` uses `primal_names::BARRACUDA` (was hardcoded `"barracuda"`)
- Last hardcoded primal names in `s_live_provenance.rs` replaced with `primal_names::RHIZOCRYPT` / `LOAMSPINE` / `SWEETGRASS` — zero hardcoded primal name strings remain anywhere in the codebase
- Zero `TODO`/`FIXME`/`unimplemented!` in all Rust code
- Zero unsafe code (`#![forbid(unsafe_code)]` crate-wide)
- No files >800 lines; no mocks in production code; all dependencies Rust-native
- 45/45 papers reviewed in review queue

### plasmidBin Release Binaries

Stripped release binaries staged to `infra/plasmidBin/springs/`:

| Binary | Size | Purpose |
|--------|------|---------|
| `healthspring` | 2.9M | UniBin alias — `validate`, `certify`, `serve`, `status`, `version` |
| `healthspring_unibin` | 2.9M | UniBin canonical name (identical to `healthspring`) |
| `healthspring_primal` | 3.1M | JSON-RPC IPC server (87 methods) |

NUCLEUS workloads can validate healthSpring science without a source tree. `plasmidBin/.gitignore` updated with all three binary names.

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

healthSpring uses 7+ deploy TOMLs following the `[graph]` metadata + ordered nodes pattern from projectNUCLEUS (includes skunkBat node for audit). Each node specifies `depends_on`, `by_capability`, and `health_method` (Wire Standard L3). 4 NUCLEUS workloads are published for PK, biosignal, microbiome, and certification validation.

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

6. **Registry sync integration tests**: Validate local capabilities against primalSpring canonical 413. Catches drift early.

---

## Priority Upstream Blockers

**NestGate is the critical missing link for full data/compute chains.** healthSpring's next evolution round requires sovereign data pipelines (MIMIC-IV, openFDA FAERS, DrugBank, PubChem) — every stage of the chain is live except NestGate storage:

```
fetch (ureq/sovereign) → store (NestGate) → compute (barraCuda IPC) → provenance (rhizoCrypt/loamSpine/sweetGrass) → audit (skunkBat)
                              ↑ BLOCKED
```

| Blocker | Owner | Impact | Priority |
|---------|-------|--------|----------|
| **NestGate `storage.egress_fence`** | NestGate team | Blocks sovereign data fetch/store/egress for clinical datasets | **HIGH** |
| **NestGate not in default `PRIMAL_LIST`** | biomeOS / primalSpring | Must explicitly add `nestgate` to composition scripts; omission causes silent storage-offline | **HIGH** |
| **BearDog ionic bridge for NestGate** | BearDog + NestGate | Family-scoped encryption at rest for health data compliance (HIPAA-class) | **HIGH** |
| **skunkBat Phase 3 forwarding** | skunkBat team | Audit events to rhizoCrypt DAG + sweetGrass braid — healthSpring wired and waiting | MEDIUM |
| **guideStone L6** | primalSpring | NUCLEUS deployment validation — healthSpring has L5 (57/57) | MEDIUM |

Once NestGate + BearDog ionic bridge ship, healthSpring can wire the full sovereign data pipeline: fetch credentialed datasets via `ureq` (env-configurable bases already in place), store through NestGate with egress policy, compute via barraCuda IPC, and provenance-seal via the rhizoCrypt/loamSpine/sweetGrass trio.

## Remaining Gaps (for upstream teams)

| Gap | Owner | Description |
|-----|-------|-------------|
| NestGate storage pipeline | NestGate + BearDog | Full data fetch/store/egress chain for clinical datasets — see Priority Blockers above |
| skunkBat Phase 3 | skunkBat team | Audit forwarding to rhizoCrypt DAG + sweetGrass braid — healthSpring is wired and waiting |
| guideStone L6 | primalSpring | NUCLEUS deployment validation — healthSpring has L5 (57/57) and composition infrastructure for L6 |
| GPU Tier 2 expansion | barraCuda | Anderson eigensolve, biosignal FFT, MM population on GPU |
| NPU Tier 3 | toadStool/metalForge | Neural processing unit dispatch for ML-accelerated paths |
| NLME on GPU | barraCuda | FOCE/SAEM population PK estimation with GPU acceleration |
| MIMIC-IV / openFDA | ecosystem + NestGate | Large clinical dataset integration — blocked on NestGate storage pipeline |

---

## Downstream References

- **projectNUCLEUS**: 4 workloads in `workloads/healthspring/` (PK, biosignal, microbiome, certification)
- **foundation**: Thread 8 (Human Health/Clinical); `data/sources/thread08_health.toml`; Papers 13/22; `expressions/SOVEREIGN_HEALTH.md` (seeded); `data/targets/thread08_health_targets.toml` (12 validation targets seeded)

---

*Supersedes: V61 interstadial eukaryotic + upstream primal handoffs (archived)*
