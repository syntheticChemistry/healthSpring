# healthSpring V31 — Deep Debt Solutions + Modern Idiomatic Rust Evolution

**Date:** March 16, 2026
**From:** healthSpring V31
**To:** barraCuda, toadStool, ecosystem

---

## Summary

V31 evolves healthSpring to modern idiomatic Rust patterns: `OrExit<T>` trait (wetSpring V123), `IpcError` ecosystem-standard error type, enriched `capability.list` for Pathway Learner, magic number elimination, complete `#![forbid(unsafe_code)]`, capability-based data-provider discovery, and non-async Tier A GPU ops.

## Changes

| Change | Detail |
|--------|--------|
| `OrExit<T>` trait | Centralized zero-panic pattern in `validation::OrExit` — `Result<T,E>` and `Option<T>` impls. Dump binaries migrated. |
| `IpcError` | `SendError` evolved to `IpcError` with `RpcError{code,message}` + `Timeout`. Backward-compatible alias. |
| Enriched `capability.list` | `operation_dependencies` DAG + `cost_estimates` (CPU ms, GPU eligible) for Pathway Learner. |
| Magic number cleanup | `0.693` → `LN_2`, float `assert_eq!` → `abs() < EPSILON`, `.mul_add()` for FMA. |
| `#![forbid(unsafe_code)]` | Complete — all 73 binary crate roots now forbid unsafe. |
| Capability-based discovery | `neural-api` hardcode → `DATA_PROVIDER_SOCK_PREFIX` env with fallback. |
| Non-async GPU ops | Tier A rewire functions stripped of unused `async`. |
| barraCuda API alignment | `PopulationPkF64::simulate()` updated for upstream `u32` params. |

## Metrics

- 616 tests (up from 611)
- 0 clippy warnings (pedantic + nursery)
- 0 unsafe blocks
- 0 `#[allow()]` in production
- 0 `#[expect()]` without reason
- All files under 764 LOC

## barraCuda Absorption

- `PopulationPkF64::simulate()` now uses `u32` params — healthSpring aligned.
- Tier A ops (Hill, PopPK, Diversity) are now synchronous — `async` removed.
- `barracuda::health::*` delegations from V30 stable.

## toadStool Absorption

- `compute.dispatch.*` typed client stable from V30.
- `operation_dependencies` and `cost_estimates` in `capability.list` enable toadStool Pathway Learner to plan execution graphs.

## Ecosystem Patterns Absorbed

| Pattern | Source | Location |
|---------|--------|----------|
| `OrExit<T>` | wetSpring V123 | `validation::OrExit` |
| `IpcError` | biomeOS/airSpring/groundSpring | `ipc::rpc::IpcError` |
| Enriched capability.list | biomeOS Pathway Learner spec | `capabilities::handle_capability_list` |
| Float comparison idioms | Rust 2024 clippy nursery | All test files |
