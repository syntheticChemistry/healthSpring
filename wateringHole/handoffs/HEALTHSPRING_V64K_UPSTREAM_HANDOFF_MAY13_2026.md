# healthSpring V64k — Deep Debt Resolution + Evolution Sprint (Reconfirmation)

**Date**: May 13, 2026
**Sprint**: V64k
**Upstream audit**: ecoPrimals — Deep Debt Resolution + Evolution Sprint

---

## Summary

Full re-audit of all 7 deep debt categories after V64j wire name changes. **Zero debt confirmed** — no regressions, no new gaps. All findings unchanged from V64i with refreshed audit question answers.

---

## Audit Results — All 7 Categories at Zero Debt

| Category | Count | Evidence |
|----------|-------|---------|
| TODO/FIXME/HACK | **0** | `rg 'TODO\|FIXME\|HACK\|XXX' src/` — no matches |
| `unsafe` code | **0** | `#![forbid(unsafe_code)]` in `lib.rs` + all 5 bin crates. `rg unsafe src/` — only `forbid` declarations and doc references. |
| Production mocks | **0** | 2 mock fns (`mock_petaltongue_response`, `mock_petaltongue_error`) — both inside `#[cfg(test)] mod tests` in `visualization/ipc_push/mod.rs` |
| `panic!` in non-test | **0** | 20 `panic!` calls total — all inside `#[cfg(test)] mod tests` blocks across `gpu/mod.rs`, `microbiome/mod.rs`, `visualization/`, `ipc/`, `pkpd/nonlinear.rs` |
| Files > 800 LOC | **0** | Largest: `ipc/proptest_ipc.rs` at 597 lines. 46,265 lines across all `.rs` files. |
| Clippy pedantic+nursery | **0 warnings, 0 errors** | `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery` passes clean |
| External C deps | **0 runtime** | Default build: `blake3` uses `cc` at build-time for SIMD assembly (not a runtime C dep). `ring` only with `nestgate` feature. All 12 direct deps are pure Rust. |
| Hardcoded routing | **0** | All primal names via `primal_names::*` constants. Self-knowledge (`PRIMAL_NAME`, `PRIMAL_ID`, `TOOL_NAME`) legitimate. NCBI URLs configurable via env vars. |

---

## Dependency Audit (Default Build)

| Dependency | Version | Purpose | C deps |
|-----------|---------|---------|--------|
| `primalspring` | v0.9.25 | Ecosystem framework | `blake3` (build-time `cc` for SIMD) |
| `serde` | 1.0.228 | Serialization | None |
| `serde_json` | 1.0.149 | JSON | None |
| `toml` | 0.8.23 | Config parsing | None |
| `clap` | 4.6.0 | CLI | None |
| `thiserror` | 2.0.18 | Error derives | None |
| `tracing` | 0.1.44 | Instrumentation | None |
| `tracing-subscriber` | 0.3.23 | Log output | None |
| `approx` | 0.5.1 | Float comparison | None |
| `proptest` | 1.10.0 | Property testing (dev) | None |
| `criterion` | 0.5.1 | Benchmarks (dev) | None |

**Feature-gated deps**: `ureq` → `rustls` → `ring` (C) behind `nestgate`. `wgpu` (C GPU backends) behind `gpu`.

---

## Audit Questions

### 1. Python baselines for barraCuda CPU parity

**Two Python baseline suites:**

| Script | Operations | Rust Parity |
|--------|-----------|-------------|
| `control/validation/exp040_barracuda_cpu.py` | stats (mean, std_dev, variance, correlation), Hill, Shannon, Simpson, Chao1, Anderson | **Full** — all matched by Rust scenarios |
| `control/scripts/bench_barracuda_cpu_vs_python.py` | Hill, oral PK, Shannon/Simpson/Pielou, trapezoidal AUC, population MC | **Partial** — Hill + Shannon/Simpson covered; oral PK, Pielou, trapezoidal AUC lack dedicated scenarios |

**Gaps**: Oral one-compartment PK (Bateman absorption), Pielou evenness index, standalone trapezoidal AUC scenario.

### 2. GPU benchmarks (Kokkos/Galaxy/SciPy/LAMMPS)

| Bench | Content |
|-------|---------|
| `gpu_parity.rs` | Hill batch 10k, diversity batch 1k, population PK MC 5k, MM batch 512 — wgpu GPU or CPU fallback |
| `kokkos_parity.rs` | Kokkos-**modeled** CPU patterns: reduce, scatter-style diversity, Monte Carlo pop PK, NCA batch, FOCE |
| `cpu_parity.rs` | Mirrors Python bench + V16 ops (MM, SCFA, serotonin, antibiotic, stress, beat classify) |

**No SciPy/LAMMPS/Galaxy direct comparisons.** Sovereign WGSL shaders are not ports of external GPU frameworks.

### 3. Unimplemented/unvalidated/untested

- **~30 Python baselines** without Rust validation scenarios (exp003-006, exp012-013, exp022-023, exp031-038, exp078-082, exp091-094, exp098-099, exp101-106, exp111)
- **V16 primitives** from exp084 (antibiotic perturbation, SCFA, serotonin, EDA, beat classification) — benchmarked in `cpu_parity.rs` but not in scenario registry
- **exp084 + exp085** — timing/scaling benchmarks, not registered as validation scenarios

### 4. Unreviewed papers

**2 papers** in LTEE GuideStone Queue, both marked QUEUED:
- **E2**: Mardikoraem & Woldring 2025, "HOLIgraph" (*J Cheminformatics*) — OATP PK/PD bridge
- **E4**: Woldring Lab 2024, macrocyclic peptide screening (*bioRxiv*)

**45/45 main-track papers complete.**

### 5. Datasets

| Dataset | Fetch Script | SHA256 |
|---------|-------------|--------|
| `qs_gene_matrix` | None (manual assembly) | Empty |
| `mitbih` | `fetch_mitbih.sh` | Empty |
| `chembl_hill_panel` | `fetch_chembl.sh` | Empty |
| `hmp_16s` | `fetch_hmp_16s.sh` | Empty |
| `geo_androgen_receptor` | `fetch_geo_ar.sh` | Empty |

All 5 datasets need SHA256 population after fetch. `qs_gene_matrix` needs a fetch script.

---

## Build Status

- `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery`: **0 warnings, 0 errors**
- `cargo test`: **all pass**
- Rust edition: 2024 (rust-version 1.87)
- Nest Atomic: structural + wire verified, live-ready pending deploy

---

## Gap Status Summary

| State | Count | IDs |
|-------|-------|-----|
| **Resolved** | 28 | #1, #5-8, #11-19, #23, #28-29, #32, #34-37 |
| **Open (external)** | 9 | #2, #10, #20-22, #24-27 |
| **Open (internal, low priority)** | 5 | #3, #4, #9, #30-31 |
| **Documented (evolving)** | 4 | #33, #38-40 |
| **N/A** | 1 | #41 (GPU parity — sovereign WGSL) |
