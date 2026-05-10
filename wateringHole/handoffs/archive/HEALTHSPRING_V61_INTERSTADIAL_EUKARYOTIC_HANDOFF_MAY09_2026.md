# healthSpring V61 — Interstadial Eukaryotic Evolution Handoff

**Date:** May 9, 2026  
**Sprint:** Interstadial Primordial Extinction — Spring Evolution Wave  
**primalSpring:** v0.9.25 (pinned)  
**Architecture:** Eukaryotic (UniBin)

## What Changed

### Architecture
- **UniBin binary** (`healthspring_unibin`): Single binary with `certify`, `validate`, `serve`, `status`, `version` subcommands
- **Certification organelle** (`certification/`): Absorbed `healthspring_guidestone` binary into library module
- **Validation scenarios** (`validation/scenarios/`): 16 scenarios across 8 tracks (PkPd, Microbiome, Biosignal, Endocrine, Comparative, Discovery, Composition, Toxicology)
- **Composition module** (`composition/`): HealthCompositionContext wrapping primalSpring's CompositionContext with health-domain typed accessors

### IPC Evolution
- **CompositionContext migration**: PrimalClient, InferenceClient, discover_primal(), discover_by_capability_public() all deprecated with `note` pointing to CompositionContext
- **IPC provenance trio**: rhizocrypt (DAG), loamspine (ledger/merkle), sweetgrass (braid/analytics)
- **Expanded BarraCudaClient**: stats_variance, stats_correlation, rng_normal
- **Default features flipped**: `default = []` (IPC-first), `barracuda-lib` opt-in

### Fossilization
- `fossilRecord/guidestone_prokaryotic_may2026/`: Archived guidestone binary sources
- `fossilRecord/experiments_prokaryotic_may2026/`: Archived 16 absorbed experiment main.rs files

### Code Quality
- primalSpring v0.9.25 pinned (was path dep, optional)
- All `#[deprecated]` include `note`
- All `#[allow]` include `reason`
- Zero bare suppressions, zero TODO/FIXME/HACK/DEBT
- Zero clippy warnings (`--workspace --all-targets`)
- All tests pass

## Scenario Registry

| ID | Track | Tier | Source |
|----|-------|------|--------|
| hill-dose-response | PkPd | Rust | exp001 |
| one-compartment-pk | PkPd | Rust | exp002 |
| population-pk | PkPd | Rust | exp005 |
| michaelis-menten | PkPd | Rust | exp077 |
| diversity-indices | Microbiome | Rust | exp010 |
| anderson-gut | Microbiome | Rust | exp011 |
| pan-tompkins-qrs | Biosignal | Rust | exp020 |
| hrv-metrics | Biosignal | Rust | exp021 |
| testosterone-pk | Endocrine | Rust | exp030 |
| canine-il31 | Comparative | Rust | exp100 |
| matrix-scoring | Discovery | Rust | exp090 |
| composition-parity | Composition | Live | exp119 |
| live-provenance | Composition | Live | exp120 |
| live-health | Composition | Live | exp121 |
| barracuda-parity | Composition | Live | exp122 |
| nucleus-parity | Composition | Both | exp123 |

## For Upstream (primalSpring)

- healthSpring is now on primalSpring v0.9.25 UniBin pattern
- CompositionContext wired for all IPC; HealthCompositionContext adds health-domain typed accessors
- ValidationResult + ScenarioMeta patterns adopted (16 scenarios)
- certify() returns ValidationResult with exit_code() semantics

## For Downstream Springs

- Pattern: `certification/` organelle for self-validating binary-to-library absorption
- Pattern: `validation/scenarios/` with Track taxonomy + Tier classification
- Pattern: `composition/context.rs` wrapping primalSpring context with domain-typed accessors
- Pattern: `ipc/provenance/` per-trio modules for clean provenance separation
- Pattern: `fossilRecord/` for archiving pre-extinction code with provenance READMEs

## Next Steps

- Extract server module from `healthspring_primal` into library for UniBin `serve` subcommand
- Add Tier 2/3 live scenario testing when NUCLEUS is deployed
- Expand validation scenarios to cover remaining tracks (toxicology experiments)
- GPU provenance tracking via IPC (post-sovereign dispatch wiring)
