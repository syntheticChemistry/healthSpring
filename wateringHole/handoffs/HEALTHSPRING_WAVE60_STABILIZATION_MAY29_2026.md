# healthSpring — Wave 60 Stabilization Handoff

**Date**: May 29, 2026
**From**: healthSpring (ironGate)
**Wave**: 60 — Stabilization (post-Triad Absorption)
**Version**: V65a
**Directive**: Stabilize, deepen niche, no new upstream API surface until 14 methods ship

---

## Context

primalSpring Wave 60 follow-up: upstream frozen. The Neural API Coordination Triad
(quorumSignal, rootPulse, waterFall) is spec'd but 14 new capability methods need
to be built across 6 primals before the triad is real. Springs are directed to
stabilize, deepen niche work, and publish.

**Upstream pending** (not healthSpring work):
- rhizoCrypt: `dag.branch/diff/merge/federate` (VCS operations for rootPulse)
- nestGate: `content.sync/push/replicate/fetch_heads` (waterFall Neural API)
- songbird: `mesh.discover_remotes/mirror/publish` (mesh federation)
- biomeOS: cross-gate `graph.execute` with `gate`/`relay` hints

---

## Actions Taken

### 1. New Scenario: `s_ltee_b5` (8 checks, ALL PASS)

LTEE B5 (symbiont PK/PD, Leonard et al. 2024) had a validated binary
(`validate_ltee_b5`) and Python baseline (`control/ltee_symbiont_pkpd/`) but
no entry in the 57-scenario guideStone ladder.

**Added**: `ecoPrimal/src/validation/scenarios/s_ltee_b5.rs`
- Track: PK/PD | Tier: Rust | Source: `ltee_b5`
- Phase 1 — Colonization dynamics: day-7 CFU, final CFU, doubling time, half-capacity time
- Phase 2 — Molecule production & efficacy: steady-state molecule, monotonicity, knockdown, PK half-life
- All 8 checks pass bit-identical against Python baseline constants

### 2. New Scenario: `s_barracuda_cpu_parity` (10 checks, ALL PASS)

exp040 (`control/validation/exp040_barracuda_cpu.py`) validated CPU math primitives
but had no dedicated guideStone scenario (only partial coverage via `s_barracuda_parity`
which is Live/IPC-tier).

**Added**: `ecoPrimal/src/validation/scenarios/s_barracuda_cpu_parity.rs`
- Track: PK/PD | Tier: Rust | Source: `exp040`
- Phase 1 — Statistical primitives: mean, std_dev
- Phase 2 — Hill dose-response: at IC50, at 2x IC50, steep (n=4)
- Phase 3 — Diversity indices: Shannon (uniform-4), Simpson (uniform-4), Chao1 (ge observed)
- Phase 4 — Distance: Bray-Curtis identical (=0), Bray-Curtis disjoint (=1)
- All 10 checks pass against analytical closed-form baselines

### 3. PRIMAL_GAPS #38 Substantially Closed

Gap #38 previously claimed "~30 Python baselines without Rust scenarios." With the
addition of s_ltee_b5 and s_barracuda_cpu_parity, scenario count is now **59**.
Coverage now spans all 10 tracks + LTEE B5 + barraCuda CPU parity. Only ~5
experiment binaries (exp084/085 GPU scaling, exp112-113 dispatch parity) lack
dedicated scenarios; these are exercised by CI composition jobs.

Three audit-answer tables in PRIMAL_GAPS.md updated to reflect the new state.
Gap #38 summary row marked **SUBSTANTIALLY CLOSED**.

---

### 4. Unit Tests Added (31 new tests, 1,021 → 1,052)

Four modules with zero inline tests now have comprehensive coverage:

| Module | Tests Added | Coverage |
|--------|-------------|----------|
| `composition/routing.rs` | 4 | ALL_CAPS completeness, routing table exhaustive, unknown fallback, no duplicates |
| `certification/bare.rs` | 7 | P1/P2/P4/P5 run-and-pass, niche identity, tier coverage, wire counts |
| `microbiome/anderson.rs` | 12 | Hamiltonian structure/symmetry/tridiagonal, IPR localized/extended, localization length edges, level-spacing ratio, colonization resistance, diagonalization |
| `gpu/cpu_fallback.rs` | 8 | One test per GpuOp variant (Hill/PopPK/Diversity/MM/SCFA/Beat), determinism, wang_hash range |

### 5. `data/fetch_qs_genes.py` Implemented

The only dataset in `data/manifest.toml` with `script = ""` now has a fetch script.
Queries NCBI Gene eutils for 40 gut-associated bacterial species × 19 QS gene
families. Produces structured JSON matrix for exp107 (QS-augmented Anderson
localization). Rate-limited at 350ms per query. `--dry-run` mode available.

---

## Current State

| Metric | Value |
|--------|-------|
| Validation scenarios | **59** (was 57) |
| Tests | **1,052** (was 1,021) |
| Clippy | Zero warnings (pedantic + nursery) |
| Deep debt | All 7 categories zero |
| Registry | 470+ methods |
| NUCLEUS | 13/13 on ironGate |
| Gate identity | `.gate` = `ironGate` |
| Forge sync | cascade-pull v2.0.0, 22-repo profile |

---

## Remaining Niche Depth Opportunities (no upstream dependency)

| Priority | Item | Status |
|----------|------|--------|
| P0 | Baseline JSON parity layer for science scenarios | Not started |
| P0 | Dataset fetch + SHA256 population (5 datasets) | All 5 datasets now have fetch scripts |
| P1 | LTEE E2 (HOLIgraph) + E4 (macrocyclic peptides) paper reproduction | Queued |
| P2 | Conditional GPU vs CPU tests for 6 WGSL shaders | Feature-gated, manual only |
| P2 | Absorb exp112-113 dispatch parity into scenario registry | CI-covered, scenarios optional |

---

## Files Changed

| File | Change |
|------|--------|
| `ecoPrimal/src/validation/scenarios/s_ltee_b5.rs` | **New** — LTEE B5 scenario (8 checks) |
| `ecoPrimal/src/validation/scenarios/s_barracuda_cpu_parity.rs` | **New** — barraCuda CPU parity scenario (10 checks) |
| `ecoPrimal/src/validation/scenarios/mod.rs` | Added 2 module declarations |
| `ecoPrimal/src/validation/scenarios/registry.rs` | Added 2 scenario entries (PK/PD track) |
| `ecoPrimal/src/composition/routing.rs` | Added 4 unit tests (ALL_CAPS + routing table) |
| `ecoPrimal/src/certification/bare.rs` | Added 7 unit tests (bare properties + niche identity) |
| `ecoPrimal/src/microbiome/anderson.rs` | Added 12 unit tests (lattice physics + edge cases) |
| `ecoPrimal/src/gpu/cpu_fallback.rs` | Added 8 unit tests (all GpuOp variants + determinism) |
| `data/fetch_qs_genes.py` | **New** — NCBI Gene fetch for QS matrix (40 species × 19 families) |
| `data/manifest.toml` | `qs_gene_matrix` script field populated |
| `docs/PRIMAL_GAPS.md` | Gap #38 closed, 3 audit tables updated, header → 59 scenarios, 1,052 tests |
| `README.md` | Status → Wave 60 Stabilization, 59 scenarios, 1,052 tests |
| `CHANGELOG.md` | New stabilization entry |
| `wateringHole/README.md` | Status + handoff table updated |
