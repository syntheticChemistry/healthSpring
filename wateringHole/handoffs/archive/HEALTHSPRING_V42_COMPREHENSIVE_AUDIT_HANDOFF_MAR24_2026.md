<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring V42 — Comprehensive Audit & Deep Debt Resolution

**Date**: March 24, 2026
**Supersedes**: V42 Deep Debt Resolution (Mar 23)
**Status**: Active (< 48h)

---

## Summary

Full-stack audit against wateringHole ecosystem standards followed by
systematic resolution of all P0 and P1 findings. 13 distinct remediation
actions executed across code quality, provenance, refactoring, testing,
IPC, and CI infrastructure.

---

## Changes Made

### P0 — Code Quality

1. **Eliminated `#[expect(clippy::expect_used)]` from production library code.**
   - `visualization/scenarios/mod.rs`: Replaced infallible `serde_json` serialization
     with `try_scenario_with_edges_json` (fallible) + `scenario_with_edges_json` (match-based
     graceful degradation). Zero-panic path.
   - `visualization/scenarios/biosignal.rs`: Replaced `.expect()` on WFDB `decode_format_212`
     with `match` + early return on error with `tracing::warn!`.

2. **Evolved hardcoded `/tmp/biomeos` fallback to capability-aware with warning.**
   - `ipc/socket.rs`: `resolve_socket_dir()` now logs `tracing::warn!` when falling
     back to `/tmp/biomeos`, directing operators to set `BIOMEOS_SOCKET_DIR`.

3. **Populated provenance registry with verified check counts and baseline sources.**
   - `provenance/registry.rs`: All 54 entries updated — experiments now carry actual
     Rust `ValidationHarness` check counts; `baseline_source` filled with academic
     citations (DOIs, textbook references) or analytical derivation notes.

### P1 — Smart Refactoring

4. **metalForge/forge smart split (524 → 4 files).**
   - `types.rs`: `Substrate`, `Workload`, `PrecisionRouting`, `GpuInfo`, `NpuInfo`, `DispatchThresholds`
   - `discovery.rs`: `Capabilities` + `discover()` + `probe_gpu()/probe_npu()`
   - `routing.rs`: `select_substrate*` + all routing tests
   - `lib.rs`: 51 LOC — module declarations and public re-exports (API unchanged)

5. **GPU dispatch match arm extraction (`gpu/mod.rs`, `gpu/dispatch/mod.rs`).**
   - `execute_cpu`: 6 named helpers (`execute_cpu_hill_sweep`, etc.) — `#[expect(too_many_lines)]` removed
   - `execute_gpu`: split into `try_barracuda_tier_a`, `request_wgpu_device`, `execute_gpu_local_wgsl`

6. **Centralized stochastic seed strategy in `rng.rs`.**
   - Module-level documentation now describes the seed strategy, GPU parity notes
     (Wang hash u32 vs LCG u64), and cross-spring provenance.

### P1 — Testing

7. **Added 3 bitwise determinism tests in `rng.rs`.**
   - `lcg_sequence_bitwise_determinism`: 1000-step LCG chain identity
   - `normal_chain_bitwise_determinism`: 500 Box-Muller samples identity
   - `population_pk_determinism`: 200 seeded AUC values identity

8. **Added WFDB round-trip integration test (`tests/integration_wfdb.rs`).**
   - Format 212 encode → decode → re-encode binary identity
   - Format 16 encode → decode → re-encode binary identity
   - ADC → physical → ADC recovery
   - Header parse determinism
   - Annotation parse + beat type code coverage

### P2 — Infrastructure

9. **Added `rustfmt.toml`** — pins `edition = "2024"` to prevent formatting drift.

10. **Added barraCuda version assertion to CI.**
    - Quality job verifies `c04d848` pin comment in `ecoPrimal/Cargo.toml`.

11. **Extended biomeOS lifecycle IPC client** with graph and niche operations:
    - `graph_deploy`, `graph_status`, `graph_teardown` for pipeline coordination
    - `niche_list`, `primal_list` for ecosystem introspection
    - All with tests verifying graceful failure without orchestrator

12. **Updated `data/manifest.toml`** — added checksum computation instructions,
    updated `last_updated` to Mar 24.

### V42 Active Handoff

13. **Created this handoff document** in `wateringHole/handoffs/` (not archive).

---

## Current State (Post-Audit)

| Metric | Value |
|--------|-------|
| Workspace members | 86 (ecoPrimal + forge + toadstool + 83 experiments) |
| `.rs` files | 244 (~63K LOC) |
| WGSL shaders | 6 (all GPU LIVE) |
| Validation checks | 83/83 `ValidationHarness` experiments |
| Provenance records | 54 (all with check counts + baseline sources) |
| Named tolerances | 87+ (centralized in `tolerances.rs`) |
| `#[allow()]` in production | 0 |
| `unsafe` blocks | 0 (`#![forbid(unsafe_code)]` workspace-wide) |
| `TODO`/`FIXME` | 0 |
| Max file LOC | 732 (`scenarios/tests.rs`) |
| CI coverage gate | 90%+ line (llvm-cov) |
| barraCuda version | v0.3.7 (rev `c04d848`) |

---

## Remaining Items (Not Blocking)

- **GPU CI**: `test-gpu` job still `if: false` — blocked on GPU runners
- **Data checksums**: `sha256` fields in `data/manifest.toml` awaiting first download
- **2 planned datasets**: `hmp_16s`, `geo_androgen_receptor` not yet fetched
- **WGSL absorption**: 6 shaders ready for upstream barraCuda absorption (P0: Hill, PopPK, Diversity)
- **coralReef sovereign dispatch**: Feature-gated, not yet exercised in experiments
- **TensorSession fused pipeline**: Awaiting barraCuda `TensorSession` API

---

## Absorption Requests for barraCuda Team

### P0 — Immediate

1. **`hill_dose_response`** WGSL shader + element-wise primitive
2. **`population_pk_cpu`** WGSL shader + Monte Carlo ODE primitive
3. **`shannon_index` + `simpson_index`** WGSL shader + workgroup reduction

### P1 — Health-Specific

4. `pbpk_iv_simulate` + tissue profiles → multi-compartment PBPK ODE
5. `auc_trapezoidal` → parallel prefix candidate
6. `foce_estimate` → per-subject gradient batch parallel

### P2 — Signal Processing

7. `pan_tompkins_qrs` → streaming detection pipeline (NPU path)
8. `fuse_channels` → multi-modal biosignal fusion

---

## Cross-Spring Notes

- **neuralSpring**: `microbiome_transfer` module provides Anderson gut params
  for neuralSpring's ESN models via IPC (no compile-time coupling)
- **wetSpring**: `ValidationSink` trait absorbed from wetSpring V132
- **hotSpring**: `ValidationHarness` pattern follows hotSpring canonical form
- **toadStool**: Local `toadstool/` crate mirrors pipeline model;
  absorption into upstream toadStool is a future target
