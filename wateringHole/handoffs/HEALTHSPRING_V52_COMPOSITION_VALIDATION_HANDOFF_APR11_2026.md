<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HEALTHSPRING V52 — Composition Validation Handoff

**Date**: 2026-04-11
**From**: healthSpring V52
**To**: barraCuda, toadStool, primalSpring, biomeOS
**barraCuda**: v0.3.11 (rev 7f6649f)
**Previous**: V51 Hardened Composition Patterns

---

## Summary

V52 completes the shift from Rust-validates-Python to **NUCLEUS-validates-composition**.
Python baselines validated Rust science; Rust validated Python baselines; now Tier 5
experiments validate that the NUCLEUS composition patterns themselves are internally
consistent — deploy graphs align with proto-nucleate declarations, fragment metadata
is accurate, bonding policies match, and capability surfaces are complete.

---

## Changes

### P0: Zero clippy warnings restored
- Fixed `match_wildcard_for_single_variants` in `gpu/sovereign.rs` — the wildcard
  arm in the not-yet-wired sovereign dispatch now explicitly names `GpuOp::HillSweep`.

### P1: Typed IPC clients wired into production
- `PrimalClient.call()` now uses `resilient_send` (retry with exponential backoff)
  instead of single-attempt `try_send`. New `try_call()` for single-attempt paths.
- `handle_primal_forward` in `server/routing.rs` migrated from raw `rpc::resilient_send`
  to `PrimalClient` — gains health probe fallback chains and structured discovery.
- Gap #11 in `PRIMAL_GAPS.md` resolved.

### P1: Deploy graph validation (Tier 5)
- **exp118** (`exp118_composition_deploy_graph_validation`): 99 checks validating:
  - TOML parsing of `healthspring_niche_deploy.toml`
  - Fragment metadata (`tower_atomic`, `nest_atomic`, `neutron_heavy`, proto-nucleate ref)
  - Required/optional node presence (beardog, songbird, healthspring, nestgate, trio, etc.)
  - Bonding policy (ionic, `dual_tower_enclave`, encryption tiers)
  - Capability coverage — all 58+ science and 14+ infra capabilities in deploy graph
  - Squirrel optional node (`required=false`, `by_capability=inference`)
  - Primal identity constants match deploy graph
- Gap #12 in `PRIMAL_GAPS.md` added and resolved.
- Added to CI composition job (Tier 4 + Tier 5).

### P1: tolerances.py policy clarification
- `control/tolerances.py` docstring updated to document it as an **intentional subset**
  of `tolerances.rs`. Removed misleading "update BOTH files" language. The Rust file
  is the authoritative source of truth; Python contains only constants used by
  cross-validation baselines.

### P1: GPU tests on every PR
- `test-gpu` CI job now runs `barracuda-ops` feature tests on every PR (not just
  weekly schedule). Full `--features gpu` still runs on weekly schedule only.

### P2: CI improvements
- `cargo llvm-cov` scope expanded from `--lib` to full workspace (lib + integration).
- New `bench` CI job compiles benchmarks on every PR (regression check).
- Integration tests now run as a separate CI step.
- `specs/CODE_QUALITY_AUDIT.md` section 10 updated to reflect current CI pipeline.

### Provenance
- exp118 provenance record added to `records_infra.rs`.
- Total experiments: **90** (84 science + 7 composition Tier 4/5).
- Total tests: **985+** (845 lib + proptest + IPC fuzz + 37 integration + 90 experiment
  bins + 9 doc-tests + exp118's 99 checks).

---

## Ecosystem Asks

### To primalSpring
- Validate that `healthspring_enclave_proto_nucleate.toml` trust_model
  (`btsp_enforced`) aligns with deploy graph trust_model (`dual_tower_enclave`).
  healthSpring uses the deploy graph value as canonical — confirm or update
  the proto-nucleate.

### To BearDog
- Ionic bond runtime (`crypto.ionic_bond`, `crypto.verify_family`) still needed
  for dual-tower enforcement. Gap #2 unchanged.
- BTSP server endpoint still needed for end-to-end handshake. Gap #10 unchanged.

### To Squirrel / neuralSpring
- Canonical inference namespace (`inference.*` vs `model.*` vs `ai.*`) coordination
  still pending. healthSpring supports all three. Gap #4.

---

## Validation Evidence

```
cargo clippy --workspace -- -D warnings -W clippy::pedantic -W clippy::nursery  → 0 warnings
cargo fmt --check --all                                                         → pass
cargo test --workspace                                                          → 985+ tests, 0 failures
exp118_composition_deploy_graph_validation                                      → 99/99 PASS
```
