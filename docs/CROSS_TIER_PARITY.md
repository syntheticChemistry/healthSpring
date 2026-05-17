# healthSpring Cross-Tier Parity

**Version**: 1.0
**Date**: May 17, 2026 (V64x)
**Reference**: `primalSpring/docs/VALIDATION_TIERS.md` — Tier 3 + parity sections

---

## Principle

Cross-tier parity proves that Python (Tier 0) and Rust (Tier 1) produce
numerically identical results for the same science. This is the three-layer
proof:

- **Tier 0**: Python baseline confirms science (peer-reviewed math)
- **Tier 1**: Rust implementation confirms implementation fidelity
- **Tier 2**: Live IPC confirms primal integration (NUCLEUS deployment)
- **Tier 3**: Provenance trio confirms attributable, immutable record

Parity between Tier 0 and Tier 1 proves the Rust port is faithful.

---

## B5 Leonard PK/PD — Cross-Tier Parity Report

**Paper**: Leonard SP et al. (2024) "One-step genome engineering of E. coli
colonizing the honeybee gut." mBio 15(3):e03342-23.

**lithoSpore Module**: `ltee-symbiont-pk` (Module 8 candidate)

### Parity Results (May 17, 2026)

| Check | Python (Tier 0) | Rust (Tier 1) | Rel Error | Tolerance | Status |
|-------|-----------------|----------------|-----------|-----------|--------|
| CFU at day 7 | 307,839,277.318 | 307,839,277.318 | 0.0 | 1e-4 | PASS |
| Final CFU | 999,494,652.935 | 999,494,652.935 | 0.0 | 1e-4 | PASS |
| Doubling time (h) | 13.863 | 13.863 | 0.0 | 1e-6 | PASS |
| t_half_max (days) | 7.675 | 7.675 | 0.0 | 1e-6 | PASS |
| SS molecule (ng) | 424.509 | 424.509 | 0.0 | 1e-3 | PASS |
| Monotonic post-d1 | true | true | 0.0 | exact | PASS |
| Knockdown SS | 0.8525 | 0.8525 | 0.0 | 1e-3 | PASS |
| PK half-life (h) | 8.318 | 8.318 | 0.0 | 1e-6 | PASS |

**Result**: 8/8 PASS. Python and Rust produce **bit-identical** IEEE 754 f64
results for all 8 checks. Zero floating-point divergence.

### How to Reproduce

```bash
# Tier 0: Python baseline
python3 control/ltee_symbiont_pkpd/ltee_symbiont_pkpd.py
# → writes benchmark_ltee_symbiont.json

# Tier 1: Rust validation
cargo run --bin validate_ltee_b5
# → compares against benchmark constants
```

### Artifacts

| File | Purpose |
|------|---------|
| `control/ltee_symbiont_pkpd/expected_values.json` | Paper parameters + expected outcomes |
| `control/ltee_symbiont_pkpd/benchmark_ltee_symbiont.json` | Python Tier 0 output values |
| `control/ltee_symbiont_pkpd/parity_report.json` | Structured parity comparison |
| `control/ltee_symbiont_pkpd/tolerances.toml` | lithoSpore tolerance definitions |
| `ecoPrimal/src/bin/validate_ltee_b5.rs` | Rust Tier 1 validation binary |
| `control/ltee_symbiont_pkpd/ltee_symbiont_pkpd.py` | Python Tier 0 baseline |

---

## Future Parity Candidates

healthSpring has 95 experiments with Python Tier 0 baselines and Rust
Tier 1 validation binaries. The following are high-priority candidates
for formal cross-tier parity reporting:

| Experiment | Domain | Priority | Notes |
|-----------|--------|----------|-------|
| exp001–exp005 | microbiome diversity | Medium | Shannon/Simpson/Pielou/Chao1 |
| exp010 | Anderson gut localization | Medium | Shared with groundSpring |
| exp020–exp025 | PK/PD core models | High | Hill, 1-/2-compartment, PBPK |
| exp060–exp065 | biosignal processing | Medium | Pan-Tompkins, HRV, SpO2 |
| exp080 | gut-brain serotonin | Medium | Tryptophan pathway |

---

## lithoSpore Module 8 Readiness

B5 is confirmed ready for lithoSpore Module 8 ingestion:

- [x] `expected_values.json` — complete with paper provenance
- [x] `benchmark_ltee_symbiont.json` — Python baseline output
- [x] `parity_report.json` — structured cross-tier comparison
- [x] `tolerances.toml` — CI tolerance definitions
- [x] Rust binary passes all 8 checks with zero divergence
- [x] Stable method names: `science.pkpd.hill_dose_response`, `science.pkpd.one_compartment_pk`
- [ ] Coordinate with lithoSpore on exact `expected_values.json` schema version
