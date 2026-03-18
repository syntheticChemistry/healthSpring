# healthSpring V33 Handoff — Protocol Evolution + Centralized Cast Algebra

**Date:** March 16, 2026
**From:** healthSpring V33
**To:** toadStool, biomeOS, all springs (informational)
**License:** AGPL-3.0-or-later (scyBorg Provenance Trio)
**Covers:** V32 → V33

---

## Executive Summary

- **635 tests** (up from 618), zero clippy warnings, zero unsafe, zero `#[allow()]`
- **`IpcError::is_recoverable()`** classifies transient vs permanent IPC failures
- **`DispatchOutcome` enum** separates protocol vs application RPC errors
- **Generic discovery helpers** replace per-primal env-var boilerplate
- **Centralized `cast` module** consolidates numeric casts from 15+ call sites
- All changes absorbed from sibling springs — zero novel patterns

---

## What Changed

### 1. `IpcError` Evolution (`ipc::rpc`)

- Added `is_recoverable()` — classifies `Connect`, `Timeout`, `Write`, `Read`
  as transient; `InvalidJson`, `NoResult`, `RpcError` as permanent.
- Added `is_protocol_error()` — identifies JSON-RPC spec codes (-32700..=-32600).
- Pattern source: neuralSpring S161 `IpcError::is_recoverable()`.
- 4 new tests.

### 2. `DispatchOutcome` Enum (new `ipc::protocol` module)

- `DispatchOutcome::Ok(Value)`, `ProtocolError { code, message }`,
  `ApplicationError { code, message }`.
- `classify_response()` parses JSON-RPC response values into structured outcomes.
- `parse_rpc_response()` for string input with `serde_json::Error` propagation.
- `is_method_not_found()` and `is_protocol_error()` query helpers.
- Pattern source: groundSpring V112 / biomeOS v2.46.
- 6 new tests.

### 3. Generic Discovery Helpers (`ipc::protocol`)

- `socket_from_env(env_var)` — resolve socket path from env var, returns `None`
  if unset or file doesn't exist.
- `discover_primal_socket(env_override, name_prefix)` — generic env-then-scan
  discovery replacing per-primal boilerplate.
- `discover_compute_primal()` now accepts `HEALTHSPRING_COMPUTE_SOCKET` (direct
  socket path) before name-based scan.
- `discover_data_primal()` likewise accepts `HEALTHSPRING_DATA_SOCKET`.
- Songbird discovery evolved to use `socket_from_env()`.
- 2 new tests.

### 4. `cast` Module (new top-level module)

- `usize_f64(n)` — exact for lengths up to 2^53.
- `u64_f64(n)` — exact for values up to 2^53.
- `f64_usize(x)` — truncating, caller ensures non-negative.
- `usize_u32(n)` — saturating at `u32::MAX`.
- Each documents its precision guarantee. `#[expect]` with reasons.
- `biosignal::fft::idx_to_f64`/`u64_to_f64` replaced with re-exports.
- Pattern source: groundSpring V112.
- 5 new tests.

---

## Ecosystem Relevance

### For toadStool

- `DispatchOutcome` can be used to classify RPC responses from springs that
  invoke `compute.dispatch`. Protocol errors → retry; application errors → propagate.
- `socket_from_env()` / `discover_primal_socket()` reduce boilerplate for any
  primal needing multi-primal discovery.

### For biomeOS

- `IpcError::is_recoverable()` enables smarter retry logic in orchestration
  graphs that call healthSpring science methods.
- `DispatchOutcome::is_method_not_found()` supports graceful fallback when a
  requested capability isn't available on a given primal version.

### For all springs

- `cast` module pattern worth absorbing — centralizes numeric precision lints.
- Generic discovery helpers reduce per-spring env-var proliferation.

---

## Files Changed

| File | Change |
|------|--------|
| `ecoPrimal/src/ipc/rpc.rs` | `is_recoverable()`, `is_protocol_error()` on `IpcError`, 4 tests |
| `ecoPrimal/src/ipc/protocol.rs` | **New** — `DispatchOutcome`, `classify_response()`, `socket_from_env()`, `discover_primal_socket()`, 8 tests |
| `ecoPrimal/src/ipc/mod.rs` | Added `pub mod protocol` |
| `ecoPrimal/src/ipc/socket.rs` | `discover_compute_primal()`/`discover_data_primal()` use `socket_from_env()` |
| `ecoPrimal/src/cast.rs` | **New** — `usize_f64`, `u64_f64`, `f64_usize`, `usize_u32`, 5 tests |
| `ecoPrimal/src/lib.rs` | Added `pub mod cast` |
| `ecoPrimal/src/biosignal/fft.rs` | `idx_to_f64`/`u64_to_f64` → re-exports from `cast` |
| `ecoPrimal/src/visualization/capabilities.rs` | Songbird discovery → `socket_from_env()` |

---

## Metrics

| Metric | V32 | V33 |
|--------|-----|-----|
| Tests | 618 | **635** |
| Clippy warnings | 0 | 0 |
| `#[allow()]` | 0 | 0 |
| Unsafe blocks | 0 | 0 |
| New modules | — | `ipc::protocol`, `cast` |
| New public APIs | — | 8 functions, 1 enum |

---

## Patterns Absorbed

| Pattern | Source | healthSpring Module |
|---------|--------|-------------------|
| `is_recoverable()` | neuralSpring S161 | `ipc::rpc::IpcError` |
| `DispatchOutcome` | groundSpring V112 / biomeOS v2.46 | `ipc::protocol` |
| Generic discovery | wetSpring V125 / groundSpring V112 | `ipc::protocol` |
| Safe cast helpers | groundSpring V112 | `cast` |

---

## Superseded Handoffs

- V32 Ecosystem Convergence → `archive/`
- V32 toadStool/barraCuda Absorption → `archive/`
