<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V43 — Primal & Spring Evolution Handoff

**Date**: March 24, 2026
**From**: healthSpring V43 (888 tests, 83 experiments, 54 Python baselines)
**To**: All primal teams + sibling spring teams
**Scope**: Cross-spring absorption insights, patterns for upstream evolution, requests

---

## Summary

V43 absorbed patterns from groundSpring (cast module), neuralSpring (self-knowledge
compliance), and wetSpring (7-primal discovery, validation harness submodules).
This handoff documents what we learned, what we built, and what we need from upstream.

---

## 1. For barraCuda Team

### Cast Module — Request for Upstream Absorption

healthSpring and groundSpring independently arrived at the same solution: centralized
named numeric cast functions to eliminate scattered `#[expect(clippy::cast_*)]`
annotations. healthSpring's `cast.rs` now provides 14 named conversions:

| Function | Conversion | Notes |
|----------|-----------|-------|
| `usize_f64` | `usize` → `f64` | Exact for lengths up to 2^53 |
| `usize_u32` | `usize` → `u32` | Saturating at `u32::MAX` |
| `usize_u64` | `usize` → `u64` | Lossless widening |
| `u64_f64` | `u64` → `f64` | Exact for values up to 2^53 |
| `u64_u32_truncate` | `u64` → `u32` | Intentional low-32-bit truncation (PRNG seeds) |
| `u32_f64` | `u32` → `f64` | Lossless |
| `u32_usize` | `u32` → `usize` | Lossless on 32/64-bit |
| `i32_f64` | `i32` → `f64` | Lossless |
| `i16_f64` | `i16` → `f64` | Lossless (WFDB sample data) |
| `f64_usize` | `f64` → `usize` | Truncation, caller-ensured non-negative |
| `f64_u32` | `f64` → `u32` | Truncation (GPU workgroup sizes) |
| `f64_u64` | `f64` → `u64` | Truncation |

**Request**: Absorb a `barraCuda::cast` module with at minimum: `usize_f64`, `usize_u32`,
`u64_u32_truncate`, `f64_usize`, `u32_f64`. This prevents every spring from reinventing
the same cast infrastructure. groundSpring V122 independently recommends the same.

### Upstream Contract Tolerances

healthSpring now has an `upstream_contract` tolerance section in `tolerances.rs`
documenting cross-spring agreed values:

| Constant | Value | Agreement |
|----------|-------|-----------|
| `UPSTREAM_GPU_HILL_PARITY` | `1e-4` | f32 transcendental precision |
| `UPSTREAM_GPU_DIVERSITY_PARITY` | `1e-4` | Workgroup reduction |
| `UPSTREAM_GPU_FUSED_PARITY` | `1e-4` | Fused vs sequential dispatch |
| `UPSTREAM_PRNG_DETERMINISM` | `0.0` | LCG seed identity |
| `UPSTREAM_DIVERSITY_CROSS_VALIDATE` | `1e-8` | Shannon/Simpson vs Python |

**Request**: Document these as barraCuda's contract guarantees so all springs can
reference the same tolerance values.

### Existing Absorption Requests (from V42)

The V42 absorption handoff remains active. Key items:

- **P0**: 6 WGSL shaders ready for absorption (Hill, PopPK, Diversity, MM, SCFA, Beat)
- **P1**: ODE codegen registry (3 `OdeSystem` impls)
- **P2**: TensorSession API for fused multi-op pipelines

---

## 2. For toadStool Team

### Self-Knowledge Compliance

healthSpring V43 neutralized all cross-primal name references in production error
strings. MCP tool descriptions now use capability-based language:

| Before | After |
|--------|-------|
| "Offload compute to Node Atomic (toadStool) GPU" | "Offload compute to Node Atomic GPU via compute.dispatch" |
| "Sovereign WGSL shader compilation via coralReef" | "via shader.compile capability" |
| "Route model inference to Squirrel primal" | "via model.inference capability" |

**Recommendation**: All primals should audit their error strings for cross-primal
name leaks. neuralSpring achieved this in V124; healthSpring in V43.

### Extended Discovery

healthSpring now discovers 7 capability domains (up from 4):

| Capability | Domain | Primal Role |
|-----------|--------|-------------|
| `compute.*` | GPU dispatch | (toadStool) |
| `data.*` | Data fetch | (NestGate) |
| `shader.*` | Shader compile | (coralReef) |
| `model.*` | ML inference | (Squirrel) |
| `ephemeral.*` | Session isolation | (rhizoCrypt) |
| `permanence.*` | Durable storage | (loamSpine) |
| `attribution.*` | Provenance chain | (sweetGrass) |

All discovery is capability-based with env-var override fallback. No hardcoded
primal names in the discovery code path.

---

## 3. For Sibling Springs

### Patterns Available for Absorption

| Pattern | Source | What It Does |
|---------|--------|-------------|
| Cast module (14 functions) | `ecoPrimal/src/cast.rs` | Named numeric casts, zero scattered `#[expect]` |
| Safe cast module (fallible) | `ecoPrimal/src/safe_cast.rs` | `Result`-returning casts for boundary values |
| Upstream contract tolerances | `ecoPrimal/src/tolerances.rs` | Cross-spring agreed tolerance constants |
| 7-primal discovery | `ecoPrimal/src/ipc/socket.rs` | ephemeral/permanence/attribution capability probes |
| Self-knowledge compliance | All `Display` impls | Zero cross-primal names in error strings |
| Validation harness submodules | `ecoPrimal/src/validation/` | check, sink, harness, or_exit, metrics |
| Simulation submodules | `ecoPrimal/src/simulation/` | stress, population, ecosystem, causal_chain |

### What healthSpring Absorbed from Springs

| Source | Pattern | V43 Status |
|--------|---------|-----------|
| groundSpring V122 | Cast module pattern | Adopted + expanded to 14 conversions |
| neuralSpring V124 | Self-knowledge compliance | Full compliance achieved |
| neuralSpring V124 | Upstream contract tolerances | Section added to tolerances.rs |
| wetSpring V135 | 7-primal discovery | 3 new capability domains added |
| wetSpring V134 | Validation harness submodules | validation.rs smart-refactored |
| hotSpring | Sovereign pipeline patterns | Acknowledged in GPU dispatch |

### What healthSpring Exports to Springs

| Pattern | Absorbers | Version |
|---------|----------|---------|
| `normalize_method()` IPC | primalSpring | V42 |
| `DispatchOutcome` enum | airSpring | V42 |
| `OrExit<T>` trait | wetSpring (origin) | V41 |
| Health domain capabilities | petalTongue | V22+ |
| Workload routing patterns | toadStool | V19+ |
| 6 WGSL shaders | coralReef | V17+ |

---

## 4. For biomeOS Team

### Deploy Graph Integration

healthSpring uses `lifecycle_dispatch.rs` for graph-related operations:
- `graph_deploy` / `graph_status` / `graph_teardown`
- `niche_list` / `primal_list`

These are wired but not yet exercised in integration tests. When biomeOS deploy
graph API stabilizes, healthSpring is ready for automated niche deployment.

### Capability Registration

healthSpring registers 59 capabilities (46 science + 13 infra) via
`capability.register` at startup. The capability surface is stable.

---

## 5. Debris Status

| Metric | Value |
|--------|-------|
| TODOs/FIXMEs in Rust | 0 |
| Temp files | 0 |
| Dead directories | 0 |
| Stale archive | `wateringHole/handoffs/archive/` (36 files, preserved as fossil record) |
| Python controls | 54 (legitimate baseline provenance) |
| Shell scripts | 5 (operational tools) |

---

## 6. Metrics Summary (V43)

| Metric | Value |
|--------|-------|
| Tests | 888 |
| Experiments | 83 |
| Python baselines | 54 |
| JSON-RPC capabilities | 59 |
| MCP tool schemas | 23 |
| GPU ops (barraCuda) | 6/6 LIVE |
| Named tolerances | 87+ |
| Named cast functions | 14 |
| Discovery domains | 7 |
| Unsafe blocks | 0 |
| `#[allow()]` in production | 0 |
| Clippy warnings | 0 |
| Max file LOC | 732 |

---

**License**: AGPL-3.0-or-later (code), CC-BY-SA-4.0 (this document)
