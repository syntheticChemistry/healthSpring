<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V64 — Upstream Primal & Spring Handoff

**Date**: May 12, 2026
**Version**: V64
**primalSpring**: v0.9.25 (pinned)
**Tests**: 1,014 (874 lib + 9 doc + 131 integration/workspace)
**Capabilities**: 88 JSON-RPC methods (58 science + 30 infra)
**Architecture**: Eukaryotic UniBin + IPC-first defaults + 4 NUCLEUS workloads + 17 validation scenarios + Tier 2 wired

---

## What Changed in V64

### LTEE B5 Tier 1 — `validate_ltee_b5` Rust Binary

Leonard et al. 2024 "Symbiont PK/PD" (LTEE B5) is now a **complete Tier 0+1
reproduction**: Python baseline (`control/ltee_symbiont_pkpd/`) plus Rust
validation binary (`ecoPrimal/src/bin/validate_ltee_b5.rs`), 8/8 checks PASS.

The binary validates four models against analytical expectations:
- Logistic colonization dynamics (steady-state, growth rate, half-colonization time)
- Biomass-proportional metabolite production (steady-state ∝ biomass)
- One-compartment gut-lumen PK (peak timing, clearance, absorption)
- Hill dose-response efficacy (EC50, max efficacy, slope)

**Upstream relevance (lithoSpore)**: Module candidate `ltee-symbiont-pk`. The
`control/ltee_symbiont_pkpd/expected_values.json` + Rust binary are at the same
readiness level as groundSpring's B1–B3 data. BLAKE3-hash and ingest directly.

**Upstream relevance (groundSpring)**: healthSpring's B5 reproduction follows
your B1–B3 pattern — `expected_values.json` + `validate_ltee_*` binary with
`ValidationHarness`. No format changes needed.

### `--format json` on Validation Binaries

Both `validate_pk_models` and `validate_ltee_b5` now accept `--format json`,
producing structured output:

```json
{"name":"validate_ltee_b5","passed":8,"failed":0,"total":8}
```

**Upstream relevance (projectNUCLEUS)**: This enables Tier 2 ingestion — your
workload pipeline can capture structured pass/fail counts without parsing human
output. All springs with validation binaries should adopt this convention.

**Upstream relevance (all springs)**: Pattern to adopt: check `std::env::args()`
for `--format` + `json`, use `ValidationHarness::silent()` to suppress stdout
during validation, then emit the JSON line on finish.

### Foundation Threads 3, 5, 8 — Expressions Wired, Status → Active

| Thread | Title | Expression | Status Change |
|--------|-------|-----------|---------------|
| 3 | Immunology/Drug Discovery | `IMMUNO_DRUG_DISCOVERY.md` | mapped → **active** |
| 5 | LTEE/Evolutionary Biology | `LTEE_EVOLUTIONARY_DYNAMICS.md` | seeded → **active** (springs: +healthSpring, +airSpring) |
| 8 | Human Health/Clinical | `SOVEREIGN_HEALTH.md` | mapped → **active** |

Thread 5 now lists healthSpring (B5) and airSpring (E3) as contributing springs,
with B5/E2/E3/E4 papers in the reproduction table.

**Upstream relevance (sporeGarden/foundation)**: 8/10 threads now have
expressions (T3, T5, T8 newly active). Remaining: T4 (Environmental Genomics —
wetSpring owns), T9 (Gaming/Creative — ludoSpring owns), T10 (Provenance/Economics
— needs co-seeding with primalSpring).

**Upstream relevance (airSpring)**: Thread 5 now references your E3 (Dolgikh
FLS2) reproduction. When E3 baseline ships, update the thread targets.

### `s_toxicology` Validation Scenario (17th Scenario)

`Track::Toxicology` had a registered taxonomy slot but no scenario exercising it.
V64 ships `s_toxicology` with 9 structural checks:
- `compute_toxicity_landscape`: tissue count, systemic burden, IPR bounds, clearance
- `biphasic_dose_response`: zero/low/high dose, hormetic optimum existence and bounds

**Upstream relevance (all springs)**: If your spring has domain tracks without
scenarios, fill them — dead taxonomy slots are tech debt that erodes trust in
the scenario registry.

### Tier 2 Wiring — `toadstool.validate` + `precision.route` (V64e)

Ecosystem wave sync response: Tier 2 Live Science API is now unblocked.

**`compute_dispatch.rs`** — two new typed IPC clients:
- `validate_workload(path)` → `ValidationReport` — wraps `toadstool.validate` for workload pre-flight (valid, gpu_available, precision_tier, estimated_dispatch_time_ms, warnings, required_capabilities)
- `list_workloads()` → `Vec<String>` — wraps `toadstool.list_workloads` for available compute workloads

**`barracuda_client.rs`** — precision advisory:
- `precision_route(domain)` → `PrecisionAdvisory` — wraps `barracuda.precision.route` for domain-aware precision tier recommendations (e.g. `population_pk` → FP64/compute, `eigensolve` → DF64/tensor_core)

**Upstream relevance (all springs)**: Pattern to adopt — create typed IPC wrappers for `toadstool.validate` and `barracuda.precision.route`. Use `discover_compute_primal()` for capability-first socket discovery. The `ValidationReport` and `PrecisionAdvisory` structs model the full Tier 2 wire contract from `primalSpring/docs/LIVE_SCIENCE_API.md`.

**Upstream relevance (toadStool)**: healthSpring calls `toadstool.validate` with `dry_run: true` and parses the S250 response schema. If the response shape changes, update `ValidationReport` fields.

**Upstream relevance (barraCuda)**: healthSpring calls `precision.route` with `domain` param and parses `tier`, `hardware_hint`, `compiler_required`. The `population_pk`, `eigensolve`, `bioinformatics`, `statistics` domains are directly relevant to our science tracks.

### Ionic Bridge IPC Stubs (V64b)

`TowerAtomic` now has `ionic_propose`, `ionic_countersign`, and `ionic_verify` methods calling `BearDog`'s `crypto.contract.*` JSON-RPC methods. These are the first steps toward cross-tower/cross-family trust bonds. Tests verify graceful failure when BearDog is unavailable.

**Upstream relevance (bearDog)**: healthSpring is ready to exercise the propose → countersign → verify lifecycle when bearDog ships `crypto.contract.*` handlers.

### `healthspring_unibin validate --format json` (V64b)

The UniBin `validate` subcommand now accepts `--format json`, producing structured output with pass/fail/skipped counts. Uses primalSpring's `NullSink` to suppress verbose output in JSON mode.

### Dead Feature Gate Cleanup

The `npu = []` feature in `Cargo.toml` was declared but never used (no
`#[cfg(feature = "npu")]` anywhere). Removed. The `guidestone` feature comment
was clarified to explain its purpose (Level 5 lineage verification).

### Experiment Primal Name Centralization

`exp115` and `exp118` replaced hardcoded `"biomeos"`, `"beardog"`, `"nestgate"`,
etc. with `primal_names::*` constants. This completes the centralization started
in V63 — zero hardcoded primal name strings remain across the entire codebase.

---

## Audit Results

| Metric | Value |
|--------|-------|
| Tests | **1,014** (874 lib + 9 doc + 131 integration/workspace) |
| Clippy warnings | **0** |
| Unsafe blocks | **0** (`forbid(unsafe_code)`) |
| TODO/FIXME/HACK | **0** in production code |
| `unwrap()` / `panic!` in production | **0** |
| Mocks in production | **0** (all mocks isolated to `#[cfg(test)]`) |
| Files >800 lines | **0** |
| Hardcoded primal name strings | **0** |
| Validation scenarios | **17** (16 + s_toxicology) |
| Validation binaries | **3** (unibin, validate_pk_models, validate_ltee_b5) |
| Foundation threads (healthSpring-owned) | **3 active** (T3, T5 co-owned, T8) |
| Python baselines | **54** scripts + **53** notebooks |
| Provenance entries | **96+** (100% coverage) |

---

## Composition Patterns for NUCLEUS Deployment via neuralAPI

### The Three-Stage Validation Ladder

```
Python baseline (Tier 0) → Rust binary (Tier 1) → NUCLEUS composition (Tier 2+)
```

Every healthSpring science claim follows this ladder. V64 extends it:
- Tier 0: 54 Python baselines (53 track scripts + 1 LTEE B5)
- Tier 1: 3 Rust validation binaries (unibin certify, validate_pk_models, validate_ltee_b5)
- Tier 2: `--format json` enables projectNUCLEUS pipeline ingestion
- Tier 3–5: 12 composition experiments (exp112–123) validate IPC dispatch parity

### biomeOS Deployment Model

healthSpring deploys as a biomeOS niche via `healthspring_primal` (88 JSON-RPC
capabilities over Unix socket). The deployment graph
(`graphs/healthspring_niche_deploy.toml`) specifies:

- **barraCuda**: GPU compute (Hill, PopPK, diversity shaders)
- **toadStool**: Pipeline dispatch (stage→GPU routing)
- **nestgate**: Data provider (PubMed fetch, storage)
- **beardog**: Identity + crypto signing (ionic bridge ready)
- **skunkBat**: Defense node
- **rhizoCrypt + loamSpine + sweetGrass**: Provenance trio

All primal interactions use `CompositionContext` for runtime discovery. No
compile-time coupling. `primal_names.rs` centralizes wire conventions.

### Workload TOML Convention

4 workloads staged under `projectNUCLEUS/workloads/healthspring/`:
1. PK validation workload
2. Biosignal validation workload
3. Microbiome validation workload
4. Certification workload

Binary invocation: `healthspring validate --format json` or
`healthspring certify`.

---

## Evolution Patterns for Upstream Teams

### For barraCuda Team

1. **Parameter struct pattern**: healthSpring's `DosingRegimen`,
   `PopulationPkVariability`, `ToxicityModelParams`, `AntibioticSimConfig` replaced
   all multi-parameter function signatures. Consider this for GPU dispatch configs.
2. **Wire prefix convention**: `primal_names::wire_prefix::BARRACUDA` centralizes
   all JSON-RPC method normalization. A prefix change requires one constant update.

### For biomeOS / neuralAPI Team

1. **Socket discovery**: All socket paths go through `primal_names::BIOMEOS_DIR_NAME`
   and `primal_names::FALLBACK_SOCKET_DIR`.
2. **Capability-based routing**: healthSpring demonstrates domain + infrastructure
   capability separation (`science.*` for domain, `health.*` for probes).
3. **NUCLEUS workloads**: 4 staged, all use `--format json` for structured output.

### For lithoSpore Team

1. **LTEE B5 ready**: `control/ltee_symbiont_pkpd/expected_values.json` +
   `validate_ltee_b5` binary. Same format as groundSpring B1–B3.
2. **Module candidate**: `ltee-symbiont-pk` covering colonization dynamics,
   metabolite production, gut-lumen PK, and Hill efficacy.
3. **LTEE queue**: E2 (Lenski 2017, 50K generations fitness dynamics) and E4
   (Dollhopf/Liu FMT) are queued — Python baselines next.

### For sporeGarden/foundation Team

1. **Thread 3 (Immunology)**: Expression wired, Papers 12+13+22 mapped, 5 springs.
2. **Thread 5 (LTEE)**: healthSpring + airSpring added, B5/E2/E3/E4 papers in table.
3. **Thread 8 (Human Health)**: Expression wired, 6 tracks mapped to targets.
4. **Thread coverage**: 8/10 active; T4 (wetSpring), T9 (ludoSpring), T10
   (primalSpring co-seed) remain.

### For All Springs

1. **`--format json`**: Adopt on your validation binaries for projectNUCLEUS
   ingestion. Pattern: `ValidationHarness::silent()` + JSON line on `finish()`.
2. **Validation scenario coverage**: Fill dead taxonomy slots. healthSpring's
   `s_toxicology` closed the last dead slot — check yours.
3. **Primal name centralization**: Use a `primal_names.rs` or equivalent. Zero
   hardcoded strings means one-constant adaptation when primals evolve conventions.
4. **Fossil record**: Archive prokaryotic-era sources under `fossilRecord/` with
   dated subdirectories rather than deleting them.

### For primalSpring Coordination

1. **healthSpring is V64, audit-clean**: Zero clippy, zero unsafe, zero TODO,
   zero mocks in production, zero files >800L. The interstadial exit gate for
   healthSpring is met on all mechanical metrics.
2. **LTEE progress**: B5 COMPLETE (Tier 0+1). E2, E4 queued. Each reproduction
   feeds lithoSpore.
3. **Foundation**: T3+T8 expressions written, T5 updated. 8/10 threads active.
4. **Remaining blocked items**: Ionic bridge (beardog upstream), NestGate content
   pipeline, Songbird canonical name resolution — documented in `docs/PRIMAL_GAPS.md`.

---

## Forward Work (Low Priority)

- **LTEE E2** (Lenski 2017, 50K-gen fitness): Python baseline → Rust binary
- **LTEE E4** (Dollhopf/Liu FMT): Python baseline → Rust binary
- **NLME GPU shaders**: FOCE/SAEM on GPU via barraCuda v0.4.x
- **NestGate integration**: Data provider spec exists; blocked on NestGate endpoint
- **Foundation T4+T9+T10**: Not healthSpring-owned but available for contribution

---

## Supersedes

- [V63 handoff](archive/HEALTHSPRING_V63_UPSTREAM_HANDOFF_MAY11_2026.md) — now archived
