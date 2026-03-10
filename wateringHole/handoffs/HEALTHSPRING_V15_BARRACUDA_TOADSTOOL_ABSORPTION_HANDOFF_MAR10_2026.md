# healthSpring V15 → barraCuda + toadStool Absorption Handoff

**Date**: March 10, 2026
**From**: healthSpring V15
**To**: barraCuda team, toadStool team
**License**: AGPL-3.0-or-later
**Purpose**: Absorption guidance — what healthSpring has validated, what's ready for upstream consumption, and what we learned about precision, shaders, and clinical compute.

---

## Executive Summary

healthSpring V15 completes the upstream rewire:
- **rng** delegates to `barracuda::rng` (canonical LCG)
- **eigensolver** delegates to `barracuda::special::anderson_diagonalize`
- **Precision routing** (`PrecisionRouting` enum) mirrors toadStool S128
- **Cross-validation** confirms bit-identical results between local and upstream
- **Benchmarks** show performance-neutral rewiring (within 2% for most ops)
- **368 tests**, 48 experiments, 0 clippy warnings, 0 unsafe code

---

## Part 1: For barraCuda — What to Absorb Next

### 1.1 P0: NLME GPU Shaders

healthSpring Exp075-076 validate FOCE and SAEM estimation. These are the
highest-value GPU promotion candidates:

| Primitive | Source | GPU Pattern | Subject Threshold |
|-----------|--------|-------------|-------------------|
| `foce_estimate` | `pkpd/nlme.rs` | Per-subject gradient → batch parallel | > 100 subjects |
| `saem_estimate` | `pkpd/nlme.rs` | E-step Metropolis-Hastings | > 100 subjects |
| `vpc_simulate` | `pkpd/diagnostics.rs` | Embarrassingly parallel | > 50 simulations |

**toadStool action:** Add `Workload::NlmeIteration` to dispatch rules with 100-subject GPU crossover threshold (from Kokkos benchmark `kokkos_nlme_iteration`).

**barraCuda action:** Create `ops::NlmeFoceF64` and `ops::NlmeSaemF64` GPU ops. The per-subject gradient in FOCE is embarrassingly parallel — each subject's log-likelihood and gradient can be computed independently.

### 1.2 P0: Biosignal FFT

healthSpring's `biosignal/fft.rs` is a Cooley-Tukey radix-2 FFT (replaced O(n²) DFT in V14.1). This should be absorbed as `barracuda::signal::fft`:

```rust
pub fn rfft(signal: &[f64]) -> (Vec<f64>, Vec<f64>)
pub fn irfft(re: &[f64], im: &[f64], n: usize) -> Vec<f64>
```

**Key learning:** When padding non-power-of-two inputs, `irfft` must reconstruct from the full `n_padded/2 + 1` frequency bins, then truncate to original `n`. Downstream callers must use `n_effective = (n_freq - 1) * 2` for frequency calculations.

### 1.3 P1: VPC Monte Carlo Shader

VPC (Visual Predictive Check) runs 200-1000 independent ODE simulations with random parameters. This is the same embarrassingly-parallel pattern as `PopulationPkF64`:

```wgsl
// Each thread: sample params → solve ODE → store median/CI
@compute @workgroup_size(256)
fn vpc_simulate(@builtin(global_invocation_id) id: vec3<u32>) {
    let sim_idx = id.x;
    // Wang hash + xorshift32 PRNG (same as population_pk)
    // 1-compartment ODE per sim
    // Store percentile contributions
}
```

### 1.4 P2: Streaming Biosignal Patterns

Pan-Tompkins QRS detection and PPG SpO2 estimation are streaming pipelines:
- Fixed-size sliding windows (150ms MWI for QRS, 4-beat windows for SpO2)
- Per-sample state machines (refractory period, peak detection)
- NPU candidates (Akida) for edge deployment

**toadStool action:** These are `Workload::Streaming` types. When Akida driver integration lands, route biosignal streams to NPU.

### 1.5 Precision Learnings

| Finding | Impact | Where |
|---------|--------|-------|
| f32 `exp`/`log` intermediates in f64 WGSL shaders give ~7 decimal digits | Sufficient for PK/PD (dose-response variability >> 7 digits) | All 3 GPU shaders |
| u32 PRNG (xorshift32 + Wang hash) sufficient for Monte Carlo | No need for `SHADER_INT64` feature | `population_pk_f64.wgsl` |
| `workgroup_size(256)` optimal across tested GPUs | Convention for all healthSpring WGSL | All 3 shaders |
| f64 shared-memory reduction unreliable on some drivers | Use per-thread accumulators, not workgroup shared memory for f64 | `diversity_f64.wgsl` |

**coralReef action:** When f64 lowering replaces f32 transcendental workarounds, healthSpring's Hill shader gains full f64 precision. The `power_f64(base, exp)` workaround (`exp(f32(n * log(c)))`) can be removed.

---

## Part 2: For toadStool — Dispatch and Precision

### 2.1 Kokkos Crossover Thresholds

From `barracuda/benches/kokkos_parity.rs`, recommended dispatch thresholds:

| Pattern | GPU Crossover (N) | Workload Type | Source |
|---------|-------------------|---------------|--------|
| Reduction | > 10,000 | `Workload::Reduce` | `kokkos_reduction` |
| Scatter | > 50,000 | `Workload::Parallel` | `kokkos_scatter` |
| Monte Carlo | > 100,000 | `Workload::Parallel` | `kokkos_monte_carlo` |
| ODE batch | > 5,000 patients | `Workload::Sweep` | `kokkos_ode_batch` |
| NLME iteration | > 100 subjects | `Workload::Sweep` | `kokkos_nlme_iteration` |

### 2.2 PrecisionRouting Alignment

healthSpring's `metalForge::PrecisionRouting` is semantically identical to toadStool's `PrecisionRoutingAdvice`. When healthSpring integrates with the ecosystem daemon, it should consume the toadStool type directly.

**toadStool action:** Consider re-exporting `PrecisionRoutingAdvice` from a shared crate (e.g., `toadstool-types`) that springs can depend on without pulling the full toadStool runtime.

### 2.3 StageProgress Pattern

healthSpring's `Pipeline::execute_streaming()` uses `FnMut(usize, usize, &StageResult)`. toadStool S140 has `StageProgress { stage_index, total_stages, stage_name, elapsed_secs }`. healthSpring should adopt `StageProgress` for richer telemetry.

### 2.4 PipelineGraph DAG

healthSpring's `Pipeline` is a linear sequence of stages. toadStool S139's `PipelineGraph` DAG with Kahn sort would benefit complex clinical pipelines where biosignal and microbiome can run in parallel before fusion.

---

## Part 3: Cross-Spring Shader Evolution Notes

### 3.1 What healthSpring Consumes From Other Springs

| From | What | healthSpring Use |
|------|------|------------------|
| hotSpring | df64_core.wgsl, df64_transcendentals.wgsl | f64 precision on consumer GPUs |
| hotSpring | NVVM poisoning workarounds | Safe GPU compute on NVIDIA |
| groundSpring | PrecisionRoutingAdvice (V84-85) | f64 shader variant selection |
| groundSpring | chi_squared_f64.wgsl | PK/PD diagnostic GOF tests |
| groundSpring | welford_mean_variance_f64.wgsl | Streaming biosignal statistics |
| neuralSpring | fused_kl_divergence_f64.wgsl | NLME model selection |
| wetSpring | smith_waterman_banded_f64.wgsl | 16S microbiome classification |
| wetSpring | hmm_forward_f64.wgsl | Biosignal rhythm detection |

### 3.2 What Other Springs Should Know About healthSpring

| Pattern | Value to Other Springs |
|---------|------------------------|
| Clinical per-person pipeline (`diagnostic.rs`) | Model for per-sample analysis in wetSpring |
| NLME population modeling (FOCE/SAEM) | Applicable to wetSpring population genetics |
| Fused GPU pipeline pattern (`execute_fused`) | Zero CPU roundtrips between stages |
| WFDB streaming parser | Generalizable to other signal formats |
| Anderson gut microbiome ↔ physics mapping | Novel cross-domain validation |

---

## Part 4: Benchmark Results

### CPU Parity (Rust vs Python)

| Benchmark | Python | Rust CPU | Speedup |
|-----------|--------|----------|---------|
| Hill sweep (50 concs) | 12 µs | 0.8 µs | 15x |
| PK curve (101 points) | 45 µs | 2.1 µs | 21x |
| Diversity (7 genera) | 8 µs | 25 ns | 320x |
| Population MC (500) | 180 ms | 1.2 ms | 150x |

### Upstream Parity (Local vs barraCuda)

| Function | Local | Upstream | Delta |
|----------|-------|----------|-------|
| Shannon | 25.3 ns | 23.5 ns | upstream 7% faster |
| Simpson | 2.4 ns | 6.6 ns | local 2.7x faster (simpler for frequencies) |
| Bray-Curtis | 5.6 ns | 3.5 ns | upstream 38% faster |
| Hill 1K | 8.66 µs | 8.51 µs | ≈ same |
| LCG 1M steps | 102 µs | 102 µs | identical |
| Anderson L=50 | 636 µs | 636 µs | identical (delegated) |

---

## Part 5: Verification State

| Check | Result |
|-------|--------|
| `cargo fmt --all --check` | 0 diffs |
| `cargo clippy --workspace -- -D warnings -W clippy::pedantic` | 0 warnings |
| `cargo doc --workspace --no-deps` | Clean |
| `cargo test --workspace` | 368 passed, 0 failed |
| All 48 experiments | PASSED |
| Cross-validation tests | 4 new, all passing |
| `#![forbid(unsafe_code)]` | All lib crates |
| Files > 1000 lines | None |
