<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HEALTHSPRING V52 — Primal Evolution & Composition Patterns Handoff

**Date**: 2026-04-11
**From**: healthSpring V52
**To**: primalSpring, biomeOS, neuralSpring, BearDog, Squirrel, all springs
**Purpose**: Document primal usage patterns, composition learnings, and
evolution opportunities discovered during V49–V52 composition validation.

---

## Context

healthSpring has completed a full pass through the composition evolution spiral:

```
Read proto-nucleate → Wire IPC to primals → Validate composition → Discover gaps
    → Hand back to primalSpring → Primals evolve → Cycle continues
```

This handoff captures what we learned — what worked, what blocked, what the
ecosystem should absorb, and what shapes the next spiral iteration.

---

## healthSpring's Primal Usage (Current State)

### Primals We Consume (by capability, not by identity)

| Primal | Capabilities Used | Discovery Method | Status |
|--------|------------------|-----------------|--------|
| **BearDog** | `crypto.ionic_bond`, `crypto.verify_family` | env override → well-known path | Blocked (awaiting runtime) |
| **Songbird** | `net.announce`, `net.discover` | glob-based socket discovery | Active |
| **toadStool** | `compute.dispatch.submit/result/capabilities` | typed `compute_dispatch` client | Active |
| **barraCuda** | Path dependency (GPU math), `barracuda::health::*` delegations | Compile-time | Active |
| **coralReef** | `SovereignDevice` (sovereign dispatch feature flag) | Feature-gated | Ready, awaiting device |
| **NestGate** | `data.ncbi_search`, `data.ncbi_fetch` | env override → capability probe | Active |
| **rhizoCrypt** | `provenance.session.open/close/record` | capability probe | Active |
| **loamSpine** | `provenance.verify` | capability probe | Ready |
| **sweetGrass** | `provenance.attribute` | capability probe | Ready |
| **Squirrel** | `inference.complete`, `inference.embed`, `inference.models` | optional node, auto-discover | Active (Ollama fallback) |
| **petalTongue** | `render.push`, `render.append`, `render.replace`, `gauge.update` | IPC push client | Active |
| **biomeOS** | Registration, heartbeat, graph execution | well-known socket | Ready |

### Capabilities We Expose

84+ JSON-RPC methods across 10 domains:
- `science.pkpd.*` (15 methods: Hill, compartmental, NLME, NCA, population)
- `science.microbiome.*` (12 methods: diversity, Anderson, SCFA, QS, FMT)
- `science.biosignal.*` (10 methods: QRS, HRV, PPG, EDA, fusion, WFDB)
- `science.endocrine.*` (8 methods: testosterone PK, TRT outcomes, gut axis)
- `science.diagnostic.*` (5 methods: patient assessment, population, composite)
- `science.toxicology.*` (3 methods: biphasic, landscape, hormetic)
- `science.simulation.*` (2 methods: mechanistic fitness, ecosystem)
- `science.discovery.*` (5 methods: MATRIX, HTS, compound, fibrosis, selectivity)
- `science.comparative.*` (4 methods: species PK, canine, feline, cross-species)
- Infrastructure: `capability.list`, `health.*`, `identity.get`, `inference.*` (routed), `compute.*` (routed), `data.*` (routed), `provenance.*` (routed)

### LOCAL vs ROUTED Split

- **LOCAL** (62 methods): All `science.*` capabilities — dispatched in-process
- **ROUTED** (22 methods): `compute.*`, `data.*`, `inference.*`, `provenance.*` — forwarded to canonical providers (toadStool, NestGate, Squirrel, sweetGrass)

---

## Composition Learnings for primalSpring

### 1. Deploy Graph ↔ Proto-Nucleate Drift

**Problem**: The proto-nucleate declares `trust_model = "btsp_enforced"` but the
deploy graph uses `trust_model = "dual_tower_enclave"`. Both are valid but they
describe different enforcement levels.

**Recommendation**: primalSpring should define canonical trust_model vocabulary
(e.g., `dual_tower_enclave` subsumes `btsp_enforced`) and add a validation rule
to `deployment_matrix.toml` checks.

### 2. Fragment Metadata Is Critical

exp118 validates that deploy graph fragment metadata (which NUCLEUS atomics are
present) matches reality. Without this validation, biomeOS could deploy a graph
that claims `nucleus = true` when only Tower + Node are present (missing Nest).

**Recommendation**: primalSpring should ship a `composition_validate` helper crate
that any spring can depend on for automated fragment metadata checks. healthSpring's
exp118 pattern (parse TOML, assert booleans, walk nodes, verify bonding) is
straightforward and reusable.

### 3. Optional Nodes Need Discovery Contracts

Squirrel is optional in healthSpring's deploy graph (`required = false`). The
discovery contract is: probe socket bus for `inference.*` capability → if found,
route to provider → if not found, skip gracefully (no error, no degraded state
for capabilities that don't depend on inference).

**Recommendation**: primalSpring's composition guidance should formalize the
optional-node discovery contract: `by_capability` field → runtime probe →
graceful absence.

### 4. Bonding Policy Enforcement Is Aspirational

healthSpring declares ionic bonds at tower↔node boundaries in the deploy graph,
but BearDog doesn't yet provide `crypto.ionic_bond` runtime. The deploy graph is
*correct* (it declares what *should* happen), but enforcement is deferred.

**Recommendation**: BearDog should ship ionic bond runtime. Until then,
springs should declare bonding policy in deploy graphs but not block
on enforcement — the graph is forward-compatible.

### 5. Capability Surface Completeness Matters

exp118 checks that every capability registered in `ALL_CAPABILITIES` appears in
the deploy graph's capability list for the healthspring node. This caught 13
missing infrastructure capabilities in the first run (they were handled by the
routing layer but not declared in the graph).

**Recommendation**: Capability registration should be the *source of truth* for
deploy graph capability lists. Consider auto-generating the deploy graph capability
section from the primal's `capabilities.list` response.

---

## Composition Patterns for NUCLEUS Deployment via Neural API

### The Full Deployment Flow (as healthSpring understands it)

```
1. primalSpring defines proto-nucleate (desired composition)
2. Spring writes deploy graph (concrete TOML: nodes, deps, bonds, caps)
3. Spring validates deploy graph (Tier 5: exp118 pattern)
4. biomeOS loads deploy graph → resolves node binaries → checks deps
5. biomeOS starts primals in dependency order (deploy graph ordering)
6. Each primal binds socket → creates domain symlink → registers with biomeOS
7. biomeOS Neural API receives graph execution request (workflow TOML)
8. Neural API routes capabilities to primals via IPC
9. Pathway Learner optimizes graph execution over time
```

### What biomeOS Needs from Each Spring

| Requirement | healthSpring Implementation |
|-------------|---------------------------|
| UniBin binary | `healthspring_primal serve` / `server` / `version` / `capabilities` |
| TCP + UDS | `--port` flag, `HEALTHSPRING_PORT` env, UDS default |
| Domain symlink | `health.sock` → `healthspring-{family}.sock` |
| Capability list | 84+ methods, `methods: [string]` array, LOCAL/ROUTED metadata |
| Health probes | `health.liveness` (unconditional), `health.readiness` (subsystem status) |
| Identity | `identity.get` (name, version, domain, license, proto-nucleate ref) |
| SIGTERM | Clean socket removal, graceful shutdown |
| Deploy graph | Validated TOML with fragment metadata, bonding, nodes, capabilities |

### What Springs Should Hand Back to primalSpring

After each composition validation spiral:
1. **Gaps discovered** → `docs/PRIMAL_GAPS.md` (numbered, status-tracked)
2. **New patterns proven** → wateringHole handoff with adoption path
3. **Fragment metadata corrections** → if proto-nucleate/deploy graph don't align
4. **Capability surface evolution** → new capabilities that need deploy graph/proto-nucleate updates
5. **Trust model questions** → bonding policy mismatches or unclear enforcement

---

## Evolution Opportunities (What Primals Can Absorb)

### For primalSpring
- **exp118 pattern** → generic `composition_validate` crate for all springs
- **Fragment metadata validation** → deployment_matrix.toml CI check
- **Trust model vocabulary** → canonical enum for deploy graph trust_model field
- **Optional node contract** → formalized `by_capability` + graceful absence

### For barraCuda
- **TensorSession** → healthSpring has 6 fused-pipeline patterns waiting for
  multi-op TensorSession API. When shipped, healthSpring can remove local
  individual-dispatch wrappers (Write → Absorb → Lean).
- **Anderson eigensolve** → GPU shader candidate for gut lattice localization
- **NLME GPU** → FOCE per-subject gradient, VPC Monte Carlo shader candidates

### For neuralSpring / Squirrel
- **Inference routing** → healthSpring already routes `inference.*` to Squirrel;
  when neuralSpring ships WGSL ML shaders, the routing is transparent
- **Canonical namespace** → `inference.complete` / `inference.embed` / `inference.models`
  is what healthSpring uses; confirm or evolve

### For BearDog
- **BTSP client** → `ipc/btsp.rs` is a pure-Rust BTSP client handshake;
  ready for server endpoint
- **Ionic bond runtime** → deploy graph declares ionic bonds; runtime enforcement
  awaiting `crypto.ionic_bond` capability

### For biomeOS
- **Deploy graph validated** → healthSpring's `healthspring_niche_deploy.toml` has
  been structurally validated (exp118, 99 checks); ready for Neural API loading
- **Workflow graphs** → 4 workflow graphs (patient assessment, TRT scenario,
  microbiome analysis, biosignal monitor) ready for execution
- **Feature gate** → biomeOS feature gate in healthSpring CI still pending;
  will auto-pass composition jobs when biomeOS CI endpoint available

---

## Validation State

| Metric | Value |
|--------|-------|
| Tests | 985+ |
| Experiments | 90 (84 science + 7 composition Tier 4/5) |
| Composition checks | 172+ (73 Tier 4 + 99 Tier 5) |
| Clippy | 0 warnings (pedantic + nursery) |
| Unsafe | 0 (`forbid(unsafe_code)`) |
| Coverage target | 90%+ (llvm-cov, lib + integration) |
| barraCuda | v0.3.11 (7f6649f) |
| ecoBin | 0.8.0 (static-PIE x86_64-musl) |

---

## Next Spiral

The composition evolution spiral continues:
1. primalSpring evolves proto-nucleate based on this handoff
2. BearDog ships ionic bond runtime → bonding policy enforced
3. neuralSpring ships WGSL ML → `inference.*` goes native
4. barraCuda ships TensorSession → local shaders absorbed upstream
5. healthSpring validates next composition iteration → Tier 5 evolves
6. Cycle continues
