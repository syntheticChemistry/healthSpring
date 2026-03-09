<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring V7 → barraCuda / toadStool Full Visualization Handoff

**Date:** March 9, 2026
**From:** healthSpring V7
**To:** barraCuda / toadStool / coralReef / petalTongue teams
**Covers:** V6 → V7 (GPU pipeline + petalTongue absorption + full visualization)
**License:** AGPL-3.0-or-later
**Status:** Complete — 31 experiments, 201 unit tests, 418 binary checks, full petalTongue visualization

---

## Executive Summary

- **V6** delivered 3 WGSL shaders, `GpuContext`, fused pipeline, and GPU scaling to 10M elements
- **V6.1** completed petalTongue absorption: `DataChannel`, `ClinicalRange`, renderers, and clinical theme absorbed upstream; healthSpring leaned (petaltongue-health removed)
- **V7** adds per-track scenario builders: **22 nodes, 62 data channels, 13 clinical ranges** across all 4 study tracks, generating petalTongue-compatible JSON from live math functions
- **Exp056** validates 47 structural checks across all scenarios (723 KB combined JSON)
- Zero clippy warnings, zero unsafe blocks, 201 unit tests + 418 binary checks all green

---

## Part 1: What healthSpring Built (V6 → V7)

### 1.1 GPU Pipeline (V6 — Unchanged)

| Component | Location | Status |
|-----------|----------|--------|
| `hill_dose_response_f64.wgsl` | `barracuda/shaders/health/` | LIVE |
| `population_pk_f64.wgsl` | `barracuda/shaders/health/` | LIVE |
| `diversity_f64.wgsl` | `barracuda/shaders/health/` | LIVE |
| `GpuContext` + `execute_fused()` | `barracuda/src/gpu.rs` | LIVE |
| `Pipeline::execute_gpu()` | `toadstool/src/pipeline.rs` | LIVE |

### 1.2 petalTongue Absorption (V6.1)

| Component | healthSpring Source | petalTongue Target | Status |
|-----------|--------------------|--------------------|--------|
| `DataChannel` enum | `visualization/types.rs` | `petal-tongue-core/data_channel.rs` | **Absorbed** |
| `ClinicalRange` struct | `visualization/types.rs` | `petal-tongue-core/data_channel.rs` | **Absorbed** |
| Chart renderers (TimeSeries, Distribution, Bar, Gauge) | ~~petaltongue-health/render.rs~~ | `petal-tongue-graph/chart_renderer.rs` | **Absorbed** |
| Clinical theme colors | ~~petaltongue-health/theme.rs~~ | `petal-tongue-graph/clinical_theme.rs` | **Absorbed** |
| Version string parsing fix | N/A | `dynamic_schema.rs` | **Fixed** |

### 1.3 Per-Track Scenario Builders (V7 — NEW)

`barracuda/src/visualization/scenarios.rs` — calls live math functions, not mock data:

| Builder | Track | Nodes | Channels | Ranges | Key Math Called |
|---------|-------|:-----:|:--------:|:------:|----------------|
| `pkpd_study()` | PK/PD | 6 | 18 | 4 | `hill_sweep`, `pk_one_compartment`, `pk_two_compartment`, `pk_mab_allometric`, `population_pk_cpu`, `pbpk_iv_simulate` |
| `microbiome_study()` | Microbiome | 4 | 10 | 2 | `shannon_index`, `simpson_index`, `pielou_evenness`, `evenness_to_disorder`, `colonization_resistance`, `fmt_blend`, `bray_curtis` |
| `biosignal_study()` | Biosignal | 4 | 15 | 4 | `pan_tompkins`, `heart_rate_from_peaks`, `sdnn_ms`, `rmssd_ms`, `pnn50`, `ppg_r_value`, `spo2_from_r`, `fuse_channels` |
| `endocrine_study()` | Endocrinology | 8 | 19 | 3 | `pk_im_depot`, `pk_pellet`, `testosterone_decline`, `trt_weight`, `trt_lipid_response`, `hba1c_response`, `lognormal_params`, `hrv_trt_response` |
| `full_study()` | Combined | 22 | 62 | 13 | All of the above |

### 1.4 DataChannel Types Used

| Type | Count | Example |
|------|:-----:|---------|
| TimeSeries | 28 | PK concentration-time curves, RR tachogram, EDA SCL |
| Distribution | 3 | Population AUC distribution, Monte Carlo risk |
| Bar | 8 | Shannon/Simpson across communities, genus abundances |
| Gauge | 23 | HR, SpO2, SDNN, HbA1c, colonization resistance, cardiac risk |

### 1.5 Experiments Added (V6 → V7)

| Exp | Purpose | Checks |
|-----|---------|:------:|
| 053 | GPU parity: WGSL vs CPU (V6) | 17 |
| 054 | Fused pipeline + toadStool GPU dispatch (V6) | 11 |
| 055 | GPU scaling: 1K→10M sweep (V6) | Benchmark |
| 056 | Full petalTongue visualization for all 4 tracks (V7) | **47** |

---

## Part 2: barraCuda Evolution — Review and Recommendations

### 2.1 healthSpring's barraCuda Usage Summary

healthSpring uses barraCuda as its math+compute library with these patterns:

| Pattern | Usage | Notes |
|---------|-------|-------|
| Pure math functions | `hill_dose_response`, `pk_one_compartment`, etc. | Called directly from experiments and scenario builders |
| GPU dispatch | `GpuOp` → `GpuContext::execute()` | Feature-gated `gpu` |
| Fused pipeline | `GpuContext::execute_fused()` | Multiple ops, one encoder submit |
| WGSL shaders | 3 health-specific shaders | Compiled into binary, not loaded at runtime |
| Visualization | `scenarios.rs` → petalTongue JSON | Math → DataChannel → JSON |
| toadStool bridge | `Stage::to_gpu_op()` | Converts pipeline stages to GPU ops |

### 2.2 Absorption Candidates for barraCuda Core

Tier 1 — Ready now (used across multiple experiments, stable API):

| Function / Pattern | Location | Target | Priority |
|--------------------|----------|--------|:--------:|
| `power_f64` WGSL helper | `hill_dose_response_f64.wgsl` | `barraCuda::wgsl::math` | **P0** |
| `log_f64` WGSL helper | `diversity_f64.wgsl` | `barraCuda::wgsl::math` | **P0** |
| `strip_f64_enable()` | `gpu.rs` | `barraCuda::wgsl::preprocess` | **P0** |
| `GpuContext` (persistent device/queue) | `gpu.rs` | `barraCuda::gpu` | **P1** |
| `dispatch_and_readback()` generic | `gpu.rs` | `barraCuda::gpu` | **P1** |
| `hill_dose_response` | `pkpd/dose_response.rs` | `barraCuda::bio::pharmacology` | P2 |
| `population_pk_cpu` | `pkpd/population.rs` | `barraCuda::monte_carlo` | P2 |
| `lognormal_params` | `pkpd/population.rs` | `barraCuda::stats::distributions` | P2 |
| `allometric_scale` | `pkpd/allometry.rs` | `barraCuda::math::scale` | P2 |

Tier 2 — Need design discussion:

| Function | Notes |
|----------|-------|
| `pan_tompkins_qrs` | Streaming signal detection — needs NPU path in toadStool |
| `pbpk_iv_simulate` | Multi-compartment ODE — depends on wetSpring ODE absorption status |
| `anderson_hamiltonian_1d` | Shared with wetSpring — coordinate which spring owns the abstraction |

Tier 3 — Stays local (too health-specific):

| Function | Reason |
|----------|--------|
| `assess_patient` | healthSpring-specific diagnostic pipeline |
| `cardiac_risk_composite` | Clinical scoring, not general math |
| `fuse_channels` | Multi-modal clinical fusion, specific to health |
| All `trt_*` functions | Testosterone-specific clinical models |

### 2.3 Absorption Candidates for toadStool

| Component | Location | Target | Priority |
|-----------|----------|--------|:--------:|
| `Pipeline::execute_gpu()` | `toadstool/src/pipeline.rs` | toadStool core pipeline | **P1** |
| `Stage::to_gpu_op()` bridge | `toadstool/src/stage.rs` | toadStool core stage | **P1** |
| `Pipeline::execute_auto()` | `toadstool/src/pipeline.rs` | toadStool routing | P2 |
| GPU fallback on failure | `pipeline.rs` | Resilience pattern | P2 |

### 2.4 Absorption Candidates for coralReef

| Pattern | Description | Priority |
|---------|-------------|:--------:|
| f32 transcendental workaround | `pow(f64,f64)` via f32 intermediates for NVIDIA NVVM | **P0** |
| u32-only PRNG | `xorshift32 + wang_hash` — no SHADER_INT64 dep | **P0** |
| `@workgroup_size(256)` convention | Avoids 65K dispatch limit to 10M+ elements | P1 |
| 2D dispatch for > 16M elements | Not yet implemented, but architecture supports it | P2 |

---

## Part 3: Critical Learnings (V1 → V7)

### 3.1 GPU (Learnings for barraCuda/toadStool/coralReef)

1. **`enable f64;` must be stripped** — wgpu's naga parser rejects it; f64 support enabled via `wgpu::Features::SHADER_F64` at device level
2. **`pow(f64, f64)` crashes NVIDIA** — NVVM backend; use `exp(n * log(c))` through f32 intermediates (~7 decimal digits)
3. **u64 PRNG not portable** — `SHADER_INT64` not widely available; u32 xorshift32 + Wang hash works
4. **GPU/CPU parity is statistical for Monte Carlo** — different PRNG families produce different sequences; validate mean/std/range, not bit-exact
5. **Fused pipeline 31.7x faster at small sizes** — single encoder eliminates 2 device roundtrips; no advantage at compute-bound sizes
6. **wgpu v28 API changes** — `PollType::Wait`, `experimental_features`, `PipelineCompilationOptions::default()` all required
7. **Workgroup dispatch limit** — max 65,535 workgroups; `@workgroup_size(256)` handles up to ~16M; 2D dispatch needed beyond

### 3.2 petalTongue Integration (Learnings for petalTongue/biomeOS)

1. **`ClinicalRange.status` must be `String`** — not `&'static str`; petalTongue deserializes this field
2. **Version field** — petalTongue `DynamicData` expected `{major, minor, patch}` object but JSON has `"2.0.0"` string; fixed upstream
3. **Schema stability** — `HealthScenario`, `ScenarioNode`, `DataChannel` types are wire-stable; downstream can parse without breakage
4. **Per-track scenarios** — better for selective rendering than one giant scenario; `full_study()` combines when needed

### 3.3 Validation Architecture (Learnings for All Springs)

1. **Four-tier pipeline works** — Python (Tier 0) → Rust CPU (Tier 1) → GPU WGSL (Tier 2) → metalForge (Tier 3)
2. **Binary check pattern** — macro-based `check!()` in experiment binaries gives clear pass/fail reporting
3. **Cross-validation** — 104 Python ↔ Rust checks catch subtle numerical differences early
4. **Clippy pedantic + `#[expect(...)]`** — strict linting with targeted exceptions is maintainable at scale

---

## Part 4: healthSpring metalForge Status

| Component | Location | Status |
|-----------|----------|--------|
| `Nucleus` (Tower/Node/Nest) atomics | `metalForge/forge/src/nucleus.rs` | Built, 27 tests |
| PCIe P2P transfer planning | `metalForge/forge/src/transfer.rs` | Built |
| `select_substrate()` | `metalForge/forge/src/lib.rs` | Returns routing decisions; not wired to live GPU |

**Action for toadStool**: Wire `select_substrate()` to actual `GpuContext` dispatch for field deployment.

---

## Part 5: Files Changed (V6 → V7)

| File | Change |
|------|--------|
| `barracuda/src/visualization/scenarios.rs` | **New** — per-track scenario builders |
| `barracuda/src/visualization/mod.rs` | Added `pub mod scenarios` |
| `barracuda/src/visualization/types.rs` | `ClinicalRange.status`: `&str` → `String` |
| `barracuda/src/visualization/nodes.rs` | Updated ClinicalRange instantiations |
| `barracuda/src/lib.rs` | Updated doc table (population_pk shader) |
| `experiments/exp056_study_scenarios/` | **New** — 47-check visualization validation |
| `Cargo.toml` | +exp056, -petaltongue-health |
| ~~`petaltongue-health/`~~ | **Removed** (absorbed upstream) |

---

## Part 6: Status Summary

```
cargo test --workspace                    # 201 tests pass
cargo clippy --workspace -- -D warnings   # zero warnings
cargo run --release --bin exp056_study_scenarios  # 47/47 pass (723 KB JSON)
```

| Metric | V6 | V7 |
|--------|:--:|:--:|
| Experiments | 30 | **31** |
| Unit tests | 200 | **201** |
| Binary checks | 346 | **418** |
| WGSL shaders | 3 | 3 |
| petalTongue nodes | 8 (diagnostic only) | **22** (all 4 tracks) |
| Data channels | 12 | **62** |
| Clinical ranges | 3 | **13** |
| Unsafe blocks | 0 | 0 |
| Clippy warnings | 0 | 0 |

Upstream pins: barraCuda v0.3.3 (local), toadStool (local), wgpu v28.

---

## Part 7: Recommended Next Steps

### For barraCuda Team
1. [ ] Extract `power_f64` / `log_f64` WGSL helpers into a shared shader library
2. [ ] Absorb `strip_f64_enable()` into a `preprocess_wgsl()` utility
3. [ ] Absorb `GpuContext` pattern for reuse by other springs
4. [ ] Add `PipelineBuilder` auto-dispatch strategy based on element count

### For toadStool Team
1. [ ] Absorb `Pipeline::execute_gpu()` and `Stage::to_gpu_op()` into core
2. [ ] Wire `metalForge::select_substrate()` to live GPU dispatch
3. [ ] Add NPU dispatch path for streaming biosignal workloads (Pan-Tompkins)
4. [ ] Document that GPU/CPU Monte Carlo parity is statistical, not bitwise

### For coralReef Team
1. [ ] Standardize f32 transcendental workaround as a shader preamble
2. [ ] Provide u32-only PRNG as a shader library function
3. [ ] Support 2D dispatch for > 16M element workloads

### For petalTongue Team — LOCAL EVOLUTION READY FOR ABSORPTION

healthSpring has evolved the local petalTongue copy (3 non-invasive, additive changes).
These wire the DataChannel chart rendering that petalTongue already absorbed from healthSpring.

**Files changed (for absorption review):**

| File | Change | Lines Added |
|------|--------|:-----------:|
| `crates/petal-tongue-ui/src/scenario/ecosystem.rs` | Add `data_channels: Vec<DataChannel>` and `clinical_ranges: Vec<ClinicalRange>` to `PrimalDefinition` with `#[serde(default)]` | 4 |
| `crates/petal-tongue-ui/src/scenario/convert.rs` | Serialize data_channels/clinical_ranges to JSON string in properties during `to_primal_info()` | 12 |
| `crates/petal-tongue-ui/src/app_panels/primal_details.rs` | Read data_channels from properties and call `draw_node_detail()` — handles both typed and DynamicScenarioProvider paths | 25 |

**What these changes enable:**
- `petaltongue ui --scenario healthspring-full-study.json` shows 22-node topology
- Clicking any node renders TimeSeries, Distribution, Bar, and Gauge charts via the already-absorbed `chart_renderer.rs`
- All 62 data channels and 13 clinical ranges are visible

**Scenario files (generated by `dump_scenarios` binary):**

| File | Nodes | Channels | Size |
|------|:-----:|:--------:|:----:|
| `healthspring-pkpd.json` | 6 | 18 | 217 KB |
| `healthspring-microbiome.json` | 4 | 10 | 7 KB |
| `healthspring-biosignal.json` | 4 | 15 | 89 KB |
| `healthspring-endocrine.json` | 8 | 19 | 410 KB |
| `healthspring-full-study.json` | 22 | 62 | 723 KB |
| `healthspring-diagnostic.json` | 8 | 12 | 56 KB |

**Remaining petalTongue team items:**
1. [ ] Absorb the 3 local changes above (review, merge, test)
2. [ ] Add clinical range threshold rendering (green/yellow/red zones on charts)
3. [ ] Fix winit event loop threading (pre-existing — `EventLoop` on non-main thread panics)
4. [ ] Wire edge topology from scenario JSON `edges` array (currently auto-generates ring/NUCLEUS)

---

This handoff is unidirectional: healthSpring → barraCuda / toadStool / coralReef / petalTongue. No response expected.
