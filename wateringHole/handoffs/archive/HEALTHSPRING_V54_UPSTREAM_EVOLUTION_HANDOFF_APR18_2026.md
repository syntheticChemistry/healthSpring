<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring V54 — Upstream Evolution Handoff

**Date**: 2026-04-18
**From**: healthSpring (V54, guideStone Level 2)
**To**: primalSpring, barraCuda, toadStool, metalForge, biomeOS, all springs
**Purpose**: Communicate what healthSpring learned, what changed, what upstream teams should absorb, and what composition patterns emerged for NUCLEUS deployment via Neural API from biomeOS.

---

## 1. What Changed in V54

### guideStone Binary

healthSpring now ships `healthspring_guidestone` — a self-validating binary per
`GUIDESTONE_COMPOSITION_STANDARD` v1.0.0 from primalSpring v0.9.15.

The binary validates five certified properties without requiring NUCLEUS:
1. **Deterministic Output** — same inputs → same outputs across runs
2. **Reference-Traceable** — every constant and tolerance traces to a DOI or source
3. **Self-Verifying** — the binary validates its own science (no external test runner)
4. **Environment-Agnostic** — passes on any x86_64 Linux (no GPU, no network)
5. **Tolerance-Documented** — every numerical tolerance is named, justified, catalogued

When NUCLEUS is deployed, the guideStone additionally validates IPC parity via
`primalspring::composition` for generic barraCuda math and probes manifest
capabilities.

### barraCuda IPC Migration — Corrected Understanding

**Previous framing (V53):** "healthSpring has 9 pending barraCuda wire handlers —
upstream must add 9 JSON-RPC methods."

**Corrected framing (V54):** barraCuda's 32 IPC methods are generic math
primitives (`stats.mean`, `stats.std_dev`, `stats.variance`, `stats.correlation`,
`linalg.*`, `tensor.*`, `rng.*`). healthSpring's 9 domain-specific functions
(Hill dose-response, Shannon diversity, Simpson index, Chao1 richness,
Bray-Curtis dissimilarity, Anderson eigenvalue, MM-AUC, antibiotic perturbation,
SCR rate) are LOCAL compositions built from these primitives — they are
healthSpring's responsibility, not barraCuda's.

**Impact on barraCuda team:** The V53 ask for 9 new wire handlers is
**withdrawn**. barraCuda's existing 32-method surface is sufficient.
healthSpring's `math_dispatch` module is reframed as a "validation window"
comparing local compositions against direct library calls during development.

---

## 2. For primalSpring Team

### guideStone Integration Report

healthSpring is the first spring to implement `GUIDESTONE_COMPOSITION_STANDARD`
v1.0.0. Observations for the standard:

| Property | Implementation Notes |
|----------|---------------------|
| P1 (Deterministic) | LCG PRNG seed + tolerance-bounded comparison. Straightforward. |
| P2 (Traceable) | `PROVENANCE_REGISTRY` with 94 entries, all DOI-cited. Required significant upfront investment but pays off in auditability. |
| P3 (Self-Verifying) | **Not yet implemented** — needs `CHECKSUMS` generation infrastructure. This is the only blocked property. Suggest primalSpring provide a `checksums::generate()` helper. |
| P4 (Env-Agnostic) | `niche::NICHE_DOMAIN` comparison validates no hardware-specific paths leak into science. Easy to implement. |
| P5 (Tolerance-Documented) | `tolerances.rs` with 70+ named constants, `TOLERANCE_REGISTRY.md` cross-reference. Works well with `validate_tolerance_documented()`. |

### Suggested primalSpring Enhancements

1. **`checksums::generate(binary_path) -> ChecksumSet`** — P3 blocker for all springs.
2. **`ValidationResult` should support `skip` reason strings** — healthSpring
   skips NUCLEUS tests when offline; a structured skip reason would improve
   reporting.
3. **`CompositionContext::discover()` timeout** — when NUCLEUS is not running,
   discovery blocks. A configurable timeout (defaulting to e.g. 2s) would help
   bare-mode fallback.

### healthSpring's guideStone Readiness

| Level | Meaning | Status |
|-------|---------|--------|
| 1 | Validation exists | DONE (94 experiments, 948+ tests) |
| 2 | Properties documented | DONE (V54: `GUIDESTONE_PROPERTIES` in `niche.rs`) |
| 3 | Checksums certified | BLOCKED (P3 needs `checksums::generate()`) |
| 4 | Cross-substrate certified | READY (6 GPU ops, metalForge routing validated) |
| 5 | NUCLEUS-deployed | READY (ecoBin 0.9.0, deploy graph exists) |

---

## 3. For barraCuda Team

### What healthSpring Consumes (Current)

| barraCuda Module | healthSpring Usage | Version |
|------------------|--------------------|---------|
| `ops::HillFunctionF64` | GPU Hill dose-response | v0.3.12 |
| `ops::PopulationPkF64` | GPU population PK Monte Carlo | v0.3.12 |
| `ops::bio::DiversityFusionGpu` | GPU Shannon+Simpson+Pielou | v0.3.12 |
| `ops::health::MichaelisMentenBatchGpu` | GPU MM batch PK | v0.3.12 |
| `ops::health::ScfaBatchGpu` | GPU SCFA production | v0.3.12 |
| `ops::health::BeatClassifyGpu` | GPU beat classification | v0.3.12 |
| `stats::mean` | Uncertainty module | v0.3.12 |
| `rng::lcg_step` / `state_to_f64` | Deterministic PRNG | v0.3.12 |
| `special::tridiagonal_ql` | Anderson eigensolver | v0.3.12 |
| `numerical::OdeSystem` | ODE→WGSL codegen | v0.3.12 |

### What healthSpring Does NOT Need from barraCuda

The following domain functions are healthSpring local compositions — **not**
barraCuda IPC candidates:

| Function | Why Local |
|----------|-----------|
| `hill_dose_response` | Composes `stats.mean` + local Hill equation |
| `shannon_index` | Local `-Σ(p·ln(p))` over abundance vector |
| `simpson_index` | Local `Σ(p²)` over abundance vector |
| `chao1` | Local richness estimator from singletons/doubletons |
| `bray_curtis` | Local `Σ|a-b| / Σ(a+b)` dissimilarity |
| `anderson_eigenvalue` | Local Hamiltonian construction + `tridiagonal_ql` |
| `mm_auc` | Local Michaelis-Menten AUC integration |
| `antibiotic_perturbation` | Local ODE model of diversity recovery |
| `scr_rate` | Local skin conductance response rate |

### What healthSpring Contributes Back

| Contribution | Target | Status |
|--------------|--------|--------|
| 6 WGSL shaders (health domain) | `barracuda::ops::health::*` | Absorbed upstream |
| `OdeSystem` implementations (3 ODEs) | WGSL codegen corpus | Available |
| Dual-tower ionic bridge pattern | Composition patterns doc | New (V54) |

---

## 4. For toadStool / metalForge Teams

### Composition Patterns Learned

healthSpring's deployment reveals a **dual-tower ionic bridge** pattern for
NUCLEUS composition:

```
Tower A: Patient Data Enclave
  ├── beardog (identity + family seed)
  ├── nestgate (NCBI data, content-addressed cache)
  └── healthspring_primal (science capabilities)

Tower B: Analytics / Inference
  ├── toadstool (compute dispatch: CPU/GPU/NPU)
  ├── metalForge (substrate selection, PCIe P2P)
  └── barraCuda ecobin (generic math IPC)

Bridge: Strictly typed IPC over Unix sockets
  ├── Patient data NEVER leaves Tower A
  ├── Analytics results flow A→B for compute, B→A for results
  └── Provenance trio (sweetGrass/rhizoCrypt/loamSpine) anchors every crossing
```

This pattern is health-specific but generalizable: any spring handling
sensitive data (financial, legal, personal) can adopt the dual-tower model
with the ionic bridge enforcing data egress policy.

### metalForge Substrate Routing — Validated

All 9 `Workload` variants route correctly via `select_substrate()`:

| Workload | Substrate | Gate (LAN mesh) |
|----------|-----------|-----------------|
| HillSweep | GPU at >1K | Northgate RTX 5090 |
| PopulationPkBatch | GPU at >10K | Northgate |
| DiversityBatch | GPU at >1K | Northgate |
| MichaelisMentenBatch | GPU at >1K | Northgate |
| ScfaBatch | GPU at >100 | Northgate |
| BeatClassifyBatch | GPU at >100 | Northgate |
| OdeRK4Batch | CPU (codegen) | Eastgate |
| AndersenLattice | CPU | Eastgate |
| GenericCompute | CPU | Eastgate |

---

## 5. For biomeOS Team

### Neural API Deployment via biomeOS

healthSpring's niche consists of:
- 1 primal binary (`healthspring_primal`)
- 84+ JSON-RPC capabilities (62 science + 22 infra)
- 5 workflow graphs (patient assessment, TRT scenario, microbiome analysis,
  biosignal monitor, deploy)
- 1 guideStone binary (`healthspring_guidestone` — self-validation)

### Deploy Graph Status

`graphs/healthspring_niche_deploy.toml` defines a 10-stage DAG:
```
beardog → songbird → nestgate → toadstool → healthspring → petaltongue
```

All stages are defined with capability requirements. The graph is structurally
validated by exp118 (99 checks).

### ecoBin

Static-PIE x86_64-musl binary, 3.2 MB, harvested to `infra/plasmidBin/`.
Version 0.9.0. Zero C dependencies in default build. Passes
`healthspring_guidestone` bare properties on any x86_64 Linux.

---

## 6. For All Springs

### Patterns Worth Absorbing

| Pattern | Origin | Benefit |
|---------|--------|---------|
| **guideStone binary** | primalSpring v0.9.15 → healthSpring V54 | Self-validating deployable. No external test runner needed on clean machine. |
| **Validation window** (`math_dispatch`) | healthSpring V54 | Temporary module comparing local vs IPC math during development. Remove when IPC is the sole path. |
| **Domain vs generic IPC** | healthSpring V54 correction | Spring-specific science is local composition, not upstream IPC gaps. Only generic primitives (stats, linalg, tensor, rng) are IPC candidates. |
| **Dual-tower ionic bridge** | healthSpring V54 | Sensitive data stays in Tower A; compute in Tower B; typed IPC bridges. |
| **ValidationResult + exit codes** | primalSpring → healthSpring | `0` = all pass, `1` = failure, `2` = partial (bare-only mode). Standard for CI and clean-machine deployment. |

### Cross-Spring Shader Evolution (V54 State)

healthSpring contributes 6 WGSL shaders to `barracuda::ops::health::*`. All 6
compile to SM70 via coralReef. The ecosystem now has 794 shaders across all
springs. See `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md` for the full
pollination map.

---

## 7. Known Gaps for Upstream

| Gap | Owner | Priority | Notes |
|-----|-------|----------|-------|
| P3 CHECKSUMS infrastructure | primalSpring | High | Blocks guideStone Level 3 for all springs |
| `CompositionContext::discover()` timeout | primalSpring | Medium | Blocks clean bare-mode fallback |
| `ValidationResult` skip reason | primalSpring | Low | Improves reporting when NUCLEUS offline |
| coralReef f64 lowering for Hill shader | coralReef | Low | f32 transcendental workaround; 3 V17 shaders already use df64 |
| 10GbE cables for LAN mesh | Hardware | Medium | Switches + NICs installed; cables pending |

---

## 8. Fossil Record

All V1–V53 handoffs preserved in `wateringHole/handoffs/archive/`. The
Python → Rust → GPU → composition evolution is fully documented across
53 archived handoffs. This handoff (V54) is the first to address
the guideStone standard and the corrected barraCuda IPC framing.
