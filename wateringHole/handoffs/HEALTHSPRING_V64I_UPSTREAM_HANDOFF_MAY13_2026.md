# healthSpring V64i — Deep Debt Resolution + Evolution Sprint Handoff

**From**: healthSpring
**To**: primalSpring, upstream primals
**Date**: May 13, 2026
**Version**: V64i

---

## Summary

healthSpring has completed a comprehensive deep debt resolution and evolution
sprint. All 7 audit categories are at zero debt. The codebase passes clippy
pedantic+nursery with zero warnings and zero errors.

---

## Audit Results

### Priority 1: Deep Debt (TODO/FIXME/HACK)

**Result: ZERO.** No TODO, FIXME, HACK, `unimplemented!`, `todo!`, or
production `panic!` markers exist in the codebase.

### Priority 2: Modern Idiomatic Rust

**Result: CLEAN.** Edition 2024, rust-version 1.87. Clippy pedantic+nursery
passes with zero warnings on `--all-targets`. Fixes applied:
- `match` → `if let` (3 sites)
- `unwrap()` → `f64::total_cmp` (3 sites)
- `i32 as f64` → `f64::from` (7 sites)
- `unwrap_or` → `unwrap_or_else` (1 site)
- `&Option<T>` → `Option<&T>` (1 site)
- Doc backticks for primal names (7 sites)
- `const fn` promotions (2 functions)
- Long function decomposition (1 scenario → 9 phase functions)

### Priority 3: External Dependencies

**Result: CLEAN in default build.** Zero C dependencies in `default = []`.
- `ring` (via `ureq` → `rustls`): gated behind `nestgate` feature
- `wgpu` GPU backends: gated behind `gpu` feature
- All other deps pure Rust: serde, serde_json, tracing, clap, thiserror, bytemuck

### Priority 4: Large Files

**Result: NONE.** No file exceeds 800 LOC. Largest: 597 lines.

### Priority 5: Unsafe Code

**Result: ZERO.** `#![forbid(unsafe_code)]` enforced at lib.rs and workspace
`[lints.rust]` level. No `unsafe` blocks anywhere.

### Priority 6: Hardcoding

**Result: ZERO production hardcoding.** All primal routing via
`primal_names::*` constants and capability-based discovery. Fixed: hardcoded
`"healthSpring"` → `crate::PRIMAL_NAME` in provenance session JSON.

### Priority 7: Mocks

**Result: CLEAN.** All mocks isolated to `#[cfg(test)]` modules. Zero
production mocks.

---

## Audit Questions

### Do we have Python baselines for barraCuda CPU parity?

**Yes.** `control/scripts/exp040_barracuda_cpu.py` provides baselines for
stats.mean, stats.std_dev, stats.variance, stats.correlation, Hill dose-response,
Shannon diversity, Simpson diversity, Chao1, and Anderson diagonalization.
All are matched by Rust validation scenarios. CPU benchmarks in
`benches/cpu_parity.rs` cover population Monte Carlo at 500 and 5000 patients.

### Do we have Kokkos/Galaxy/LAMMPS benchmarks for GPU parity?

**No, and this is by design.** barraCuda uses sovereign WGSL shaders compiled
via coralReef — it is not porting Kokkos, LAMMPS, or other external frameworks.
GPU parity is validated against Python baselines (SciPy/NumPy) through the
`gpu` feature gate. There is no external GPU benchmark suite to compare against
because the compute model is fundamentally different (shader dispatch via
toadStool, not CUDA/HIP kernels).

### What have we not implemented, verified, validated, or tested?

**~30 Python baselines** under `control/` lack corresponding Rust validation
scenarios:

| Track | Missing Scenarios |
|-------|------------------|
| PK/PD | exp003 (two-compartment), exp004 (mAb transfer), exp006 (PBPK) |
| Microbiome | exp012 (colonization), exp013 (antibiotic response), exp078-080 (SCFA, QS, lattice) |
| Biosignal | exp022 (SpO2), exp023 (stress fusion), exp081-082 (ppg/eda) |
| Endocrine | exp031-038 (TRT outcomes, decline, andropause, etc.) |
| Comparative | exp101-106 (species PK, canine disorders, allometric) |
| Discovery | exp091-094 (HTS, IC50, fibrosis modeling) |
| Toxicology | exp098-099 (landscape detail, hormesis detail) |
| Simulation | exp111 (causal terrarium) |

These are valid science baselines. The existing 18 Rust scenarios cover the
critical structural identity checks. Adding remaining scenarios is incremental
work, not architectural debt.

### What papers remain unreviewed?

**2 LTEE papers queued:**
- **E2**: Mardikoraem & Woldring 2025 — "HOLIgraph" holistic CDR engineering
- **E4**: Woldring Lab 2024 — macrocyclic peptide display platforms

45/45 main-track papers are complete.

### What datasets should we examine?

5 datasets in `data/manifest.toml`, all lacking SHA256 checksums:

| Dataset | Script | Status |
|---------|--------|--------|
| `qs_gene_matrix` | **No script** | Manual assembly, needs automation |
| `mitbih` | `fetch_mitbih.sh` | Scripted, unchecksummed |
| `chembl_hill_panel` | `fetch_chembl.sh` | Scripted, unchecksummed |
| `hmp_16s` | `fetch_hmp_16s.sh` | Scripted, unchecksummed |
| `geo_androgen_receptor` | `fetch_geo_ar.sh` | Scripted, unchecksummed |

As the larger project comes together, these datasets should be:
1. Fetched and BLAKE3-hashed for `manifest.toml`
2. Stored via NestGate `storage.store` with content-addressed keys
3. Provenance-tracked through the Nest Atomic pipeline
4. Validated against `expected_values.json` tolerances

---

## New Gaps

| # | Gap | Severity | Action |
|---|-----|----------|--------|
| 38 | ~30 Python baselines without Rust scenarios | LOW | Incremental, not architectural |
| 39 | LTEE E2+E4 queued | LOW | Review when provenance work matures |
| 40 | Dataset SHA256 + `qs_gene_matrix` fetch | MEDIUM | Populate post-fetch |
| 41 | No GPU parity benchmarks | N/A | Sovereign WGSL, not framework port |

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 902 (847 lib + 55 integration/doc) |
| Scenarios | 18 |
| Capabilities | 88 |
| Clippy pedantic+nursery | 0 warnings, 0 errors |
| `unsafe` | 0 (forbidden) |
| TODO/FIXME/HACK | 0 |
| Production mocks | 0 |
| Files > 800 LOC | 0 |
| External C deps (default) | 0 |
| Edition | 2024 |
