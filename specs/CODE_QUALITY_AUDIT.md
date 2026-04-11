# healthSpring Code Quality Audit Report

> **Note**: This is a V42 historical snapshot. Current metrics: V52, 985+ tests,
> zero clippy/fmt/doc warnings. See `README.md` and `CHANGELOG.md` for current state.

## V42 (March 23, 2026)

| Metric | Value |
|--------|-------|
| Clippy errors | 0 |
| Clippy warnings | 0 |
| `cargo test` (workspace) | 928 passing |
| `cargo deny` | clean |
| `cargo doc` | 0 warnings |

---

## Historical snapshot (pre-V39, March 19, 2026)

**Scope**: ecoPrimal/src/, metalForge/, toadstool/ (excluding archive/)

---

## 1. Line Counts — Files Over 1000 Lines

**Result**: **No file exceeds 1000 lines.**

### ecoPrimal/src/ — Largest Files (all under 1000)

| Lines | File |
|------|------|
| 800 | `ecoPrimal/src/provenance.rs` |
| 732 | `ecoPrimal/src/visualization/scenarios/tests.rs` |
| 549 | `ecoPrimal/src/qs.rs` |
| 537 | `ecoPrimal/src/microbiome/mod.rs` |
| 537 | `ecoPrimal/src/gpu/mod.rs` |
| 528 | `ecoPrimal/src/visualization/scenarios/v16.rs` |
| 528 | `ecoPrimal/src/visualization/scenarios/nlme.rs` |
| 514 | `ecoPrimal/src/visualization/scenarios/biosignal.rs` |
| 510 | `ecoPrimal/src/gpu/context.rs` |
| 501 | `ecoPrimal/src/visualization/ipc_push/mod.rs` |

### metalForge/forge/src/

| Lines | File |
|------|------|
| 484 | `metalForge/forge/src/lib.rs` |
| 363 | `metalForge/forge/src/dispatch.rs` |
| 304 | `metalForge/forge/src/nucleus.rs` |
| 264 | `metalForge/forge/src/transfer.rs` |

### toadstool/src/

| Lines | File |
|------|------|
| 403 | `toadstool/src/pipeline/tests.rs` |
| 305 | `toadstool/src/pipeline/mod.rs` |
| 283 | `toadstool/src/stage/mod.rs` |
| 193 | `toadstool/src/stage/tests.rs` |
| 125 | `toadstool/src/stage/exec.rs` |

---

## 2. lib.rs — `#![forbid(unsafe_code)]`

**Result**: **All three crates enforce forbid(unsafe_code).**

| Crate | File | Line |
|-------|------|------|
| ecoPrimal | `ecoPrimal/src/lib.rs` | 2 |
| metalForge | `metalForge/forge/src/lib.rs` | 2 |
| toadstool | `toadstool/src/lib.rs` | 2 |

ecoPrimal additionally has:
- `#![deny(clippy::unwrap_used)]` (line 6)
- `#![deny(clippy::expect_used)]` (line 7)

---

## 3. `#[allow()]` in Production Code

**Result**: **No `#[allow()]` found.** The codebase uses `#[expect()]` instead (clippy’s documented suppression).

`#[expect()]` appears in production code for:
- `clippy::cast_precision_loss`, `clippy::cast_possible_truncation`, `clippy::cast_sign_loss` (numeric casts)
- `clippy::too_many_arguments`, `clippy::too_many_lines`, `clippy::type_complexity`
- `clippy::expect_used` (e.g. `ecoPrimal/src/visualization/scenarios/mod.rs:288` — serde serialization)
- `clippy::unwrap_used`, `clippy::expect_used` in test modules (with `reason = "test code"`)

---

## 4. `unsafe` Blocks or Functions

**Result**: **No `unsafe` blocks or functions.** One mention is in a comment:

| File | Line | Content |
|------|------|---------|
| `ecoPrimal/src/bin/healthspring_primal/server/signal.rs` | 18 | `// Since we forbid unsafe, we rely on the accept loop timeout...` |

---

## 5. `unwrap()` / `expect()` in Non-Test Code

**Result**: **ecoPrimal library production code is compliant.** All `unwrap`/`expect` in ecoPrimal are either:

1. **Inside `#[cfg(test)]` modules** (wfdb, ipc_push, capabilities, microbiome_transfer, visualization, etc.)
2. **Inside `#[test]` functions** with `#[expect(clippy::unwrap_used, ...)]` where needed
3. **One production use** with explicit `#[expect(clippy::expect_used)]`:
   - `ecoPrimal/src/visualization/scenarios/mod.rs:288-292` — `serde_json::to_string_pretty(...).expect("serialization cannot fail")`

**Note**: `unwrap_or()` and `unwrap_or_else()` are used in production (e.g. `wfdb/parser.rs`) and are safe (provide defaults).

**Experiments (validation binaries)** use `unwrap`/`expect` in several places (e.g. `exp074_interaction_roundtrip`, `exp072_compute_dashboard`, `exp055_gpu_scaling`). These crates do not enable `deny(clippy::unwrap_used)`.

---

## 6. Error Handling Patterns

**Result**: **Good use of Result, thiserror, and `?`.**

- **thiserror**: Used in `ecoPrimal/src/ipc/error.rs` (`#[derive(Debug, Error)]`).
- **Custom error enums**: `DataError` in `data/mod.rs`, `WfdbError` in `wfdb/mod.rs`, `PushError` in `ipc_push/mod.rs`, `CapabilityError` in `capabilities.rs`.
- **`?` operator**: Used across `data/`, `ipc/`, `wfdb/`, `gpu/`, etc.
- **Result types**: Public APIs return `Result<T, E>` with structured errors.

---

## 7. Zero-Copy Patterns in I/O Parsing

**Result**: **wfdb/ has strong zero-copy support.**

### wfdb/ module

- **`Format212Iter<'a>`** (`parser.rs:231-266`): Iterator over `&'a [u8]`, yields `(i16, i16)` without buffering.
- **`Format16Iter<'a>`** (`parser.rs:272-308`): Iterator over `&'a [u8]`, yields `Vec<i16>` per iteration.
- **`AdcToPhysicalIter<I>`** (`parser.rs:314-344`): Zero-allocation ADC → physical conversion.
- **`decode_format_212` / `decode_format_16`**: Take `&[u8]`, return `Vec<Vec<i16>>` (allocates output; input is slice).
- **`parse_annotations`** (`annotations.rs:53`): Takes `&[u8]`, parses in-place.

### data/ module

- **No binary I/O parsing** — uses JSON (serde), RPC, HTTP. No low-level byte parsing in `data/`.

---

## 8. deny.toml — C-Dependency Bans

**Result**: **14 crates banned** for C/native dependencies.

| Crate | Notes |
|-------|-------|
| `openssl-sys`, `openssl` | Banned |
| `native-tls` | Banned |
| `aws-lc-sys`, `aws-lc-rs` | Banned |
| `ring` | Banned except as wrapper of `rustls`, `rustls-webpki` |
| `libz-sys`, `bzip2-sys`, `curl-sys`, `libsqlite3-sys` | Banned |
| `cmake`, `cc`, `pkg-config`, `vcpkg` | Banned (cc allowed only for ring, blake3) |

Comment in deny.toml: *"Default build has zero C deps. Exception: ring/cc allowed ONLY as transitive deps of ureq (feature-gated behind `nestgate`)."*

---

## 9. Non-Pure-Rust Dependencies (C/C++ FFI)

**Result**: **Default build is pure Rust.** Optional `nestgate` feature pulls in `ureq` → `rustls` → `ring` (allowed via deny.toml wrappers). No `openssl`, `reqwest`, or other heavy FFI in default build.

---

## 10. CI Workflow (.github/workflows/ci.yml)

**Jobs** (V52):

| Job | Steps |
|-----|-------|
| **quality** | `cargo fmt --check`, `cargo clippy --workspace` (pedantic + nursery), `cargo doc`, `cargo-deny`, barraCuda version pin check |
| **test** | `cargo build`, `cargo test`, integration tests, `cargo llvm-cov` (90% line coverage, lib + integration) |
| **validate** | Build release, run all 90 validation binaries, Python cross-validation |
| **composition** | Integration composition tests + Tier 4/5 composition binaries (exp112–exp118) |
| **cross-compile** | musl matrix (x86_64 + aarch64), static PIE check, ecoBin artifact upload |
| **bench** | Compile benchmarks (regression check) |
| **test-gpu** | `barracuda-ops` on every PR; full GPU features on weekly schedule |

**RUSTFLAGS**: `-D warnings`

---

## 11. Module Organization (Single-Responsibility)

**Result**: **Clear module boundaries.**

| Module | Responsibility |
|--------|----------------|
| `pkpd` | Dose-response, compartmental PK, population, NCA, PBPK |
| `microbiome` | Diversity indices, Anderson lattice |
| `biosignal` | ECG, HRV, PPG, EDA, fusion, classification |
| `endocrine` | Testosterone, TRT outcomes |
| `diagnostic` | 4-track patient pipeline |
| `discovery` | MATRIX, HTS, compound, fibrosis |
| `data` | Fetch, provenance, storage, RPC |
| `wfdb` | PhysioNet format parsing |
| `gpu` | WGSL dispatch, context, fused ops |
| `ipc` | Socket, RPC, MCP, dispatch handlers |
| `visualization` | petalTongue schema, IPC push, capabilities |

---

## 12. Dead Code, Unused Imports, Redundant Patterns

**Result**: **No dead code or unused import warnings** from `cargo build -p healthspring-barracuda`.

`#[expect(dead_code)]` used in `experiments/exp004_mab_pk_transfer` for documented constants (`NEMOLIZUMAB_HL_RANGE`, `NEMOLIZUMAB_VD_RANGE`).

---

## 13. Cargo.toml Dependencies — Pure Rust

**Result**: **All direct dependencies are pure Rust** (or optional with pure-Rust alternatives).

| Crate | Dependencies | Notes |
|-------|--------------|------|
| ecoPrimal | serde, serde_json, tracing, thiserror, clap, wgpu, tokio, bytemuck, ureq | ureq (nestgate) uses rustls, not openssl |
| metalForge | healthspring-barracuda, wgpu, pollster | Pure Rust |
| toadstool | healthspring-barracuda, healthspring-forge, tokio | Pure Rust |

**No openssl, ring (direct), or reqwest** in Cargo.toml. Songbird is a discovery service (socket path), not a crate.

---

## Summary

| Check | Status |
|-------|--------|
| Files > 1000 lines | None |
| `#![forbid(unsafe_code)]` | All crates |
| `#[allow()]` in production | None (uses `#[expect]`) |
| `unsafe` blocks | None |
| `unwrap`/`expect` in library production | One, with `#[expect]` |
| Error handling | Result, thiserror, `?` |
| Zero-copy in wfdb | Yes (Format212Iter, Format16Iter, AdcToPhysicalIter) |
| deny.toml C bans | 14 crates |
| Non-pure-Rust deps | None in default build |
| CI checks | fmt, clippy, doc, deny, test, coverage, validate |
| Module organization | Single-responsibility |
| Dead code | None reported |
| Pure Rust deps | Yes |
