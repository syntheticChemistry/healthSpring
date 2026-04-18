# healthSpring — Health of Living Systems via Sovereign Scientific Computing

**An ecoPrimals Spring** — species-agnostic health applications validating PK/PD, microbiome, biosignal, endocrine, comparative medicine, and drug discovery pipelines against Python baselines via Pure Rust + barraCuda GPU. Follows the **Write → Absorb → Lean** cycle adopted from wetSpring/hotSpring.

**Date:** April 17, 2026 (V53)
**License:** scyBorg (AGPL-3.0-or-later code + ORC mechanics + CC-BY-SA 4.0 creative content)
**MSRV:** 1.87
**Status:** V53 — Composition Parity (Live IPC). 936+ tests, 93 experiments (84 science + 10 composition Tier 3–4), 54 Python baselines, 93 provenance entries (100% coverage). barraCuda v0.3.12 (workspace current). Zero `dyn` dispatch (enum `ValidationSink`), typed errors (`ServerError`, `TrioError`), capability routing by domain (not primal identity). Three new live IPC experiments (exp119–121): science parity, provenance trio, health probes — all over Unix socket JSON-RPC. `niche.rs` composition experiment registry. ecoBin 0.9.0 (3.2 MB static-PIE x86_64-musl) at `infra/plasmidBin/`. Zero clippy warnings (pedantic+nursery). Four-layer validation: Python validates science, Rust validates Python, in-process dispatch validates composition, **live IPC validates NUCLEUS wire path**.

---

## What This Is

healthSpring is the sixth ecoPrimals spring. Where the other five springs validate published science — reproducing papers to prove the pipeline — healthSpring builds **usable applications** of that validated science for the health of living systems.

The other springs do the chemistry. healthSpring makes the drug.

**New in V22**: healthSpring becomes a **biomeOS niche** — a composed set of primals and workflow graphs orchestrated by the Neural API. The `healthspring_primal` binary exposes all science capabilities via JSON-RPC 2.0 over Unix sockets. biomeOS graphs compose these capabilities into diagnostic pipelines (patient assessment, TRT scenario, microbiome analysis, biosignal monitoring). The primal provides the science; the graphs define the workflows; biomeOS orchestrates the composition.

See [wateringHole/SPRING_NICHE_SETUP_GUIDE.md](wateringHole/SPRING_NICHE_SETUP_GUIDE.md) for how this pattern applies to all springs.

| Spring | Role | healthSpring relationship |
|--------|------|--------------------------|
| **wetSpring** | Life science validation (16S, LC-MS, immunology) | Gut microbiome analytics, Anderson colonization resistance, joint low-affinity binding experiment, shared hormesis framework (`bio::hormesis` ↔ `toxicology`) |
| **neuralSpring** | ML primitives, PK/PD surrogates | Hill dose-response, population PK, clinical prediction |
| **hotSpring** | Plasma physics, lattice methods | Lattice tissue modeling, Anderson spectral theory |
| **airSpring** | Agricultural IoT, evapotranspiration | Environmental chemical exposure, hygiene hypothesis, hormesis transition |
| **groundSpring** | Uncertainty, spectral theory | Error propagation, pesticide hormesis, ecological dose-response |

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Version | **V53** (Composition Parity — Live IPC) |
| **Total tests** | **936+** (852 lib + proptest + IPC fuzz + 33 forge + 51 toadstool + 93 experiment bins) |
| Experiments complete | 93 (84 science Tracks 1–9 + 10 composition Tier 3–4) |
| Composition validation (Tier 3–4) | 10 experiments (exp112–121) — in-process dispatch parity, proto-nucleate, wire round-trip, deploy graph, **live IPC science parity**, **live provenance trio**, **live health probes** |
| JSON-RPC capabilities | 84+ (62 science + 22 infrastructure — `capability.list`, `health.*`, `identity.get`, `inference.*`, provenance, compute/data routing) |
| Paper queue | **30/30 complete** (Tracks 1–5), 10 complete (Tracks 6–7), 5 queued |
| Python baselines | **54** with structured provenance registry (90 total entries, 100% experiment coverage) |
| Cross-validation | **113/113** checks (all tracks, `cross_validate.py`) |
| ecoBin | Static-PIE x86_64-musl, 3.2 MB, harvested to `infra/plasmidBin/healthspring/` (v0.9.0) |
| GPU validation (Tier 2) | **Live** — 6 WGSL shaders, fused pipeline, 42/42 parity |
| CPU parity | Rust 84× faster than Python across V16 primitives |
| biomeOS niche | **Live** — `UniBin`-compliant primal binary (`serve`/`server`/`version`/`capabilities`), `--port` TCP, domain symlink, SIGTERM/SIGINT |
| NLME population PK | FOCE + SAEM estimation, NCA metrics, CWRES/VPC/GOF diagnostics |
| Faculty | Gonzales (MSU Pharm/Tox), Lisabeth (ADDRC), Neubig (Drug Discovery), Ellsworth (Med Chem), Mok (Allure Medical) |
| Unsafe blocks | **0** (`forbid(unsafe_code)` in `[workspace.lints]`) |
| TODO/FIXME in production | **0** |
| Clippy | **0 warnings** (workspace `deny(clippy::{all,pedantic,nursery,unwrap_used,expect_used})`) |
| License | **AGPL-3.0-or-later** (scyBorg trio compliant across all .rs, .py, .sh, .toml, .md) |

---

## V53 Composition Parity — Live IPC (from V52)

V53 completes the composition evolution spiral: Python baselines validated Rust science; Rust validated Python; in-process dispatch validated the composition layer; now **live IPC experiments validate the NUCLEUS wire path** — Unix socket JSON-RPC to a running primal server, comparing results against direct Rust calls.

| Change | Impact |
|--------|--------|
| **Live IPC parity (exp119)** | Science methods called via Unix socket JSON-RPC produce identical results to direct Rust calls. Graceful skip when primal offline. |
| **Live provenance trio (exp120)** | Provenance session lifecycle (create → record → complete → Merkle root) validated over IPC. |
| **Live health probes (exp121)** | `health.liveness`, `health.readiness`, `capability.list`, `identity.get` exercised over real wire path. |
| **Zero `dyn` dispatch** | `Box<dyn ValidationSink>` replaced with `ValidationSink` enum dispatch — stadial zero-dyn compliance. |
| **Typed errors** | `ServerError` and `TrioError` enums replace `Result<_, String>` in server and provenance IPC paths. |
| **Capability routing by domain** | `ROUTED_CAPABILITIES` maps to `by_capability` domains, not hardcoded primal names. |
| **`niche.rs` composition registry** | `COMPOSITION_EXPERIMENTS` constant maps all 10 composition experiments to validation tiers. |
| **ecoBin 0.9.0** | 3.2 MB static-PIE x86_64-musl, harvested to `infra/plasmidBin/`. barraCuda v0.3.12. |
| **936+ tests** | 93 experiments (84 science + 10 composition). Zero clippy, zero `dyn`, zero `async-trait`. |

---

## V44 Deep Debt Resolution & Modern Idiomatic Evolution (from V43)

V44 executes on a comprehensive audit: centralizing primal names, evolving hardcoded values to capability-based patterns, smart-refactoring large modules, expanding test coverage, and migrating inline tolerances across experiment binaries.

| Change | Impact |
|--------|--------|
| **`primal_names` module** | Centralized all primal string literals (`BEARDOG`, `SONGBIRD`, `PETALTONGUE`, `BIOMEOS`) with `socket_env_var()` / `prefix_env_var()` helpers. Zero hardcoded primal names in production. |
| **`PopulationPkConfig` provenance** | `10.0` / `0.5` / `1.5` defaults → `tolerances::POP_PK_BASE_CL` / `POP_PK_CL_LOW` / `POP_PK_CL_HIGH` with documented Python source. |
| **`gpu/mod.rs` smart refactor** | 696→413 lines. Extracted `gpu/types.rs` (105 LOC: `GpuOp`, `GpuResult`, `GpuError`) and `gpu/cpu_fallback.rs` (175 LOC: all CPU reference implementations). |
| **Provenance registry enhanced** | `tracks` module with const identifiers, `records_for_track()`, `record_for_experiment()`, `distinct_tracks()` accessors. Data stays consolidated (smart refactor for data tables). |
| **toadStool coverage: 51 tests** | 31 new tests covering Michaelis-Menten batch, SCFA batch, beat classification, biosignal fusion, AUC, Bray-Curtis, variance, GPU mappability. |
| **WFDB annotations coverage** | 11 new tests for `parse_annotations()` and `BeatType` decoding (empty, terminators, multi-beat, AUX/SKIP, truncation). |
| **Tolerance migration (8 experiments)** | exp022/052/062/067/077/078/093/111: inline literals → `tolerances::*` constants. |
| **FFT absorption documented** | `barraCuda` FFT is GPU-only; local CPU FFT justified for non-GPU biosignal pipelines. |
| **`TensorSession` evaluated** | Complementary pattern (dependent chains vs independent parallel ops). Integration deferred until dependent multi-op pipelines needed. |
| **`wang_hash_uniform` scoped** | Visibility tightened to private in `cpu_fallback` — internal WGSL parity helper, not a public API. |
| **928 tests** | Up from 888 (V43). Zero failures, zero clippy warnings (pedantic+nursery), zero fmt diffs, zero doc warnings. |

---

## V40 Cross-Ecosystem Absorption Sprint (from V39)

V40 reviews all 7 springs, 10+ primals, and 49 handoffs for absorption opportunities. Bumps barraCuda to v0.3.7. Resolves toxicology module conflict. Extracts 17 magic numbers to named constants. Smart-refactors provenance.rs (850→201+164 lines). Adds 39 new tests covering IPC dispatch handlers, MCP, WFDB, NLME solver.

| Change | Impact |
|--------|--------|
| **barraCuda v0.3.7** | Unlocks `KimuraGpu`, `JackknifeGpu`, `DeviceClass`, hydrology Monte Carlo |
| **Module conflict resolved** | Deleted stale `toxicology.rs` (1060 lines), module directory is canonical |
| **17 magic numbers → constants** | FOCE/SAEM tuning, tissue cap, ecosystem defaults, VPC defaults |
| **provenance/ refactor** | 850→201+164 lines (types vs data), public API unchanged |
| **39 new tests** | Handler dispatch (22), MCP tools (5), WFDB parser (9), NLME solver (4) |
| **Zero `#[allow]`** | Last `allow(unused_variables)` → `#[expect(reason)]` |
| **855 tests** | Up from 848. Zero failures. Workspace-level `[lints]` consolidation. |

---

## V39 Toxicology, Simulation, Hormesis + Cross-Spring Integration (from V38)

V39 builds three new science domains: toxicology (Anderson delocalization of toxic burden, hormesis, mithridatism), simulation (multi-scale causal chain from molecular binding to ecosystem dynamics), and low-affinity binding landscape (computation as preprocessor for experiment design).

| Change | Impact |
|--------|--------|
| **3 new Rust modules** | `toxicology/` (32 tests), `simulation.rs` (18 tests), `discovery/affinity_landscape.rs` (15 tests) |
| **4 new experiments** | exp097 (affinity landscape), exp098 (toxicity landscape), exp099 (hormesis), exp111 (causal terrarium) |
| **5 new IPC capabilities** | `science.toxicology.biphasic_dose_response`, `science.toxicology.toxicity_landscape`, `science.toxicology.hormetic_optimum`, `science.simulation.mechanistic_fitness`, `science.simulation.ecosystem_simulate` |
| **4 new Python baselines** | `control/discovery/exp097_*`, `control/toxicology/exp098_*`, `control/toxicology/exp099_*`, `control/simulation/exp111_*` |
| **Cross-spring hormesis** | Biphasic dose-response shared with wetSpring V130 `bio::hormesis`, groundSpring (pesticide), airSpring (environmental) |
| **Computation as preprocessor** | 26× selectivity from weak binding → testable prediction for Gonzales/Lisabeth wet lab |
| **809 tests** | Up from 719 (V38). Zero failures, zero clippy warnings (pedantic + nursery). |

---

## V38 Deep Debt Completion + Standardized Validation (from V37)

V38 completes the deep debt audit execution — every experiment uses standardized validation, every tolerance is named, every provenance record has citations.

| Change | Impact |
|--------|--------|
| **All 79 experiments → `ValidationHarness`** | 23 remaining legacy experiments migrated from manual `passed`/`failed` counters. Zero ad-hoc validation patterns remain. |
| **~120+ inline tolerances → named constants** | Magic numbers in `#[cfg(test)]` modules replaced with `tolerances::*`. New: `TEST_ASSERTION_2_PERCENT`. 30+ files updated. |
| **`clippy::nursery` across all experiments** | All 79 experiment crates now enforce `clippy::nursery` in addition to `pedantic`. Zero warnings. |
| **Typed IPC dispatch in routing** | `server/routing.rs` evolved from raw `rpc::send()` to typed dispatch clients (`compute_dispatch`, `shader_dispatch`, `inference_dispatch`, `data_dispatch`). |
| **Provenance with DOI citations** | `ProvenanceRecord` gains `baseline_source` field. All entries enhanced with literature citations (Hill 1910, Shannon 1948, Pan & Tompkins 1985, Gonzales 2013-2016, etc.). `git_commit`/`run_date`/`exact_command` populated for all Python-controlled experiments. |
| **719 tests** | Up from 706 (V37). Zero failures, zero clippy warnings (pedantic + nursery). |

---

## V33 Protocol Evolution + Centralized Cast Algebra (from V32)

V33 evolves the IPC protocol layer and centralizes numeric cast patterns across the codebase.

| Change | Impact |
|--------|--------|
| **`IpcError::is_recoverable()`** | Classifies transient (Connect, Timeout, Write, Read) vs permanent (InvalidJson, NoResult, RpcError) failures. Enables smarter retry decisions across all IPC consumers. Absorbed from neuralSpring S161. |
| **`IpcError::is_protocol_error()`** | Identifies JSON-RPC protocol-level errors (-32700 to -32600) vs application errors. |
| **`DispatchOutcome` enum** | New `ipc::protocol` module separating RPC responses into `Ok`, `ProtocolError`, `ApplicationError`. Absorbed from groundSpring V112 / biomeOS v2.46. Includes `classify_response()`, `parse_rpc_response()`, `is_method_not_found()`. 6 new tests. |
| **Generic discovery helpers** | `protocol::socket_from_env()` and `protocol::discover_primal_socket()` replace per-primal env-var boilerplate. `discover_compute_primal()` and `discover_data_primal()` now accept direct socket paths via `HEALTHSPRING_COMPUTE_SOCKET`/`HEALTHSPRING_DATA_SOCKET`. Songbird discovery also evolved to use `socket_from_env()`. |
| **`cast` module** | Centralized safe numeric cast helpers: `usize_f64`, `u64_f64`, `f64_usize`, `usize_u32`. Absorbed from groundSpring V112. Each documents its precision guarantee. 5 new tests. |
| **FFT cast consolidation** | `biosignal::fft::idx_to_f64`/`u64_to_f64` replaced with re-exports from `cast` module — single source of truth for all numeric casts. |
| **635 tests** | Up from 618 (V32). 17 new tests across `ipc::rpc`, `ipc::protocol`, `cast`. Zero failures, zero clippy warnings. |

---

## V32 Cross-Spring Absorption + Ecosystem Convergence (from V31)

V32 absorbs proven patterns from the full ecoPrimals ecosystem: structured logging, health probes, and resilient IPC.

| Change | Impact |
|--------|--------|
| **Structured `tracing`** | All `eprintln!` in `healthspring_primal` binary evolved to `tracing::info!/warn!/error!` with structured key-value fields. `tracing-subscriber` with `EnvFilter` (default `healthspring=info`, configurable via `RUST_LOG`). Aligns with all 6 sibling springs (wetSpring V124, airSpring v0.8.6, groundSpring V110, neuralSpring S159, hotSpring v0.6.32, ludoSpring V22). |
| **`health.liveness` probe** | New lightweight JSON-RPC method returning `{"alive": true}` unconditionally — confirms process responsiveness. Aligned with coralReef Iter 51 health probe standard. |
| **`health.readiness` probe** | New JSON-RPC method reporting subsystem availability (`science_dispatch`, `provenance_trio`, `compute_provider`, `data_provider`). Confirms the primal can accept science workloads. |
| **Resilient provenance trio IPC** | `capability_call` wrapped with circuit breaker (5s cooldown on failure) + exponential backoff retry (50ms base, 2 retries). Absorbed from sweetGrass v0.7.18 resilience patterns. Pure Rust, zero new dependencies beyond `tracing`. 2 new tests. |
| **Dispatch refactor** | `dispatch_request` evolved to pattern-match health probes first, then delegate to `dispatch_extended` for science and infrastructure methods. Unknown methods logged via `tracing::warn!`. |
| **57+ JSON-RPC capabilities** | Up from 55+. 2 new: `health.liveness`, `health.readiness`. All registered in `ALL_CAPABILITIES`. |
| **618 tests** | Up from 616 (V31). 2 new tests: circuit breaker open/close, epoch_ms sanity. Zero failures, zero clippy warnings. |
| **Dependencies** | Added `tracing = "0.1"` + `tracing-subscriber = "0.3"` (env-filter). Both pure Rust, ecoBin compliant. |

---

## V31 Deep Debt Solutions + Modern Idiomatic Rust Evolution (from V30)

V31 executes deep debt solutions and evolves to modern idiomatic Rust patterns across the entire codebase.

| Change | Impact |
|--------|--------|
| **`OrExit<T>` trait** | Absorbed from wetSpring V123. Centralized zero-panic pattern for `Result<T,E>` and `Option<T>` — replaces per-site `let-else` with `val.or_exit("context")`. Used in dump binaries. 2 new tests. |
| **`IpcError` ecosystem type** | `SendError` evolved to `IpcError` with `RpcError{code,message}` and `Timeout` variants, aligning with biomeOS/airSpring/groundSpring. Backward-compatible `SendError` alias preserved. 2 new tests. |
| **Enriched `capability.list`** | Response now includes `operation_dependencies` (DAG of science ops) and `cost_estimates` (CPU ms + GPU eligibility) for Pathway Learner execution graph planning. |
| **Magic numbers eliminated** | `0.693` → `std::f64::consts::LN_2` in Michaelis-Menten half-life. `assert_eq!` on floats → `abs() < EPSILON`. `suboptimal_flops` → `.mul_add()`. |
| **`#![forbid(unsafe_code)]` complete** | Added to 3 remaining dump binaries (`exp050`, `exp056`, `exp063`). All 73 binary crate roots now forbid unsafe. |
| **Capability-based data provider** | `neural-api` hardcoded socket prefix evolved to `DATA_PROVIDER_SOCK_PREFIX` env override with `DEFAULT_DATA_PROVIDER_PREFIX` fallback. Any primal exposing `dag.*` capabilities works. |
| **Non-async Tier A GPU ops** | `execute_hill_barracuda`, `execute_pop_pk_barracuda`, `execute_diversity_barracuda` stripped of unused `async` — barraCuda ops are synchronous. Eliminates false async overhead. |
| **barraCuda API alignment** | `PopulationPkF64::simulate()` signature updated for upstream `u32` parameters (was `usize`/`u64`). |
| **616 tests** | Up from 611 (V30). 5 new tests: 2 OrExit, 2 IpcError, 1 doc. Zero failures, zero clippy warnings. |

---

## V30 Cross-Spring Absorption + Zero-Panic Evolution (from V29)

V30 absorbs proven patterns from all 6 sibling springs and evolves validation binaries to zero-panic production quality.

| Change | Impact |
|--------|--------|
| **Dual-format capability parsing** | `probe_capability()` now handles healthSpring (`result.science`), neuralSpring/ludoSpring (`result.capabilities`), nested object, and raw array formats. Cross-primal discovery works with any response shape. 4 new tests. (Absorbed from neuralSpring S157 / ludoSpring V22.) |
| **Zero-panic validation binaries** | ~100 `.expect()`, `.unwrap()`, and `panic!()` sites evolved to graceful `let Ok(...) else { eprintln!(); exit(1); }` pattern across ~25 experiment binaries. Validation failures now produce structured error messages instead of stack traces. (Absorbed from groundSpring V109.) |
| **`compute_dispatch` IPC client** | Typed wrappers for toadStool `compute.dispatch.submit` / `result` / `capabilities` protocol. Capability-based compute primal discovery. 3 new tests. (Absorbed from ludoSpring V22 / toadStool S156.) |
| **`barracuda::health::*` delegation** | `mm_auc()` → `barracuda::health::pkpd::mm_auc`, `scr_rate()` → `barracuda::health::biosignal::scr_rate`, `antibiotic_perturbation_abundances()` → `barracuda::health::microbiome::antibiotic_perturbation`. 1 new delegation test. Write → Absorb → **Lean**. |
| **`deny.toml`** | `wildcards = "deny"`, license allowlist, advisory/source controls. (Absorbed from airSpring v0.8.4.) |
| **Python dependency provenance** | `control/requirements.txt` documented with PRNG drift warning and exact pinning rationale. (Absorbed from groundSpring V109.) |
| **611 tests** | Up from 603 (V29). 8 new tests: 4 capability format, 3 compute dispatch, 1 barraCuda delegation. Zero failures, zero clippy warnings. |

---

## V29 Deep Debt Solutions + Modern Idiomatic Rust (from V28)

V29 executes all remediation from the V28 comprehensive audit — eliminating duplicate math, centralizing all inline constants, evolving hardcoded patterns to capability-based, and wiring barraCuda delegation into the persistent GPU context.

| Change | Impact |
|--------|--------|
| **Experiment refactoring** | `exp090`, `exp092`, `exp093`, `exp100` refactored from monolithic `main()` to domain-coherent helpers (`validate_pathway_selectivity`, `validate_batch_ranking`, `validate_hill_properties`, `validate_il31_kinetics`, etc.). Zero `clippy::too_many_lines` violations workspace-wide. |
| **Tolerance centralization** | 3 inline `1e-15` in `validation.rs` → `MACHINE_EPSILON_STRICT`. `1e-30` in `uncertainty.rs` → `DECOMPOSITION_GUARD`. `BOX_MULLER_CLAMP` moved from `rng.rs` to `tolerances.rs`. IPC constants (`IPC_PROBE_BUF`, `IPC_RESPONSE_BUF`, `IPC_TIMEOUT_MS`) added. |
| **IPC error extraction** | New `extract_rpc_error()` in `ipc/rpc.rs` replaces 4 scattered `unwrap_or(-1)` / `unwrap_or("unknown")` patterns across `data/rpc.rs`, `data/provenance.rs`, `visualization/capabilities.rs`, `visualization/ipc_push/client.rs`. |
| **barraCuda `mean()` delegation** | `uncertainty.rs` local `mean()` → `barracuda::stats::mean`. Zero duplicate math between healthSpring and upstream. |
| **GPU context rewire** | `GpuContext::execute()` now delegates Tier A ops (Hill, PopPK, Diversity) to `barracuda_rewire` when `barracuda-ops` feature is enabled. Previously only `dispatch::execute_gpu()` had this delegation. |
| **Python tolerance mirror** | `control/tolerances.py` created with all 70+ named constants mirroring `tolerances.rs` — ensures Python baselines use identical thresholds. |
| **Hardcoding evolution** | Health response `nestgate` / `toadstool` → `data_provider` / `compute_provider` (capability-based, no primal self-knowledge of others). Songbird well-known paths documented as intentional bootstrap exception. |
| **NLME Cholesky documented** | Local `cholesky_solve()` in NLME solver documented as intentional optimization (2×2/3×3 matrices with fallback, not a candidate for barraCuda delegation). |
| **Tolerance registry updated** | `specs/TOLERANCE_REGISTRY.md` expanded with guard constants (`DECOMPOSITION_GUARD`, `BOX_MULLER_CLAMP`) and IPC constants (`IPC_PROBE_BUF`, `IPC_RESPONSE_BUF`, `IPC_TIMEOUT_MS`). |

---

## V28 Deep Debt + Ecosystem Maturity (from V27)

V28 evolves debt solutions to production quality: capability-based discovery, Result-based IPC, smart module refactoring, and full provenance coverage across all 7 tracks.

| Change | Impact |
|--------|--------|
| **IPC evolution** | `rpc::try_send()` returns `Result<Value, SendError>` with structured error variants (Connect, Write, Read, InvalidJson, NoResult). `rpc::send()` preserved as fire-and-forget convenience. Server registration/heartbeat upgraded with `eprintln!` observability. |
| **Socket discovery** | Removed hardcoded primal name fallbacks (`COMPUTE_FALLBACK_NAMES`, `DATA_FALLBACK_NAMES`). `discover_compute_primal()` and `discover_data_primal()` now pure capability-based. `data/discovery.rs` evolved: `discover_nestgate_socket()` → `discover_data_provider_socket()` (name-agnostic). |
| **microbiome smart refactor** | `microbiome/mod.rs` 680 → 480 LOC. FMT, SCFA, antibiotic perturbation, gut-brain serotonin extracted to `microbiome/clinical.rs` (203 LOC). Tests stay in mod.rs via `pub use clinical::*` re-export. |
| **WGSL shader provenance** | All 6 shaders documented: Wang hash (Thomas Wang 2007), phenytoin C0 (Winter 5th ed), CL variation (Rowland & Tozer), f32 transcendental path rationale, correlation guard derivation. |
| **Tolerance centralization** | exp020: 5 inline thresholds → `tolerances::QRS_PEAK_MATCH_MS`, `QRS_SENSITIVITY`, `HR_DETECTION_BPM`, `SDNN_UPPER_MS`. Added ANSI/AAMI EC57:2012 and ESC/NASPE provenance comments. |
| **Track 6-7 baselines** | 12 baseline JSON files generated and registered in `update_provenance.py`. 5 Python scripts fixed (`from datetime import datetime, timezone`). All 42 baselines carry git-tracked provenance. |
| **Cross-validation** | `cross_validate.py` extended from 24 experiments (Tracks 1-5) to all 7 tracks. 113/113 checks pass. |
| **Binary lint evolution** | `pub(crate)` → `pub` in binary-private modules (nursery `redundant_pub_crate`). `gpu/dispatch/mod.rs` `too_many_lines` → documented `#[expect]`. |
| **603 tests** | Up from 601 (V27). 2 new IPC tests (`try_send_connect_fails_gracefully`, `send_error_display`). Zero failures, zero clippy warnings. |

---

## V27 Deep Evolution Sprint (from V25)

V27 continues deep debt evolution, cross-spring absorption, and modern idiomatic Rust patterns.

| Change | Impact |
|--------|--------|
| **D1: IPC cast safety** | Added `sz`/`sz_or`/`sza` helpers to eliminate ~40 raw `as usize` casts in IPC handlers. `len_f64()` utility for safe precision-loss-annotated casts. |
| **D6: ODE→WGSL codegen** | Absorbed barraCuda `OdeSystem` pattern from wetSpring. 3 implementations: `MichaelisMentenOde`, `OralOneCompartmentOde`, `TwoCompartmentOde`. CPU+GPU integration via `BatchedOdeRK4::generate_shader()`. 7 new tests. |
| **D7: Uncertainty quantification** | Absorbed bootstrap CI, jackknife variance, bias-variance decomposition, MBE, and Monte Carlo propagation from groundSpring. `uncertainty.rs` module. 11 new tests. |
| **D8: `core::` imports** | `std::fmt` → `core::fmt`, `std::f64` → `core::f64` for `no_std` readiness. |
| **601 tests** | Up from 583 (V26) / 501 (V25). 18 new tests from D6+D7 absorptions. Zero failures, zero clippy warnings. |

---

## V25 Track 6+7 Buildout — Comparative Medicine + Drug Discovery (from V24)

V25 completes the Track 6 (Comparative Medicine) and Track 7 (Drug Discovery) paper queues, adding 12 new experiments with 173 validation checks and 7 Python Tier 0 baselines.

| Change | Impact |
|--------|--------|
| **Track 7 DD-001–DD-005 complete** | Anderson-augmented MATRIX scoring (Exp090), ADDRC HTS analysis (Exp091), compound IC50 profiling (Exp092), ChEMBL JAK panel (Exp093), Rho/MRTF/SRF fibrosis scoring (Exp094). 5 experiments, ~70 validation checks. |
| **Track 6 CM-001–CM-007 complete** | Canine IL-31 kinetics (Exp100), JAK1 selectivity (Exp101), IL-31 pruritus time-course (Exp102), lokivetmab dose-duration (Exp103), cross-species PK (Exp104), canine gut Anderson (Exp105), feline hyperthyroidism MM PK (Exp106). 7 experiments, ~103 validation checks. |
| **New library modules** | `discovery/` (matrix_score, hts, compound, fibrosis) + `comparative/` (species_params, canine, feline) — 8 new Rust files, 17 named tolerances. |
| **TissueContext struct** | Reduces `score_compound` argument count from 8 to 5 via parameter grouping. |
| **Species-agnostic PK bridge** | Allometric scaling validated across 5 species (mouse, rat, dog, human, horse). |
| **7 Python Tier 0 baselines** | NumPy controls for all Track 6+7 experiments with provenance JSON. |
| **501 tests** | Up from 485. 173 validation checks across 12 new experiment binaries. Zero failures, zero clippy warnings. |

---

## V24 Deep Audit Execution + Modern Rust Evolution (from V23)

V24 executes on the comprehensive audit — eliminating duplication, evolving hardcoded patterns to capability-based runtime discovery, and modernizing Rust idioms.

| Change | Impact |
|--------|--------|
| **toadStool Hill/AUC delegation** | `stage.rs` no longer reimplements Hill or AUC — delegates to `pkpd::hill_sweep()` and `pkpd::auc_trapezoidal()`. Zero duplicate math. |
| **gpu/context.rs smart refactor** | 968 → 350 LOC. Per-op buffer preparation extracted to `gpu/fused.rs` (340 LOC) by responsibility. `execute_fused` now clean 3-phase: prepare → submit → readback. |
| **Capability-based primal discovery** | Removed hardcoded `COMPUTE_PRIMAL_DEFAULT`/`DATA_PRIMAL_DEFAULT`. `discover_compute_primal()`/`discover_data_primal()` now probe socket dir via `capability.list` with well-known name fallback. |
| **Songbird wired** | `announce_to_songbird()` called during primal startup — advertises `health.*` capabilities to petalTongue and other primals. |
| **Tolerance constants expanded** | 12 new named constants: `HILL_AT_EC50`, `DETERMINISM`, `PCIE_BANDWIDTH`, `PCIE_GEN{3,4,5}_16X_GBPS`, `TRP_RANGE_*`, `SEROTONIN_MIDPOINT_*`, `HILL_SATURATION_100X`. |
| **ValidationHarness migration** | exp050, exp070, exp080 migrated from ad-hoc counters. All use named tolerances. |
| **exp089 exit code fix** | Replaced `assert_eq!` panic with proper `exit(1)`. |
| **cross_validate.py docstring** | Corrected misleading "Python vs Rust" claim to accurate "baseline self-consistency". |
| **CI clippy nursery** | `.github/workflows/ci.yml` now enforces `-W clippy::nursery` matching `lib.rs`. |
| **Python provenance** | Added provenance headers to exp078-082 control scripts. |
| **baseCamp cleanup** | Removed duplicate files (gonzales.md, mok_testosterone.md, drug_matrix_comparison.md → canonical subdirectory versions). |

---

## V23 Deep Debt Remediation + Production Hardening (from V22)

V23 is a zero-debt deep evolution. Every audit finding from the V22 comprehensive audit is resolved.

| Change | Impact |
|--------|--------|
| **License compliance** | AGPL-3.0-or-later across all files (was AGPL-3.0-only). Includes .rs, .py, .sh, .toml, .md. scyBorg trio compliant. |
| **clippy::nursery enforced** | `#![deny(clippy::nursery)]` added to crate root. 5 nursery findings fixed (`const fn`, `map_or_else`, `or_fun_call`, `suboptimal_flops`, `must_use_candidate`). |
| **#[allow] → #[expect]** | All `#[allow()]` in production code eliminated. Replaced with `#[expect()]` with explicit `reason` strings. |
| **UniBin compliance** | Primal binary now uses `clap` with `serve`, `version`, `capabilities` subcommands. `--help` and `--version` flags. |
| **SIGTERM handling** | Accept loop handles `Interrupted` errors; clean socket removal on shutdown. |
| **13 capabilities wired** | NLME (FOCE, SAEM), CWRES, VPC, GOF, QS gene profile, QS effective disorder, WFDB decode, population TRT, population Monte Carlo, TRT scenario, patient parameterize, risk annotate. |
| **dispatch.rs refactored** | 1193-line monolith → 5 domain modules (pkpd 363, microbiome 174, biosignal 186, clinical 295, mod 149). All under 400 LOC. |
| **Discovery unified** | Capability-based primal discovery. Zero hardcoded primal names — all use named constants and env var overrides. |
| **Three-tier fetch** | biomeOS → NestGate → local cache fully implemented (was TODO stubs). |
| **Tolerances centralized** | 15 experiments migrated from inline magic numbers to `tolerances::*` constants. |
| **ValidationHarness** | 10 experiments migrated from ad-hoc counters to `ValidationHarness` (hotSpring pattern). |
| **unwrap/expect eliminated** | All production `.unwrap()` and `.expect()` replaced with safe patterns. |
| **GPU rewire documented** | Tier A → barraCuda upstream ops (Hill, PopPK, Diversity) with clear rewire plan. |
| **435 tests** | Up from 414. Zero failures, zero clippy warnings (pedantic + nursery). |

---

## V22 biomeOS BYOB Niche Deployment (from V21)

V22 transforms healthSpring from experiment binaries into a biomeOS niche — a composed set of primals and workflow graphs discoverable and orchestrable by the Neural API.

| Change | Impact |
|--------|--------|
| **healthspring_primal binary** | `ecoPrimal/src/bin/healthspring_primal.rs` — JSON-RPC 2.0 server over Unix socket, XDG path, biomeOS registration + heartbeat, SIGTERM cleanup. |
| **IPC dispatch module** | `ecoPrimal/src/ipc/dispatch.rs` — maps 55+ JSON-RPC methods to science functions across 6 domains (PK/PD, microbiome, biosignal, endocrine, diagnostic, clinical). |
| **IPC infrastructure** | `ecoPrimal/src/ipc/{rpc,socket}.rs` — JSON-RPC response formatting, Unix socket path resolution, primal discovery. |
| **Niche manifest** | `graphs/healthspring_niche.toml` — declares healthSpring as a niche, lists primals + workflow graphs. |
| **Patient assessment graph** | `graphs/healthspring_patient_assessment.toml` — ConditionalDag: 4 parallel science tracks → cross-track → composite → visualize. |
| **TRT scenario graph** | `graphs/healthspring_trt_scenario.toml` — Sequential: testosterone PK → outcomes → HRV → cardiac → gut → scenario → visualize. |
| **Microbiome analysis graph** | `graphs/healthspring_microbiome_analysis.toml` — Sequential: diversity (parallel) → Anderson → resistance → SCFA → gut-brain → Bray-Curtis → antibiotic. |
| **Biosignal monitor graph** | `graphs/healthspring_biosignal_monitor.toml` — Continuous @ 250 Hz: ECG/PPG/EDA → QRS → HRV (feedback) → stress → arrhythmia → fusion → render. |
| **Niche deploy graph** | `graphs/healthspring_niche_deploy.toml` — startup ordering for all primals in the niche. |
| **414 tests** | 337 ecoPrimal + 33 forge + 30 toadStool + 8 IPC + 3 doc-tests + 3 integration. |

---

## V14 NLME + Full Pipeline Evolution (from V13)

V14 adds NLME population pharmacokinetics, NCA, WFDB parsing, diagnostics, Kokkos-equivalent benchmarks, full petalTongue pipeline visibility, and industry benchmark mapping.

| Change | Impact |
|--------|--------|
| **NLME population PK** | FOCE + SAEM estimation in `ecoPrimal/src/pkpd/nlme.rs` — sovereign replacement for NONMEM/Monolix. 30 subjects, 150 FOCE iterations, 200 SAEM iterations. Theta/omega/sigma recovery validated. |
| **NCA** | Non-compartmental analysis in `ecoPrimal/src/pkpd/nca.rs` — sovereign WinNonlin replacement. Lambda-z, AUC_inf, MRT, CL, Vss. |
| **NLME diagnostics** | CWRES, VPC (50 simulations), GOF in `ecoPrimal/src/pkpd/diagnostics.rs`. CWRES mean <2.0, GOF R²≥0. |
| **WFDB parser** | PhysioNet Format 212/16 streaming parser in `ecoPrimal/src/wfdb.rs`. Beat annotation parsing. |
| **Kokkos-equivalent benchmarks** | Reduction, scatter, Monte Carlo, ODE batch, NLME iteration in `ecoPrimal/benches/kokkos_parity.rs`. GPU readiness evidence. |
| **Full petalTongue pipeline** | 28 nodes, 29 edges, 121 channels across all 7 DataChannel types. NLME scenario builder (5 nodes: population, NCA, CWRES, VPC, GOF). WFDB ECG node. |
| **Exp075** | NLME cross-validation: FOCE/SAEM parameter recovery, NCA metrics, CWRES, GOF. 19 binary checks. |
| **Exp076** | Full pipeline petalTongue scenario validation. 197 binary checks across all 5 tracks + full study. |
| **Industry benchmarks** | SnapGene, Chromeleon, NONMEM, Monolix, WinNonlin profiled. Sovereign replacements mapped to ecoPrimals stack. |

---

## V13 Deep Audit Evolution (from V12)

V13 is a code quality and correctness evolution — no new experiments, but significant structural improvements:

| Change | Impact |
|--------|--------|
| **Anderson eigensolver** | Fixed IPR bug: Hamiltonian diagonal was used instead of actual eigenvectors. Implemented tridiagonal QL algorithm in `microbiome.rs` for correct eigenvalue/eigenvector computation. Fixes `diagnostic.rs` and `scenarios/microbiome.rs`. |
| **Smart clinical.rs refactor** | 1177 → 374 lines (clinical.rs) + 819 lines (clinical_nodes.rs). Eight node-building functions extracted by domain responsibility, not arbitrary split. Both files under 1000-line limit. |
| **LCG PRNG centralization** | New `rng.rs` module (37 lines): `LCG_MULTIPLIER`, `lcg_step()`, `state_to_f64()`. Replaced hardcoded `6_364_136_223_846_793_005` in 4 files. |
| **Math deduplication** | `endocrine::evenness_to_disorder` → delegates to `microbiome::evenness_to_disorder`. `endocrine::lognormal_params` → delegates to `pkpd::LognormalParam::to_normal_params`. |
| **Capability-based discovery** | Replaced hardcoded `/tmp/songbird.sock` in `capabilities.rs` with glob-based `songbird*.sock` discovery. |
| **Flaky IPC test fix** | `AtomicU64` unique socket paths + refactored test harness eliminates `Barrier` race conditions. |
| **Doc-tests** | 4 added: `shannon_index`, `hill_dose_response`, `auc_trapezoidal`, `state_to_f64`. |
| **Tolerance registry** | Added `exp067` and `exp069` CPU parity class entries. |

---

## Domains

### Track 1: Pharmacokinetic / Pharmacodynamic Modeling (Exp001-006)

Pure Rust PK/PD tools replacing Python/NONMEM dependency chains. Extends neuralSpring nS-601–605 (veterinary) to human therapeutics.

- Hill dose-response (4 human JAK inhibitors + canine reference) — Exp001
- One-compartment PK (IV bolus + oral Bateman + multiple dosing + AUC) — Exp002
- Two-compartment PK (biexponential α/β phases, peripheral compartment) — Exp003
- mAb PK cross-species transfer (lokivetmab → nemolizumab/dupilumab) — Exp004
- Population PK Monte Carlo (1,000 virtual patients, lognormal IIV) — Exp005
- PBPK multi-compartment (5-tissue: liver, kidney, muscle, fat, rest) — Exp006

### Track 2: Gut Microbiome and Colonization Resistance (Exp010-013)

Extends wetSpring's Anderson localization framework from soil to gut.

- Shannon/Simpson/Pielou diversity indices + Chao1 richness — Exp010
- Anderson localization in gut lattice (1D localization length ξ) — Exp011
- C. difficile colonization resistance score — Exp012
- FMT RCDI (fecal microbiota transplant, recurrent C. difficile) — Exp013

### Track 3: Biosignal Processing (Exp020-023)

Real-time physiological signal analysis on sovereign hardware.

- Pan-Tompkins QRS detection (ECG R-peak, 5-stage intermediates) — Exp020
- HRV metrics (RMSSD, pNN50, LF/HF, power spectrum) — Exp021
- PPG SpO₂ (pulse oximetry, reflectance) — Exp022
- Biosignal fusion (ECG + PPG + EDA multi-modal) — Exp023

### Track 4: Endocrinology — Testosterone PK and TRT Outcomes (Exp030-038)

Clinical claim verification pipeline: extracting quantifiable claims from Dr. Charles Mok's clinical reference and validating against published registry data.

- Testosterone PK: IM injection steady-state (weekly vs biweekly) — Exp030
- Testosterone PK: pellet depot (5-month, zero-order release) — Exp031
- Age-related testosterone decline (Harman 2001 BLSA model) — Exp032
- TRT metabolic response: weight/BMI/waist (Saad 2013 registry) — Exp033
- TRT cardiovascular: lipids + CRP + BP (Saad 2016, Sharma 2015) — Exp034
- TRT diabetes: HbA1c + insulin sensitivity (Kapoor 2006 RCT) — Exp035
- Population TRT Monte Carlo (10K virtual patients, IIV + age-adjustment) — Exp036
- Testosterone–gut axis: microbiome stratification (cross-track 2×4) — Exp037
- HRV–TRT cardiovascular (cross-track 3×4) — Exp038

### Track 5: NLME Population Pharmacokinetics (Exp075-076)

Sovereign replacement for NONMEM (FOCE), Monolix (SAEM), and WinNonlin (NCA). Full population PK modeling with diagnostics.

- NLME cross-validation: FOCE + SAEM parameter recovery, NCA metrics, CWRES, GOF — Exp075
- Full pipeline petalTongue scenario validation (all 5 tracks, 28 nodes, 121 channels) — Exp076

### Track 6: Comparative Medicine / One Health (V25 — Complete)

Species-agnostic mathematics validated on animal models. Study disease where it naturally occurs, gain causal insight, translate to humans via parameter substitution.

- 7 experiments complete (Exp100–106): canine IL-31 kinetics, JAK1 selectivity, IL-31 pruritus time-course, lokivetmab dose-duration, cross-species PK, canine gut Anderson, feline hyperthyroidism MM PK
- See [specs/PAPER_REVIEW_QUEUE.md](specs/PAPER_REVIEW_QUEUE.md) for details

### Track 7: Drug Discovery / ADDRC (V25 — Complete)

Anderson-augmented MATRIX scoring → ADDRC HTS → Gonzales iPSC → Ellsworth med chem pipeline.

- 5 experiments complete (Exp090–094): MATRIX scoring, ADDRC HTS analysis, compound IC50 profiling, ChEMBL JAK panel, Rho/MRTF/SRF fibrosis scoring
- See [specs/PAPER_REVIEW_QUEUE.md](specs/PAPER_REVIEW_QUEUE.md) for details

### Integrated Diagnostics (Exp050-052)

- Integrated patient diagnostic pipeline (4 tracks + cross-track + composite risk) — Exp050
- Population diagnostic Monte Carlo (1,000 virtual patients) — Exp051
- petalTongue scenario schema validation (DataChannel, ClinicalRange) — Exp052

### GPU Pipeline (Exp053-055)

- GPU parity: WGSL shader output vs CPU baseline (Hill, PopPK, Diversity) — Exp053
- Fused pipeline: all ops in one GPU submission, toadStool dispatch — Exp054
- GPU scaling: 1K→10M sweep, crossover analysis, field deployment thesis — Exp055

### Visualization (Exp056)

- Full petalTongue 5-track scenario generation (57 checks, 7 channel types, 14 scenarios) — Exp056

### Validation Track (Exp040)

- barraCuda CPU parity (Tier 0+1 baseline for GPU migration) — Exp040

### CPU vs GPU Parity & Mixed Dispatch (Exp060-062)

- CPU vs GPU pipeline comparison (full matrix, 27 parity checks) — Exp060
- Mixed hardware dispatch via NUCLEUS topology (22 dispatch route checks) — Exp061
- PCIe P2P transfer validation (DMA planning, 26 transfer checks) — Exp062

### Clinical TRT Scenarios & petalTongue Integration (Exp063-065)

- Patient-parameterized clinical TRT scenarios (5 archetypes, 8 nodes/8 edges each) — Exp063
- IPC push to petalTongue (Unix socket discovery, JSON-RPC render push, fallback to file) — Exp064
- Live streaming dashboard (ECG, HRV, PK via StreamSession with backpressure) — Exp065

### Compute & Benchmark (Exp066-072)

- barraCuda CPU benchmark (Hill, PopPK, Diversity timing) — Exp066
- GPU parity extended (additional kernel validation) — Exp067
- GPU benchmark (throughput at scale) — Exp068
- toadStool dispatch matrix (stage assignment validation) — Exp069
- PCIe P2P bypass (NPU→GPU direct transfer) — Exp070
- Mixed system pipeline (CPU+GPU+NPU coordinated execution) — Exp071
- Compute dashboard (toadStool streaming → petalTongue live gauges) — Exp072

### petalTongue Evolution (Exp073-074)

- Clinical TRT live dashboard (PK trough streaming, HRV improvement, cardiac risk replace) — Exp073
- Interaction roundtrip (mock petalTongue: render, append, replace, gauge, capabilities, subscribe — 12/12) — Exp074

### NLME + Full Pipeline (Exp075-076)

- NLME cross-validation: FOCE/SAEM parameter recovery, NCA (λz, AUC∞), CWRES, GOF (19 checks) — Exp075
- Full pipeline petalTongue scenario validation: 5 tracks, 28 nodes, 29 edges, 121 channels, 197 checks — Exp076

### V16 Primitives (Exp077-082)

Six new domain experiments closing the paper queue (30/30):

- Michaelis-Menten nonlinear PK (capacity-limited elimination) — Exp077
- Antibiotic perturbation (diversity decline/recovery dynamics) — Exp078
- SCFA production (Michaelis-Menten kinetics: acetate, propionate, butyrate) — Exp079
- Gut-brain serotonin (tryptophan metabolism pathway) — Exp080
- EDA stress detection (SCL, phasic decomposition, SCR) — Exp081
- Arrhythmia beat classification (template correlation: Normal, PVC, PAC) — Exp082

### GPU V16 Parity (Exp083)

- GPU parity for V16 primitives: 3 new WGSL compute shaders (MM batch, SCFA batch, Beat classify) — Exp083

### CPU Parity Benchmarks (Exp084)

- V16 CPU parity bench: Rust 84× faster than Python across 6 primitives (33 Rust checks, 17 Python checks) — Exp084

### GPU Scaling + toadStool Dispatch + NUCLEUS Routing (Exp085-087)

- barraCuda GPU vs CPU V16 scaling bench (4 scales × 3 ops, fused pipeline, metalForge routing) — Exp085
- toadStool V16 streaming dispatch (execute_cpu + streaming callbacks, GPU-mappability) — Exp086
- metalForge mixed NUCLEUS V16 dispatch (Tower/Node/Nest topology, PCIe P2P bypass, plan_dispatch) — Exp087

### petalTongue V16 Visualization + Patient Explorer (Exp088-089)

- Unified dashboard: all scenarios (5 tracks + V16 + compute), 326 validation checks, JSON dump + IPC push — Exp088
- Patient explorer: CLI-parameterized diagnostic + V16 analysis, streaming to petalTongue — Exp089

---

## Validation Protocol

```
Tier 0: Python control (published algorithm, reference implementation)
Tier 1: Rust CPU (Pure Rust, f64-canonical, tolerance-documented)
Tier 2: Rust GPU (barraCuda WGSL shaders, math parity with CPU)
Tier 3: metalForge (toadStool dispatch, cross-substrate routing)
Tier 4: Primal composition (IPC dispatch vs direct Rust — the NUCLEUS composition surface)
Tier 5: Deploy graph validation (TOML graph ↔ proto-nucleate ↔ capability surface consistency)
```

**Current state**: Tier 0+1 validation complete for **84** experiments (ValidationHarness **84/84**). **Tier 2 live**: 6 WGSL shaders (3 Tier A + 3 Tier B), fused pipeline, CPU vs GPU parity matrix. **Tier 3 live**: metalForge NUCLEUS routing for all Workload variants, toadStool streaming dispatch, PCIe P2P bypass. **Tier 4 live** (V47): 6 experiments (exp112–117), IPC dispatch parity, proto-nucleate alias resolution, wire protocol round-trip. **Tier 5 live** (V52): exp118, deploy graph vs proto-nucleate structural alignment (99 checks). **Three-layer validation**: Python → Rust (science), Rust → Python (baselines), NUCLEUS → composition (deploy graphs, fragments, bonding, capabilities).

---

## Directory Structure

```
healthSpring/
├── ecoPrimal/           # Rust library — PK/PD, microbiome, biosignal, endocrine
│   └── src/
│       ├── lib.rs       # #![forbid(unsafe_code)]
│       ├── pkpd/        # Track 1: Hill, 1/2-compartment, allometric, pop PK, PBPK, NLME (FOCE/SAEM), NCA, diagnostics
│       ├── microbiome/   # Track 2: diversity indices, Anderson, clinical models
│       │   ├── mod.rs       # Shannon, Simpson, Pielou, Chao1, communities
│       │   ├── anderson.rs  # Anderson lattice, IPR, localization length
│       │   └── clinical.rs  # FMT, SCFA, antibiotic perturbation, gut-brain serotonin
│       ├── biosignal/    # Track 3 (submodules after V14.1 refactor)
│       │   ├── mod.rs    # Re-exports all public items for API compatibility
│       │   ├── ecg.rs    # Pan-Tompkins QRS detection, synthetic ECG
│       │   ├── hrv.rs    # SDNN, RMSSD, pNN50, heart rate from peaks
│       │   ├── ppg.rs    # SpO2 R-value calibration, synthetic PPG
│       │   ├── eda.rs    # SCL, phasic decomposition, SCR detection
│       │   ├── fusion.rs # Multi-channel FusedHealthAssessment
│       │   └── fft.rs    # DFT/IDFT utilities (centralized)
│       ├── endocrine.rs  # Track 4: testosterone PK, decline, TRT outcomes, gut axis
│       ├── wfdb.rs      # WFDB parser (PhysioNet Format 212/16, annotations)
│       ├── rng.rs       # Deterministic LCG PRNG (centralized)
│       ├── safe_cast.rs # Checked numeric conversions (usize_u32, usize_f64, f64_f32)
│       ├── gpu/         # Tier 2: GPU dispatch + GpuContext + fused pipeline
│       │   ├── mod.rs
│       │   ├── dispatch.rs
│       │   ├── context.rs  # GpuContext (350 LOC — single-op + fused orchestrator)
│       │   ├── fused.rs    # Per-op buffer prep + readback decode (extracted from context)
│       │   └── sovereign.rs # Sovereign GPU dispatch via SovereignDevice
│       ├── ipc/          # JSON-RPC IPC infrastructure
│       │   ├── mod.rs       # Module root
│       │   ├── socket.rs    # XDG socket resolution + primal discovery
│       │   ├── rpc.rs       # JSON-RPC response helpers + client
│       │   ├── error.rs     # IpcError (8 variants + query helpers)
│       │   ├── resilience.rs # CircuitBreaker + RetryPolicy
│       │   ├── btsp.rs      # BTSP client handshake (BearDog Transport Security)
│       │   ├── client.rs    # Typed PrimalClient + InferenceClient wrappers
│       │   ├── discover.rs  # Structured DiscoveryResult + DiscoverySource
│       │   └── tower_atomic.rs # Tower Atomic integration
│       ├── discovery/    # Track 7: MATRIX, HTS, compound, fibrosis
│       │   ├── matrix_score.rs
│       │   ├── hts.rs
│       │   ├── compound.rs
│       │   └── fibrosis.rs
│       ├── comparative/  # Track 6: species-agnostic PK, canine, feline
│       │   ├── species_params.rs
│       │   ├── canine.rs
│       │   └── feline.rs
│       └── visualization/ # petalTongue integration
│           ├── ipc_push.rs      # JSON-RPC client (render, append, replace, gauge, caps, interact)
│           ├── stream.rs        # StreamSession with backpressure
│           ├── clinical.rs      # Patient-parameterized TRT scenario builder (374 lines)
│           ├── clinical_nodes.rs # TRT node builders (819 lines)
│           ├── scenarios/       # Per-track + topology + dispatch scenario builders
│           └── capabilities.rs  # Songbird capability announcement (glob-based discovery)
│   └── shaders/health/  # WGSL compute kernels (f64)
│       ├── hill_dose_response_f64.wgsl
│       ├── population_pk_f64.wgsl
│       ├── diversity_f64.wgsl
│       ├── michaelis_menten_batch_f64.wgsl
│       ├── scfa_batch_f64.wgsl
│       └── beat_classify_batch_f64.wgsl
├── control/             # Python baselines (Tier 0) — 194 + 7 Track 6+7 cross-validation checks
│   ├── pkpd/            # exp001–exp006, exp077 + cross_validate.py
│   ├── microbiome/      # exp010–exp013, exp078–exp080
│   ├── biosignal/       # exp020–exp023, exp081–exp082
│   ├── endocrine/       # exp030–exp038
│   ├── validation/      # Exp040 CPU parity
│   ├── discovery/       # exp090–094
│   ├── comparative/     # exp100–106
│   └── scripts/         # Benchmark scripts + timing JSON results
├── experiments/         # 90 validation binaries
│   ├── exp001–exp006/   # Track 1: PK/PD
│   ├── exp010–exp013/   # Track 2: Microbiome
│   ├── exp020–exp023/   # Track 3: Biosignal
│   ├── exp030–exp038/   # Track 4: Endocrinology
│   ├── exp040/          # barraCuda CPU parity
│   ├── exp050–exp052/   # Integrated diagnostics
│   ├── exp053–exp056/   # GPU pipeline + visualization
│   ├── exp060–exp062/   # CPU vs GPU + mixed dispatch + PCIe
│   ├── exp063–exp065/   # Clinical TRT + IPC + live streaming
│   ├── exp066–exp072/   # Compute benchmarks + dashboard
│   ├── exp073–exp074/   # petalTongue evolution
│   ├── exp075–exp076/   # NLME + full pipeline
│   ├── exp077–exp082/   # V16 primitives (MM PK, antibiotic, SCFA, serotonin, EDA, arrhythmia)
│   ├── exp083/          # GPU V16 parity (25/25)
│   ├── exp084/          # CPU parity bench (Rust 84× faster)
│   ├── exp085–exp087/   # GPU scaling + toadStool dispatch + NUCLEUS routing
│   ├── exp088–exp089/   # petalTongue V16 visualization + patient explorer
│   ├── exp090–exp096/   # Track 7: Drug Discovery (+ iPSC skin, niclosamide)
│   ├── exp100–exp110/   # Track 6: Comparative Medicine (+ QS Anderson, real 16S, MIT-BIH, equine)
│   ├── ipc/              # biomeOS IPC (JSON-RPC 2.0 dispatch)
│   │   ├── mod.rs
│   │   ├── dispatch/     # 79 method → science function routing
│   │   │   ├── mod.rs    # Central dispatch table
│   │   │   └── handlers/ # Domain handlers (pkpd, microbiome, biosignal, clinical)
│   │   ├── rpc.rs        # JSON-RPC response helpers + client
│   │   ├── socket.rs     # XDG socket path resolution + primal discovery
│   │   ├── error.rs      # thiserror IpcError (8 variants + query helpers)
│   │   └── resilience.rs # CircuitBreaker + RetryPolicy
│   └── bin/
│       └── healthspring_primal.rs  # UniBin-compliant biomeOS primal binary
├── graphs/             # biomeOS niche definition + workflow graphs
│   ├── healthspring_niche.toml              # Niche manifest
│   ├── healthspring_niche_deploy.toml       # Primal startup order
│   ├── healthspring_patient_assessment.toml # ConditionalDag diagnostic pipeline
│   ├── healthspring_trt_scenario.toml       # Sequential TRT workflow
│   ├── healthspring_microbiome_analysis.toml # Sequential microbiome pipeline
│   └── healthspring_biosignal_monitor.toml  # Continuous 250 Hz monitoring
├── metalForge/          # Cross-substrate dispatch (Tier 3)
│   └── forge/
│       └── src/
│           ├── nucleus.rs    # NUCLEUS atomics (Tower, Node, Nest)
│           ├── dispatch.rs   # DispatchPlan, StageAssignment
│           └── transfer.rs   # PCIe P2P transfer planning
├── toadstool/           # Compute dispatch pipeline
│   └── src/
│       ├── pipeline/    # execute(), execute_gpu(), execute_streaming(), execute_auto()
│       └── stage/      # StageOp, BiosignalFusion, AucTrapezoidal, BrayCurtis
├── specs/               # Paper queue, evolution map, compute profile, integration plan
├── whitePaper/          # Scientific documentation
│   ├── baseCamp/        # Faculty-linked sub-theses
│   └── experiments/     # Experiment plan and status
├── wateringHole/        # Cross-spring handoffs
│   └── handoffs/        # → barraCuda, toadStool, petalTongue
├── scripts/             # Dashboard, visualization, sync scripts
├── Cargo.toml           # Workspace (93 members)
└── README.md            # This file
```

---

## Build

```bash
cargo test --workspace                  # 985+ tests
cargo clippy --workspace --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery  # Zero warnings (pedantic denied at crate level)
cargo fmt --check --all                 # Zero diffs
cargo doc --workspace --no-deps         # Zero warnings

# Full validation (all experiments + Python cross-checks)
cargo build --workspace --release
# Run each exp* binary, then:
python3 control/pkpd/cross_validate.py

# Run individual validation binaries
cargo run --bin exp050_diagnostic_pipeline
cargo run --bin exp051_population_diagnostic
cargo run --bin exp052_petaltongue_render

# GPU experiments (requires GPU)
cargo run --release --bin exp053_gpu_parity    # 17 parity checks
cargo run --release --bin exp054_gpu_pipeline  # Fused pipeline + toadStool
cargo run --release --bin exp055_gpu_scaling   # 1K→10M scaling benchmark

# CPU vs GPU and mixed dispatch
cargo run --release --bin exp060_cpu_vs_gpu_pipeline    # 27 parity checks
cargo run --release --bin exp061_mixed_hardware_dispatch # 22 NUCLEUS dispatch checks
cargo run --release --bin exp062_pcie_transfer_validation # 26 PCIe P2P checks

# Full petalTongue visualization — per-track scenario JSON generation
cargo run --bin exp056_study_scenarios  # 57 checks across 5 tracks
cargo run --release --bin dump_scenarios # Write 16 scenario JSON files to sandbox/scenarios/

# NLME + Full Pipeline
cargo run --bin exp075_nlme_cross_validation     # 19 checks (FOCE/SAEM/NCA/CWRES/GOF)
cargo run --bin exp076_full_pipeline_scenarios    # 197 checks (all 5 tracks + full study)

# V16 primitives
cargo run --release --bin exp077_michaelis_menten_pk      # Nonlinear PK
cargo run --release --bin exp084_v16_cpu_parity_bench     # CPU parity: Rust 84× faster

# GPU scaling + dispatch + NUCLEUS
cargo run --release --bin exp085_gpu_vs_cpu_v16_bench     # 47 checks — GPU scaling
cargo run --release --bin exp086_toadstool_v16_dispatch   # 24 checks — toadStool dispatch
cargo run --release --bin exp087_mixed_nucleus_v16        # 35 checks — NUCLEUS routing

# petalTongue V16 visualization + patient explorer
cargo run --release --bin exp088_unified_dashboard             # 326 checks — all scenarios
cargo run --release --bin exp089_patient_explorer              # 14 checks — patient diagnostic + V16
cargo run --release --bin exp089_patient_explorer -- --age 55 --weight 220 --baseline-t 280

# Track 6: Comparative Medicine
cargo run --release --bin exp100_canine_il31
cargo run --release --bin exp106_feline_hyperthyroid

# Track 7: Drug Discovery
cargo run --release --bin exp090_matrix_scoring
cargo run --release --bin exp094_rho_mrtf_fibrosis

# Python controls
python3 control/scripts/bench_v16_cpu_vs_python.py       # V16 Python timing baseline
python3 control/scripts/compare_v16_benchmarks.py        # Rust vs Python comparison
python3 control/scripts/control_exp085_gpu_scaling.py    # GPU scaling validation
```

---

## Relationship to ecoPrimals

healthSpring is a biomeOS **niche** in the ecoPrimals ecosystem. It consumes `barraCuda` (vendor-agnostic GPU math library) and exposes health science capabilities as a discoverable primal via JSON-RPC 2.0. biomeOS composes these capabilities into diagnostic workflows via TOML graphs. The Neural API orchestrates, and the Pathway Learner optimizes.

The springs validate science. healthSpring applies it — as a deployable niche.

---

## V14.1 Deep Debt Evolution (from V14)

V14.1 is a code quality evolution — zero-warning `#![deny(clippy::pedantic)]` enforcement, smart modular refactoring, and DFT deduplication.

| Change | Impact |
|--------|--------|
| **biosignal.rs → biosignal/ submodules** | 953-line monolith split into 6 domain-coherent modules (ecg, hrv, ppg, eda, fusion, fft) with `mod.rs` re-exporting all public items for API compatibility. |
| **clippy::pedantic promoted to deny** | All three lib crates (`barracuda`, `toadstool`, `metalForge/forge`) now use `#![deny(clippy::pedantic)]` instead of `#![warn(...)]`. All warnings resolved — `mul_add`, `must_use`, `const fn`, `while_float`, `branches_sharing_code`, `option_if_let_else`, `significant_drop_tightening`. |
| **DFT deduplication** | `visualization/scenarios/biosignal.rs` HRV power spectrum now delegates to `biosignal::fft::rfft` instead of local DFT reimplementation. |
| **Dead code removal** | Removed unused `cpu_stages` vector in toadStool pipeline. |
| **Idiomatic Rust** | `if let Some(prev) = prev_nest { if prev == id { ... } }` chains replaced with `prev_nest.filter().map()`. Shared code hoisted from if/else branches. |
| **exp023 provenance fix** | Corrected `exp023_biosignal_fusion.py` → `exp023_fusion.py` in baseline JSON and provenance script. |
