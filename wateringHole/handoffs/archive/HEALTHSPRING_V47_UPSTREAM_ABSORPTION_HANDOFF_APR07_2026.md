# healthSpring V47 -- Upstream Absorption Handoff

**Date:** April 7, 2026
**From:** healthSpring V47
**To:** primalSpring, wetSpring, biomeOS, ecosystem
**License:** AGPL-3.0-or-later
**Supersedes:** HEALTHSPRING_V46_COMPOSITION_CONVERGENCE_HANDOFF_APR07_2026.md

---

## Summary

Pull upstream patterns from primalSpring, wetSpring, biomeOS, and
`infra/wateringHole` ecosystem standards. Absorb concrete requirements,
fix health triad compliance, add biomeOS-format deploy graph, implement
wetSpring hormesis cross-validation, and document remaining gaps.

## Sources Reviewed

| Source | Key documents |
|--------|--------------|
| **primalSpring** | `deploy/mod.rs` (DeployGraph, topological_waves), `coordination/mod.rs` (AtomicType), `specs/GEN4_COMPOSITION_AUDIT.md`, `graphs/spring_byob_template.toml`, `harness/mod.rs`, all 48 handoffs |
| **wetSpring** | `bio/hormesis.rs` (Anderson biphasic), `ipc/dispatch.rs`, `work/anderson_hormesis/JOINT_EXPERIMENT.md`, V130/V137 handoffs, `niches/wetspring-ecology.yaml` |
| **biomeOS** | `neural_graph.rs` (parser), `neural_executor.rs` (dispatch), `capability_registry.toml`, `niche.rs` (templates), `socket_discovery/mod.rs`, CHANGELOG v2.78-v2.82 |
| **wateringHole** | `DEPLOYMENT_AND_COMPOSITION.md`, `DEPLOYMENT_VALIDATION_STANDARD.md`, `COMPOSITION_HEALTH_STANDARD.md`, `SPRING_COORDINATION_AND_VALIDATION.md`, `SPRING_INTERACTION_PATTERNS.md`, `PRIMAL_IPC_PROTOCOL.md` v3.1 |

## Changes

### 1. Health triad compliance (DEPLOYMENT_VALIDATION_STANDARD.md)

**`health.readiness`** response now includes `primal`, `version`, and `domain`
fields as required by the deployment validation standard. Previously only
returned `ready`, `uptime_secs`, and `subsystems`.

**`health.check`** response now includes `"healthy": true` boolean alongside
the existing `"status": "healthy"` string. biomeOS's `call_primal_health`
checks for the `healthy` boolean specifically.

### 2. biomeOS-format deploy graph

**New file:** `graphs/healthspring_biomeos_deploy.toml`

biomeOS's `neural_graph.rs` parses `[[nodes]]` or `[[graph.nodes]]` -- not
primalSpring's `[[graph.node]]`. healthSpring now carries both:

| Graph | Format | Consumer |
|-------|--------|----------|
| `healthspring_niche_deploy.toml` | `[[graph.node]]` (Format A) | primalSpring structural validation |
| `healthspring_biomeos_deploy.toml` | `[[nodes]]` + `action`/`params` | biomeOS `niche.deploy` / `graph.execute` |

The biomeOS graph includes all 11 capability domains (medical, pkpd,
microbiome, biosignal, endocrine, diagnostic, clinical, comparative,
discovery, toxicology, simulation) and a `composition.health_health`
verification step.

**Note for biomeOS:** The bundled `primals/biomeOS/graphs/healthspring_deploy.toml`
is stale (only registers 7 domains, missing comparative/discovery/toxicology/
simulation). healthSpring's `graphs/healthspring_biomeos_deploy.toml` is the
authoritative source.

### 3. wetSpring hormesis cross-validation

**New file:** `ecoPrimal/src/toxicology/wetspring_cross_validation.rs`

Cross-validation tests verify that healthSpring's `biphasic_dose_response`
produces identical results to wetSpring's `bio::hormesis` biphasic model
for the shared parameter set from joint experiment Exp379.

**Algebraic mapping:**

| wetSpring | healthSpring | Constraint |
|-----------|-------------|------------|
| `A` | `s_max` | Equal |
| `K_stim` | `k_stim` | Equal |
| `n_s` | (hardcoded 1) | wetSpring may use n_s > 1 |
| `K_inh` | `ic50` | Equal |
| `n_i` | `hill_n` | Equal |
| (implicit 1.0) | `baseline` | Set to 1.0 for cross-validation |

Tests cover: zero dose, hormetic zone (0.1-5.0), transition zone (10-40),
toxic zone (100-500), hormetic optimum location, stimulation threshold, and
biphasic curve shape (single peak).

**New tolerance:** `HORMESIS_CROSS_SPRING = 1e-12` in `tolerances.rs` for
cross-spring numerical agreement.

**Limitation:** When wetSpring uses stimulation Hill n_s > 1, the models
diverge in the stimulation term. Joint experiment Exp379 documents the
parameter regime where both agree.

### 4. Composition guidance refresh

Updated `infra/wateringHole/healthspring/HEALTHSPRING_COMPOSITION_GUIDANCE.md`
from V34 to V47:

- Capability count updated: 58 science across 10 domains (was "79 across 8")
- Added comparative, discovery, toxicology, simulation domain rows
- Added `composition.health_health` to infrastructure capabilities
- Added health triad to infrastructure capabilities

### 5. Niche manifest update

`graphs/healthspring_niche.toml` now references the biomeOS deploy graph as
an additional `[[graphs]]` entry alongside the primalSpring format.

## Upstream findings -- already aligned

| Pattern | Status |
|---------|--------|
| `by_capability` on all deploy graph nodes | Fixed in V46 |
| `composition.health_health` per COMPOSITION_HEALTH_STANDARD | Implemented in V46 |
| proptest IPC fuzzing (trio witness wire types) | Implemented in V45 |
| Capability four-way sync (ALL_CAPABILITIES, REGISTRY, YAML, deploy graph) | Synced in V45/V46 |
| `normalize_method` with `healthspring.` legacy prefix | Already handled by `rpc::normalize_method` |
| primalSpring `primal_names.rs` slug mapping | Uses `healthspring` -> `healthSpring` |
| Circuit breaker epoch pattern (trio resilience) | Originated in healthSpring V42, absorbed by primalSpring Phase 16 |

## Remaining gaps for future work

| Gap | Source | Priority |
|-----|--------|----------|
| **wetSpring hormesis n_s > 1 divergence** | Joint experiment Exp379 | P2 -- add generalized stimulation Hill to `biphasic_dose_response` |
| **biomeOS graph staleness** | biomeOS `graphs/healthspring_deploy.toml` | P2 -- upstream PR to update biomeOS's bundled copy |
| **Graph structural validation module** | wetSpring `graph_validate.rs` | P3 -- add deploy graph validation tests to healthSpring |
| **Trio circuit breaker policy alignment** | wetSpring V137 (3 failures / 30s) | P3 -- verify healthSpring uses same thresholds |
| **`composition.medical_health` alias** | SPRING_INTERACTION_PATTERNS registers healthSpring under domain `medical` | P3 -- biomeOS v2.90 may add aliases; healthSpring should accept both |
| **ProvenancePipeline trait adoption** | SPRING_COORDINATION_AND_VALIDATION | P3 -- optional `provenance-trio-types` integration |
| **Full NUCLEUS composition health surface** | wetSpring exposes tower/node/nest/nucleus + science_health | P3 -- healthSpring only exposes `composition.health_health` |

## Files changed

| File | Action |
|------|--------|
| `ecoPrimal/src/bin/healthspring_primal/server/routing.rs` | Fixed health triad response shapes |
| `graphs/healthspring_biomeos_deploy.toml` | New: biomeOS-format deploy graph |
| `graphs/healthspring_niche.toml` | Added biomeOS deploy graph reference |
| `ecoPrimal/src/toxicology/wetspring_cross_validation.rs` | New: cross-validation tests |
| `ecoPrimal/src/toxicology/mod.rs` | Registered cross-validation module |
| `ecoPrimal/src/tolerances.rs` | Added HORMESIS_CROSS_SPRING tolerance |
| `infra/wateringHole/healthspring/HEALTHSPRING_COMPOSITION_GUIDANCE.md` | Refreshed to V47 |
