<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Context — healthSpring

## What This Is

healthSpring is a pure Rust health science compute library validating PK/PD,
microbiome, biosignal, endocrine, comparative medicine, drug discovery,
toxicology, and multi-scale simulation models against Python baselines.
GPU-accelerated via WGSL shaders through barraCuda. It is part of the
ecoPrimals sovereign computing ecosystem — a collection of self-contained
binaries that coordinate via JSON-RPC 2.0 over Unix sockets, with zero
compile-time coupling between components.

## Role in the Ecosystem

healthSpring is the sixth ecoPrimals **spring** — a validation target that
proves Python scientific baselines can be faithfully ported to pure Rust and
promoted to GPU acceleration. It bridges five other springs:

- **wetSpring**: shared gut microbiome Anderson, colonization resistance, hormesis
- **neuralSpring**: Hill dose-response, population PK surrogates
- **hotSpring**: lattice tissue modeling, Anderson spectral theory
- **airSpring**: environmental chemical exposure, hygiene hypothesis
- **groundSpring**: uncertainty propagation, ecological dose-response

The runtime primal (`healthspring_primal`) exposes all science as JSON-RPC 2.0
capabilities over Unix sockets for biomeOS graph composition.

## Technical Facts

- **Language**: 100% Rust, zero C dependencies (wgpu optional for GPU)
- **Architecture**: workspace with 3 library crates + 90 experiment binaries
  - `healthspring-barracuda` — core science library
  - `healthspring-forge` — metalForge hardware dispatch
  - `healthspring-toadstool` — pipeline orchestration
- **IPC**: JSON-RPC 2.0 over Unix domain sockets, 59 capabilities (46 science + 13 infrastructure); `normalize_method()` maps legacy-prefixed names before routing
- **License**: AGPL-3.0-or-later (scyBorg trio)
- **Tests**: 985+ (lib + proptest + IPC fuzz + doc + integration + experiment bins)
- **Coverage**: target 90% line (llvm-cov)
- **Clippy**: 0 warnings, 0 errors (pedantic + nursery + doc-markdown, all promoted to error), workspace-level `[lints]`
- **Validation harness**: `ValidationSink` trait (pluggable check output for experiments)
- **Unsafe code**: 0 (`forbid(unsafe_code)` in workspace lints)
- **MSRV**: 1.87 (Edition 2024)
- **Crates**: 93 workspace members (3 lib + 90 experiments)
- **GPU**: 6 WGSL shaders via barraCuda v0.3.12 (Hill, PopPK, Diversity, MM, SCFA, Beat); availability probe cached in `OnceLock`
- **Tracing**: library code uses `tracing` (no `println!` in lib)

## Key Capabilities (JSON-RPC)

- `capability.list` / `capabilities.list` (alias) / `primal.capabilities` — enumerate methods
- `science.pkpd.*` — Hill dose-response, compartmental PK, NLME (FOCE/SAEM), NCA
- `science.microbiome.*` — Shannon/Simpson diversity, Anderson gut lattice, SCFA, QS
- `science.biosignal.*` — Pan-Tompkins QRS, HRV, PPG SpO2, EDA, WFDB parsing
- `science.endocrine.*` — Testosterone PK, TRT outcomes, cardiac risk
- `science.diagnostic.*` — Patient assessment, population Monte Carlo, composite risk
- `science.toxicology.*` — Biphasic dose-response, toxicity landscape, hormetic optimum
- `science.simulation.*` — Mechanistic cell fitness, ecosystem Lotka-Volterra

## What This Does NOT Do

- No wet-lab data generation — consumes published datasets (SRA, NCBI, ChEMBL, PhysioNet)
- No clinical decision-making — computes models, does not prescribe treatments
- No proprietary data — all datasets public with documented accession numbers
- No ML training — delegates to neuralSpring for surrogates
- No raw GPU shader authoring — delegates to barraCuda for WGSL primitives

## Related Repos

| Repo | Relationship |
|------|-------------|
| **barraCuda** | GPU math library (path dep, v0.3.11) |
| **coralReef** | WGSL compiler pipeline |
| **toadStool** | Dispatch orchestration |
| **wetSpring** | Shared gut microbiome, hormesis framework |
| **hotSpring** | Anderson spectral theory, lattice methods |
| **wateringHole** | Ecosystem standards, handoffs, coordination |

## Evolution Path

```
Python baseline → Rust CPU validation → barraCuda GPU → coralReef/toadStool sovereign pipeline
```

## Design Philosophy

These binaries are built using AI-assisted constrained evolution. Rust's
compiler constraints (ownership, lifetimes, type system) reshape the fitness
landscape and drive specialization. Primals are self-contained — they know
what they can do, never what others can do. Complexity emerges from runtime
coordination, not compile-time coupling.

healthSpring treats IC50, Hill coefficients, and clearance as composable
primitives that chain from molecular through cellular to population and
ecosystem scales. Every tolerance is a named constant. Every expected value
traces to a Python run with commit hash.
