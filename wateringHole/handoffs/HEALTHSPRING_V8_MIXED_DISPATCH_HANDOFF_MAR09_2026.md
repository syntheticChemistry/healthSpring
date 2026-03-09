# healthSpring V8 — Mixed Hardware Dispatch Handoff

**Date**: March 9, 2026
**From**: healthSpring
**To**: barraCuda, toadStool, coralReef, biomeOS
**Status**: V8 — 34 experiments, 211 tests, 526 binary checks

---

## Part 1: What V8 Adds

| Component | What | Validation |
|-----------|------|------------|
| toadStool `StageOp` expansion | `PopulationPk` + `DiversityReduce` map to GPU ops | 4 new tests |
| CPU vs GPU parity matrix | 3 kernels x 3 scales through Pipeline | Exp060 (27/27) |
| metalForge `DispatchPlan` | Per-stage substrate assignment + transfer planning | 6 new tests |
| metalForge `Workload` expansion | `BiosignalFusion`, `EndocrinePk` variants | Routing tests |
| Exp061 mixed dispatch | NUCLEUS topology validation (CPU+GPU+NPU) | 22/22 |
| Exp062 PCIe transfer | Gen3/4/5 bandwidth, overhead analysis | 26/26 |
| Absorption annotations | `ABSORPTION CANDIDATES` in all key modules | — |

---

## Part 2: Absorption Tables

### For barraCuda Team

| healthSpring Source | What to Absorb | Why |
|---------------------|----------------|-----|
| `barracuda/src/gpu.rs` `GpuContext` | Persistent device + queue pattern | Avoids per-op adapter enumeration |
| `barracuda/src/gpu.rs` `execute_fused()` | Single-encoder multi-op dispatch | 1.1-3x speedup vs individual dispatch |
| `barracuda/src/gpu.rs` `execute_cpu()` | CPU reference for any `GpuOp` | Parity testing, fallback |
| `barracuda/src/gpu.rs` `strip_f64_enable()` | WGSL f64 preprocessor | naga rejects `enable f64;`, wgpu uses feature flag |
| `barracuda/src/gpu.rs` `shader_for_op()` | GpuOp -> WGSL source registry | Shader management pattern |
| `metalForge/forge/src/lib.rs` `Substrate`, `Workload` | Workload classification enum | Dispatcher needs to know workload type |
| `metalForge/forge/src/lib.rs` `Capabilities::discover()` | Runtime GPU/NPU probe | Hardware discovery pattern |

### For toadStool Team

| healthSpring Source | What to Absorb | Why |
|---------------------|----------------|-----|
| `toadstool/src/pipeline.rs` `execute_auto()` | metalForge-routed pipeline | Auto CPU/GPU/NPU selection per stage |
| `toadstool/src/pipeline.rs` `stage_to_workload()` | Stage -> Workload mapping | Connects pipeline stages to dispatch routing |
| `toadstool/src/stage.rs` `StageOp::PopulationPk` | GPU-native population PK stage | Complete stage-to-GpuOp mapping |
| `toadstool/src/stage.rs` `StageOp::DiversityReduce` | GPU-native diversity stage | Batch community analysis |
| `metalForge/forge/src/dispatch.rs` `DispatchPlan` | Per-stage substrate + transfer plan | Pipeline scheduling with topology awareness |
| `metalForge/forge/src/dispatch.rs` `plan_dispatch()` | Workload -> Nest assignment | NUCLEUS-aware dispatch planning |

### For coralReef Team

| healthSpring Source | What to Absorb | Why |
|---------------------|----------------|-----|
| `barracuda/src/gpu.rs` `strip_f64_enable()` | Should become a naga preprocessor pass | Every f64 shader needs this workaround |
| `shaders/health/*.wgsl` (3 files) | Shader library patterns | f64 workgroup dispatch conventions |

### For biomeOS Team

| healthSpring Source | What to Absorb | Why |
|---------------------|----------------|-----|
| `metalForge/forge/src/nucleus.rs` NUCLEUS topology | Tower/Node/Nest hierarchy | Hardware graph representation |
| `metalForge/forge/src/transfer.rs` transfer planning | PCIe P2P / HostStaged / NetworkIpc | Inter-device data movement |
| `metalForge/forge/src/dispatch.rs` `DispatchPlan` | Graph annotations for substrate assignment | Pipeline stage -> Nest mapping |

---

## Part 3: New Experiments

| Experiment | Checks | What It Validates |
|------------|:------:|-------------------|
| Exp060 | 27 | CPU vs GPU parity for Hill (50-5K), PopPK (100-10K), Diversity (10-1K) through toadStool Pipeline |
| Exp061 | 22 | Mixed hardware dispatch: GPU+NPU+CPU pipeline, threshold routing, substrate transitions, PCIe P2P |
| Exp062 | 26 | PCIe transfer: method selection, Gen3/4/5 bandwidth, realistic workload sizes, overhead analysis |

---

## Part 4: Key Design Decisions

1. **PRNGs differ between CPU and GPU** — CPU uses LCG, GPU uses xorshift32+Wang hash. Population PK comparison is statistical (mean AUC ± 25%), not bitwise.

2. **DispatchPlan is topology-aware** — uses `Tower`/`Node`/`Nest` to select devices and plan transfers. Falls back to device 0 (CPU) if requested substrate is unavailable.

3. **PCIe P2P bypasses CPU** — GPU->NPU transfers use direct DMA when both devices are on the same `Node`. This avoids the CPU roundtrip for biosignal data flowing from GPU preprocessing to NPU inference.

4. **Threshold routing** — small workloads (<100-1000 elements depending on type) stay on CPU to avoid GPU dispatch overhead. Thresholds are configurable via `DispatchThresholds`.

5. **Workload expansion** — `BiosignalFusion` and `EndocrinePk` workloads route to NPU and CPU respectively. `prefers_npu()` now includes fusion workloads.

---

## Part 5: Metrics

| Metric | V7 | V8 |
|--------|:--:|:--:|
| Experiments | 31 | **34** |
| Unit tests | 201 | **211** |
| Binary checks | 418 | **526** |
| Forge tests | 27 | **33** |
| toadStool tests | 13 | **17** |
| GPU kernels via pipeline | 1 (Hill) | **3 (Hill, PopPK, Diversity)** |
| metalForge modules | 3 (lib, nucleus, transfer) | **4 (+dispatch)** |
| Workload variants | 5 | **7** |

---

This handoff is unidirectional: healthSpring -> barraCuda / toadStool / coralReef / biomeOS. No response expected.
