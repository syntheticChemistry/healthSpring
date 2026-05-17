# healthSpring V64w — Comprehensive Upstream Handoff

**Date**: May 17, 2026
**From**: healthSpring (V64w — Deep Debt Re-Audit + Science Buildout)
**To**: primalSpring, all delta springs, upstream primal teams (barraCuda, toadStool, coralReef, metalForge, biomeOS, NestGate, rhizoCrypt, loamSpine, sweetGrass)
**Registry**: 452 methods (Wave 20)
**State**: 1,018 tests, 57 scenarios, 0 clippy, 0 unsafe, 0 TODO

---

## What healthSpring Proved

healthSpring is the **Nest Atomic Specialist** — the sixth ecoPrimals spring,
validating health-of-living-systems science (PK/PD, microbiome, biosignal,
endocrine, comparative medicine, drug discovery, toxicology, simulation) from
Python baselines through sovereign Rust + GPU to NUCLEUS composition.

### The Validation Ladder

```
L1  Python baseline     54 scripts + 53 notebooks → DOI-cited peer-reviewed science
L2  Rust CPU            95 experiment binaries → faithful port, named tolerances
L3  barraCuda CPU       6 WGSL shaders → CPU fallback parity (Rust 84× faster)
L4  barraCuda GPU       GPU execution → sovereign shader parity (RTX 4070 validated)
L5  guideStone          57/57 checks → self-verifying binary (bare + IPC)
L6  NUCLEUS             Composition deploy → plasmidBin ecobins on clean machine
```

**This ladder is reusable.** Every spring can follow the same pattern:
Python → Rust → barraCuda CPU → barraCuda GPU → guideStone → NUCLEUS.

### Science Coverage

| Track | Scenarios | Experiments | Key Functions |
|-------|:---------:|:-----------:|---------------|
| PK/PD | 7 | exp001-006, exp077 | Hill, compartmental, PBPK, MM, allometry, population MC |
| Microbiome | 9 | exp010-013, exp078-080, exp107-108 | Shannon, Anderson lattice, FMT, SCFA, serotonin, QS |
| Biosignal | 7 | exp020-023, exp081-082, exp109 | Pan-Tompkins, HRV, PPG SpO2, EDA, beat classification |
| Endocrine | 9 | exp030-038 | Testosterone PK, TRT outcomes, gut axis, HRV cross-track |
| Discovery | 7 | exp090-096, exp111 | MATRIX, HTS, compound library, JAK panel, fibrosis, iPSC, niclosamide |
| Toxicology | 3 | exp097-099 | Biphasic dose-response, toxicity landscape, hormesis |
| Comparative | 8 | exp100-106, exp110 | Canine IL-31/JAK1, feline methimazole, equine laminitis, cross-species PK |
| Composition | 7 | exp119-123 + nest_atomic | IPC parity, provenance, health, barraCuda IPC, NUCLEUS |

---

## Primal Use and Evolution — What healthSpring Consumed

### Tower Atomic (bearDog + songBird + skunkBat)

healthSpring validates the full Tower via `s_nest_atomic` and deploy graphs:
- **bearDog** `crypto.sign` — base64 `message` field (wire correction absorbed upstream)
- **songBird** — `announce_to_songbird` for biomeOS registration
- **skunkBat** `security.audit_log` — in all 7 deploy graphs

**Learning**: Tower should always appear as a triple. Deploy graphs that omit
skunkBat are incomplete — we caught this at V64n and every graph since includes it.

### Node Atomic (barraCuda + toadStool + coralReef)

healthSpring is the deepest Node consumer among delta springs:
- **barraCuda v0.4.0** — 6 WGSL shaders (Hill, PopPK, Diversity, MM, SCFA, Beat),
  `execute_cpu` fallback for all ops, GPU crossover analysis
- **toadStool** — `Pipeline` with `execute_cpu`, `execute_gpu`, `execute_streaming`,
  `execute_auto` (substrate-aware). StageOps for all V16 primitives.
- **coralReef** — Referenced in exp123 for sovereign shader compilation; healthSpring's
  WGSL shaders are `wgpu`-native, coralReef migration is future

**metalForge** — Substrate discovery (`Capabilities::discover`, `select_substrate`),
dispatch planning (`plan_dispatch`, `plan_transfer`), PCIe P2P bypass modeling,
9 `Workload` variants for mixed Tower/Node/Nest routing.

### Nest Atomic (NestGate + rhizoCrypt + loamSpine + sweetGrass)

healthSpring owns the **Nest Atomic validation pattern**:

```
session.create → event.append → crypto.sign → content.put
  → dag.event.append → spine.seal → braid.create → session.commit
```

`NestComposition` facade (`ipc/provenance/nest.rs`) tries signal dispatch first:
`ctx.dispatch("nest.store")` / `ctx.dispatch("nest.commit")` via biomeOS, then
falls back to manual multi-call chain. This dual-path pattern is the reference
implementation for provenance-heavy springs.

### biomeOS Neural API

healthSpring consumes biomeOS for:
- `capability.list` / `primal.list` canonical envelope
- `signal.dispatch` for nest.store / nest.commit
- `primal.announce` single-call registration (replaces 3-call pattern)
- Graph execution via Neural API Pathway Learner
- 7 deploy graphs (niche, patient assessment, TRT scenario, microbiome analysis,
  biosignal monitor, nest atomic, niche deploy)

---

## Composition Patterns for NUCLEUS Deployment

### Pattern 1: Domain-Based Capability Routing

```rust
match domain {
    "science" => HEALTHSPRING,      // local science calls
    "security" | "crypto" => BEARDOG,
    "content" | "storage" => NESTGATE,
    "signal" | "graph" => BIOMEOS,
    "compute" | "gpu" => BARRACUDA,
    _ => UNKNOWN,
}
```

New methods appearing under existing domains route automatically — no code changes.
`normalize_method()` handles legacy name compatibility (e.g., `capabilities.list` → `capability.list`).

### Pattern 2: Signal-First with Manual Fallback

```rust
match ctx.dispatch("nest.store", params).await {
    Ok(_) => { /* biomeOS handled the graph */ },
    Err(_) => {
        // Manual chain: content.put → dag.event.append → spine.seal → braid.create
        ctx.call("content.put", ...)?;
        ctx.call("dag.event.append", ...)?;
        // ...
    }
}
```

Adopt signal dispatch immediately — the fallback means it works with or without biomeOS.

### Pattern 3: Scenario-Based Validation

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
    v.section("Phase 1: Structural — monotonicity");
    // Call production science functions, check structural properties
    v.check_bool("hill_monotone", resp_high > resp_low, &format!("..."));
}
```

Scenarios test **structural properties** (monotonicity, bounds, conservation laws),
not exact values — robust against parameter tuning. Named tolerances from
`tolerances.rs` with DOI citations. Never `assert!((x - y).abs() < 1e-10)`.

### Pattern 4: Deploy Graph Composition

healthSpring's 7 deploy graphs follow the NUCLEUS atomic model:

```toml
[[nodes]]
name = "skunkBat"
capability = "security.audit_log"
required = true

[[nodes]]
name = "bearDog"
capability = "crypto.sign"
required = true

[[edges]]
from = "healthSpring"
to = "skunkBat"
trigger = "on_session_start"
```

Every graph includes all three Tower primals (bearDog + songBird + skunkBat).
Nest primals (NestGate, rhizoCrypt, loamSpine, sweetGrass) appear in provenance graphs.
Node primals (barraCuda, toadStool) appear in compute graphs.

---

## Upstream Primal Gap Asks

### sweetGrass TCP (GAP-42)
sweetGrass listens on TCP port 5488 instead of Unix domain socket. healthSpring's
`try_sweet_grass()` probes TCP as fallback. **Ask**: Plan for UDS migration?

### toadStool Sandbox (GAP-43)
`toadStool.sandbox.create` returns `-32601 MethodNotFound`. Sandbox isolation for
GPU workloads is documented but not implemented. **Ask**: Sandbox timeline?

### coralReef WGSL Ingest
healthSpring has 6 validated WGSL shaders. coralReef v0.1.0 supports Blackwell +
dual-vendor. **Ask**: naga::Module ingest path for existing WGSL?

### biomeOS Schema Validation
No delta spring has a `s_schema_standard` scenario that probes live biomeOS
response shapes. healthSpring could add this when biomeOS stabilizes v3.57+.

### Dataset Fetch Gaps
5 external datasets have pending checksums in `config/dataset_checksums.toml`:
qs_gene_matrix (NCBI), mitbih_records (PhysioNet), ncbi_gene_il31, ncbi_gene_jak1,
chembl_jak_panel. These require NestGate data provider wiring (Tier 2+).

---

## For Other Delta Springs

### Scenario Expansion Pattern
healthSpring expanded from 18 → 57 scenarios in two sprints by systematically
matching experiment binaries to production science functions. Key technique:
read the experiment `main.rs`, identify the science functions called, write a
scenario that exercises the same functions with structural checks. One scenario
per experiment ID gives 1:1 traceability.

### Clippy Pedantic+Nursery Patterns
Common fixes across scenario code:
- `x >= 0.0 && x <= 1.0` → `(0.0..=1.0).contains(&x)` (manual_range_contains)
- Similar variable names (`pac_class` / `pvc_class`) → distinct names (`atrial_class` / `ventricular_class`)
- Float-to-usize casts → `#[expect(clippy::cast_possible_truncation)]` with documented reason

### Dataset Manifest Pattern
`config/dataset_checksums.toml` documents all external data dependencies with
source URLs, SHA-256 checksums (or "pending"), and fetch gap descriptions.
Even synthetic/literature datasets get entries with `sha256 = "N/A"`.

### Named Tolerance Pattern
Every numerical check uses a named constant from `tolerances.rs`:
```rust
pub const MACHINE_EPSILON: f64 = 1e-12;
pub const DETERMINISM: f64 = 0.0;
pub const HILL_SATURATION_100X: f64 = 0.99;
```
Each constant has a doc comment citing the literature source or derivation.

---

## LTEE Foundation Threads

| Thread | Domain | healthSpring Contribution | Status |
|--------|--------|--------------------------|--------|
| 3 | Immunology | External research thread | Assessed |
| 8 | Human Health | External research thread | Assessed |
| 10 | Provenance/Economics | PK/PD provenance workload candidate | Ready |

**B5 symbiont PK/PD** is complete (Python + Rust `validate_ltee_b5`, 8/8 checks).
E2 (HOLIgraph) and E4 (macrocyclic peptide) remain queued — experiment IDs TBD.

---

## Deep Debt — Final State

| Category | Count | Detail |
|----------|:-----:|--------|
| TODO/FIXME/HACK | **0** | 246 .rs files audited |
| Unsafe code | **0** | `forbid(unsafe_code)` workspace-wide |
| Files >800L | **0** | Max 597 lines (test-only) |
| Hardcoding | **0** | Domain routing, convention socket paths |
| Production mocks | **0** | All in `#[cfg(test)]` |
| unwrap/expect in prod | **0** | Workspace lint denies |
| Clippy warnings | **0** | pedantic + nursery, promoted to error |
| External deps to evolve | **0** | All pure Rust, no C FFI |

---

## Files Changed (V64v–V64w)

- All root documentation synced to V64w: `README.md`, `CONTEXT.md`,
  `whitePaper/README.md`, `whitePaper/baseCamp/README.md`,
  `whitePaper/METHODOLOGY.md` (v0.5), `specs/README.md`,
  `specs/PAPER_REVIEW_QUEUE.md`, `wateringHole/README.md`, `docs/PRIMAL_GAPS.md`
- This handoff document
