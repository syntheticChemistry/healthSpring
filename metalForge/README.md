# healthSpring metalForge

Cross-substrate dispatch validation for health application pipelines.

**Status**: Scaffold — awaiting Tier 1/2 implementations

---

## Structure

```
metalForge/
├── forge/        # metalForge test crate (Cargo project)
│   └── src/
├── gpu/          # GPU-specific benchmark scripts
├── npu/          # NPU-specific benchmark scripts (Akida)
├── shaders/      # Local WGSL (Write phase → absorbed into barraCuda)
└── benchmarks/   # Cross-substrate performance comparisons
```

## Dispatch Targets

| Substrate | Use Case | Driver |
|-----------|----------|--------|
| CPU | Baseline, portable, IEC 62304 reference | Native Rust |
| GPU (NVIDIA/AMD) | Population PK Monte Carlo, batch biosignal | BarraCUDA WGSL + wgpu |
| NPU (Akida) | Real-time ECG/PPG inference, edge health | ToadStool akida-driver |

## Write → Absorb → Lean Cycle

healthSpring follows the proven spring pattern:
1. **Write**: New WGSL shaders in `metalForge/shaders/` for health-specific ops
2. **Absorb**: Upstream validated shaders to `barraCuda` standalone library
3. **Lean**: Remove local copies, consume from `barraCuda` dependency

Target: 0 local WGSL (fully lean) once all primitives are upstreamed.
