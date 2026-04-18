# HEALTHSPRING V50 — Composition Evolution Handoff

**Date**: 2026-04-11
**Version**: V50 (0.10.0)
**Phase**: Rust validation → Primal composition validation
**From**: healthSpring
**To**: primalSpring, ecosystem

---

## Context

healthSpring has completed three validation tiers:

1. **Python → Rust**: 54 Python baselines faithfully ported to Rust with full
   provenance (git commit, exact command, DOI citations, reproducible runs).
2. **Rust → barraCuda GPU**: Six WGSL shaders (Hill, PopPK, Diversity, MM,
   SCFA, BeatClassify) absorbed upstream; CPU/GPU parity validated.
3. **Rust → IPC dispatch parity**: Composition experiments (exp112–exp117)
   validate that JSON-RPC dispatch produces bit-identical results to direct
   Rust calls.

V50 moves into **primal composition validation** — the Python and Rust
baselines are now validation targets for NUCLEUS patterns. Springs prove
that primal compositions faithfully reproduce the science validated in
tiers 1–3.

---

## V50 Changes (this handoff)

### Code Quality

- **4 clippy errors fixed**: exp114 (`if_same_then_else`, `unnecessary_map_or`),
  exp112 (`unnecessary_struct_initialization`), exp116 (`boolean logic bug`)
- **7 clippy warnings fixed**: doc backticks (`BearDog`, `PopPK`), decimal
  bitwise operand (hex literal), long first doc paragraph
- **`cargo fmt --all`**: 579-line drift resolved
- **`provenance/registry.rs` split**: 1224 LOC → 3 files (registry.rs ~80 LOC,
  records_science.rs ~460 LOC, records_infra.rs ~720 LOC). Public API
  preserved with `all_records()`, `registry_len()`.
- **exp112/exp113 refactored**: `main()` extracted into helper functions to
  satisfy `too_many_lines` (pedantic, 100-line limit)
- **Inline tolerances migrated**: toadstool/metalForge tests now use named
  `healthspring_barracuda::tolerances::` constants instead of magic numbers

### Composition Evolution

- **`primal.forward` migrated**: capability-based discovery first, name-based
  fallback. Callers sending `primal.forward` with a capability domain as
  `target` now get routed by capability.
- **`tower_atomic.rs` dual-method discovery**: `find_capability()` tries
  `discovery.find_by_capability` first, falls back to legacy
  `net.discovery.find_by_capability` (PRIMAL_GAPS §3 partial resolution).
- **Squirrel optional node**: `squirrel_b` added to
  `healthspring_niche_deploy.toml` with `required = false`. biomeOS deploys
  Squirrel when available; healthSpring degrades gracefully without it.
- **PRIMAL_GAPS.md updated**: V50 resolutions for §3 (discovery naming) and
  §9 (Squirrel in deploy graph).

### Validation State

- **954 `#[test]` functions**: all passing
- **89 experiment binaries**: all follow `ValidationHarness` → `h.exit()` pattern
- **9 doc-tests**: all passing
- **CI gate**: 90% line coverage (`cargo llvm-cov --workspace --lib --fail-under-lines 90`)
- **Zero** `unsafe`, `#[allow()]`, `TODO`, `FIXME` in production code

---

## Evolution Path Forward

### Composition Validation Tier (current)

The three-tier validation ladder:

```
Tier 1: Python baseline    → source of truth (control/)
Tier 2: Rust validation    → direct function call parity (experiments/)
Tier 3: IPC dispatch       → JSON-RPC wire parity (exp112–exp117)
Tier 4: Primal composition → NUCLEUS graph deploys reproduce Tiers 1–3
```

Tier 4 validates that biomeOS deploying the healthSpring proto-nucleate graph
(`healthspring_enclave_proto_nucleate.toml`) reproduces the same science
results as direct Rust calls and Python baselines. The validation targets
are now Python AND Rust — both serve as ground truth for composition testing.

### ecoBin Harvest

healthSpring binaries are candidates for `infra/plasmidBin` once:
- `cargo tree` audit confirms zero C deps in the default feature set
- `wgpu` (optional GPU feature) is excluded from the ecoBin submission
- Static musl build tested: `cargo build --target x86_64-unknown-linux-musl`
- Single binary UniBin mode confirmed (`healthspring_primal serve|version|capabilities`)

### GPU Shader Absorption Complete

All 6 local WGSL shaders absorbed by barraCuda. Local copies retained until
`TensorSession` API enables fused pipeline migration. Removal sequence
documented in `gpu/mod.rs` § Shader Removal Plan (V49).

### Remaining Primal Gaps

| Gap | Status | Blocker |
|-----|--------|---------|
| Ionic bridge enforcement | Blocked | BearDog `crypto.ionic_bond` |
| Inference canonical namespace | Partial | primalSpring/Squirrel alignment |
| Discovery method naming | V50 dual fallback | Songbird canonical names |
| Squirrel in deploy graph | V50 optional node | Squirrel ecoBin compliance |

---

## For primalSpring

1. **Validate proto-nucleate alignment**: `healthspring_enclave_proto_nucleate.toml`
   now has matching deploy graph metadata (V49) and optional Squirrel (V50).
2. **Discovery naming**: healthSpring sends `discovery.find_by_capability`
   first — please confirm Songbird supports this method name.
3. **Inference namespace**: healthSpring handles `inference.*` and `model.*` —
   pick the canonical one.
4. **Composition validation pattern**: exp112–exp117 is a reusable pattern
   for other springs. Consider absorbing into primalSpring's composition
   validation framework.

---

## For ecosystem (wateringHole)

- healthSpring demonstrates the **three-tier validation ladder** that other
  springs can follow: Python → Rust → IPC → Composition.
- The `provenance/registry.rs` pattern (static data table with git commit,
  run date, exact command per experiment) is available for cross-spring use.
- The `primal.forward` capability-first pattern should become the ecosystem
  standard for cross-primal routing.
