<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V50 — Ecosystem & Primal Evolution Handoff

**Date**: 2026-04-11
**From**: healthSpring
**To**: barraCuda, toadStool, primalSpring, biomeOS, neuralSpring, coralReef, ecosystem

---

## The Validation Ladder

healthSpring has proven a reusable four-tier validation ladder that every
spring can follow. This is the pattern for evolving from scientific validation
to sovereign primal composition:

```
Tier 0: Python control     → peer-reviewed science (DOI-cited baselines)
Tier 1: Rust CPU            → faithful port (f64-canonical, tolerance-documented)
Tier 2: GPU parity          → barraCuda WGSL (CPU vs GPU bit-identical)
Tier 3: metalForge dispatch → NUCLEUS routing (cross-substrate, PCIe P2P)
Tier 4: Primal composition  → IPC dispatch parity (JSON-RPC wire = direct Rust)
```

**Key insight**: At Tier 4, both Python and Rust become validation targets for
the composition layer. The science doesn't change — we validate that the
NUCLEUS composition patterns faithfully reproduce it through IPC dispatch.

---

## For barraCuda

### Absorption Complete

All 6 local WGSL shaders have been absorbed upstream:

| Shader | barraCuda location | Status |
|--------|-------------------|--------|
| Hill dose-response f64 | `barracuda::shaders::math::hill_f64.wgsl` | Absorbed |
| Population PK f64 | `barracuda::shaders::science::population_pk_f64.wgsl` | Absorbed |
| Diversity fusion f64 | `barracuda::shaders::bio::diversity_fusion_f64.wgsl` | Absorbed |
| Michaelis-Menten batch f64 | `barracuda::ops::health::MichaelisMentenBatchGpu` | Absorbed |
| SCFA batch f64 | `barracuda::ops::health::ScfaBatchGpu` | Absorbed |
| Beat classify batch f64 | `barracuda::ops::health::BeatClassifyGpu` | Absorbed |

### Local Copies Retained Until `TensorSession`

healthSpring retains local shader copies solely for the single-encoder fused
pipeline path (`execute_fused_local`). Once `TensorSession` API is available:

1. **Tier A** (Hill, PopPK, Diversity): remove local copies, switch to
   `barracuda::ops::{HillFunctionF64, PopulationPkF64, DiversityFusionGpu}`
2. **Tier B** (MM, SCFA, BeatClassify): remove once `barracuda::ops::health`
   covers the fused pipeline path
3. Remove `shader_for_op()` and `shaders` module entirely

### Ecosystem Request

- **`TensorSession` API**: healthSpring's fused pipeline (`execute_fused_local`)
  orchestrates 3 ops in a single GPU submission. `TensorSession` would let us
  express this as a declarative pipeline instead of manual encoder management.
  Priority: enables local shader removal across all springs.

- **`barracuda::stats::correlation::std_dev` → public**: Currently used via
  delegation from `uncertainty::std_dev`. If this becomes a first-class
  `barracuda::stats::std_dev`, all springs benefit.

### Pin

barraCuda v0.3.11 (`7f6649f`, 2026-04-10). Path dep for local dev.

---

## For toadStool

### What healthSpring Uses

| Feature | API | Status |
|---------|-----|--------|
| CPU pipeline dispatch | `execute_cpu()` | Live, 51 tests |
| GPU dispatch routing | `execute_auto()` | Live, GPU-mappability probe |
| Streaming dispatch | `execute_streaming()` | Live, callback-based |
| Pipeline stages | `StageOp::{HillSweep, AucTrapezoidal, BrayCurtisPairwise, ...}` | 8 V16 ops |
| Substrate routing | `Substrate::{Cpu, Gpu, Npu}` | metalForge integration |

### Tolerance Migration

healthSpring V50 migrated all inline `1e-10` / `1e-6` values in toadStool
and metalForge tests to `healthspring_barracuda::tolerances::MACHINE_EPSILON`
and `tolerances::HALF_LIFE_POINT`. This pattern (named constants in a
centralized registry) should be adopted by other springs.

### Ecosystem Request

- toadStool stages that healthSpring doesn't use yet: variance batch, FFT
  pipeline, spectral analysis. As springs evolve, these become relevant.

---

## For primalSpring

### Proto-Nucleate Alignment Confirmed

`primalSpring/graphs/downstream/healthspring_enclave_proto_nucleate.toml`
defines a dual-tower ionic composition. healthSpring's deploy graph now
accurately reflects this:

- `fragments = ["tower_atomic", "nest_atomic"]`
- `particle_profile = "neutron_heavy"`
- `proto_nucleate = "healthspring_enclave_proto_nucleate"`
- `[graph.bonding]` with `bond_type = "ionic"`, `trust_model = "dual_tower_enclave"`

### Composition Validation as a Pattern

The exp112–117 composition experiments are a reusable pattern. They validate:

1. **Dispatch parity** (exp112/113): `dispatch_science(method, params)` produces
   bit-identical results to direct Rust function calls
2. **Registry completeness** (exp114): every registered capability dispatches
3. **Proto-nucleate alignment** (exp115): socket conventions, capability list
4. **Provenance lifecycle** (exp116): data session begin/record/complete
5. **Wire round-trip** (exp117): full JSON-RPC serialization fidelity

**Recommendation**: Abstract this into a `composition_validation` crate in
primalSpring that any spring can depend on for Tier 4 testing.

### Gaps Fed Back

See `docs/PRIMAL_GAPS.md` — 9 gaps identified, 6 fixed (V48/V49/V50), 3
remaining:

| Gap | Status | Blocker |
|-----|--------|---------|
| Ionic bridge enforcement | Blocked | BearDog `crypto.ionic_bond` capability |
| Inference canonical namespace | Partial | primalSpring/Squirrel alignment |
| Discovery method naming | V50 dual fallback | Songbird canonical names |

### Capability-First Routing

healthSpring's `primal.forward` now routes by capability first:

```
discover_by_capability_public(target)  // try capability domain match
  .or_else(|| discover_primal(target)) // fall back to name-based
```

This should become the ecosystem standard. Name-based routing is a bootstrap
convenience; capability-based is the NUCLEUS pattern.

---

## For biomeOS

### Deploy Graph Ready

`graphs/healthspring_niche_deploy.toml` is biomeOS-deployable:

```
biomeos deploy --graph graphs/healthspring_niche_deploy.toml
```

Deploy order: BearDog → Songbird → rhizoCrypt → loamSpine → sweetGrass →
NestGate → ToadStool → healthSpring → (Squirrel, optional)

### 80+ Capabilities Registered

healthSpring advertises 80+ capabilities via `capability.list`:
- 62 `science.*` methods (PK/PD, microbiome, biosignal, endocrine, diagnostic,
  clinical, comparative, discovery, toxicology, simulation)
- 5 `health.*` proto-nucleate aliases
- 13 infrastructure capabilities (provenance, compute, data, inference routing)
- Health probes: `health.liveness`, `health.readiness`

### Workflow Graphs

| Graph | Pattern | Use |
|-------|---------|-----|
| `healthspring_patient_assessment.toml` | ConditionalDag | 4-track diagnostic → composite risk |
| `healthspring_trt_scenario.toml` | Sequential | Testosterone PK → outcomes → visualization |
| `healthspring_microbiome_analysis.toml` | Sequential | Diversity → Anderson → resistance → SCFA |
| `healthspring_biosignal_monitor.toml` | Continuous @ 250 Hz | ECG → HRV → stress → arrhythmia → fusion |

### ecoBin Harvest

healthSpring ecoBin (static-PIE x86_64-musl, 2.5 MB) has been harvested to
`infra/plasmidBin/healthspring/`. Zero C dependencies in default feature set.

---

## For neuralSpring / Squirrel

### Optional Squirrel Integration

healthSpring V50 adds `squirrel_b` to the deploy graph with `required = false`.
When Squirrel reaches ecoBin compliance and exposes stable `inference.*`
capabilities, healthSpring gains AI inference for clinical decision support
with zero code changes — Squirrel discovers neuralSpring as a provider.

### What healthSpring Needs

- `inference.complete`: Clinical narrative generation from diagnostic results
- `inference.embed`: Patient similarity search for population analysis
- `inference.models`: Available model inventory for routing decisions

### Blocker

Squirrel ecoBin compliance (pure Rust, zero C deps) + stable inference
capability set.

---

## For coralReef

### Sovereign Dispatch Ready

healthSpring's `gpu/sovereign.rs` module is prepared for coralReef native
compilation. The `sovereign-dispatch` feature flag exists but is gated on
coralReef device availability. Once coralReef can compile WGSL → native
binary, healthSpring replaces the `strip_f64_enable()` workaround with
sovereign dispatch.

---

## Composition Patterns Learned

### 1. Capability-First, Name-Second

Every cross-primal interaction should resolve by capability domain, not by
primal name. Names are bootstrap convenience; capabilities are the contract.
healthSpring's V50 `primal.forward` demonstrates this.

### 2. Proto-Nucleate Aliases Enable Domain Vocabulary

The 5 `health.*` aliases (`health.pharmacology`, `health.gut`, `health.cardiac`,
`health.clinical`, `health.genomics`) let biomeOS graphs reference health
capabilities without knowing the `science.*` method hierarchy. Other springs
should define their own domain aliases.

### 3. Validation Ladder is the Evolution Path

Every spring should implement Tiers 0–4 in sequence. Skipping tiers creates
composition debt. The tier order maps exactly to the evolution path:

```
Python baseline → Rust validation → barraCuda GPU → toadStool dispatch →
  primal composition → NUCLEUS deployment
```

### 4. Provenance Registry Pattern

A static data table with one entry per experiment, carrying `git_commit`,
`run_date`, `exact_command`, and `baseline_source` (DOI citation), ensures
every validation is traceable and reproducible. This pattern should be
standardized across springs.

### 5. Bonding Policy in Deploy Graphs

Cross-atomic compositions must declare bond type, trust model, and encryption
tiers per boundary. healthSpring's dual-tower ionic model with de-identified
aggregate bridge is the first working example.

---

## What Springs Can Harvest

| Pattern | Location | Reusable? |
|---------|----------|-----------|
| `ValidationHarness` | `ecoPrimal/src/validation/` | Yes — any spring |
| `OrExit<T>` trait | `ecoPrimal/src/validation/or_exit.rs` | Yes — zero-panic binaries |
| Provenance registry | `ecoPrimal/src/provenance/` | Yes — per-spring data table |
| Tolerance centralization | `ecoPrimal/src/tolerances.rs` | Pattern — per-spring registry |
| IPC dispatch table | `ecoPrimal/src/ipc/dispatch/` | Pattern — method → function map |
| Composition experiments | `experiments/exp112–117/` | Pattern — Tier 4 validation |
| Deploy graph + bonding | `graphs/healthspring_niche_deploy.toml` | Template |
| Niche YAML manifest | `niches/healthspring-health.yaml` | Template |

---

## Summary for Each Team

| Team | Action |
|------|--------|
| **barraCuda** | Ship `TensorSession` → unlocks local shader removal across all springs |
| **toadStool** | Named tolerance pattern available for adoption |
| **primalSpring** | Abstract composition validation into reusable crate; confirm `discovery.find_by_capability` in Songbird; pick canonical inference namespace |
| **biomeOS** | healthSpring niche deploy graph is ready; 80+ capabilities registered |
| **neuralSpring** | Squirrel optional node waiting; `inference.*` capability set needed |
| **coralReef** | Sovereign dispatch feature flag ready; waiting on device availability |
| **All springs** | Follow the 4-tier validation ladder; adopt provenance registry pattern |
