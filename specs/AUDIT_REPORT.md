# healthSpring Codebase Completion Audit Report

> **Note**: This is a V42 historical snapshot. Current metrics: V53, 948+ tests,
> 90 experiments, zero clippy/fmt/doc warnings. See `README.md` for current state.

## V42 (March 23, 2026)

V42 deep debt resolution: all 548 missing_docs resolved; clippy pedantic+nursery+doc-markdown passes with `-D warnings`; RUSTSEC-2026-0049 resolved; 3 barraCuda API drift errors fixed; `transport.rs` panic evolved to `Result`; `ValidationSink` trait added; `normalize_method()` IPC; `OnceLock` GPU probe; 928 tests all passing.

**V42 — Sections 6–8 findings closed:** The items recorded under **§6 Hardcoded tolerance values**, **§7 tolerances.rs completeness** (no change required beyond registry sync), and **§8 provenance.rs** in the historical audit snapshot below are **resolved** in code: `ecoPrimal/src/biosignal/ppg.rs` uses `tolerances::DIVISION_GUARD` (no duplicate local constant); PROVENANCE_REGISTRY entries for Python baselines populate `git_commit`, `run_date`, and `exact_command` where applicable; `experiments/exp050_diagnostic_pipeline` uses `tolerances::ENDOCRINE_TESTOSTERONE_PASSTHROUGH` instead of an inline `0.1` tolerance. The subsections are retained for traceability.

---

## Historical snapshot (pre-V39, 2026-03-19)

**Scope:** All Rust source files in `ecoPrimals/springs/healthSpring`

---

## 1. TODO / FIXME / HACK / XXX / UNIMPLEMENTED / stub / mock / placeholder / hardcoded / "expected value"

### Findings

| File | Line | Finding |
|------|------|---------|
| `experiments/exp003_two_compartment_pk/src/main.rs` | 34 | Comment: `// Expected values from exp003_baseline.json` — documents expected values (informational, not a TODO) |
| `ecoPrimal/src/provenance.rs` | 4, 657 | Comments about "hardcoded expected value" — documentation of provenance pattern (informational) |
| `experiments/exp108_real_16s_anderson/src/main.rs` | 7 | Comment: `Uses published HMP reference community profiles (hardcoded with provenance)` — documented intentional hardcoding |

**No TODO, FIXME, HACK, XXX, UNIMPLEMENTED, todo!(), unimplemented!(), or stub found.**

**Mock/placeholder notes:**
- `experiments/exp074_interaction_roundtrip/src/main.rs` L246–250: `mock_server` — **inside `#[cfg(test)]` mod** ✓
- `ecoPrimal/src/visualization/ipc_push/mod.rs` L249–266: `mock_petaltongue_response`, `mock_petaltongue_error` — **inside `#[cfg(test)]` mod** ✓

---

## 2. #[allow()] Attributes in Non-Test Code

**Finding:** No `#[allow()]` attributes found in the codebase.

**Note:** `#[expect()]` (Clippy) is used extensively — these are intentional, documented suppressions (e.g. `cast_precision_loss`, `too_many_lines`). These are acceptable per project conventions.

---

## 3. `unsafe` Usage

**Finding:** No `unsafe` blocks or `unsafe fn` in the codebase.

| File | Line | Context |
|------|------|---------|
| `ecoPrimal/src/bin/healthspring_primal/server/signal.rs` | 18 | Comment only: "Since we forbid unsafe, we rely on..." — documents policy |

---

## 4. Mock Structs/Functions Outside #[cfg(test)]

**Finding:** All mock functions are inside `#[cfg(test)]` blocks.

| Location | Mock | Status |
|----------|------|--------|
| `experiments/exp074_interaction_roundtrip/src/main.rs` | `mock_server`, `mock_server_roundtrip_receives_expected_requests` | Inside `mod tests` ✓ |
| `ecoPrimal/src/visualization/ipc_push/mod.rs` | `mock_petaltongue_response`, `mock_petaltongue_error`, `run_socket_test` | Inside `mod tests` ✓ |

---

## 5. Hardcoded File Paths, Socket Paths, Primal Names

### Hardcoded Paths (Non-Configurable)

| File | Line | Path | Notes |
|------|------|------|-------|
| `ecoPrimal/src/ipc/rpc.rs` | 171 | `/tmp/nonexistent_healthspring_rpc_test.sock` | Test-only |
| `ecoPrimal/src/visualization/ipc_push/mod.rs` | 169 | `/tmp/test-socket.sock` | Test-only |
| `ecoPrimal/src/visualization/ipc_push/mod.rs` | 374 | `/tmp/nonexistent_hs_test.sock` | Test-only |
| `ecoPrimal/src/ipc/tower_atomic.rs` | 244, 251–252 | `/tmp/nonexistent_biomeos_test_dir/`, `/tmp/test-crypto.sock`, `/tmp/test-discovery.sock` | Test-only |
| `ecoPrimal/src/ipc/lifecycle_dispatch.rs` | 121 | `/tmp/test.sock` | Test-only |
| `ecoPrimal/src/data/rpc.rs` | 120 | `/tmp/nonexistent_healthspring_test.sock` | Test-only |
| `ecoPrimal/src/visualization/capabilities.rs` | 20 | `SONGBIRD_PATHS: ["biomeos/songbird.sock", "songbird/songbird.sock"]` | **Production** — documented as intentional (Songbird is discovery service) |
| `ecoPrimal/src/visualization/capabilities.rs` | 273, 281 | `/tmp/healthspring.sock`, `/tmp/test.sock` | Test assertions |
| `ecoPrimal/src/visualization/stream.rs` | 331, 339, 352 | `/tmp/test.sock`, `/tmp/nonexistent_hs_stream.sock` | Test-only |

### Baseline Paths (Relative / Configurable)

| File | Path Pattern | Configurable? |
|------|--------------|--------------|
| `experiments/exp001_hill_dose_response` | `control/pkpd/exp001_baseline.json` | Via `CARGO_MANIFEST_DIR` / relative |
| `experiments/exp021_hrv_metrics` | `../../control/biosignal/exp021_baseline.json` | Relative to manifest |
| `experiments/exp022_ppg_spo2` | `../../control/biosignal/exp022_baseline.json` | Relative |
| `experiments/exp023_biosignal_fusion` | `../../control/biosignal/exp023_baseline.json` | Relative |
| `experiments/exp010_diversity_indices` | `include_str!("../../../control/microbiome/exp010_baseline.json")` | Compile-time include |
| `experiments/exp011_anderson_gut_lattice` | `include_str!("../../../control/microbiome/exp011_baseline.json")` | Compile-time |
| `experiments/exp012_cdiff_resistance` | `include_str!("../../../control/microbiome/exp012_baseline.json")` | Compile-time |

### Data Paths

| File | Path | Configurable? |
|------|------|---------------|
| `ecoPrimal/src/data/fetch.rs` | `HEALTHSPRING_DATA_ROOT`, `data/qs_gene_matrix.json` | Env var + fallback |
| `ecoPrimal/src/data/storage.rs` | `$HEALTHSPRING_DATA_ROOT/ncbi_cache/{db}/{id}.json` | Env var |
| `ecoPrimal/src/bin/healthspring_primal/main.rs` | `$XDG_RUNTIME_DIR/biomeos/healthspring-{family_id}.sock` | Env-based |
| `ecoPrimal/src/ipc/socket.rs` | `BIOMEOS_SOCKET_DIR`, `$XDG_RUNTIME_DIR/biomeos`, `/tmp/biomeos` | Env + fallback |

---

## 6. Hardcoded Tolerance Values (Should Use tolerances.rs)

**Status (V42):** Addressed — exp050 uses `ENDOCRINE_TESTOSTERONE_PASSTHROUGH`; `ppg.rs` uses `tolerances::DIVISION_GUARD`. Remaining library inline literals below are algorithm defaults or optional future centralization (unchanged from historical audit).

### In Experiment Binaries

| File | Line | Value | Suggested Constant |
|------|------|-------|--------------------|
| `experiments/exp050_diagnostic_pipeline/src/main.rs` | 82 | ~~`0.1`~~ → `tolerances::ENDOCRINE_TESTOSTERONE_PASSTHROUGH` | **Resolved (V42)** |

### In Library Code (Non-Test)

| File | Line | Value | Notes |
|------|------|-------|-------|
| `ecoPrimal/src/biosignal/ppg.rs` | 20 | ~~local `DIVISION_GUARD`~~ → `tolerances::DIVISION_GUARD` | **Resolved (V42)** — uses shared constant |
| `ecoPrimal/src/pkpd/nlme/mod.rs` | 69, 111, 190, 249, 355 | `1e-6`, `1e-12`, `1e-4`, `1e-6` | Solver/algorithm internals — consider named constants |
| `ecoPrimal/src/pkpd/nca.rs` | 125, 144 | `1e-15` | Could use `tolerances::DIVISION_GUARD` or `MACHINE_EPSILON_STRICT` |
| `ecoPrimal/src/pkpd/allometry.rs` | 41 | `1e-12` | Could use `tolerances::MACHINE_EPSILON_TIGHT` |
| `ecoPrimal/src/discovery/compound.rs` | 93, 171 | `1e-15` | Could use `tolerances::DIVISION_GUARD` |
| `ecoPrimal/src/discovery/hts.rs` | 45, 65, 82 | `1e-15` | Same |
| `ecoPrimal/src/biosignal/classification.rs` | 120 | `1e-14` | Could use `tolerances::MACHINE_EPSILON_STRICT` |
| `ecoPrimal/src/ipc/dispatch/handlers/pkpd.rs` | 97, 198–199, 204, etc. | `0.01`, `1e-6`, `0.04`, `0.09` | Default params — acceptable as defaults |
| `ecoPrimal/src/pkpd/diagnostics/vpc.rs` | 114 | `1e-10` | Could use `tolerances::MACHINE_EPSILON` |
| `ecoPrimal/src/pkpd/diagnostics/gof.rs` | 34, 83 | `1e-15` | Could use `tolerances::DIVISION_GUARD` |
| `ecoPrimal/src/pkpd/diagnostics/cwres.rs` | 30 | `1e-15` | Same |
| `ecoPrimal/src/gpu/ode_systems.rs` | 47, 58, 98, 113 | `1e-30` | ODE guard — could use `tolerances::DECOMPOSITION_GUARD` (1e-30) |
| `ecoPrimal/src/pkpd/nlme/solver.rs` | 41, 82, 113, 124, 136, 181, 246, 266 | Various | Algorithm internals — consider centralizing |
| `ecoPrimal/src/wfdb/parser.rs` | 219, 323 | `1e-15` | Could use `tolerances::DIVISION_GUARD` |
| `ecoPrimal/src/endocrine/trt_outcomes.rs` | 23 | `1e-15` | Same |

---

## 7. tolerances.rs — Completeness

**Status (V42):** No open audit gap — all tolerances are named and documented (included in the §6–§8 V42 resolution note for cross-reference only).

**Status:** All tolerances are named and documented.

- **Machine Epsilon Class:** MACHINE_EPSILON, MACHINE_EPSILON_TIGHT, MACHINE_EPSILON_STRICT, ANDERSON_IDENTITY, TWO_COMPARTMENT_IDENTITY, DIVERSITY_CROSS_VALIDATE
- **Numerical Method Class:** AUC_TRAPEZOIDAL, TMAX_NUMERICAL, HALF_LIFE_POINT, ALLOMETRIC_CL_RATIO, ABUNDANCE_NORMALIZATION, LEVEL_SPACING_RATIO, W_CROSS_VALIDATE, PIELOU_BOUNDARY, etc.
- **Population/Statistical:** LOGNORMAL_RECOVERY, POP_VD_MEDIAN, POP_KA_MEDIAN, etc.
- **Clinical Plausibility:** FRONT_LOADED_WEIGHT, QRS_SENSITIVITY, HR_DETECTION_BPM, SDNN_UPPER_MS, QRS_PEAK_MATCH_MS
- **GPU/CPU Parity:** GPU_F32_TRANSCENDENTAL, GPU_STATISTICAL_PARITY, CPU_PARITY, etc.
- **Test/Guard:** TEST_ASSERTION_TIGHT, TEST_ASSERTION_LOOSE, DIVISION_GUARD, NCA_TOLERANCE, etc.

All constants have doc comments. `TOLERANCE_REGISTRY.md` provides experiment-level justification.

---

## 8. provenance.rs — Baseline Provenance

**Status (V42):** Registry fields **`git_commit`**, **`run_date`**, and **`exact_command`** are populated for Python-controlled baselines per the provenance update workflow. The table below is a **historical** snapshot of the pre-V39 gap.

### PROVENANCE_REGISTRY — Entries with Empty Provenance (historical)

Many entries had empty `git_commit`, `run_date`, `exact_command` at the time of the original audit:

| Experiment | script | git_commit | run_date | exact_command |
|------------|--------|------------|----------|---------------|
| exp001 | control/pkpd/exp001_hill_dose_response.py | "" | "" | "" |
| exp002 | control/pkpd/exp002_one_compartment_pk.py | "" | "" | "" |
| exp003 | control/pkpd/exp003_two_compartment_pk.py | "" | "" | "" |
| exp004–exp006, exp077 | pkpd scripts | "" | "" | "" |
| exp020–exp023, exp081, exp082 | biosignal | "" | "" | "" |
| exp030–exp038 | endocrine | "" | "" | "" |
| exp040 | validation | "" | "" | "" |
| exp078–exp080 | microbiome | "" | "" | "" |
| bench scripts, tolerances, update_provenance | scripts | "" | "" | "" |

### Entries with Complete Provenance ✓

- exp010, exp011, exp012, exp013 (microbiome): git_commit, run_date, exact_command filled
- exp100–exp106 (comparative): filled
- exp090–exp094 (discovery): filled

**Recommendation (superseded in V42):** Populate `git_commit`, `run_date`, `exact_command` for all Python baseline scripts that generate baseline JSON files. **Done** — see V42 note at top of this report.

---

## 9. validation.rs — hotSpring Pattern Implementation

**Status:** Implemented correctly.

- `ValidationHarness` with `check_abs`, `check_rel`, `check_upper`, `check_lower`, `check_bool`, `check_exact`
- `OrExit` trait for panic-free error handling (replaces `.expect()` / `.unwrap()`)
- `rmse`, `mae`, `nse`, `index_of_agreement` for metrics
- Uses `tolerances::MACHINE_EPSILON_STRICT` for relative check near-zero guard
- Doc comments reference hotSpring pattern and `TOLERANCE_REGISTRY.md`

---

## 10. panic!, unwrap(), expect() in Non-Test Code

### panic! in Non-Test Code

| File | Line | Context |
|------|------|---------|
| `ecoPrimal/src/pkpd/nonlinear.rs` | 160 | `panic!("250 mg/day < vmax 500 should yield Some");` — **production** |
| `ecoPrimal/src/microbiome/mod.rs` | 484 | `panic!("antibiotic_perturbation must return at least one point");` — **production** |
| `ecoPrimal/src/ipc/dispatch/mod.rs` | 279, 282, 292, 295, 310 | `panic!("dispatch returned None...")` — **inside `#[test]`** ✓ |
| `ecoPrimal/src/gpu/mod.rs` | 397, 413, 425, 466, 483, 506 | `panic!("wrong result type");` — **inside `#[test]`** ✓ |

### unwrap() / expect() in Non-Test Code

| File | Line | Context |
|------|------|---------|
| `ecoPrimal/src/wfdb/mod.rs` | 69, 86, 108, 121, 165, 246 | `expect("parse header")`, `unwrap()` — **production** (wfdb parsing) |
| `ecoPrimal/src/pkpd/pbpk.rs` | 249 | `profile.last().expect("non-empty profile")` — **production** |
| `ecoPrimal/src/ipc/dispatch/mod.rs` | 279–310 | Inside `#[test]` ✓ |
| `ecoPrimal/src/visualization/ipc_push/mod.rs` | 194, 219–226, 241, 251–262, etc. | **Inside `#[cfg(test)]`** ✓ |
| `ecoPrimal/src/visualization/capabilities.rs` | 279, 284 | `.expect("capabilities array")` — **production** |
| `ecoPrimal/src/microbiome_transfer.rs` | 125, 129 | `.unwrap()` — **production** |
| `ecoPrimal/src/visualization/mod.rs` | 186, 220, 233, 236, 245, 259–260 | `expect`, `unwrap` — **production** |
| `ecoPrimal/src/visualization/nodes/mod.rs` | 238, 258–259, 276, 292 | `expect`, `unwrap` — **production** |
| `ecoPrimal/src/visualization/scenarios/mod.rs` | 292 | `expect("serialization cannot fail")` — **production** |
| `ecoPrimal/src/visualization/stream.rs` | 325 | `expect("should have avg")` — **production** |
| `ecoPrimal/src/visualization/clinical.rs` | 238, 241, 247, 274, 280, etc. | Multiple `expect`/`unwrap` — **production** |
| `ecoPrimal/src/gpu/ode_systems.rs` | 178, 199, 218, 236 | `.unwrap()` — **production** |
| `metalForge/forge/src/dispatch.rs` | 304 | `.unwrap()` — **production** |

**Recommendation:** Replace production `unwrap()`/`expect()` with `OrExit` or proper `Result`/`Option` handling where feasible.

---

## 11. Data Paths — Relative? Configurable?

| Path Type | Location | Configurable? |
|-----------|----------|---------------|
| NCBI cache | `$HEALTHSPRING_DATA_ROOT/ncbi_cache/{db}/{id}.json` | Yes (env) |
| QS matrix | `HEALTHSPRING_DATA_ROOT` then `data/qs_gene_matrix.json` | Yes |
| Baseline JSON | `control/{track}/{exp}_baseline.json` | Relative to repo; some use `include_str!` |
| Socket dir | `BIOMEOS_SOCKET_DIR`, `$XDG_RUNTIME_DIR/biomeos`, `/tmp/biomeos` | Yes |
| Benchmark outputs | `../../control/scripts/bench_results_*.json` | Relative to manifest |

---

## 12. Experiment Binaries — ValidationHarness Usage

### Sample Verified: exp001, exp020, exp050, exp090

| Experiment | ValidationHarness | Uses tolerances:: | h.exit() |
|------------|-------------------|-------------------|----------|
| exp001 | ✓ | MACHINE_EPSILON, MACHINE_EPSILON_STRICT, HILL_SATURATION_100X | ✓ |
| exp020 | ✓ | MACHINE_EPSILON_TIGHT, QRS_PEAK_MATCH_MS, QRS_SENSITIVITY, HR_DETECTION_BPM, SDNN_UPPER_MS | ✓ |
| exp050 | ✓ | HILL_AT_EC50, DETERMINISM, ENDOCRINE_TESTOSTERONE_PASSTHROUGH | ✓ |
| exp090 | ✓ | DETERMINISM, DISORDER_IMPACT, MATRIX_COMBINED, MATRIX_PATHWAY, TISSUE_GEOMETRY | ✓ |

### Experiments Using ValidationHarness

**83/83** workspace experiment binaries use `ValidationHarness` (V42). Integration demos **exp064**, **exp065**, and **exp088** use the harness with integration/dashboard-oriented checks (not numeric Tier 0 cross-validation). Older greps that reported “62” or listed benchmarks as non-harness are **obsolete**.

---

## Summary of Action Items

1. ~~**Provenance:** Populate `git_commit`, `run_date`, `exact_command` for PROVENANCE_REGISTRY entries with empty provenance.~~ **Resolved (V42)** — see §8.
2. ~~**Hardcoded tolerance:** Replace `0.1` in exp050 L82 with a named constant (e.g. `ENDOCRINE_TESTOSTERONE_MATCH`).~~ **Resolved (V42)** — `ENDOCRINE_TESTOSTERONE_PASSTHROUGH`.
3. ~~**Duplicate constant:** Use `tolerances::DIVISION_GUARD` in `ecoPrimal/src/biosignal/ppg.rs` instead of local `DIVISION_GUARD`.~~ **Resolved (V42)** — see §6.
4. **panic! in production:** Consider replacing `panic!` in `pkpd/nonlinear.rs` L160 and `microbiome/mod.rs` L484 with `Option`/`Result` handling.
5. **unwrap/expect in production:** Audit and replace with `OrExit` or proper error handling in: wfdb, pbpk, visualization (capabilities, mod, nodes, scenarios, stream, clinical), gpu/ode_systems, microbiome_transfer, metalForge.
6. **Hardcoded tolerances in library:** Migrate inline `1e-15`, `1e-12`, `1e-10` etc. to `tolerances` constants where they represent validation thresholds.
