# Changelog

All notable changes to healthSpring are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses internal versioning (V-series) for development milestones.

## V49 — 2026-04-10 — Composition Audit Remediation

### Added
- `health.genomics` capability + proto-nucleate alias → `science.microbiome.qs_gene_profile`.
  All five `health.*` proto-nucleate aliases are now wired.
- `exp117_composition_ipc_roundtrip` — Tier 4 IPC wire protocol validation (71 checks):
  round-trip serialization, proto-nucleate alias dispatch, health probe routing, full
  capability surface completeness.
- Deploy graph fragment metadata: `fragments`, `particle_profile`, `proto_nucleate`,
  `[graph.bonding]` with bond type, trust model, encryption tiers per atomic boundary.
- Bonding policy matrix documented in `tower_atomic.rs` module docs.
- WGSL shader removal plan documented in `gpu/mod.rs` (all 6 absorbed upstream).
- 36 new provenance registry entries for non-Python experiments → 89 total (100% coverage).
- `PRIMAL_GAPS.md` gaps §8 (deploy fragment metadata — fixed) and §9 (Squirrel in deploy).
- V49 handoff: `wateringHole/handoffs/HEALTHSPRING_V49_COMPOSITION_AUDIT_HANDOFF_APR10_2026.md`.
- V49 classification table for 21 tolerance-exempt experiments in `TOLERANCE_REGISTRY.md`.

### Changed
- barraCuda pin updated from v0.3.7 (`c04d848`) to v0.3.11 (`7f6649f`).
- `uncertainty::std_dev` delegated to `barracuda::stats::correlation::std_dev`.
- Sovereign dispatch error messages now name each unsupported op + reference EVOLUTION_MAP.
- `provenance::tests::registry_complete` split into `registry_covers_all_python_scripts`
  and `registry_covers_all_experiments` (decoupled from 1:1 Python file assumption).
- All spec docs (`BARRACUDA_REQUIREMENTS`, `EVOLUTION_MAP`, `TOLERANCE_REGISTRY`) updated to V49.

### Fixed
- `cross_validate.py`: `TOL_AUC` aliased to `LEVEL_SPACING_RATIO` (0.02) instead of
  `AUC_TRAPEZOIDAL` (0.01). Fixed to use correct registry constant.
- CI: added `pip install -r control/requirements.txt` before Python cross-validation.
- `PRIMAL_GAPS.md` gap §1 resolved — `health.*` science aliases implemented (option a).

## V48 — 2026-04-10

### Added
- `docs/PRIMAL_GAPS.md` — primal composition gap registry per NUCLEUS alignment protocol.
- `CHANGELOG.md` — presentation standard compliance.
- `inference.*` capability aliases alongside `model.*` for Squirrel/neuralSpring alignment.
- `health.*` proto-nucleate aliases (`health.pharmacology`, `health.clinical`,
  `health.de_identify`, `health.aggregate`) registered in `ALL_CAPABILITIES`.
- `resilient_send` in `ipc/rpc.rs` — retry with exponential backoff for retriable IPC errors.
- Infrastructure capabilities in YAML niche manifest (`niches/healthspring-health.yaml`).
- Cross-compile CI targets (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`).
- GPU CI job enabled as weekly scheduled run.
- **Tier 4 composition validation** — 5 new experiments (exp112–exp116, 73 checks total).
- ecoBin static-PIE binary (x86_64-musl, 2.5 MB) harvested to `infra/plasmidBin/healthspring/`.
- CI cross-compile job with artifact upload and static linkage verification.

### Changed
- `health.readiness` gates on science dispatch status.
- `build_semantic_mappings()` refactored to programmatic map construction.

### Fixed
- YAML niche manifest now includes all capabilities the binary serves.

## V47 — 2026-04-07

### Added
- `HEALTHSPRING_V47_UPSTREAM_ABSORPTION_HANDOFF_APR07_2026.md` — upstream
  absorption, health triad, deploy graph references.
- `HEALTHSPRING_V46_COMPOSITION_CONVERGENCE_HANDOFF_APR07_2026.md`.
- `HEALTHSPRING_V45_CAPABILITY_SYNC_IPC_FUZZ_HANDOFF_APR07_2026.md`.

## V44 — 2026-03-24

### Added
- 83 validation experiments with `ValidationHarness` (hotSpring pattern).
- JSON-RPC primal server (`healthspring_primal`) with 75+ capabilities.
- 6 WGSL shaders for GPU dispatch (Hill, PopPK, Diversity, MM, SCFA, BeatClassify).
- barraCuda integration: 6 GPU ops + CPU primitives (stats, rng, health modules).
- Provenance trio session API (begin/record/complete/status).
- Tower Atomic integration (BearDog + Songbird discovery).
- petalTongue visualization IPC push with clinical scenario nodes.
- `metalForge/forge` — NUCLEUS dispatch and composition.
- `toadstool` — compute pipeline staging.
- Proptest IPC fuzzing (protocol, transport, dispatch).
- `cargo-deny` configuration banning C/native dependencies.
- 90%+ library line coverage enforced in CI.
- SPDX `AGPL-3.0-or-later` headers on all source files.
