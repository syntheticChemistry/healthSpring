# healthSpring V64r — Wave 20 Schema Standardization Handoff

**Date:** May 16, 2026
**From:** healthSpring (Nest Atomic Specialist)
**To:** primalSpring, biomeOS, upstream primals, all springs
**primalSpring Wave:** 20 (452 methods, schema standardization)

---

## Changes Applied

### 1. `capability.list` Canonical Envelope (DONE)

healthSpring's response now includes the Wave 20 required subset:

```json
{
  "capabilities": ["science", "health", "provenance", "compute", "data", ...],
  "count": 22,
  "primal": "healthspring",
  "version": "0.1.0",
  "domain": "health",
  "methods": [...],
  "total": 88,
  "science": [...],
  "infrastructure": [...],
  "provided_capabilities": {...},
  "operation_dependencies": {...},
  "cost_estimates": {...}
}
```

`"capabilities"` is a flat array of unique top-level capability domains (from
`ALL_CAPS` routing table + method prefixes). `"count"` is the domain count.
Enriched fields preserved for downstream consumers (biomeOS Pathway Learner,
petalTongue dashboard, etc.).

**Location:** `ecoPrimal/src/bin/healthspring_primal/capabilities.rs`

### 2. Registry Sync: 452 Methods (DONE)

`primal.list` added to `[primal_registry]` in `config/capability_registry.toml`
and to `CONSUMED_CAPABILITIES` in `niche.rs`.

### 3. `nest.commit` Signal-Path Status (ALREADY WIRED)

healthSpring already uses signal-first dispatch since V64o:

- `NestComposition.full_lifecycle()` → tries `signal.dispatch("nest.commit", ...)`
  via biomeOS, falls back to manual chain
- `data/provenance.rs` → tries `signal.dispatch("nest.commit", ...)` via
  orchestrator, falls back to `dag.dehydrate → spine.create → braid.create`

This aligns with primalSpring's `s_nest_commit_live` pattern. No additional
wiring needed — healthSpring is signal-path-first for both `nest.store` and
`nest.commit`.

### 4. `--provenance-dir` (FUTURE)

Thread 10 candidate. healthSpring's validation binaries (`validate_pk_models`,
`validate_ltee_b5`) already support `--format json`. Adding `--provenance-dir`
to write `results.json` + `provenance.toml` to projectFOUNDATION's convention
is an incremental addition when workloads require it. No blocking gap.

### 5. Schema Validation Scenario (FUTURE)

A `s_schema_standard`-style scenario would probe:
- Registry presence of all 452 methods
- Local `capability.list` response shape validation
- Live biomeOS `capability.list` + `primal.list` schema compliance

healthSpring's `integration_registry_sync` test already validates structural
cross-sync against primalSpring's registry. A schema scenario adds live
response shape probing — candidate for next sprint.

---

## Assessment Summary

| Item | Status | Notes |
|------|--------|-------|
| `capability.list` canonical envelope | **DONE** | `capabilities` + `count` present |
| `count` field | **DONE** | Domain count from routing table |
| `primal.list` registry sync (452) | **DONE** | Added to registry + niche |
| `nest.commit` signal dispatch | **DONE** (V64o) | Signal-first with fallback |
| `--provenance-dir` | Future | Incremental when Foundation calls |
| `s_schema_standard` scenario | Future | CI sync test covers most |

---

## Metrics

- **Tests:** 1,018 (workspace), 0 failures
- **Registry:** 452 methods synced
- **Clippy:** 0 warnings
- **Deep debt:** 0 across all 7 categories
