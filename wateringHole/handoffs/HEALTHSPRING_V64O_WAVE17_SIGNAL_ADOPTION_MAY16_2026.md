# healthSpring V64o — Wave 17 Signal Adoption Handoff

**Date:** May 16, 2026
**From:** healthSpring (Nest Atomic Specialist)
**To:** primalSpring, biomeOS, upstream primals, all springs
**primalSpring Wave:** 17 (451 methods, 41 scenarios)

---

## Signal Adoption Status

### 1. `primal.announce` Registration (ADOPTED)

`server/registration.rs` now tries Wave 17 single-call `primal.announce` first:

```
primal.announce → { primal_id, transport, methods[], lifecycle: { state: "running" } }
```

Falls back to legacy `lifecycle.register` + N × `capability.register` when biomeOS
does not support `primal.announce` (returns `-32601`). Zero behavioral change for
older deployments.

**Location:** `ecoPrimal/src/bin/healthspring_primal/server/registration.rs`

### 2. `nest.store` + `nest.commit` Signal Dispatch (WIRED, PENDING LIVE)

#### NestComposition Facade

`NestComposition.full_lifecycle()` now has a two-tier execution path:

**Signal path (preferred):**
1. `signal.dispatch("nest.store", { experiment, event, content, hash_algorithm })` → biomeOS graph
2. `signal.dispatch("nest.commit", { session_id, experiment, merkle_root, agents })` → biomeOS graph

**Manual path (fallback):**
1. `storage.store` → NestGate
2. `dag.event.append` → rhizoCrypt
3. `crypto.sign` → BearDog
4. `spine.create` → loamSpine
5. `braid.create` + `braid.commit` → sweetGrass

Falls back to manual when either signal.dispatch returns an error (biomeOS unavailable,
signal not registered, or `-32601`).

**Location:** `ecoPrimal/src/ipc/provenance/nest.rs`

#### Data Provenance Pipeline

`complete_data_session()` tries `signal.dispatch("nest.commit", ...)` via the
orchestrator socket before falling back to manual `dag.dehydrate → spine.create →
braid.create` chain.

**Location:** `ecoPrimal/src/data/provenance.rs`

### 3. 451-Method Registry Sync (COMPLETE)

`config/capability_registry.toml` now includes all Wave 17 entries:

| Section | Methods | Source |
|---------|---------|--------|
| `[fido2]` | `beardog.fido2.authenticate`, `.discover`, `.register` | BearDog Wave 103 |
| `[genetic]` | `genetic.ceremony_init`, `.ceremony_finalize`, `.derive_key`, `.entropy_contribute` | Playbook Artifact 4/7 |
| `[certificate]` | `certificate.verify` | Playbook Artifact 3 |
| `[primal_registry]` | `primal.announce`, `primal.info` | Songbird Wave 205 |
| `[signals]` | All 14 atomic signals + `signal.dispatch` | biomeOS Neural API |

### 4. Routing Domain Expansion (COMPLETE)

`routing.rs` `ALL_CAPS` expanded: `signal`, `certificate`, `genetic`, `fido2`, `primal`.

| Domain | Provider |
|--------|----------|
| `signal` | biomeOS |
| `fido2` | bearDog |
| `primal` | primalSpring |
| `certificate`, `genetic` | ecosystem |

### 5. Niche Consumed Capabilities (COMPLETE)

`niche.rs` `CONSUMED_CAPABILITIES` gains: `signal.dispatch`, `primal.announce`,
`primal.info`, `certificate.verify`.

### 6. GAP-GS-015 (CONFIRMED)

`cargo check --workspace` passes clean. `ALL_CAPS` and `BTSP_EXTRA_CAPS` properly
re-exported from `composition/mod.rs`.

---

## Foundation Threads 3+8

Threads 3 (Immunology) and 8 (Human Health) expression artifacts (`IMMUNO_DRUG_DISCOVERY.md`,
`SOVEREIGN_HEALTH.md`, `THREAD_INDEX.toml`) are **not present** in healthSpring's workspace.
They are documented as "active" in CHANGELOG and handoffs but live externally in
primalSpring/sporeGarden.

healthSpring's contribution is **B5 (symbiont PK/PD)** — the lithoSpore module candidate
for Threads 3+8 content. Package: `control/ltee_symbiont_pkpd/` with `tolerances.toml` +
`LITHO_MODULE_README.md`.

**Ask:** primalSpring confirm sporeGarden Thread 3+8 structure so healthSpring can verify
B5 aligns with the expression format.

---

## New Gaps Filed

| # | Gap | Owner | Action |
|---|-----|-------|--------|
| 46 | Foundation Threads 3+8 expressions not in healthSpring workspace | primalSpring | Confirm sporeGarden structure; healthSpring contributes B5 |
| 47 | Signal dispatch live validation pending | healthSpring | Run `s_nest_atomic` with biomeOS `signal.dispatch` to validate `nest.store`/`nest.commit` signal path end-to-end |

---

## Signal Adoption Mapping (healthSpring-specific)

Per `SIGNAL_ADOPTION_STANDARD.md` provenance-heavy archetype:

| Signal | healthSpring Use Case | Status |
|--------|----------------------|--------|
| `nest.store` | PK/PD experiment data ingestion (storage + DAG) | **Wired** (NestComposition) |
| `nest.commit` | Session finalization (dehydrate + sign + seal + braid) | **Wired** (NestComposition + data/provenance) |
| `nest.retrieve` | Provenance chain retrieval | Not yet needed |
| `tower.authenticate` | BTSP dual-tower ionic negotiation | Blocked on BTSP server (Gap #10) |
| `tower.publish` | Signed result publication | Future (post-composed) |
| `node.compute` | GPU pipeline dispatch (toadStool → coralReef → barraCuda) | Future (compute-heavy workloads) |

Domain-specific calls (`stats.mean`, `health.pkpd.mm_auc`, etc.) remain as `ctx.call()` — they are NOT signal candidates.

---

## Metrics

- **Tests:** 1,018 (workspace), 0 failures
- **Clippy:** 0 warnings (pedantic + nursery)
- **Unsafe:** 0 (`#![forbid(unsafe_code)]`)
- **Deep debt:** 0 across all 7 categories
- **Deploy graphs:** 7
- **Validation scenarios:** 17
- **guideStone:** Level 5
