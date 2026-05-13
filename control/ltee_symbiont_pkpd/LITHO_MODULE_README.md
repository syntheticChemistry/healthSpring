<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# lithoSpore Module Candidate: `ltee-symbiont-pk`

**Paper**: Leonard SP et al. (2024) "One-step genome engineering of *E. coli*
colonizing the honeybee gut." mBio 15(3):e03342-23.
**DOI**: 10.1128/mbio.03342-23
**LTEE Queue ID**: B5
**Spring**: healthSpring V64e

---

## Module Layout

```
control/ltee_symbiont_pkpd/
├── expected_values.json       # Ground-truth parameters and tolerance envelopes
├── tolerances.toml            # Machine-readable tolerances for lithoSpore CI
├── ltee_symbiont_pkpd.py      # Python Tier 0 baseline (deterministic, no GPU)
├── benchmark_ltee_symbiont.json  # Benchmark metadata
├── LITHO_MODULE_README.md     # This file
└── __init__.py
```

**Rust Tier 1 binary**: `ecoPrimal/src/bin/validate_ltee_b5.rs`

---

## Reproduction Commands

### Python Baseline (Tier 0)

```bash
cd /path/to/healthSpring
python3 control/ltee_symbiont_pkpd/ltee_symbiont_pkpd.py
```

Expected output: `8/8 PASS`, printing colonization dynamics, production
kinetics, PK half-life, and Hill dose-response results with named
tolerances from `expected_values.json`.

**Commit**: Created in V63 (May 11, 2026)
**Determinism**: Pure-Python fallback if numpy unavailable — same results
regardless of numpy presence. No random seeds, no GPU.

### Rust Validation (Tier 1)

```bash
cd /path/to/healthSpring
cargo run --bin validate_ltee_b5 -- --format json
```

Expected output: JSON validation report with 8 checks, all passing.
Cross-validates colonization CFU at day 7, steady-state molecule
concentration, PK half-life, and knockdown fraction against the same
`expected_values.json` tolerances.

**Commit**: Created in V64 (May 12, 2026)

---

## Tolerance Summary

| Check | Expected | Tolerance |
|-------|----------|-----------|
| Colonization CFU day 7 | 3.1e8 | ±20% relative |
| Steady-state molecule ng | 425.0 | ±15% relative |
| PK half-life hours | 8.3 | ±10% relative |
| Knockdown at steady state | 0.85 | ±10% relative |
| Time to therapeutic level | 8.0 days | ±2.0 days |

---

## BLAKE3 Provenance

lithoSpore should BLAKE3-hash:
- `expected_values.json`
- `tolerances.toml`
- `ltee_symbiont_pkpd.py`
- The `validate_ltee_b5` binary (musl-static)

These four artifacts form the complete provenance chain for module
`ltee-symbiont-pk`.
