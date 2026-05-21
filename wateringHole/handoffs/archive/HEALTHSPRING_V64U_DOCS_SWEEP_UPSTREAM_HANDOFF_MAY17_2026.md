# healthSpring V64u — Docs Sweep + Science Expansion Upstream Handoff

**Date**: May 17, 2026
**From**: healthSpring (V64u)
**To**: primalSpring, all delta springs, upstream primal teams
**Registry**: 452 methods (Wave 20: `primal.list`)

---

## What Changed (V64r → V64u)

### V64t: Science Expansion — 32 New Validation Scenarios

healthSpring expanded its validation scenario registry from 18 to **50 scenarios**,
covering all 95 experiments across 7 science tracks:

| Track | New Scenarios | Total | Source Experiments |
|-------|:------------:|:-----:|-------------------|
| PK/PD | 3 | 10 | exp003, exp004, exp006 |
| Microbiome | 4 | 8 | exp012, exp013, exp078, exp079 |
| Biosignal | 4 | 8 | exp022, exp023, exp081, exp082 |
| Endocrine | 8 | 9 | exp031-038 |
| Discovery | 5 | 7 | exp091-094, exp111 |
| Toxicology | 2 | 4 | exp098, exp099 |
| Comparative | 6 | 8 | exp101-106 |
| Infrastructure | 0 | 4 | (unchanged) |

Each scenario follows the standard pattern: `SCENARIO()` → `ScenarioMeta` + `run()`,
exercising core science functions from production modules with structural validation
checks. Registry is organized by track with section comments in `build_registry()`.

### V64u: Docs Sweep

All root-level documentation synchronized to V64u:
- `README.md`, `CONTEXT.md`, `whitePaper/README.md`, `whitePaper/baseCamp/README.md`,
  `whitePaper/METHODOLOGY.md` (v0.4), `experiments/README.md`, `specs/README.md`,
  `wateringHole/README.md`, `CHANGELOG.md`, `docs/PRIMAL_GAPS.md`

Wave 20 debt item resolved: `wateringHole/README.md` status line updated from V64o/451
to V64u/452.

### Dataset Checksum Manifest

`config/dataset_checksums.toml` created — documents all external data dependencies
(NCBI Gene, PhysioNet MIT-BIH, ChEMBL JAK panel) with source URLs, verification
status, and fetch gaps. qs_gene_matrix fetch gap formally documented.

---

## Composition Patterns — What healthSpring Proved

### 1. Nest Atomic End-to-End

healthSpring is the **Nest Atomic Specialist**. The `NestComposition` facade
(`ipc/provenance/nest.rs`) orchestrates:

```
session.create → event.append → crypto.sign → content.put
    → dag.event.append → spine.seal → braid.create → session.commit
```

Signal-first: tries `ctx.dispatch("nest.store")` / `ctx.dispatch("nest.commit")`
via biomeOS before falling back to manual multi-call chain. This pattern validated
the full provenance trio (rhizoCrypt + loamSpine + sweetGrass) through NestGate.

**Learnings for other springs**:
- Signal dispatch with manual fallback is the right pattern — biomeOS may not be
  running during development/testing
- The `ferment transcript` model (session → events → dehydrate → mint) maps
  naturally to scientific experiment lifecycles
- `certificate.verify` should be consumed but not required — validation works
  offline with structural checks

### 2. Capability Discovery — Domain-Based Routing

healthSpring routes 88 capabilities across 8 science domains + infrastructure.
The routing table (`composition/routing.rs`) maps capability prefixes to primal
names at runtime — zero compile-time coupling.

```rust
match domain {
    "science" => HEALTHSPRING,      // local science
    "security" | "crypto" | "fido2" => BEARDOG,
    "content" | "storage" => NESTGATE,
    "signal" | "graph" | "plan" => BIOMEOS,
    ...
}
```

**Learnings for other springs**:
- Keep routing domain-based, not method-based — new methods appear without
  routing changes
- `normalize_method()` for legacy name compatibility (e.g., `capabilities.list`
  → `capability.list`) prevents breaking changes
- `capability_domains()` helper for the canonical envelope is a one-liner

### 3. Validation Scenario Pattern

The 50-scenario registry demonstrates scalable science validation:

```rust
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "hill-dose-response",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp001",
            description: "...",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — ...");
    // Call production science functions, check structural properties
}
```

**Learnings for other springs**:
- Scenarios test structural properties (monotonicity, bounds, conservation laws),
  not exact values — robust against parameter tuning
- One scenario per experiment gives 1:1 traceability
- `check_abs_or_rel()` for numerical checks, `check_bool()` for structural
- Named tolerances from `tolerances.rs` — never magic numbers

### 4. Wave 20 `capability.list` Canonical Envelope

healthSpring's response includes both the canonical subset and enriched fields:

```json
{
    "capabilities": ["biosignal", "comparative", "diagnostic", ...],
    "count": 12,
    "primal": "healthSpring",
    "version": "0.9.0",
    "methods": ["science.pkpd.hill", ...],
    "total": 88,
    "science": [...],
    "infrastructure": [...]
}
```

**Pattern**: canonical fields first, enriched fields alongside. Never break the
canonical subset. `capability_domains()` extracts unique top-level domains from
`ALL_CAPABILITIES` + `ALL_CAPS`.

---

## Upstream Primal Gaps — Still Open

### sweetGrass TCP (GAP-42)
`sweetGrass` listens on TCP port `5488` instead of Unix domain socket. All other
primals use UDS. healthSpring's `try_sweet_grass()` probes TCP as fallback, but
this breaks the UDS-only assumption.

**Ask**: sweetGrass team — plan for UDS migration?

### toadStool Sandbox (GAP-43)
`toadStool.sandbox.create` returns `-32601 MethodNotFound`. Sandbox isolation
for GPU workloads is documented but not implemented.

**Ask**: toadStool team — sandbox timeline?

### barraCuda/coralReef GPU Parity
healthSpring has 6 validated WGSL shaders. The `coralReef` sovereign shader
compiler (v0.1.0) supports Blackwell + dual-vendor (NVIDIA + AMD). healthSpring's
shaders are `wgpu`-native; coralReef migration is a future opportunity.

**Ask**: coralReef team — naga::Module ingest path for existing WGSL?

### biomeOS Schema Validation
No delta spring has a `s_schema_standard` scenario that probes live biomeOS
`capability.list` and `primal.list` response shapes. healthSpring's
`integration_registry_sync` test validates registry cross-sync but not live
biomeOS response shapes.

**Candidate**: healthSpring could add `s_schema_standard` when biomeOS
stabilizes v3.57+ response shapes.

---

## For Upstream primalSpring

1. **50 scenarios** — healthSpring now has the second-largest scenario registry
   after primalSpring (41). Scenario patterns are stable and reusable.
2. **Wave 20 fully absorbed** — canonical envelope, 452 registry, `primal.list`
   consumed, `nest.commit` signal-path confirmed.
3. **Zero debt** — all 7 categories across V64i/V64p/V64s audits confirmed.
4. **Foundation Thread 10** — PK/PD validation is a strong provenance workload
   candidate. `--provenance-dir` ready to add when Foundation calls.
5. **Foundation Threads 3+8** — Immunology/Human Health are external research
   threads; B5 symbiont PK/PD is a lithoSpore module candidate.

## For Other Delta Springs

- **Scenario expansion pattern**: healthSpring created 32 scenarios in one sprint
  by systematically matching experiments to production functions. Same pattern
  works for any spring with implemented science functions.
- **Dataset manifest**: `config/dataset_checksums.toml` pattern — document
  external data dependencies even if checksums are pending.
- **Tolerance constants**: Named constants in `tolerances.rs` with doc comments
  citing literature sources. Never `assert!((x - y).abs() < 1e-10)`.

---

## Files Changed (V64t–V64u)

### New Files (V64t)
- 32 scenario files in `ecoPrimal/src/validation/scenarios/s_*.rs`
- `config/dataset_checksums.toml`

### Modified Files (V64t–V64u)
- `ecoPrimal/src/validation/scenarios/mod.rs` — 32 new module declarations
- `ecoPrimal/src/validation/scenarios/registry.rs` — `build_registry()` expanded
  to 50 scenarios, organized by track
- `README.md`, `CONTEXT.md`, `CHANGELOG.md`
- `whitePaper/README.md`, `whitePaper/baseCamp/README.md`,
  `whitePaper/METHODOLOGY.md`
- `experiments/README.md`, `specs/README.md`
- `wateringHole/README.md`
- `docs/PRIMAL_GAPS.md`
