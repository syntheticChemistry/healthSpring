# healthSpring V46 -- Composition Convergence Handoff

**Date:** April 7, 2026
**From:** healthSpring V46
**To:** primalSpring coordination, biomeOS, ecosystem
**License:** AGPL-3.0-or-later
**Supersedes:** HEALTHSPRING_V45_CAPABILITY_SYNC_IPC_FUZZ_HANDOFF_APR07_2026.md

---

## Summary

Converge healthSpring's deployment artifacts onto the canonical composition
pattern defined by primalSpring's `DeployGraph` (`[graph]` + `[[graph.node]]`
with mandatory `by_capability` on every node). Retire the old `[[stages]]`
deploy graph. Add `composition.health_health` per
`COMPOSITION_HEALTH_STANDARD.md`.

## Changes

### 1. Retired `wateringHole/healthspring_deploy.toml` (old `[[stages]]` pattern)

The file used a non-canonical `[[stages]]` schema that is not parseable by
primalSpring's `load_graph()` / `topological_waves()`. It was fully redundant
with `graphs/healthspring_niche_deploy.toml` (Format A) plus the workflow
graphs. Archived to `wateringHole/handoffs/archive/`.

### 2. Deploy graph convergence (`graphs/healthspring_niche_deploy.toml`)

- **`by_capability` on every node**: beardog (`security`), songbird
  (`discovery`), toadstool (`compute`) were missing this field. All 8 nodes
  now carry `by_capability`, satisfying primalSpring's
  `all_graphs_have_by_capability_on_every_node` structural test.
- **`coordination = "sequential"`** added to `[graph]` header (was implicit).
- **`health_method`** normalized: beardog and songbird now use
  `health.liveness` (was bare `health`).
- **healthspring node capabilities** synced with V45 `ALL_CAPABILITIES`: added
  comparative (3), discovery (4), toxicology (3), simulation (2),
  `composition.health_health`, `compute.shader_compile`,
  `model.inference_route`, `health.liveness`, `health.readiness`. Removed
  stale entries (`science.discovery.ipsc_skin_model`,
  `science.discovery.niclosamide_delivery`,
  `science.microbiome.qs_expanded_matrix`).
- **Graph version** bumped to `0.3.0`.

### 3. Niche manifest update (`graphs/healthspring_niche.toml`)

healthspring primal `provides` expanded to include `science.comparative`,
`science.discovery`, `science.toxicology`, `science.simulation`, and
`composition.health_health`.

### 4. `composition.health_health` implementation

Per `COMPOSITION_HEALTH_STANDARD.md`, added:

- **Route** in `routing.rs`: `"composition.health_health" => handle_composition_health(state)`
- **Handler** returns the mandatory response shape:
  - `healthy: bool` -- true when all subsystems are up
  - `deploy_graph: "healthspring_health_niche"` -- active graph name
  - `subsystems: { science_dispatch, provenance_trio, compute_provider, data_provider }`
  - Plus `capabilities_count`, `science_domains`, `uptime_secs`
- **Registered** in `ALL_CAPABILITIES`, `build_semantic_mappings()`,
  niche YAML, and deploy graph.

### 5. Niche YAML sync (`niches/healthspring-health.yaml`)

Added `composition.health_health` to the capabilities list (now 59 science +
1 composition = 60 domain capabilities).

## Verification

| Source | Science caps | `by_capability` coverage | `composition.health_health` |
|--------|-------------|--------------------------|----------------------------|
| `ALL_CAPABILITIES` | 58 | n/a | present |
| Dispatch `REGISTRY` | 58 | n/a | routed in `routing.rs` |
| Deploy graph | 58 | 8/8 nodes | present |
| Niche YAML | 58 | n/a | present |
| Niche manifest | 10 domains | n/a | present |

## Pattern Summary

healthSpring now uses exactly one composition pattern:

| Layer | File | Format |
|-------|------|--------|
| **Deploy graph** | `graphs/healthspring_niche_deploy.toml` | `[graph]` + `[[graph.node]]` (Format A) |
| **Workflow graphs** | `graphs/healthspring_patient_assessment.toml` etc. | `[graph]` + `[[nodes]]` (Format B -- execution DAGs) |
| **Niche manifest** | `graphs/healthspring_niche.toml` | `[niche]` + `[[primals]]` + `[[graphs]]` |
| **Niche YAML** | `niches/healthspring-health.yaml` | BYOB capability declaration |
| **Composition health** | `routing.rs` | `composition.health_health` JSON-RPC method |

The old `[[stages]]` format is retired. No other composition schemas remain.

## Impact on Ecosystem

- **primalSpring**: healthSpring deploy graph now passes
  `all_graphs_have_by_capability_on_every_node` structural validation.
- **biomeOS**: `composition.health_health` enables biomeOS to probe the health
  science pipeline as a first-class subsystem alongside
  `composition.tower_health`, `composition.node_health`, etc.
- **Other springs**: This handoff documents the convergence path for springs
  that still use `[[stages]]` or lack `by_capability` coverage.

## Files Changed

| File | Action |
|------|--------|
| `wateringHole/healthspring_deploy.toml` | Archived to `handoffs/archive/` |
| `graphs/healthspring_niche_deploy.toml` | Rewritten (by_capability, caps sync, v0.3.0) |
| `graphs/healthspring_niche.toml` | Updated provides list |
| `ecoPrimal/src/bin/healthspring_primal/capabilities.rs` | Added composition.health_health |
| `ecoPrimal/src/bin/healthspring_primal/server/routing.rs` | Added composition health handler |
| `niches/healthspring-health.yaml` | Added composition.health_health |
