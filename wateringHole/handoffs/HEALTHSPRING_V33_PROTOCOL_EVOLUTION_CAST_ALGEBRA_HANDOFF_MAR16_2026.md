# healthSpring V33 — Protocol Evolution + Centralized Cast Algebra

**Date:** March 16, 2026
**Version:** V33
**Previous:** V32 (Cross-Spring Absorption + Ecosystem Convergence)
**License:** AGPL-3.0-or-later (scyBorg Provenance Trio)

---

## Summary

V33 evolves the IPC protocol layer with structured error classification and
centralizes numeric cast patterns across the codebase. All changes absorbed
from sibling springs and core primals — zero novel patterns.

---

## Changes

### 1. `IpcError` Evolution

- Added `is_recoverable()` — classifies `Connect`, `Timeout`, `Write`, `Read`
  as transient; `InvalidJson`, `NoResult`, `RpcError` as permanent.
- Added `is_protocol_error()` — identifies JSON-RPC spec codes (-32700..=-32600).
- Pattern source: neuralSpring S161 `IpcError::is_recoverable()`.
- 4 new tests.

### 2. `DispatchOutcome` Enum

- New `ipc::protocol` module with `DispatchOutcome::Ok`, `ProtocolError`,
  `ApplicationError`.
- `classify_response()` parses JSON-RPC responses into structured outcomes.
- `parse_rpc_response()` for string input with `serde_json::Error` propagation.
- `is_method_not_found()` and `is_protocol_error()` helpers.
- Pattern source: groundSpring V112 / biomeOS v2.46.
- 6 new tests.

### 3. Generic Discovery Helpers

- `protocol::socket_from_env(env_var)` — resolve socket path from env var.
- `protocol::discover_primal_socket(env_override, name_prefix)` — generic
  env-then-scan discovery replacing per-primal boilerplate.
- `discover_compute_primal()` now accepts `HEALTHSPRING_COMPUTE_SOCKET` (direct
  socket path) in addition to existing `HEALTHSPRING_COMPUTE_PRIMAL` (name scan).
- `discover_data_primal()` likewise accepts `HEALTHSPRING_DATA_SOCKET`.
- Songbird discovery evolved to use `socket_from_env()`.
- 2 new tests.

### 4. `cast` Module

- Centralized safe numeric cast helpers: `usize_f64`, `u64_f64`, `f64_usize`,
  `usize_u32` (saturating).
- Each function documents precision guarantee and carries `#[expect]` with reason.
- Pattern source: groundSpring V112 `casts` module.
- `biosignal::fft::idx_to_f64`/`u64_to_f64` replaced with re-exports from `cast`.
- 5 new tests.

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

## Next Priorities (P2/P3)

- JSON-RPC proptest fuzz (requires `proptest` dependency)
- NestGate `model.register`/`model.locate` integration
- SourDough `sourdough validate primal/unibin/ecobin` CI
- Squirrel AI bridge for health-domain inference
- Manifest-based primal discovery (toadStool pattern)
