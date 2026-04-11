<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V49 — Composition Audit Handoff

**Date**: 2026-04-10
**From**: healthSpring
**To**: primalSpring, barraCuda, biomeOS, ecosystem

---

## Summary

V49 is a comprehensive audit remediation release. healthSpring was audited
against all ecosystem standards (wateringHole), sibling spring conventions,
and the NUCLEUS atomic model. This handoff documents all changes made.

## Evolution context

Python was the validation target for Rust. Now Rust and Python are both
validation targets for ecoPrimal NUCLEUS composition patterns. The spring
validates not just scientific correctness but that the composition wire
(IPC dispatch, proto-nucleate aliases, capability registration, deploy
graphs) faithfully reproduces the same results as direct Rust calls.

## Changes

### Critical fixes

1. **cross_validate.py tolerance bug**: `TOL_AUC` was aliased to
   `LEVEL_SPACING_RATIO` (0.02) instead of `AUC_TRAPEZOIDAL` (0.01).
   Fixed to use the correct registry constant.

2. **CI Python reproducibility**: Added `pip install -r control/requirements.txt`
   before running `cross_validate.py` in the validate job. NumPy 2.1.3 is
   now enforced in CI, not just locally.

3. **Provenance registry coverage**: Added 35 `ProvenanceRecord` entries
   covering all 88 workspace experiments. Non-Python experiments are
   classified by category (GPU bench, demo, composition, analytical).
   Registry is now 1:1 with workspace crate count.

### Standards alignment

4. **barraCuda version**: Updated pin from v0.3.7 (c04d848) to v0.3.11
   (7f6649f). CI pin check updated. `specs/BARRACUDA_REQUIREMENTS.md`
   header updated to V49.

5. **health.genomics capability**: Added to `ALL_CAPABILITIES` and wired
   via `resolve_proto_alias` to `science.microbiome.qs_gene_profile`.
   Proto-nucleate gap §1 is now resolved — all five `health.*` proto
   aliases are implemented.

6. **Deploy graph fragment metadata**: Both `healthspring_niche_deploy.toml`
   and `healthspring_biomeos_deploy.toml` now declare `fragments`,
   `particle_profile`, `proto_nucleate`, and `[graph.bonding]` with bond
   type, trust model, and encryption tiers per atomic boundary.

7. **Bonding policy documentation**: `tower_atomic.rs` module docs now
   include the bonding policy matrix (Tower A, Tower B, ionic bridge,
   within-tower IPC) with bond types, trust models, and encryption tiers.

### Code quality

8. **uncertainty::std_dev delegation**: Replaced local Bessel-corrected
   std_dev with `barracuda::stats::correlation::std_dev`. Eliminates the
   last stats duplication between healthSpring and barraCuda.

9. **WGSL shader removal plan**: Documented in `gpu/mod.rs` — all six
   local shaders are absorbed upstream; removal sequence tied to
   TensorSession availability.

10. **Sovereign dispatch expansion**: Error messages now name the specific
    unsupported op and reference `specs/EVOLUTION_MAP.md` for tracking.

### Composition validation

11. **exp117_composition_ipc_roundtrip**: New Tier 4 composition experiment
    validating JSON-RPC wire protocol round-trips, proto-nucleate alias
    resolution, health probe routing, capability surface completeness, and
    the new `health.genomics` alias. Added to CI composition job.

### Documentation

12. **PRIMAL_GAPS.md**: Updated to V49. Gap §1 marked fixed (aliases added).
    New gaps §8 (deploy fragment metadata — fixed) and §9 (Squirrel not in
    deploy graphs — blocked on Squirrel maturity) added. Summary matrix
    updated.

13. **TOLERANCE_REGISTRY.md**: Added V49 classification table for 21
    tolerance-exempt experiments with category and rationale.

## Metrics (V49)

| Metric | Value |
|--------|-------|
| Workspace experiments | 89 (88 + exp117) |
| Provenance registry entries | 88 (100% coverage) |
| `#[test]` functions | 920+ |
| Science capabilities | 62 |
| Proto-nucleate aliases | 5/5 (health.pharmacology/genomics/clinical/de_identify/aggregate) |
| barraCuda version | v0.3.11 |
| Local WGSL shaders | 6 (absorbed upstream, removal pending TensorSession) |
| llvm-cov target | 90% line (CI-enforced) |

## Upstream actions needed

- **barraCuda**: Expose `TensorSession` API for fused pipeline migration
- **primalSpring**: Confirm `health.*` alias approach (option a) vs proto revision (option b)
- **Songbird**: Standardize discovery method names (`discovery.*` vs `net.discovery.*`)
- **Squirrel**: Reach ecoBin compliance for deploy graph inclusion
- **BearDog/NestGate**: Evolve ionic bond + egress fence capabilities
