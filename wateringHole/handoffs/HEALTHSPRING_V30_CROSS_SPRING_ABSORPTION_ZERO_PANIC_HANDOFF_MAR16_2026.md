# HEALTHSPRING V30 — Cross-Spring Absorption + Zero-Panic Evolution Handoff

**Date:** March 16, 2026
**From:** healthSpring V30
**To:** barraCuda, toadStool, All Springs
**Supersedes:** V29 (Deep Debt Solutions)
**License:** scyBorg (AGPL-3.0-or-later code + ORC mechanics + CC-BY-SA 4.0 creative content)
**Covers:** V29 → V30 (cross-spring pattern absorption, zero-panic validation, compute dispatch, barraCuda health delegation)

---

## Executive Summary

- **Cross-spring absorption** from all 6 sibling springs — patterns proven elsewhere, adopted here
- **~100 panic sites** eliminated across ~25 validation binaries (groundSpring zero-panic pattern)
- **Dual-format capability parsing** interoperates with all ecosystem response formats
- **Typed `compute_dispatch` client** for toadStool `compute.dispatch.submit/result/capabilities`
- **3 barraCuda health delegations** — `mm_auc`, `scr_rate`, `antibiotic_perturbation` (Write → Absorb → **Lean**)
- **611 tests pass** (up from 603), zero failures, zero clippy warnings
- **`deny.toml`** with `wildcards = "deny"` for dependency hygiene

---

## Part 1: What Changed (V29 → V30)

### 1.1 Dual-Format Capability Parsing

New `extract_capability_strings()` in `ipc/socket.rs` handles all ecosystem response formats:

| Format | Source | Field Path |
|--------|--------|------------|
| healthSpring | `handle_capability_list()` | `result.science` + `result.infrastructure` |
| neuralSpring/ludoSpring | `parse_capability_list()` | `result.capabilities` (flat array) |
| Nested object | biomeOS Neural API | `result.capabilities.capabilities` |
| Raw array | Songbird | `result` (direct array) |

`probe_capability()` now works with any primal in the ecosystem.

### 1.2 Zero-Panic Validation Binaries

~100 `.expect()`, `.unwrap()`, and `panic!()` sites evolved to graceful error handling:

| Pattern | Before | After |
|---------|--------|-------|
| JSON parsing | `serde_json::from_str(x).expect("msg")` | `let Ok(v) = serde_json::from_str(x) else { eprintln!("FAIL: msg"); exit(1); }` |
| File I/O | `fs::write(p, d).expect("msg")` | `if fs::write(p, d).is_err() { eprintln!("FAIL: msg"); exit(1); }` |
| Serialization | `to_string_pretty(x).expect("serialize")` | `.unwrap_or_default()` (infallible for valid data) |
| Float comparison | `partial_cmp(a, b).unwrap()` | `.unwrap_or(Ordering::Equal)` |
| Iterator results | `iter.last().unwrap()` | `let Some(v) = iter.last() else { exit(1); }` |
| Mutex lock | `.expect("lock")` | Kept as-is (poisoning is panic-worthy) |

Affected: exp002, exp003, exp006, exp021-023, exp036, exp052, exp055, exp056, exp060, exp063-064, exp066, exp068, exp072-074, exp076-078, exp084-085, exp088-089.

### 1.3 Typed Compute Dispatch Client

New `ipc::compute_dispatch` module wraps toadStool `compute.dispatch.*` protocol:

- `submit(workload_type, params)` → `DispatchHandle { job_id, socket }`
- `result(handle)` → `serde_json::Value`
- `capabilities()` → `Vec<String>`

All three use capability-based discovery — no hardcoded compute primal names.

### 1.4 barraCuda Health Delegation

Three CPU primitives now delegate to `barracuda::health::*`:

| healthSpring Function | Delegates To | Module |
|----------------------|--------------|--------|
| `mm_auc()` | `barracuda::health::pkpd::mm_auc` | `pkpd/nonlinear.rs` |
| `scr_rate()` | `barracuda::health::biosignal::scr_rate` | `biosignal/stress.rs` |
| `antibiotic_perturbation_abundances()` | `barracuda::health::microbiome::antibiotic_perturbation` | `microbiome/clinical.rs` |

Local implementations kept where signatures diverge (documented rationale in each module).

### 1.5 Dependency Hygiene

- `deny.toml` created with `wildcards = "deny"`, license allowlist, advisory controls
- Python `requirements.txt` enhanced with PRNG drift warning and pinning rationale

---

## Part 2: Patterns Absorbed from Sibling Springs

| Pattern | Source | What We Took |
|---------|--------|-------------|
| Zero-panic validation | groundSpring V109 | `let Ok(...) else { eprintln!(); exit(1); }` for all JSON/file ops |
| Dual-format capability | neuralSpring S157, ludoSpring V22 | `extract_capability_strings()` multi-format parser |
| Direct dispatch client | ludoSpring V22, toadStool S156 | `compute.dispatch.submit/result/capabilities` typed wrappers |
| `deny.toml` | airSpring v0.8.4 | `wildcards = "deny"`, license allowlist |
| Python dep pinning | groundSpring V109 | Exact version + PRNG drift documentation |
| Write → Absorb → Lean | All springs | `barracuda::health::*` delegation instead of local reimplementation |

---

## Part 3: barraCuda Absorption Status

### Active Delegations

| healthSpring | barraCuda Primitive | Type |
|-------------|-------------------|------|
| `uncertainty::mean()` | `barracuda::stats::mean` | CPU (V29) |
| `pkpd::mm_auc()` | `barracuda::health::pkpd::mm_auc` | CPU (V30) |
| `biosignal::scr_rate()` | `barracuda::health::biosignal::scr_rate` | CPU (V30) |
| `clinical::antibiotic_perturbation_abundances()` | `barracuda::health::microbiome::antibiotic_perturbation` | CPU (V30) |
| HillSweep GPU | `barracuda::ops::HillFunctionF64` | GPU (V29) |
| PopPK GPU | `barracuda::ops::PopulationPkF64` | GPU (V29) |
| Diversity GPU | `barracuda::ops::bio::DiversityFusionGpu` | GPU (V29) |

### Local Kept (divergent signatures)

| Function | Reason |
|----------|--------|
| `mm_pk_simulate` | healthSpring: dose-based API; barraCuda: concentration-based |
| `scfa_production` | healthSpring: fiber-first args; barraCuda: params-first |
| `gut_serotonin_production` | healthSpring: microbiome_factor ∈ [0,1]; barraCuda: same but different call sites |
| EDA pipeline | healthSpring: `moving_window_integration` + `min_interval_samples`; barraCuda: simpler |
| Beat classification | healthSpring: `min_correlation` threshold; barraCuda: no threshold |

---

## Part 4: toadStool Evolution Targets

| Target | Priority | Description |
|--------|----------|-------------|
| **`compute.dispatch.submit` validation** | P0 | healthSpring V30 has a typed client — validate round-trip with toadStool S156. |
| **Dispatch capabilities registry** | P1 | Return supported workload types from `compute.dispatch.capabilities`. |
| **Streaming dispatch** | P2 | healthSpring `exp065_live_dashboard` needs streaming dispatch for real-time biosignal. |

---

## Part 5: Metrics

| Metric | V29 | V30 |
|--------|-----|-----|
| Tests | 603 | **611** (+8) |
| Panic sites (validation binaries) | ~100 | **0** |
| Capability response formats | 1 | **4** (science, capabilities, nested, raw) |
| `compute_dispatch` client | No | **Yes** (submit/result/capabilities) |
| barraCuda health delegations | 1 (mean) | **4** (mean, mm_auc, scr_rate, antibiotic) |
| `deny.toml` | No | **Yes** |
| `#[allow()]` in codebase | 0 | **0** |
| Clippy warnings | 0 | **0** |

---

## Part 6: Next Evolution Targets

1. **HMM for biosignal regime detection** — absorb `HmmBatchForwardF64` from neuralSpring S157
2. **ESN for clinical prediction** — absorb Echo State Network from neuralSpring
3. **3D Tissue Anderson** — absorb from groundSpring for tissue heterogeneity
4. **TensorSession** — when barraCuda ships `TensorSession`, wire `GpuContext::execute_fused()`
5. **Tier B shader absorption** — push MM batch, SCFA batch, Beat classify upstream
6. **llvm-cov** — target 90%+ line coverage
7. **CytokineBrain** — absorb from airSpring for clinical cytokine network visualization
