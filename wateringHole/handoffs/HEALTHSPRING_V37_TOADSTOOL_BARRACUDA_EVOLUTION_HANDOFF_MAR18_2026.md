<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# healthSpring V37 → toadStool / barraCuda Evolution Handoff

**Date**: 2026-03-18
**From**: healthSpring V37
**To**: toadStool team, barraCuda team, coralReef team
**License**: AGPL-3.0-or-later
**Pins**: barraCuda v0.3.5 (rev a60819c3), toadStool S158+, coralReef Phase 10 Iteration 55+
**Supersedes**: HEALTHSPRING_V37_CROSS_ECOSYSTEM_ABSORPTION_HANDOFF_MAR18_2026.md (general handoff)

---

## Executive Summary

healthSpring V37 completes the **Write → Absorb → Lean** cycle for all 6 GPU ops, adds MCP tool definitions for Squirrel AI, and identifies the remaining local math and patterns the upstream teams should consider absorbing. This handoff focuses on actionable evolution items for barraCuda and toadStool.

- **6/6 GPU ops fully rewired** to `barracuda::ops` — Tier A (V35) + Tier B (V36) complete
- **Local `std_dev`** remains in `uncertainty.rs` — candidate for `barracuda::stats::sample_variance`
- **Local FFT** remains in `biosignal/fft.rs` — 208 lines, CPU radix-2 Cooley-Tukey
- **`mul_add()` FMA sweep** done (8 new sites) — barraCuda CPU reference code should follow
- **MCP tool definitions** — 23 typed schemas via `mcp.tools.list` for Squirrel coordination
- **Centralized `extract_rpc_result()`** — pattern all primals should adopt
- **706 tests**, 79 experiments, 80 capabilities, zero clippy/unsafe/allow

---

## Part 1: barraCuda Primitive Consumption (V37 Complete Map)

### CPU Primitives Used

| Category | Primitive | healthSpring Module | Usage |
|----------|-----------|-------------------|-------|
| **rng** | `lcg_step`, `LCG_MULTIPLIER`, `state_to_f64`, `uniform_f64_sequence` | `rng.rs` (re-export) | PPG, EDA, ECG signal generation; SAEM Monte Carlo; bootstrap resampling |
| **stats** | `mean` | `uncertainty.rs` | Bootstrap CI, jackknife, bias-variance decomposition |
| **numerical** | `OdeSystem`, `BatchedOdeRK4` | `gpu/ode_systems.rs` | Michaelis-Menten, oral 1-compartment, 2-compartment ODE |

### GPU Ops Used (All 6 — Complete Rewire)

| healthSpring Op | barraCuda Op | Rewire Module | Status |
|-----------------|-------------|---------------|--------|
| `HillSweep` | `barracuda::ops::hill_f64::HillFunctionF64` | `barracuda_rewire.rs` | **Tier A — LIVE** |
| `PopulationPkBatch` | `barracuda::ops::population_pk_f64::PopulationPkF64` | `barracuda_rewire.rs` | **Tier A — LIVE** |
| `DiversityBatch` | `barracuda::ops::bio::diversity_fusion::DiversityFusionGpu` | `barracuda_rewire.rs` | **Tier A — LIVE** |
| `MichaelisMentenBatch` | `barracuda::ops::health::michaelis_menten_batch::MichaelisMentenBatchGpu` | `barracuda_rewire.rs` | **Tier B — LIVE (V36)** |
| `ScfaBatch` | `barracuda::ops::health::scfa_batch::ScfaBatchGpu` | `barracuda_rewire.rs` | **Tier B — LIVE (V36)** |
| `BeatClassifyBatch` | `barracuda::ops::health::beat_classify::BeatClassifyGpu` | `barracuda_rewire.rs` | **Tier B — LIVE (V36)** |

### What's NOT Delegated (Remaining Local Math)

| Local Implementation | File | LOC | Why Not Delegated | Recommended Action |
|---------------------|------|-----|-------------------|-------------------|
| `fn std_dev(data)` | `uncertainty.rs:290` | 8 | Uses `barracuda::stats::mean` internally, adds manual variance loop | **barraCuda: add `stats::sample_variance()`** |
| `fn fft_complex_inplace()`, `rfft()`, `irfft()` | `biosignal/fft.rs` | 208 | CPU radix-2 FFT for HRV analysis (<512 pts) | **barraCuda: GPU FFT for >512pt workloads** |
| `fn fused_pipeline()` | `gpu/fused.rs` | ~300 | Local multi-op upload→compute→readback | **barraCuda: stabilize `TensorSession` API** |

---

## Part 2: Recommended Actions for barraCuda

### P1 — High Priority

| Action | Context | Effort |
|--------|---------|--------|
| **Add `stats::sample_variance(data) → f64`** | healthSpring `uncertainty.rs` manually computes sample variance from `mean()`. groundSpring likely duplicates. One function eliminates N local copies. | Small |
| **Validate Tier B GPU parity on hardware** | healthSpring rewired MM, SCFA, BeatClassify to `barracuda::ops::health::*`. Needs GPU hardware validation to confirm numerical parity. healthSpring tests are CPU-only. | 1 session |
| **`mul_add()` sweep in CPU reference code** | healthSpring (V37), neuralSpring (S165), airSpring (V090) have all done FMA sweeps. barraCuda's own CPU reference implementations should follow for consistency. | Small |

### P2 — Medium Priority

| Action | Context | Effort |
|--------|---------|--------|
| **Stabilize `TensorSession` API** | healthSpring's `gpu/fused.rs` implements a local multi-op pipeline (upload → N compute → readback). This is the consumer use case for `TensorSession`. When the API stabilizes, healthSpring will rewire. | Medium |
| **GPU FFT (`spectral::fft_gpu`)** | healthSpring `biosignal/fft.rs` has 208-line CPU FFT. groundSpring also needs GPU FFT for spectral recon. Shared upstream implementation benefits both. | Medium |
| **`stats::kahan_sum()` absorption** | wetSpring V127 identified `kahan_sum` as generic math for upstream. healthSpring could use for large-array summation in population PK Monte Carlo. | Small |

### P3 — Track

| Action | Context | Effort |
|--------|---------|--------|
| **`BatchedOdeRK45F64` (adaptive step)** | airSpring V090 requested adaptive step-size ODE. healthSpring Michaelis-Menten with extreme Km/Vmax ratios would benefit. | Medium |
| **PRNG alignment** | groundSpring noted xorshift64 vs xoshiro128** divergence. healthSpring uses LCG via `barracuda::rng`. Unified PRNG policy would help reproducibility. | Discussion |

---

## Part 3: Recommended Actions for toadStool

### P1 — High Priority

| Action | Context | Effort |
|--------|---------|--------|
| **Validate healthSpring dispatch thresholds** | healthSpring's element-count → substrate routing (CPU vs GPU) was tuned on RTX 4070. Real hardware benchmarks should confirm. exp085/087 provide the test matrix. | 1 session |
| **Standardize streaming callback API** | healthSpring exp072 uses `execute_streaming()` with `fn(stage_idx, total, result)` callback. This pattern should be the standard for all toadStool consumers. | Small |

### P2 — Medium Priority

| Action | Context | Effort |
|--------|---------|--------|
| **Session-level provenance** | healthSpring now has 49 structured provenance records. toadStool's dispatch sessions should carry provenance metadata (pipeline ID, spring version, dispatch timestamp). | Medium |
| **MCP integration path** | healthSpring exposes `mcp.tools.list` returning 23 tool schemas. toadStool should consider a standard for advertising compute tools via MCP. | Discussion |

---

## Part 4: Recommended Actions for coralReef

| Action | Context | Effort |
|--------|---------|--------|
| **Verify `shader.compile` params** | healthSpring forwards `compute.shader_compile` → coralReef. Ensure `{source, target, options}` param shape is documented and stable. | Small |
| **Document WGSL preprocessing** | healthSpring's 6 local shaders use `strip_f64_enable()` preprocessing. coralReef's compile pipeline should handle this or document the expected input format. | Small |

---

## Part 5: Learnings for All Primals

### Patterns Proven in healthSpring V37

1. **`extract_rpc_result()` centralization** — 6 ad-hoc `val.get("result")` sites → 1 function. Prevents silent data loss when error responses are ignored. Every primal doing ad-hoc extraction should centralize.

2. **Provenance completeness test** — `PROVENANCE_REGISTRY` with a runtime test that walks `control/` and asserts every `.py` has a registry entry. Catches undocumented baselines at test time.

3. **`deny.toml` 14-crate C-dep ban** — explicit deny list for ecoBin compliance. `ring` allowed only as wrapper of `rustls` (feature-gated, evolution path to `rustls-rustcrypto`).

4. **MCP tool definitions** — `McpToolDef { name, description, input_schema }` exposed via `mcp.tools.list`. Simple pattern for any primal to advertise typed capabilities to Squirrel.

5. **FMA audit pattern** — search for `a * b + c` in floating-point code, replace with `a.mul_add(b, c)`. One-pass audit, measurable accuracy improvement, zero breakage risk.

---

## Part 6: WGSL Shader Status

All 6 local shaders are retained as validation targets but execution delegates to upstream barraCuda:

| Shader File | barraCuda Upstream | Validated By |
|-------------|-------------------|-------------|
| `hill_dose_response_f64.wgsl` | `ops::hill_f64::HillFunctionF64` | exp053, exp060 |
| `population_pk_f64.wgsl` | `ops::population_pk_f64::PopulationPkF64` | exp053, exp060 |
| `diversity_f64.wgsl` | `ops::bio::diversity_fusion::DiversityFusionGpu` | exp053, exp060 |
| `michaelis_menten_batch_f64.wgsl` | `ops::health::michaelis_menten_batch` | exp083 |
| `scfa_batch_f64.wgsl` | `ops::health::scfa_batch` | exp083 |
| `beat_classify_batch_f64.wgsl` | `ops::health::beat_classify` | exp083 |

**Next step**: Remove local shader copies once upstream parity is CI-validated on GPU hardware.

---

## Part 7: Quality Metrics

| Metric | Value |
|--------|-------|
| Tests | 706 |
| Experiments | 79 |
| JSON-RPC capabilities | 80 |
| MCP tool schemas | 23 |
| Provenance records | 49 |
| Proptest IPC fuzz tests | 18 |
| GPU ops (rewired to barraCuda) | 6/6 |
| FMA `mul_add()` sites (total) | 23 |
| C-dep crates banned | 14 |
| Unsafe blocks | 0 |
| `#[allow()]` in production | 0 |
| Clippy warnings | 0 |
| TODO/FIXME | 0 |
| Files > 1000 LOC | 0 (max 731) |

---

## Verification

```bash
cargo fmt --check --all          # 0 diffs
cargo clippy --workspace --all-targets  # 0 warnings
cargo test --workspace           # 706 passed, 0 failed
cargo doc --workspace --no-deps  # 0 warnings
```

---

**healthSpring V37 | 706 tests | 79 experiments | 6/6 GPU ops rewired | 80 capabilities | 23 MCP tools | AGPL-3.0-or-later**
