# Changelog

All notable changes to healthSpring are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses internal versioning (V-series) for development milestones.

## V64i — May 13, 2026

### Deep Debt Resolution + Evolution Sprint

- **Clippy pedantic+nursery: zero warnings, zero errors** — full `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery` passes clean. Fixed: `match` → `if let` (3), `unwrap()` → `f64::total_cmp` (3), `to_owned()` → `extract_str` helper (6), redundant closures (3), `i32 as f64` → `f64::from` (7), unfulfilled `#[expect]` attrs (3), doc backticks (7), `const fn` promotions (2), `unwrap_or` → `unwrap_or_else` (1), `&Option<T>` → `Option<&T>` (1), function too-many-lines refactor (1).
- **Hardcoded `"healthSpring"` → `crate::PRIMAL_NAME`** — provenance session JSON in `data/provenance.rs` now uses the canonical `PRIMAL_NAME` constant instead of string literals.
- **`s_nest_atomic` refactored** — 9-phase validation decomposed into per-phase functions (`phase1_structural` through `phase9_chain_audit`) with shared `ChainState` struct. Satisfies pedantic `too_many_lines` lint without arbitrary splitting.
- **Bench casts evolved** — `cpu_parity.rs` population benchmarks use `f64::from` instead of `i as f64` casts. Stale `#[expect(cast_precision_loss)]` removed.
- **Test sort evolved** — `tridiagonal_ql_local.rs` uses `f64::total_cmp` method reference instead of `partial_cmp().unwrap()`.

### Deep Debt Audit Results (zero-debt confirmed)

- **TODO/FIXME/HACK**: 0 across entire codebase
- **`unsafe` code**: 0 — `#![forbid(unsafe_code)]` enforced at lib + workspace level
- **Production mocks**: 0 — all mocks isolated to `#[cfg(test)]`
- **`unimplemented!`/`todo!`/`panic!`** in non-test code: 0
- **Files > 800 LOC**: 0 (largest: 597 lines)
- **Rust edition**: 2024 (rust-version 1.87)
- **External C deps**: 0 in default build. `ring` (via `ureq`→`rustls`) gated behind `nestgate` feature. `wgpu` GPU backends gated behind `gpu` feature.
- **Hardcoded routing**: 0 — all primal routing via `primal_names::*` constants and capability-based discovery

## V64h — May 13, 2026

### Nest Atomic Validation Sprint

- **`s_nest_atomic` validation scenario** — 9-phase validation exercising all 7 Nest primals (bearDog, songbird, skunkBat, nestGate, rhizoCrypt, loamSpine, sweetGrass) through clinical data pipelines. Phases: structural routing → liveness → NestGate storage round-trip → rhizoCrypt DAG chain → BearDog Merkle signing → loamSpine ledger append → sweetGrass attribution braid → Tower auxiliary → chain recoverability audit.
- **`healthspring_nest_atomic.toml` deploy graph** — 7-node Nest Atomic graph with ionic bonding, MethodGate trust, and correct dependency ordering. Registered in `healthspring_niche.toml`.
- **`NestComposition` capability domain fix** — `record_event` now routes through `"storage"` domain (was `"data"`), aligned with `capability_to_primal("storage") == nestgate`.
- **Gaps #34–37 surfaced** — wire name divergence (`content.*` vs `storage.*`, `ledger.entry.append` vs `entry.append`), trio UDS blocking live exercises, facade domain misroute.
- **Shared checklist COMPLETE** — deploy graph ✓, composition start ✓, liveness ✓, capabilities.list ✓, real data ✓, honest skip ✓, `--format json` ✓, gaps documented ✓.

## V64g — May 13, 2026

### Provenance Elevation — Auditable Data Chains

- **Phase 1: Python baseline provenance strengthened** — `expected_values.json` and `tolerances.toml` created for all 7 science tracks (pkpd, endocrine, microbiome, comparative, biosignal, discovery, toxicology, simulation). DOIs added for 30+ papers across all tracks.
- **Phase 1: `records_science.rs` DOIs updated** — 18 `ProvenanceRecord` entries now include explicit DOIs (previously journal-only references). Citations aligned to `control/<track>/expected_values.json`.
- **Phase 2: Provenance IPC wire shape unified** — `data/provenance.rs` refactored from `capability.call` envelope pattern to canonical JSON-RPC method names (`dag.session.create`, `dag.event.append`, `dag.dehydrate`, `commit.create`, `braid.create`) matching `ipc/provenance/*.rs` and `LIVE_SCIENCE_API.md`. All 8 provenance tests pass.
- **Phase 3: `NestComposition` facade** — `ipc/provenance/nest.rs` orchestrates the full Nest Atomic chain (NestGate → rhizoCrypt → BearDog → loamSpine → sweetGrass) as a single composed unit. Builder-pattern API: `begin_session() → record_event() → sign_merkle() → commit() → attribute() → finalize()`. Graceful degradation at each step. 4 new tests.

## V64f — May 13, 2026

### Tier 2 Convergence Wave Response

- **`barracuda.precision.route` wire contract aligned**: Response fields updated to canonical `LIVE_SCIENCE_API.md` — `recommended_tier`, `fma_safe`, `requires_compiler`, `hardware_hint`. Accepts optional `hardware_hint` param.
- **`validate --list` flag**: Lists all 17 scenarios without executing (plasmidBin compatibility)
- **`PRIMAL_PROOF_IPC_MAPPING.md`**: Documents all 17 domain operation → precision route mappings across PK/PD, microbiome, biosignal, toxicology, simulation
- **LTEE B5 lithoSpore module packaging**: `tolerances.toml` + `LITHO_MODULE_README.md` — exact reproduction commands, tolerance envelopes, BLAKE3 provenance chain
- **plasmidBin cell TOML updated**: Compute trio nodes (toadStool, barraCuda, coralReef) added to `healthspring_cell.toml`
- **plasmidBin niche promoted**: `nest` → `full` composition (12 NUCLEUS primals)
- **Gaps #28–31 surfaced**: Cell TOML stale, niche under-specced, blurb/LIVE_SCIENCE_API contract divergence, lithoSpore ingestion pending

## V64e — May 12, 2026

### Tier 2 Wiring (Ecosystem Wave Sync Response)

- **`toadstool.validate`**: `compute_dispatch::validate_workload()` wraps pre-flight workload validation (Tier 2 Live Science API)
- **`toadstool.list_workloads`**: `compute_dispatch::list_workloads()` queries available compute workloads
- **`barracuda.precision.route`**: `BarraCudaClient::precision_route()` queries recommended precision tier for physics domains (e.g. `population_pk`, `eigensolve`)
- **`PrecisionAdvisory`** and **`ValidationReport`** structs for typed Tier 2 responses
- 874 lib + 9 doc + 131 workspace = **1,014 tests pass**

### Doc Reconciliation (V64c–V64d)

- Test count reconciled: 999 → 1,011 → 1,014 across 19+ docs
- Scenario count: 16 → 17 (s_toxicology was uncounted)
- Capability count: 87 → 88 (58 science + 30 infra from `ALL_CAPABILITIES`)
- Cross-validation: 194 → 113 (actual `cross_validate.py` output)
- wateringHole active handoff: V63 → V64 (V63 was already archived)
- Foundation Thread 3 expression re-wired (overwritten by upstream merge)
- Foundation Thread 8 promoted seeded → active (V64 validated)
- data/manifest.toml: phantom `fetch_qs_genes.py` marked unimplemented
- control/README: toxicology exp IDs corrected

## V64 — May 12, 2026

### Added
- **`validate_ltee_b5`** Rust validation binary — LTEE B5 (Leonard et al. 2024 symbiont PK/PD) Tier 1 parity with Python baseline (8/8 checks: logistic colonization, carrying capacity, doubling time, half-max timing, steady-state molecule, monotonic production, Hill knockdown, PK half-life)
- **`--format json`** flag on `validate_pk_models` (16 checks) and `validate_ltee_b5` (8 checks) for projectNUCLEUS Tier 2 structured ingestion
- **LTEE provenance entry** in `records_science.rs` — `ltee_b5` track with Leonard 2024 mBio reference
- **Foundation Thread 3** (Immunology) expression wired in `THREAD_INDEX.toml` → `IMMUNO_DRUG_DISCOVERY.md`; Paper 22 added to `basecamp_papers`; status remains `active`
- **Foundation Thread 5** (LTEE) expression re-wired → `LTEE_EVOLUTIONARY_DYNAMICS.md` (lost during upstream rebase); healthSpring + airSpring added to springs list; B5/E2/E3/E4 entries added to reproduction papers table
- **Foundation Thread 8** (Human Health) expression wired in `THREAD_INDEX.toml` → `SOVEREIGN_HEALTH.md`; status promoted from `mapped` → `active`
- `Status:` header lines added to Thread 3, 5, and 8 expression docs for template parity
- **`s_toxicology` validation scenario** — 9 structural checks: toxicity landscape (tissue count, systemic burden, IPR bounds, clearance regime) + biphasic hormesis (zero/low/high dose, hormetic optimum existence and bounds). Track::Toxicology now has a registered scenario (was dead taxonomy slot)

### Changed
- `primal_names::wire_prefix` constants now have `#[doc]` attributes (removes `-W missing-docs` warnings)
- Provenance test `registry_covers_all_python_scripts` excludes `__init__.py` files (module markers, not science baselines)
- PAPER_REVIEW_QUEUE LTEE B5 status: `STARTED` → `COMPLETE` (Tier 0+1 parity achieved)
- **Dead `npu` feature gate removed** — was declared but had zero `#[cfg(feature = "npu")]` usage anywhere; NPU dispatch will use capability-based runtime discovery per design
- **Experiment primal name centralization**: exp115 + exp118 hardcoded `"biomeos"`, `"beardog"`, `"songbird"`, `"nestgate"`, `"rhizocrypt"`, `"loamspine"`, `"sweetgrass"`, `"toadstool"` replaced with `primal_names::*` constants

### Audit
- 871 lib + 9 doc + 131 workspace = **1,011 tests pass**; zero clippy warnings; zero unsafe; zero TODO/FIXME in production
- `validate_ltee_b5` 8/8 PASS matches Python benchmark to <1e-4 relative tolerance on all numerics
- Foundation Threads 3+5+8 now `active` with expressions wired — 10/10 threads indexed, 8/10 with expressions
- **Deep debt sweep clean**: zero files >800L (max 597), zero unsafe, zero unwrap/expect/panic in production, zero mocks in production, zero hardcoded primal names in lib code
- External deps: all standard Rust ecosystem (serde, clap, tracing, thiserror, wgpu, tokio, ureq) — no stale/unmaintained crates
- Python CPU benchmarks exist: `bench_barracuda_cpu_vs_python.py` + `bench_v16_cpu_vs_python.py` (84x Rust speedup)
- Kokkos-pattern benchmarks in `benches/kokkos_parity.rs` (conceptual parity, no library dependency)

## V63 — May 11, 2026

### Added
- **`primal_names` expansion**: `wire_prefix` sub-module (`HEALTHSPRING`, `BARRACUDA`, `BIOMEOS` — JSON-RPC method normalization prefixes); `BIOMEOS_DIR_NAME` (lowercase filesystem convention); `FALLBACK_SOCKET_DIR` (centralized `/tmp/biomeos`); `SONGBIRD_SOCKET_PATHS` (discovery service socket paths)
- **`DosingRegimen`** struct — groups `dose_mg` + `f_bioavail` for oral PK modeling
- **`PopulationPkVariability`** struct — groups `LognormalParam` priors (CL, Vd, Ka) for population PK IIV models
- **`ToxicityModelParams`** struct — groups Hill coefficient + Km + clearance threshold for toxicity landscape computation
- **`AntibioticSimConfig`** struct — groups all 7 antibiotic perturbation simulation parameters
- **`pop_baricitinib::REGIMEN`** and **`pop_baricitinib::VARIABILITY`** convenience constants
- **Foundation Thread 3** (Immunology/Drug Discovery) seeded in `sporeGarden/foundation` — `expressions/IMMUNO_DRUG_DISCOVERY.md` covering Papers 12, 13, 22 across 5 springs; `THREAD_INDEX.toml` wired (6/10 active threads toward 7+ exit gate)

### Changed
- `normalize_method()` PREFIXES array now uses `primal_names::wire_prefix::*` instead of raw string literals
- `ipc/socket.rs` — all `"biomeos"` path segments replaced with `primal_names::BIOMEOS_DIR_NAME`; `FALLBACK_SOCKET_DIR` moved to `primal_names`
- `visualization/capabilities.rs` — `SONGBIRD_PATHS` replaced with `primal_names::SONGBIRD_SOCKET_PATHS`
- `visualization/ipc_push/client.rs` — `.join("biomeos")` replaced with `primal_names::BIOMEOS_DIR_NAME`
- `data/provenance.rs` — `.join("biomeos")` replaced with `primal_names::BIOMEOS_DIR_NAME`
- `population_pk_cpu` — removed redundant `n_patients` param (derived from slice length); uses `DosingRegimen`
- `population_pk_monte_carlo` — uses `PopulationPkVariability` + `DosingRegimen` (8 params → 5)
- `compute_toxicity_landscape` — uses `ToxicityModelParams` (7 params → 5)
- `antibiotic_perturbation` — uses `AntibioticSimConfig` (7 params → 1 struct)
- All 21 call sites across lib, bins, benchmarks, IPC handlers, viz scenarios, validation scenarios, and 6 experiment crates updated

### Audit
- Deep debt audit: zero files >800 lines, zero unsafe code, zero `unwrap()`/`panic!` in production, zero mocks in production, zero TODO/FIXME/HACK, zero clippy warnings
- All external deps are standard Rust ecosystem (serde, tokio, clap, wgpu, thiserror, tracing, ureq) — no stale or unmaintained crates
- Python baselines intentionally retained as Tier 0 controls (not targeted for removal)

## V62 — May 11, 2026

### Added
- **CI cross-sync**: `health.monitor` + `health.probe` handlers complete 5/5 canonical `[health]` alignment with primalSpring's 413-method registry
- **`skunkBat` audit wiring**: `ipc/audit.rs` with `audit_log()` / `audit_certification()` via `HealthCompositionContext`; `SKUNKBAT` in `primal_names.rs`; `"audit"` routed in composition
- **biomeOS v3.51 absorption**: `composition.status` (primal health + resource pressure) and `method.register` (dynamic method registration) handlers
- **Env-configurable NCBI**: `HEALTHSPRING_NCBI_EUTILS_BASE` and `HEALTHSPRING_NCBI_SRA_BASE` for air-gapped/proxy sovereign deployments
- **`skunkBat` in deploy graphs**: `graphs/healthspring_niche_deploy.toml` (order 8, `defense.*`), `healthspring_biomeos_deploy.toml` (Phase 2b verify), `healthspring_niche.toml` (optional `defense`+`audit`)
- **`healthspring` binary alias**: `[[bin]]` entry in `Cargo.toml` — same `main.rs` as `healthspring_unibin`; NUCLEUS workloads can invoke `healthspring validate` / `healthspring certify`
- **4 NUCLEUS workloads**: `healthspring-pk-validation`, `healthspring-biosignal-validation`, `healthspring-microbiome-validation`, `healthspring-certification` in `projectNUCLEUS/workloads/healthspring/`
- **plasmidBin release binaries**: stripped `healthspring` (2.9M), `healthspring_unibin` (2.9M), `healthspring_primal` (3.1M) staged to `infra/plasmidBin/springs/`; NUCLEUS workload-ready without source tree

### Changed
- Niche `DEPENDENCIES` centralized from string literals to `primal_names::*` constants (single source of truth)
- `BarraCudaClient::discover()` uses `primal_names::BARRACUDA` instead of hardcoded `"barracuda"`
- Last hardcoded primal name strings in `s_live_provenance.rs` replaced with `primal_names::RHIZOCRYPT` / `LOAMSPINE` / `SWEETGRASS` — zero hardcoded primal names remain
- Capabilities surface expanded: 83 → 87 methods (`health.monitor`, `health.probe`, `composition.status`, `method.register`)
- `experiments/README.md` now documents Exp097 (affinity landscape), Exp098 (toxicity landscape), Exp099 (hormesis), Exp111 (causal terrarium)
- Defense routing added to `composition/routing.rs`: `"defense"` / `"defense.audit"` → `SKUNKBAT`; canonical 413 alignment for `security.audit_log` + `defense.audit` in consumed capabilities

## V61 — 2026-05-09 — Interstadial Eukaryotic Evolution

### Architecture & binaries
- **primalSpring v0.9.25 pinned** — workspace dependency with version (replaces optional path-only v0.9.17 baseline).
- **`healthspring_unibin`** — single UniBin entrypoint with `certify`, `validate`, `serve`, `status`, `version` subcommands (primalSpring UniBin pattern).
- **`certification/` organelle** — absorbed legacy `healthspring_guidestone` binary into library module; fossils under `fossilRecord/guidestone_prokaryotic_may2026/`.
- **`validation/scenarios/`** — 16 scenarios across 8 tracks (PkPd, Microbiome, Biosignal, Endocrine, Comparative, Discovery, Composition, Toxicology); absorbed experiment mains archived to `fossilRecord/experiments_prokaryotic_may2026/`.
- **`composition/`** — `HealthCompositionContext` wraps primalSpring `CompositionContext` with health-domain typed accessors.

### IPC & provenance
- **CompositionContext migration** — `PrimalClient`, `InferenceClient`, `discover_primal()`, `discover_by_capability_public()` deprecated with `note` pointing callers at `CompositionContext`.
- **IPC provenance trio** — dedicated modules: rhizocrypt (DAG), loamspine (ledger / Merkle), sweetgrass (braid / analytics).
- **BarraCudaClient** — primal-proof surface expanded: `stats_variance`, `stats_correlation`, `rng_normal`.
- **Default features flipped** — `default = []` (IPC-first); **`barracuda-lib`** opt-in for direct barraCuda library linkage and GPU library paths.

### Quality & testing
- **Parity tests** added (workspace IPC / composition coverage).
- **Lint hygiene** — all bare `#[allow]` replaced with attributed suppressions including **`reason`**; all `#[deprecated]` include **`note`**.
- **Debt markers** — zero `TODO` / `FIXME` / `HACK` / `DEBT` in production sources.
- **Tests** — full workspace test suite passes.
- **Clippy** — zero warnings across **`--workspace --all-targets`** (all targets; features as exercised in CI).

## V60 — 2026-05-08 — Deep Debt Evolution

### Architecture
- **`barracuda-lib` feature**: barraCuda/barracuda-core now optional deps behind `barracuda-lib` (default on). IPC-first sovereign NUCLEUS deployment path when disabled. `math_dispatch.rs` provides pure-Rust fallbacks for all domain functions.
- **Capability-based discovery**: `BarraCudaClient::discover()` uses `stats` capability first, `barracuda` name fallback.
- **Timeout centralization**: All scattered timeout/retry constants (`rpc.rs`, `connection.rs`, `stream.rs`, `signal.rs`, `provenance.rs`) moved to `tolerances.rs`.
- **Tolerance migration**: Inline `1e-15`/`1e-10` literals in exp122 and guidestone bare.rs replaced with named `tolerances::*` constants.

### Experiments
- **exp123_nucleus_parity**: Full NUCLEUS pipeline parity (Tower+Node+Nest+cross-atomic) for health niche, replicating primalSpring exp094.
- **exp119-122 CI coverage**: Added `[[bin]]` entries; all 5 new experiments in CI composition job.
- **validate_pk_models**: New binary for projectNUCLEUS workload (Hill, 1-compartment, PopPK, Michaelis-Menten).

### Benchmarks & Data
- **gpu_parity.rs**: Criterion GPU benchmarks (Hill, Diversity, PopPK, MM) feature-gated behind `gpu`.
- **Dataset fetch scripts**: `fetch_mitbih.sh`, `fetch_chembl.sh`, `fetch_hmp_16s.sh`, `fetch_geo_ar.sh` with BLAKE3 hashing.

### Code Quality
- All clippy errors fixed (`map_unwrap_or`, `doc_markdown`, `format_collect`, `useless_conversion`).
- barraCuda version comments updated from v0.3.12 to v0.3.13 across 17 active docs + CI.
- `records_infra.rs` (777 LOC) split into 4 domain files: `records_discovery.rs`, `records_gpu.rs`, `records_composition.rs`, `records_infra.rs`.
- `visualization/scenarios/tests.rs` (732 LOC) split into `tests_biosignal.rs`, `tests_pkpd.rs`, `tests_endocrine.rs`, `tests_microbiome.rs`.

### Documentation
- 53 Python control scripts converted to `.ipynb` notebooks with paper linkage via `tools/py_to_notebook.py`.
- CM-003/CM-004 paper queue inconsistency resolved in `specs/PAPER_REVIEW_QUEUE.md`.
- `docs/PRIMAL_GAPS.md` updated with May 8 evolution findings and upstream handback items.
- `config/capability_registry.toml` created with sync test against primalSpring canonical registry.

## V59 — 2026-04-27 — Deep Debt Resolution (Idiomatic Rust Evolution)

### Changed
- **`NodeType`, `NodeStatus`, `EdgeType`, `ClinicalStatus` enums**: Replaced
  `String` fields with typed enums across `visualization/types.rs` and all
  scenario/clinical-node call sites. Serde-compatible serialization preserved.
  Eliminates stringly-typed dispatch for closed vocabularies.
- **`timeseries()` x-values by reference**: Helper takes `&[f64]` instead of
  `Vec<f64>` for shared x-axis data, eliminating ~30 `.clone()` calls across
  8 scenario builders and 5 clinical-node builders.
- **`bar()` categories by reference**: Takes `&[String]` instead of
  `Vec<String>`, eliminating ~15 `.clone()` calls.
- **Provenance status capability-based**: `handle_provenance_status()` in
  routing.rs now uses capability domains (dag/ledger/attribution) instead of
  hardcoded primal names (rhizocrypt/loamspine/sweetgrass).
- **`NicheDependency.name` doc**: Clarified as socket-prefix fallback hint,
  not primal identity assertion. Capability domain is the primary discovery key.
- **Clinical `percentile_from_sorted`**: Extracted from `percentile_sorted`
  to avoid double-clone and double-sort for cmax percentile computation.

### Added
- **`ValidationOutcome`**: New return type from `ValidationHarness::finish()`
  — returns pass/fail/total counts without calling `process::exit()`. Library
  code can now validate without terminating the process. `exit()` delegates
  to `finish()` for binary use.

### Fixed
- All clippy errors resolved (0 warnings, 0 errors).
- BLAKE3 CHECKSUMS regenerated for modified source files.

## V58 — 2026-04-27 — Phase 46 Composition Template (Full NUCLEUS)

### Added
- **NUCLEUS composition**: healthSpring deployed and validated against a
  full 8-primal NUCLEUS using primalSpring Phase 46 composition tooling
  (`composition_nucleus.sh`, `nucleus_composition_lib.sh`).
- **`tools/healthspring_composition.sh`**: Interactive composition with
  petalTongue GUI, DAG state tracking, ledger sealing, and braid provenance.
- **`tools/healthspring_composition_headless.sh`**: Headless/CI validation
  runner — 24 automated checks across 8 capability domains.
- **`tools/socat` shim**: `nc -q 1 -U` fallback for systems without socat.
- **Gaps 23–27 documented**: Provenance trio empty UDS responses (Gap 23),
  songbird crypto provider discovery failure (Gap 24), petalTongue
  proprioception unavailable in server mode (Gap 25), nestgate not in
  default PRIMAL_LIST (Gap 26), socat dependency undocumented (Gap 27).

### Validated (18/24 pass, 4 fail, 2 skip)
- **Capability discovery**: 7/8 capabilities found (storage offline, songbird
  failed). beardog, toadstool, barracuda, rhizocrypt, loamspine, sweetgrass,
  petaltongue all have live sockets with capability domain aliases.
- **Liveness probes**: 4/4 — visualization, security, compute, tensor all
  respond to JSON-RPC.
- **barraCuda math IPC**: All 4 methods via composition_nucleus.sh NUCLEUS:
  - `stats.mean` — PASS (diff=0.0)
  - `stats.std_dev` — PASS (diff=0.0)
  - `stats.variance` — PASS (diff=1.78e-15)
  - `stats.correlation` — PASS (diff=0.0)
- **petalTongue scene push**: Accepted in server (headless) mode.
- **bearDog crypto.sign**: Ed25519 signature returned successfully.
- **toadStool compute.capabilities**: 16 cores, 64GB RAM, distributed
  coordinator active.
- **FAIL**: rhizoCrypt, loamSpine, sweetGrass — accept UDS, return empty
  (provenance trio pattern, extends known PG-45).
- **FAIL**: petalTongue proprioception — no frame_rate in server mode.

### Changed
- **Composition tools copied from primalSpring**: `nucleus_composition_lib.sh`,
  `composition_template.sh`, `composition_nucleus.sh` in `tools/`.
- **PRIMAL_GAPS.md**: Updated to V58, added gaps 23–27.

## V57 — 2026-04-20 — guideStone Level 5 (Primal Proof)

### Added
- **guideStone Level 5 — primal proof**: `healthspring_guidestone` passes
  57/57 checks (10 skipped) against live NUCLEUS (barraCuda, beardog,
  nestgate). All four generic math methods validated via IPC:
  - `stats.mean` — PASS (diff=0.00e0)
  - `stats.std_dev` — PASS (diff=0.00e0)
  - `stats.variance` — PASS (diff=1.78e-15, Sprint 44)
  - `stats.correlation` — PASS (diff=0.00e0, Sprint 44)
- **Tier 2 storage round-trip**: `storage.store` + `storage.retrieve` against
  live nestgate — PASS.
- **Gaps 20–22 documented**: BTSP production mode breaks IPC (Gap 20),
  crypto probe schema mismatch (Gap 21), missing capability socket discovery
  for DAG/AI/commit domains (Gap 22).

### Changed
- **`GUIDESTONE_READINESS`** = 5 (primal proof — first spring to reach L5).
- **`primalspring`** upgraded v0.9.16 → v0.9.17.
- **guideStone standard reference** updated v1.1.0 → v1.2.0.
- **`niche::BARRACUDA_IPC_MIGRATION`**: added `stats.variance` and
  `stats.correlation` entries.
- **Tier 2 restored**: `stats.variance` and `stats.correlation` re-added to
  Tier 2 (IPC-Wired) now that barraCuda Sprint 44 exposes them.
- **Tier 3 expanded**: primal proof validates all four math methods + domain
  science locality confirmation.

### Fixed
- **Gap 19 resolved**: `stats.variance` and `stats.correlation` now on
  barraCuda wire (Sprint 44). guideStone validates both end-to-end.
- **BTSP workaround**: `FAMILY_SEED` must be unset for guideStone runs to
  avoid BTSP handshake failures with non-BTSP primals (Gap 20).
- CHECKSUMS regenerated after domain.rs and main.rs updates.

## V56 — 2026-04-19 — guideStone Level 4 (NUCLEUS Validated)

### Added
- **Live NUCLEUS validation**: guideStone passes 49/49 checks (14 skipped)
  against live barraCuda on RTX 3070 with FAMILY_ID=healthspring-validation.
  - `stats.mean` IPC parity: composition=5.5, local=5.5, diff=0.00e0
  - `stats.std_dev` IPC parity: composition=3.027…, local=3.027…, diff=0.00e0
  - Primal proof: mean + std_dev via NUCLEUS confirmed; domain science
    (Hill, Shannon, Simpson, Bray-Curtis) validated locally in Tier 1.
- **BLAKE3 CHECKSUMS manifest**: 17 validation-critical files hashed with b3sum.
  Property 3 now PASSES (was SKIP in V55 without manifest).
- **Gap 19 documented**: barraCuda `stats.variance` and `stats.correlation` not
  on JSON-RPC wire. Removed from Tier 2, handed back to barraCuda team.

### Changed
- **`niche::GUIDESTONE_READINESS`** = 4 (NUCLEUS guideStone works).
- **Tier 3 restructured**: Domain-specific methods (Hill, Shannon, Simpson,
  Bray-Curtis) correctly classified as local compositions. Tier 3 validates
  only wire primitives (mean, std_dev) through IPC, then confirms domain
  science passed locally.
- **Tier 2 trimmed**: `stats.variance` and `stats.correlation` removed (not
  on barraCuda wire). Documented in PRIMAL_GAPS.md §19.

### Fixed
- CHECKSUMS tamper detection: regenerated after domain.rs code changes.
  guideStone caught the stale hash correctly (P3 self-verifying works).

## V55 — 2026-04-20 — guideStone Level 3 (Primal Proof Harness)

### Added
- **Three-tier primal proof harness** per `GUIDESTONE_COMPOSITION_STANDARD`
  v1.1.0 (primalSpring v0.9.16):
  - **Tier 1 (LOCAL):** Bare properties 1–5 + domain science. Always green.
  - **Tier 2 (IPC-WIRED):** barraCuda math IPC + manifest capabilities.
    `check_skip` when primals absent.
  - **Tier 3 (FULL NUCLEUS):** Primal proof — Hill, Shannon, Simpson,
    Bray-Curtis, mean via IPC vs local baseline. Deploy from plasmidBin.
- **Property 3 (Self-Verifying):** BLAKE3 checksums via
  `primalspring::checksums::verify_manifest()`. SKIPs when no manifest
  (honest scaffolding); verifies per-file hashes when manifest exists.
- **Protocol tolerance:** `skip_or_fail` now checks `is_protocol_error()`
  for HTTP-on-UDS (Songbird, petalTongue) — SKIP, not FAIL.
- **Family-aware discovery:** FAMILY_ID env var reported at startup.
  `CompositionContext` resolves `{capability}-{family}.sock` automatically
  via primalSpring v0.9.16 discovery.
- **Upstream evolution handoff:** Full handoff for primalSpring, barraCuda,
  toadStool, metalForge, biomeOS, all springs.

### Changed
- **`niche::GUIDESTONE_READINESS`** = 3 (bare guideStone works).
- **`niche::GUIDESTONE_PROPERTIES`**: P1 ✓, P2 ✓, P3 ✓, P4 ✓, P5 ✓.
- **`primalspring`** upgraded v0.9.15 → v0.9.16.
- **guideStone standard reference** updated v1.0.0 → v1.1.0.

## V54 — 2026-04-18 — guideStone Level 2

### Added
- **`healthspring_guidestone` binary**: Self-validating NUCLEUS node per
  `GUIDESTONE_COMPOSITION_STANDARD` v1.0.0. Validates bare properties 1–5
  (deterministic, traceable, env-agnostic, tolerance-documented) without
  primals. When NUCLEUS deployed: validates IPC parity via
  `primalspring::composition` for `stats.mean`, `stats.std_dev`,
  `stats.variance`, `stats.correlation`, plus 10 manifest capabilities
  (`storage`, `crypto`, `dag`, `inference`, `braid`). Exit 0/1/2.
- **`guidestone` feature**: Enables `primalspring` dep + guidestone binary.
- `primalspring` v0.9.15 as optional path dependency.

### Changed
- **`math_dispatch` reframed as "validation window"** per guideStone standard.
  The 9 domain-specific methods (Hill, Shannon, Simpson, Chao1, Bray-Curtis,
  Anderson, MM-AUC, antibiotic perturbation, SCR rate) are LOCAL compositions
  of barraCuda primitives — not missing wire handlers. Only `stats.mean` and
  `stats.std_dev` are generic IPC candidates.
- **`BARRACUDA_IPC_MIGRATION` doc corrected**: "9 pending wire handlers" → "9
  domain compositions (local)". barraCuda's 32 IPC methods are generic math;
  domain functions belong to the spring.

## V53 — 2026-04-17 — Composition Parity (Live IPC)

### Added
- **Tier 4 live IPC experiments**: Three new composition experiments that exercise the
  full Unix socket JSON-RPC wire path against a live healthSpring primal server:
  - `exp119_composition_live_parity` — science dispatch via IPC vs direct Rust
    (Hill, compartment PK, AUC, Shannon, Anderson). Graceful skip when primal offline.
  - `exp120_composition_live_provenance` — provenance trio round-trip over IPC
    (session lifecycle, Merkle root, commit, braid).
  - `exp121_composition_live_health` — NUCLEUS health probes over IPC
    (liveness, readiness, capability.list, identity.get, niche science dispatch).
- `niche::COMPOSITION_EXPERIMENTS` — centralized registry mapping all 11 composition
  experiments to their validation tier (tier3/tier4/tier5).
- `niche::PROTO_NUCLEATE_VALIDATION_CAPABILITIES` — 10 IPC methods mirrored from
  `healthspring_enclave_proto_nucleate.toml` manifest (Level 5 readiness).
- `niche::BARRACUDA_IPC_MIGRATION` — 12 library→IPC call site mappings for Level 5
  primal proof (barraCuda library deps must become IPC calls).
- **`math_dispatch` module**: Centralizes all 11 non-RNG `barracuda::` call sites.
  Behind `--features primal-proof`, wire-ready methods (`stats.mean`, `stats.std_dev`)
  route through barraCuda ecobin IPC. Falls back to library when offline.
- **`BarraCudaClient`** (`ipc/barracuda_client.rs`): Typed IPC client for barraCuda's
  JSON-RPC surface (`stats.mean`, `stats.std_dev`, `rng.uniform`, `health.liveness`).
- **`primal-proof` feature**: Feature flag for Level 5 IPC routing in `math_dispatch`.
- **`exp122_primal_proof_barracuda_parity`**: Level 5 validation — `math_dispatch`
  known-values, `BarraCudaClient` IPC vs local, wire-pending inventory check.
- PRIMAL_GAPS.md §17 — barraCuda lib→IPC gap documented with migration plan.
  (V54: reframed — 9 methods are domain compositions, not wire gaps.)
- `#![forbid(unsafe_code)]` applied directly to `ecoPrimal/src/lib.rs` crate root.
- Provenance records for exp119–122 in `records_infra.rs` (track: composition).

### Changed
- **`ValidationSink` refactored**: `Box<dyn ValidationSink>` replaced with enum dispatch
  (`ValidationSink::{Tracing, Silent, Collecting}`) — stadial zero-dyn compliance.
- **`ServerError` typed enum**: `cmd_serve` returns `ServerError` instead of `String`.
- **`TrioError` typed enum**: `capability_call`/`resilient_capability_call` in provenance
  return `TrioError` instead of `String`.
- **Capability routing by domain**: `ROUTED_CAPABILITIES` maps to capability domains
  (`compute`, `shader`, `storage`, `inference`) instead of hardcoded primal names.
- CWRES bound in exp075 sourced from `tolerances::CWRES_MEAN` (was hardcoded `3.0`).
- barraCuda version comment updated to v0.3.12 (workspace current).
- ecoBin 0.9.0 harvested to `infra/plasmidBin/healthspring/` (3.2 MB static-PIE x86_64-musl).

### Fixed
- Removed `TracingSink`, `SilentSink` type exports (replaced by `ValidationSink` enum).
- Redundant `CollectingSink` import in harness tests.

## V52 — 2026-04-11 — Composition Validation

### Added
- **Tier 5 deploy graph validation**: `exp118_composition_deploy_graph_validation` (99 checks)
  validates fragment metadata, node presence, bonding policy, capability surface coverage,
  and Squirrel optionality against proto-nucleate expectations.
- `bench` CI job — compiles all benchmarks on every PR (regression gate).
- `barracuda-ops` feature tests run on every PR (GPU code coverage without full GPU hardware).

### Changed
- `PrimalClient.call()` upgraded from `rpc::try_send` to `rpc::resilient_send` (retry + backoff
  by default). New `try_call()` for single-attempt scenarios.
- `handle_primal_forward` in routing.rs migrated from raw `rpc::resilient_send` to typed
  `PrimalClient` (resilient default, structured error reporting).
- `cargo llvm-cov` expanded from `--lib` to full workspace (lib + integration tests),
  `--fail-under-lines 90`.
- `control/tolerances.py` docstring updated: documented as intentional subset of `tolerances.rs`
  (Rust-only constants deliberately omitted — no Python consumer).

### Fixed
- `clippy::match_wildcard_for_single_variants` in `gpu/sovereign.rs` (`_ => "unknown"` →
  explicit `GpuOp::HillSweep { .. } => "HillSweep"`).

## V51 — 2026-04-11 — Hardened Composition Patterns

### Added
- TCP JSON-RPC listener via `--port` flag or `HEALTHSPRING_PORT` env var (newline-delimited
  JSON-RPC 2.0 over TCP, aligned with `PRIMAL_IPC_PROTOCOL.md` v3.1).
- `server` subcommand alias for `serve` (UniBin standard).
- `identity.get` JSON-RPC method returning primal metadata (name, version, domain, license,
  architecture, composition model, particle profile, proto-nucleate reference).
- `health.check` JSON-RPC method for lightweight health probe (status, primal, version,
  domain, uptime).
- `methods: [string]` top-level array in `capabilities.list` response per
  `PRIMAL_CAPABILITY_WIRE_STANDARD_APR08_2026.md`.
- `LOCAL_CAPABILITIES` and `ROUTED_CAPABILITIES` constants with `served_locally` and
  `canonical_provider` metadata in capability registration payloads.
- `provided_capabilities()` structured output in `capabilities.list` (local vs routed).
- Domain symlink (`health.sock`) created on bind, cleaned on shutdown (capability-domain
  discovery per `PRIMAL_IPC_PROTOCOL.md` v3.1).
- `ipc/btsp.rs` — BTSP (BearDog Transport Security Protocol) client handshake module:
  `BtspMessage` enum, `family_seed_from_env()`, `client_hello()`, pure-Rust base64 decoder.
- `ipc/client.rs` — Typed `PrimalClient` (health/capabilities fallback chains, typed calls)
  and `InferenceClient` (discover, complete, embed, models) wrappers.
- `ipc/discover.rs` — Structured `DiscoveryResult` and `DiscoverySource` for traceable
  primal discovery (env override, capability probe, well-known path, not found).
- `status` field (`"healthy"` / `"degraded"`) in `health.readiness` response.
- V51 handoff: `wateringHole/handoffs/HEALTHSPRING_V51_HARDENED_COMPOSITION_HANDOFF_APR11_2026.md`.

### Changed
- `CoralReefDevice` → `SovereignDevice` in `gpu/sovereign.rs` (upstream API rename).
- `handle_connection` refactored to generic `handle_lines<R,W>` supporting both Unix and TCP.
- `cmd_serve` accepts `tcp_port: Option<u16>` and spawns TCP listener thread when provided.
- `register_with_biomeos` iterates `LOCAL_CAPABILITIES` and `ROUTED_CAPABILITIES` separately
  with `served_locally`/`canonical_provider` metadata per primalSpring niche pattern.
- `plasmidBin/manifest.lock` healthspring version updated 0.7.0 → 0.8.0 (resolves drift).

### Fixed
- `CoralReefDevice` compile error in `gpu/sovereign.rs` (5 occurrences).
- Broken intra-doc link in `provenance/mod.rs`.
- `clippy::needless_pass_by_value` in `accept_tcp`, `handle_unix_connection`,
  `handle_tcp_connection` (justified `#[expect]` with reasons).
- `clippy::map_unwrap_or` in TCP port logging.

## V50 — 2026-04-11 — Composition Evolution

### Added
- Optional Squirrel node in `healthspring_niche_deploy.toml` (`required = false`) for
  `inference.*` capabilities when available.
- Dual-method discovery fallback in `tower_atomic.rs`: tries `discovery.find_by_capability`
  first, falls back to legacy `net.discovery.find_by_capability`.
- Provenance registry split: `registry.rs` (80 LOC logic) + `records_science.rs` (460 LOC,
  Tracks 1–5) + `records_infra.rs` (720 LOC, Tracks 6–10+). All under 1000 LOC.
- V50 handoff: `wateringHole/handoffs/HEALTHSPRING_V50_COMPOSITION_EVOLUTION_HANDOFF_APR11_2026.md`.
- Cross-team primal evolution handoff for barraCuda, toadStool, primalSpring, biomeOS.

### Changed
- `primal.forward` routing: capability-based discovery first, name-based fallback.
- exp112/exp113 refactored: `main()` extracted into domain-coherent helper functions.
- exp114: `if_same_then_else` and `unnecessary_map_or` clippy errors fixed.
- exp116: `overly_complex_bool_expr` tautologies fixed.
- Inline tolerance values in toadstool/metalForge tests migrated to `tolerances::*` constants.
- Doc backticks added for `BearDog`, `PopPK` references in module-level docs.
- `PRIMAL_GAPS.md` updated to V50: §3 (dual discovery fallback), §9 (Squirrel optional node).

### Fixed
- 4 clippy errors in composition experiments (exp112, exp114, exp116).
- 7 clippy warnings in library code (doc formatting, hex literals, paragraph breaks).
- 579-line `cargo fmt` drift resolved.
- `provenance/registry.rs` split from 1224 LOC to 3 files (all under 1000 LOC standard).

## V49 — 2026-04-10 — Composition Audit Remediation

### Added
- `health.genomics` capability + proto-nucleate alias → `science.microbiome.qs_gene_profile`.
  All five `health.*` proto-nucleate aliases are now wired.
- `exp117_composition_ipc_roundtrip` — Tier 4 IPC wire protocol validation (71 checks):
  round-trip serialization, proto-nucleate alias dispatch, health probe routing, full
  capability surface completeness.
- Deploy graph fragment metadata: `fragments`, `particle_profile`, `proto_nucleate`,
  `[graph.bonding]` with bond type, trust model, encryption tiers per atomic boundary.
- Bonding policy matrix documented in `tower_atomic.rs` module docs.
- WGSL shader removal plan documented in `gpu/mod.rs` (all 6 absorbed upstream).
- 36 new provenance registry entries for non-Python experiments → 89 total (100% coverage).
- `PRIMAL_GAPS.md` gaps §8 (deploy fragment metadata — fixed) and §9 (Squirrel in deploy).
- V49 handoff: `wateringHole/handoffs/HEALTHSPRING_V49_COMPOSITION_AUDIT_HANDOFF_APR10_2026.md`.
- V49 classification table for 21 tolerance-exempt experiments in `TOLERANCE_REGISTRY.md`.

### Changed
- barraCuda pin updated from v0.3.7 (`c04d848`) to v0.3.11 (`7f6649f`).
- `uncertainty::std_dev` delegated to `barracuda::stats::correlation::std_dev`.
- Sovereign dispatch error messages now name each unsupported op + reference EVOLUTION_MAP.
- `provenance::tests::registry_complete` split into `registry_covers_all_python_scripts`
  and `registry_covers_all_experiments` (decoupled from 1:1 Python file assumption).
- All spec docs (`BARRACUDA_REQUIREMENTS`, `EVOLUTION_MAP`, `TOLERANCE_REGISTRY`) updated to V49.

### Fixed
- `cross_validate.py`: `TOL_AUC` aliased to `LEVEL_SPACING_RATIO` (0.02) instead of
  `AUC_TRAPEZOIDAL` (0.01). Fixed to use correct registry constant.
- CI: added `pip install -r control/requirements.txt` before Python cross-validation.
- `PRIMAL_GAPS.md` gap §1 resolved — `health.*` science aliases implemented (option a).

## V48 — 2026-04-10

### Added
- `docs/PRIMAL_GAPS.md` — primal composition gap registry per NUCLEUS alignment protocol.
- `CHANGELOG.md` — presentation standard compliance.
- `inference.*` capability aliases alongside `model.*` for Squirrel/neuralSpring alignment.
- `health.*` proto-nucleate aliases (`health.pharmacology`, `health.clinical`,
  `health.de_identify`, `health.aggregate`) registered in `ALL_CAPABILITIES`.
- `resilient_send` in `ipc/rpc.rs` — retry with exponential backoff for retriable IPC errors.
- Infrastructure capabilities in YAML niche manifest (`niches/healthspring-health.yaml`).
- Cross-compile CI targets (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`).
- GPU CI job enabled as weekly scheduled run.
- **Tier 4 composition validation** — 5 new experiments (exp112–exp116, 73 checks total).
- ecoBin static-PIE binary (x86_64-musl, 2.5 MB) harvested to `infra/plasmidBin/healthspring/`.
- CI cross-compile job with artifact upload and static linkage verification.

### Changed
- `health.readiness` gates on science dispatch status.
- `build_semantic_mappings()` refactored to programmatic map construction.

### Fixed
- YAML niche manifest now includes all capabilities the binary serves.

## V47 — 2026-04-07

### Added
- `HEALTHSPRING_V47_UPSTREAM_ABSORPTION_HANDOFF_APR07_2026.md` — upstream
  absorption, health triad, deploy graph references.
- `HEALTHSPRING_V46_COMPOSITION_CONVERGENCE_HANDOFF_APR07_2026.md`.
- `HEALTHSPRING_V45_CAPABILITY_SYNC_IPC_FUZZ_HANDOFF_APR07_2026.md`.

## V44 — 2026-03-24

### Added
- 83 validation experiments with `ValidationHarness` (hotSpring pattern).
- JSON-RPC primal server (`healthspring_primal`) with 75+ capabilities.
- 6 WGSL shaders for GPU dispatch (Hill, PopPK, Diversity, MM, SCFA, BeatClassify).
- barraCuda integration: 6 GPU ops + CPU primitives (stats, rng, health modules).
- Provenance trio session API (begin/record/complete/status).
- Tower Atomic integration (BearDog + Songbird discovery).
- petalTongue visualization IPC push with clinical scenario nodes.
- `metalForge/forge` — NUCLEUS dispatch and composition.
- `toadstool` — compute pipeline staging.
- Proptest IPC fuzzing (protocol, transport, dispatch).
- `cargo-deny` configuration banning C/native dependencies.
- 90%+ library line coverage enforced in CI.
- SPDX `AGPL-3.0-or-later` headers on all source files.
