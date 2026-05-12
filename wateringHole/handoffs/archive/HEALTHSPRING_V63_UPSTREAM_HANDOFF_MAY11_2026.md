<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V63 — Upstream Primal & Spring Handoff

**Date**: May 11, 2026
**Version**: V63
**primalSpring**: v0.9.25 (pinned)
**Tests**: 999 (868 lib + 131 integration/workspace)
**Capabilities**: 87 JSON-RPC methods
**Architecture**: Eukaryotic UniBin (`healthspring_unibin` / `healthspring` alias) + IPC-first defaults + 4 NUCLEUS workloads

---

## What Changed in V63

### Deep Debt Sweep: Hardcoded String Centralization

The remaining hardcoded primal-related strings in JSON-RPC normalization, socket
discovery, and visualization wiring have been centralized into `primal_names.rs`:

| Constant / Module | Purpose | Replaces |
|-------------------|---------|----------|
| `wire_prefix::HEALTHSPRING` | JSON-RPC method normalization prefix | `"healthspring."` literals in `rpc.rs` |
| `wire_prefix::BARRACUDA` | JSON-RPC method normalization prefix | `"barracuda."` literals in `rpc.rs` |
| `wire_prefix::BIOMEOS` | JSON-RPC method normalization prefix | `"biomeos."` literals in `rpc.rs` |
| `BIOMEOS_DIR_NAME` | Filesystem convention (`"biomeos"`) | String literals in `socket.rs`, `client.rs`, `provenance.rs` |
| `FALLBACK_SOCKET_DIR` | Default socket dir (`/tmp/biomeos`) | Local `FALLBACK_SOCKET_DIR` in `socket.rs` |
| `SONGBIRD_SOCKET_PATHS` | Discovery service socket paths | Local `SONGBIRD_PATHS` in `capabilities.rs` |

**Upstream relevance (barraCuda / biomeOS / songbird)**: When these primals evolve
their socket naming or wire-prefix conventions, healthSpring only needs a constant
update in one file. No grep-and-replace across the codebase.

### Deep Debt Sweep: Parameter Struct Refactoring

Four functions with 7–8 positional parameters have been refactored to accept
named-field structs. This is idiomatic Rust API design and eliminates
argument-order bugs at all 21 call sites.

| Struct | Domain | Fields | Replaces |
|--------|--------|--------|----------|
| `DosingRegimen` | PK/PD | `dose_mg`, `f_bioavail` | 2 loose params in `population_pk_cpu` |
| `PopulationPkVariability` | PK/PD | `cl: LognormalParam`, `vd: LognormalParam`, `ka: LognormalParam` | 3 loose params in `population_pk_monte_carlo` |
| `ToxicityModelParams` | Toxicology | `hill_n`, `km`, `clearance_threshold` | 3 loose params in `compute_toxicity_landscape` |
| `AntibioticSimConfig` | Microbiome | `h0`, `depth`, `k_decline`, `k_recovery`, `treatment_days`, `total_days`, `dt` | 7 loose params in `antibiotic_perturbation` |

Convenience constants: `pop_baricitinib::REGIMEN` and `pop_baricitinib::VARIABILITY`
for the canonical baricitinib dosing scenario.

**Upstream relevance (all springs)**: These structs are public API. Any spring or
NUCLEUS workload calling healthSpring PK/PD, toxicology, or microbiome functions
should migrate to struct-based invocation. The positional parameter signatures no
longer exist.

### Foundation Thread 3 Seeded

`sporeGarden/foundation` Thread 3 (Immunology/Drug Discovery) is now seeded:
- Expression: `expressions/IMMUNO_DRUG_DISCOVERY.md` (Papers 12, 13, 22; 5 springs; drug discovery pipeline)
- Data sources: `data/sources/thread03_immuno.toml` (18 sources)
- Data targets: `data/targets/thread03_immuno_targets.toml` (12 targets)
- `THREAD_INDEX.toml` updated: 6/10 threads now seeded (7+ needed for exit gate)

**Upstream relevance (sporeGarden / all springs)**: Thread 3 cross-references
wetSpring (cytokine Anderson), neuralSpring (Fajgenbaum MATRIX), healthSpring
(PK/PD + biosignal + microbiome), groundSpring (spectral), and ludoSpring (fraud
detection). All springs with immunological or drug discovery pipelines should
review the thread for validated data source / target alignment.

---

## Audit Results

| Metric | Value |
|--------|-------|
| Tests | **999** (868 lib + 131 integration/workspace) |
| Clippy warnings | **0** |
| Unsafe blocks | **0** (`forbid(unsafe_code)`) |
| TODO/FIXME/HACK | **0** in production code |
| `unwrap()` / `panic!` in production | **0** |
| Mocks in production | **0** (all mocks isolated to `#[cfg(test)]`) |
| Files >800 lines | **0** |
| Hardcoded primal name strings | **0** (all centralized in `primal_names.rs`) |
| External dependencies | All standard Rust ecosystem (serde, tokio, clap, wgpu, thiserror, tracing, ureq) |
| Python baselines | **53** scripts + **53** notebooks (intentionally retained as Tier 0 controls) |

---

## Evolution Patterns for Upstream Teams

### For barraCuda Team

1. **Wire prefix convention**: healthSpring now uses `primal_names::wire_prefix::BARRACUDA`
   for JSON-RPC method normalization. If barraCuda changes its method prefix,
   healthSpring adapts via a single constant update.
2. **Parameter struct pattern**: Consider adopting named-field structs for barraCuda
   GPU dispatch parameters (e.g., `GpuBatchConfig` instead of positional `batch_size,
   block_size, precision`). healthSpring proved this pattern across 21 call sites
   with zero regressions.

### For biomeOS Team

1. **Socket discovery**: healthSpring uses `primal_names::BIOMEOS_DIR_NAME` and
   `primal_names::FALLBACK_SOCKET_DIR` for all socket path construction. biomeOS
   v3.51 integration is live (`composition.status`, `method.register`).
2. **NUCLEUS workloads**: 4 healthSpring workloads are staged: PK validation,
   biosignal validation, microbiome validation, certification. All use `healthspring`
   binary alias for invocation.

### For All Springs

1. **UniBin pattern**: healthSpring's `healthspring_unibin` (`certify`, `validate`,
   `serve`, `status`, `version`) is the reference implementation for spring UniBin
   architecture. New springs should follow this pattern.
2. **IPC-first defaults**: `default = []` with optional `barracuda-lib` feature
   for direct library linkage. This ensures springs can compose without compile-time
   coupling.
3. **Fossil record**: Prokaryotic-era sources are preserved under `fossilRecord/`
   with dated subdirectories. When absorbing experiment mains into `validation/scenarios/`,
   archive the originals rather than deleting them.
4. **Python → Rust (UniBin) → Primal (NUCLEUS composition)**: This is the validated
   three-stage pipeline. Python baselines establish truth (Tier 0), Rust CPU validates
   parity (Tier 1), GPU validates acceleration (Tier 2), NUCLEUS validates composition
   (Tier 3–5). healthSpring has completed all 5 tiers across 95 experiments.

### For primalSpring Team

1. **CompositionContext**: healthSpring's `HealthCompositionContext` wraps primalSpring's
   `CompositionContext` with domain-typed accessors. This pattern should be documented
   as the canonical way for springs to extend composition context.
2. **Capability registry**: 87 methods registered via `ALL_CAPABILITIES`. The
   `capability_registry.toml` config drives drift detection against the live primal.
3. **guideStone Level 5**: healthSpring maintains Level 5 lineage via
   `healthspring_unibin certify`. The certification organelle pattern (absorbing the
   legacy `healthspring_guidestone` binary) should be adopted by other springs.

### For NUCLEUS / neuralAPI Deployment

1. **Workload TOMLs**: 4 workloads staged under `projectNUCLEUS/workloads/healthspring/`.
2. **Binary alias**: `healthspring` (identical to `healthspring_unibin`) for workload
   invocation. plasmidBin release binaries: 2.9M `healthspring`, 3.1M `healthspring_primal`.
3. **Composition graph**: `graphs/healthspring_niche_deploy.toml` defines the full
   niche deployment topology including skunkBat defense node.

---

## Forward Work (Low Priority)

- **LTEE papers** (B5, E2, E4): 3 papers queued in `PAPER_REVIEW_QUEUE.md` — long-term evolution experiment validation. Not blocking any current work.
- **Foundation threads**: 6/10 seeded; 1 more needed for exit gate. Thread 4 (Neuroscience) or Thread 5 (Ecology) are natural candidates.
- **NLME on GPU**: FOCE/SAEM currently CPU-only. barraCuda v0.4.x could enable GPU-accelerated population PK estimation.
- **NestGate integration**: Data provider spec exists (`specs/NESTGATE_DATA_PROVIDER.md`); not yet wired.

---

## Supersedes

- [V62 handoff](archive/HEALTHSPRING_V62_UPSTREAM_HANDOFF_MAY10_2026.md) — now archived
