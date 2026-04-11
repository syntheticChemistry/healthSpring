# Changelog

All notable changes to healthSpring are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses internal versioning (V-series) for development milestones.

## [Unreleased] — V48

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
- **Tier 4 composition validation** — 5 new experiments (exp112–exp116, 73 checks total):
  - `exp112_composition_pkpd` — IPC dispatch vs direct Rust for PK/PD (12 checks).
  - `exp113_composition_microbiome` — IPC dispatch vs direct Rust for microbiome (10 checks).
  - `exp114_composition_health_triad` — capability surface + domain coverage (17 checks).
  - `exp115_composition_proto_nucleate` — proto-nucleate alignment + socket resolution (20 checks).
  - `exp116_composition_provenance` — provenance lifecycle + session round-trip (14 checks).
- `ecoPrimal/tests/integration_composition.rs` — 12 composition integration tests.
- `specs/COMPOSITION_VALIDATION.md` — Tier 4 validation specification.
- CI `composition` job running all composition tests and experiments.
- ecoBin static-PIE binary (x86_64-musl, 2.5 MB) harvested to `infra/plasmidBin/healthspring/`.
- CI cross-compile job now uploads ecoBin artifacts and verifies static linkage.

### Changed
- `health.readiness` now gates `ready` on science dispatch status instead of
  hardcoded `true`.
- Inline tolerance literals in integration tests replaced with named constants
  from `tolerances.rs`.
- `build_semantic_mappings()` refactored from large `json!` macro to
  programmatic map construction (avoids recursion limit with 80+ entries).

### Fixed
- YAML niche manifest now includes all 89 capabilities the binary serves
  (was missing 14 infrastructure capabilities).

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
