# healthSpring V13 — Deep Audit Evolution Handoff

**Date**: March 9, 2026
**From**: healthSpring
**To**: barraCuda, toadStool, petalTongue, metalForge
**License**: AGPL-3.0-only
**Covers**: V12→V13 — deep code audit, Anderson eigensolver, smart refactoring, math deduplication, centralized RNG, capability-based discovery, flaky test elimination, doc-tests

---

## Executive Summary

healthSpring V13 is a code quality and correctness evolution. No new experiments — instead, a comprehensive audit identified and resolved structural debt, a correctness bug in the Anderson/IPR pipeline, code duplication, hardcoded constants, and test fragility. All 47 experiments continue to pass. Test count increased from 313 → 317 (4 doc-tests added). All files are under the 1000-line wateringHole limit. Zero clippy warnings, zero fmt diffs, zero doc warnings, zero unsafe code.

**Metrics**: 317 tests, 47 experiments, 630 binary checks, 104 cross-validation checks, all green.

---

## Part 1: What Changed (V12→V13)

### 1.1 Anderson Eigensolver — Correctness Fix (P0)

**Bug**: `diagnostic.rs` and `scenarios/microbiome.rs` computed Inverse Participation Ratio (IPR) using the Hamiltonian's diagonal elements instead of actual eigenvectors. The Hamiltonian diagonal is the disorder potential, not eigenstates. IPR requires true eigenvectors of the disordered lattice.

**Fix**: Implemented a tridiagonal QL algorithm (`anderson_diagonalize` in `microbiome.rs`) that correctly computes eigenvalues and eigenvectors of the symmetric tridiagonal Anderson Hamiltonian. Both `diagnostic.rs` and `scenarios/microbiome.rs` now call `anderson_diagonalize()` and use the returned eigenvectors for IPR computation.

**Impact**: IPR values are now physically meaningful — they measure localization of true eigenstates, not just the disorder landscape. This is foundational for any downstream physics (localization length, mobility edge analysis).

**Absorption target**: barraCuda should consider absorbing the tridiagonal QL eigensolver into its linear algebra module. This is the same algorithm used in LAPACK's `dsteqr` and is needed wherever Anderson lattices are diagonalized — wetSpring Track 4 (soil), Paper 01 (Anderson-QS), Paper 12 (immunological Anderson).

### 1.2 Smart clinical.rs Refactor

**Problem**: `barracuda/src/visualization/clinical.rs` was 1177 lines, exceeding the 1000-line wateringHole limit.

**Fix**: Extracted 8 node-building functions (`assessment_node`, `protocol_node`, `metabolic_node`, `cardiovascular_node`, `glycemic_node`, `cardiac_node`, `gut_health_node`, `population_node`) into `clinical_nodes.rs` (819 lines). The original `clinical.rs` retains `PatientTrtProfile`, `TrtProtocol`, the scenario scaffold, public API, and tests (374 lines). Both files are domain-coherent — clinical_nodes builds nodes, clinical.rs orchestrates them.

**Absorption target**: When barraCuda absorbs `PatientTrtProfile` and clinical scenario builders, this same domain split should be preserved.

### 1.3 LCG PRNG Centralization

**Problem**: The Linear Congruential Generator multiplier `6_364_136_223_846_793_005_u64` was hardcoded in 4 separate files (`diagnostic.rs`, `gpu/mod.rs`, `biosignal.rs`, `toadstool/stage.rs`).

**Fix**: New `barracuda/src/rng.rs` module (37 lines):
- `LCG_MULTIPLIER: u64` — the canonical constant
- `lcg_step(state: u64) -> u64` — deterministic state transition
- `state_to_f64(state: u64) -> f64` — uniform [0, 1) mapping

All usage sites updated to import from `rng.rs`.

**Absorption target**: barraCuda should absorb `rng.rs` as a core deterministic PRNG. Springs should import this rather than defining their own LCG constants. The u32 GPU PRNG (xorshift32 + Wang hash, documented in V12 learnings) remains separate — GPU and CPU use different generators due to u64 portability constraints in WGSL.

### 1.4 Math Deduplication

**Problem**: Two functions were duplicated between modules:
- `evenness_to_disorder`: identical implementations in `endocrine.rs` and `microbiome.rs`
- `lognormal_params`: duplicate of `LognormalParam::to_normal_params()` in `pkpd/`

**Fix**: `endocrine.rs` now delegates to the canonical implementations:
```rust
pub fn evenness_to_disorder(pielou_j: f64, scale: f64) -> f64 {
    crate::microbiome::evenness_to_disorder(pielou_j, scale)
}
pub fn lognormal_params(typical: f64, cv: f64) -> (f64, f64) {
    crate::pkpd::LognormalParam { typical, cv }.to_normal_params()
}
```

**Absorption target**: When barraCuda absorbs healthSpring modules, ensure single-source-of-truth for shared math. The delegation pattern works well — public API stays stable, implementation is centralized.

### 1.5 Capability-Based Songbird Discovery

**Problem**: `barracuda/src/visualization/capabilities.rs` had a hardcoded fallback path `/tmp/songbird.sock` for the Songbird discovery service.

**Fix**: Replaced with a glob-based search (`songbird*.sock`) in `/tmp`, matching the ecosystem principle that primals discover each other at runtime via capabilities, not hardcoded paths.

**Learning for ecosystem**: All primals should discover Songbird via glob search, not hardcoded paths. The socket name includes the process ID or instance identifier — `songbird.sock`, `songbird_12345.sock`, etc. — and glob matching handles all cases.

### 1.6 Flaky IPC Test Fix

**Problem**: Tests in `visualization::ipc_push::tests` failed intermittently during parallel `cargo test --workspace` due to Unix socket path collisions and race conditions between client connect and server accept.

**Fix**:
1. `AtomicU64` counter generates unique socket paths per test (PID + sequence number)
2. Refactored test harness: removed `Barrier` synchronization, new `run_socket_test` helper spawns mock server (which binds and waits) before client connects. OS kernel queues the connection until `accept()`.

**Learning for ecosystem**: Any primal using Unix socket IPC tests should use atomic counters for path uniqueness and let the kernel handle connection queuing rather than synchronization barriers.

### 1.7 Doc-Tests

Added 4 doc-tests to key public APIs:
- `microbiome::shannon_index` — validates ln(N) for uniform distribution
- `pkpd::hill_dose_response` — validates midpoint (dose = EC50 → E = 0.5)
- `pkpd::auc_trapezoidal` — validates triangle AUC
- `rng::state_to_f64` — validates [0, 1) range

### 1.8 Tolerance Registry

Added `exp067` (GPU parity extended) and `exp069` (toadStool dispatch matrix) to `specs/TOLERANCE_REGISTRY.md` under a new CPU Parity Class at `1e-10`.

---

## Part 2: barraCuda Absorption Table (V13 Additions)

| Module | What to absorb | Source | Priority |
|--------|---------------|--------|:--------:|
| `linalg` | Tridiagonal QL eigensolver (`anderson_diagonalize`) | `barracuda/src/microbiome.rs` | **P0** |
| `rng` | Deterministic LCG PRNG (`lcg_step`, `state_to_f64`) | `barracuda/src/rng.rs` | **P0** |
| `visualization::clinical` | Node builder domain split pattern | `clinical.rs` + `clinical_nodes.rs` | P1 |

### Previously identified (V12, still active)

| Module | What to absorb | Source | Priority |
|--------|---------------|--------|:--------:|
| `visualization::ipc_push` | `push_replace()`, `push_render_with_config()`, `query_capabilities()`, `subscribe_interactions()` | `barracuda/src/visualization/ipc_push.rs` | P0 |
| `visualization::stream` | `push_replace_binding()`, `push_render_with_domain()` | `barracuda/src/visualization/stream.rs` | P0 |
| `visualization::clinical` | `PatientTrtProfile`, `TrtProtocol`, `trt_clinical_scenario()` | `barracuda/src/visualization/clinical.rs` | P1 |
| `pkpd::pbpk` | `pbpk_iv_tissue_profiles()`, `PbpkTissueProfiles` | `barracuda/src/pkpd/pbpk.rs` | P1 |

---

## Part 3: toadStool Absorption Table (V13 Additions)

| Module | What to absorb | Source | Priority |
|--------|---------------|--------|:--------:|
| `rng` | Import `healthspring_barracuda::rng::lcg_step` (already done locally, needs upstream) | `toadstool/src/stage.rs` | P0 |
| `pipeline` | `execute_streaming()` callback pattern for per-stage progress | `toadstool/src/pipeline.rs` | P0 |
| `pipeline` | Wire `replace` stream op for non-TimeSeries stage outputs | healthSpring pattern | P1 |

---

## Part 4: Learnings for the Ecosystem

### 4.1 Eigensolver Architecture

The Anderson Hamiltonian is a symmetric tridiagonal matrix. The QL algorithm with implicit shifts is the correct O(n²) method for diagonalizing it. Key implementation details:

- Wilkinson shift for convergence acceleration
- Givens rotations applied to both eigenvalue and eigenvector matrices
- Convergence criterion: `|off_diagonal[l]| <= epsilon * (|diagonal[l]| + |diagonal[l+1]|)`
- The `#[expect(clippy::float_cmp)]` on the convergence test is intentional — standard numerical analysis practice

barraCuda should have a general-purpose tridiagonal eigensolver in its linear algebra module. wetSpring's Lanczos eigensolver reduces large matrices to tridiagonal form; the QL step is the final diagonalization. These compose: Lanczos → tridiagonal → QL → eigenvalues/eigenvectors.

### 4.2 File Size Refactoring Strategy

When splitting a large file, extract by **domain responsibility**, not by arbitrary line count. `clinical.rs` (1177 lines) had two natural domains: (1) the orchestration layer (`PatientTrtProfile`, scenario assembly, public API) and (2) the individual node builders (8 functions building specific parts of the clinical graph). Extracting the node builders left both files focused and testable.

Anti-pattern: splitting alphabetically, splitting at the midpoint, or extracting private helpers that have no semantic boundary.

### 4.3 Deduplication via Delegation

When the same math appears in multiple modules (e.g., `evenness_to_disorder` in both `endocrine` and `microbiome`), the best approach is to delegate — keep the public API in both modules (for discoverability) but have one call the other. This preserves API stability while eliminating implementation duplication.

### 4.4 CPU Benchmark Summary

Python CPU benchmarks exist (`control/scripts/bench_barracuda_cpu_vs_python.py`) and show Rust is consistently faster across all domains (Hill, PK, diversity, AUC, population MC). Criterion benchmarks exist in `barracuda/benches/cpu_parity.rs`. GPU benchmarks lack industry-standard comparison targets — the CI GPU job is disabled. Kokkos/LAMMPS comparison (identified in hotSpring handoff) would be the right benchmark for GPU parity validation.

### 4.5 All Mocks Are Test-Only

Confirmed: all mock objects (`MockPetalTongue`, mock socket servers, test fixtures) are scoped to `#[cfg(test)]` modules. No mocks in production code paths.

---

## Part 5: Evolution Status

### healthSpring barraCuda Primitives — Absorption Readiness

| Primitive | Module | Tier | Absorption Status |
|-----------|--------|:----:|-------------------|
| Hill dose-response | `pkpd/dose_response.rs` | A | Ready — element-wise, WGSL validated |
| One/two-compartment PK | `pkpd/mod.rs` | A | Ready — analytical, no branching |
| Population PK Monte Carlo | `pkpd/mod.rs` | A | Ready — embarrassingly parallel |
| PBPK 5-tissue | `pkpd/pbpk.rs` | B | ODE integration needs adaptation |
| Shannon/Simpson diversity | `microbiome.rs` | A | Ready — workgroup reduction validated |
| Anderson diagonalize | `microbiome.rs` | B | QL eigensolver needs GPU adaptation |
| Pan-Tompkins QRS | `biosignal.rs` | B | Streaming pipeline, needs stage mapping |
| HRV metrics | `biosignal.rs` | A | Ready — time-domain, element-wise |
| PPG SpO₂ | `biosignal.rs` | A | Ready — Beer-Lambert, element-wise |
| Testosterone PK | `endocrine.rs` | A | Ready — reuses compartmental PK |
| LCG PRNG | `rng.rs` | — | CPU-only (GPU uses u32 xorshift) |
| Tridiagonal QL | `microbiome.rs` | B | O(n²), needs GPU parallel adaptation |

### GPU Pipeline — Current State

3 WGSL shaders validated (Hill, PopPK, Diversity). Fused pipeline eliminates 30x dispatch overhead. Scaling validated to 10M elements. CPU vs GPU parity: 27/27. Mixed dispatch: 22/22. PCIe P2P: 26/26.

**Next GPU targets**: Anderson eigensolver shader (Tier B → A promotion), biosignal FFT shader (real-time ECG/PPG), PBPK ODE stepper.

### petalTongue — Current State

Full stream ops (append, set_value, replace). Domain theming. UiConfig passthrough. Capabilities query. Interaction subscription. 5 patient archetypes. 13 scenarios. Live streaming dashboard.

**Next**: Server-side `visualization.capabilities` and `visualization.interact.subscribe` implementation. Bidirectional clinical drill-down.

---

## Part 6: Open Questions for Ecosystem

1. **GPU eigensolver**: Should barraCuda absorb a general tridiagonal eigensolver, or should Anderson diagonalization remain spring-specific? The QL algorithm is O(n²) and parallelizes differently from element-wise ops. Lanczos + QL is the standard approach for large sparse systems.

2. **GPU benchmark baseline**: healthSpring has Python CPU benchmarks. For GPU parity validation against industry tools, Kokkos/LAMMPS (identified in hotSpring) is the right target. Should healthSpring add Kokkos comparison, or is this a hotSpring/barraCuda concern?

3. **NPU biosignal**: Pan-Tompkins is a streaming signal pipeline ideal for Akida AKD1000. wetSpring has a working NPU driver. Should healthSpring wire biosignal → NPU dispatch, or wait for toadStool to absorb the NPU streaming pattern?

---

## Action Items

1. **barraCuda team**: Absorb tridiagonal QL eigensolver and LCG PRNG. Consider general-purpose `linalg::tridiagonal_ql()` API.
2. **toadStool team**: Absorb `lcg_step` from `rng.rs` (currently local import). Wire `replace` stream op for non-TimeSeries stage outputs.
3. **petalTongue team**: Implement server-side `visualization.capabilities` and `visualization.interact.subscribe`.
4. **All teams**: Use glob-based Songbird discovery, not hardcoded socket paths.
5. **CI**: Re-enable GPU CI job when GPU runner is available. Add Kokkos benchmark comparison target.

---

**License**: AGPL-3.0-only
