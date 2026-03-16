<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring V28 Deep Debt + Ecosystem Maturity — barraCuda / toadStool Handoff

**Date**: March 16, 2026
**From**: healthSpring V28 (Deep Debt + Ecosystem Maturity)
**To**: barraCuda team, toadStool team, coralReef, biomeOS
**Supersedes**: V27 Deep Evolution Sprint Handoff
**License**: scyBorg (AGPL-3.0-or-later + ORC + CC-BY-SA-4.0)

---

## Executive Summary

V28 evolves healthSpring's debt solutions to production quality. IPC upgraded from fire-and-forget to structured error handling. Socket discovery decoupled from all hardcoded primal names — fully capability-based. Smart refactoring applied to microbiome domain. All 6 WGSL shaders documented with literature provenance for magic numbers. Full Track 6-7 baseline coverage with cross-validation extended to 113 checks across all 7 tracks. 603 tests, zero clippy warnings (pedantic+nursery), zero unsafe.

---

## Part 1: V28 Changes Relevant to Upstream

### 1.1 IPC Evolution — Result-Based Error Handling

`ipc/rpc.rs` now provides two RPC client functions:

| Function | Returns | Use Case |
|----------|---------|----------|
| `try_send(socket, method, params)` | `Result<Value, SendError>` | Critical paths needing error context |
| `send(socket, method, params)` | `Option<Value>` | Fire-and-forget (delegates to `try_send().ok()`) |

`SendError` is a structured enum:
- `Connect(io::Error)` — socket connection failed
- `Write(io::Error)` — request write failed
- `Read(io::Error)` — response read failed
- `InvalidJson(serde_json::Error)` — response not valid JSON
- `NoResult` — response lacks `result` field

**toadStool action**: This pattern should be adopted for toadStool's own IPC client. The structured error enum provides observability without requiring a logging framework dependency. healthSpring's `server.rs` now logs `eprintln!` for registration, capability-register, and heartbeat failures.

### 1.2 Socket Discovery — Fully Capability-Based

Removed all hardcoded primal name fallbacks from `ipc/socket.rs`:
- Deleted `COMPUTE_FALLBACK_NAMES` array (was: `["barraCuda", "toadStool", "coralReef"]`)
- Deleted `DATA_FALLBACK_NAMES` array (was: `["nestGate", "biomeOS"]`)
- `discover_compute_primal()` and `discover_data_primal()` now rely solely on:
  1. Environment variable override (`HEALTHSPRING_COMPUTE_PRIMAL`, `HEALTHSPRING_DATA_PRIMAL`)
  2. Capability-based probing via `discover_by_capability()` which scans the socket directory

Renamed `data/discovery.rs::discover_nestgate_socket()` → `discover_data_provider_socket()` (name-agnostic).

**toadStool action**: All springs should follow this pattern. Primals have self-knowledge only; they discover peers by capability at runtime. No hardcoded primal names in discovery code.

**biomeOS action**: Ensure `discover_by_capability()` response format is documented in wateringHole so all springs probe consistently.

### 1.3 WGSL Shader Magic Number Documentation

All 6 healthSpring WGSL shaders now have inline provenance comments for every constant:

| Shader | Documented Constants |
|--------|---------------------|
| `hill_dose_response_f64.wgsl` | f32 transcendental precision path, coralReef f64 absorption candidacy |
| `population_pk_f64.wgsl` | Wang hash constant (Thomas Wang 2007), CL variation CV (Rowland & Tozer) |
| `diversity_f64.wgsl` | f32 log cast portability note, coralReef DFMA lowering |
| `michaelis_menten_batch_f64.wgsl` | C0 = 6.0 mg/L phenytoin reference (Winter 5th ed), Vmax CV ~20% (CYP2C9, Gerber et al.) |
| `scfa_batch_f64.wgsl` | Km substrate affinity ranges, healthy vs dysbiotic parameterization |
| `beat_classify_batch_f64.wgsl` | `1e-28` correlation guard derivation (near-constant signal stability) |

**barraCuda action**: When absorbing these shaders, preserve the provenance comments. They document the clinical/mathematical rationale for constants that would otherwise appear as unexplained magic numbers.

### 1.4 Smart Module Refactoring

`microbiome/mod.rs` refactored from 680 → 480 LOC. Clinical models extracted to `microbiome/clinical.rs` (203 LOC):
- `fmt_blend()`, `bray_curtis()`
- `antibiotic_perturbation()`
- `ScfaParams`, `SCFA_HEALTHY_PARAMS`, `SCFA_DYSBIOTIC_PARAMS`, `scfa_production()`
- `gut_serotonin_production()`, `tryptophan_availability()`

Tests remain in `mod.rs` via `pub use clinical::*` re-export — no API change.

### 1.5 Tolerance Centralization

exp020 (Pan-Tompkins QRS) inline thresholds replaced with named constants:
- `tolerances::QRS_PEAK_MATCH_MS` (75.0 ms — ANSI/AAMI EC57:2012)
- `tolerances::QRS_SENSITIVITY` (0.8 — ANSI/AAMI EC57:2012)
- `tolerances::HR_DETECTION_BPM` (10.0 BPM — clinical diagnostic threshold)
- `tolerances::SDNN_UPPER_MS` (200.0 ms — ESC/NASPE HRV Task Force 1996)

**barraCuda action**: If absorbing biosignal primitives, these clinical thresholds should travel with the code as documented constants, not be reimplemented.

### 1.6 Track 6-7 Baseline Coverage

- 12 baseline JSON files generated and registered in `update_provenance.py`
- 5 Python scripts fixed (`from datetime import datetime, timezone` → was `NameError`)
- `cross_validate.py` extended from Tracks 1-5 (24 experiments, 104 checks) to all 7 tracks (113 checks)
- New validation sections: MATRIX_Scoring (Exp090), ADDRC_HTS (Exp091), Canine_IL31 (Exp100), Cross_Species_PK (Exp104)

---

## Part 2: Patterns Evolved in healthSpring (Upstream Candidates)

### 2.1 Structured IPC Errors Without Logging Dependencies

healthSpring's `SendError` enum + `eprintln!` pattern provides production observability without requiring `tracing` or `log` crate dependencies. This keeps the binary lean and ecoBin-compliant.

### 2.2 `#[expect()]` with `reason` (Rust 2024)

All lint exceptions use `#[expect(lint, reason = "...")]` instead of `#[allow()]`. This is the Rust 2024 idiom: if the lint is no longer triggered, `#[expect]` emits a warning (unlike `#[allow]` which silently passes). Examples:

- `#[expect(clippy::too_many_lines, reason = "dispatch match over GpuOp variants")]`
- `#[expect(clippy::expect_used, reason = "Serialize impls on fixed structs cannot fail")]`

### 2.3 Infallible Serialization Documentation

`scenario_with_edges_json()` returns `String` (not `Result`) because `serde_json::to_string_pretty` is infallible for `Serialize` types with no dynamic content. The `#[expect(clippy::expect_used)]` is documented with a `# Panics` section explaining why it cannot actually panic.

### 2.4 Binary Module Visibility

Binary-private modules should use `pub` not `pub(crate)` — clippy nursery `redundant_pub_crate` catches this. The binary's own module privacy already restricts visibility.

---

## Part 3: barraCuda Absorption Priorities (from healthSpring V28)

### P0 — Ready for Immediate Absorption

| Primitive | File | GPU Path | Why Priority |
|-----------|------|----------|-------------|
| `hill_dose_response` | `pkpd/dose_response.rs` | `hill_dose_response_f64.wgsl` (**LIVE**) | Core pharmacology, used across 4 tracks |
| `population_pk_cpu` | `pkpd/population.rs` | `population_pk_f64.wgsl` (**LIVE**) | MC population modeling, GPU crossover at 5M |
| `shannon_index` + `simpson_index` | `microbiome/mod.rs` | `diversity_f64.wgsl` (**LIVE**) | Core ecology, workgroup reduction |
| `MichaelisMentenOde` | `gpu/ode_systems.rs` | `OdeSystem` → `generate_shader()` | First health-domain ODE, phenytoin model |
| `OralOneCompartmentOde` | `gpu/ode_systems.rs` | `OdeSystem` → `generate_shader()` | Most common PK model |
| `TwoCompartmentOde` | `gpu/ode_systems.rs` | `OdeSystem` → `generate_shader()` | Biexponential PK distribution |

### P1 — Health Module Candidates

| Primitive | File | Notes |
|-----------|------|-------|
| `foce_estimate` | `pkpd/nlme.rs` | Per-subject gradient is independent → batch parallel GPU |
| `saem_estimate` | `pkpd/nlme.rs` | E-step sampling → embarrassingly parallel MC |
| `vpc_simulate` | `pkpd/diagnostics.rs` | 50+ independent simulations → embarrassingly parallel |
| `monte_carlo_propagate` | `uncertainty.rs` | General perturb→model→summarize pattern |
| `compound_ic50_sweep` | `discovery/compound.rs` | 8K × N Hill evaluations → reuses `hill_dose_response_f64.wgsl` |
| `matrix_score` | `discovery/matrix_score.rs` | Fajgenbaum MATRIX + Anderson geometry |

### Tier B — WGSL Shaders Ready for Absorption

| Shader | Pattern | Source Domain |
|--------|---------|--------------|
| `michaelis_menten_batch_f64.wgsl` | Per-patient ODE (Euler + Wang hash PRNG) | PK/PD |
| `scfa_batch_f64.wgsl` | Element-wise 3-output MM kinetics | Microbiome |
| `beat_classify_batch_f64.wgsl` | Per-beat cross-correlation template matching | Biosignal |

---

## Part 4: toadStool Evolution Targets

### 4.1 IPC Client Upgrade

Adopt `try_send()` / `SendError` pattern from healthSpring for toadStool's own primal communication. The fire-and-forget `send()` wrapper preserves backward compatibility.

### 4.2 Capability-Based Discovery Standard

healthSpring's `discover_by_capability()` implementation should be standardized across all springs. The pattern:
1. Check environment variable override first
2. Scan socket directory for active primals
3. Probe each with `capability.list`
4. Match requested capability pattern (e.g., `compute.*`, `data.*`)
5. Cache result for session

### 4.3 ODE Dispatch Path

Three `OdeSystem` implementations are codegen-ready (`generate_shader()` produces valid WGSL). toadStool needs a `StageOp::OdeBatch` variant to wire these through the dispatch pipeline.

### 4.4 Streaming Dispatch for V16 Ops

MM batch, SCFA batch, and beat classify shaders are validated but not yet wired through toadStool streaming. These need `StageOp` variants and `execute_gpu()` paths.

---

## Part 5: Learnings for coralReef

1. **f64 transcendental precision**: healthSpring casts through f32 for `pow(f64, f64)` because NVVM doesn't support it natively. coralReef's sovereign compilation should generate native f64 transcendentals where hardware supports it.
2. **DFMA lowering**: `diversity_f64.wgsl` notes that fused multiply-add for f64 logarithms would improve precision. coralReef should prioritize DFMA lowering for ecology/statistics workloads.
3. **Wang hash PRNG**: `population_pk_f64.wgsl` uses u32 Wang hash because u64 PRNG isn't portable across GPU vendors. coralReef should document which PRNG patterns are vendor-safe.

---

## Part 6: Metrics

| Metric | V27 | V28 |
|--------|-----|-----|
| Tests | 601 | 603 |
| Experiments | 73 | 73 |
| Python baselines with provenance | ~35 | 42 |
| Cross-validation checks | 104 (Tracks 1-5) | 113 (all 7 tracks) |
| Hardcoded primal names in discovery | 5+ | 0 |
| IPC error visibility | None (Option) | Structured (Result<T, SendError>) |
| WGSL shaders with provenance docs | 0 | 6/6 |
| Centralized tolerances | ~15 | ~20 (4 new clinical thresholds) |
| Clippy warnings | 0 | 0 |
| Unsafe blocks | 0 | 0 |
| `#[allow()]` in production | 0 | 0 |

---

## Part 7: Next Evolution Targets

1. **GPU dispatch for ODE systems**: Wire `generate_shader()` → `wgpu::ComputePipeline` for batched PK parameter sweeps
2. **Tier B shader absorption**: MM PK, SCFA, beat classify → barraCuda canonical ops
3. **HMM absorption from neuralSpring**: Hidden Markov Models for biosignal regime detection
4. **ESN classifier from neuralSpring**: Echo State Network for attention state prediction
5. **Tissue Anderson from groundSpring**: 3D Anderson lattice for tissue heterogeneity modeling
6. **TensorSession integration**: Fused multi-op GPU pipelines via barraCuda's session API
7. **DD-006 iPSC validation**: Gonzales iPSC skin model readout validation
8. **DD-007 Ellsworth med chem**: Medicinal chemistry lead optimization pipeline

---

## Appendix: barraCuda Consumption Map (healthSpring V28)

| barraCuda Module | Primitives Used | healthSpring Domain |
|-----------------|-----------------|---------------------|
| `stats` | `shannon_from_frequencies`, `simpson`, `chao1`, `bray_curtis`, `hill` | Microbiome, PK/PD |
| `special` | `anderson_diagonalize` | Microbiome Anderson lattice |
| `ops` | `HillFunctionF64`, `PopulationPkF64`, `PopulationPkConfig`, `bio::diversity_fusion::DiversityFusionGpu` | GPU dispatch |
| `numerical` | `OdeSystem`, `BatchedOdeRK4` | ODE→WGSL codegen |
| `rng` | `LCG_MULTIPLIER`, `lcg_step`, `state_to_f64`, `uniform_f64_sequence` | Deterministic PRNG |
| `device` | `WgpuDevice` | GPU context |
