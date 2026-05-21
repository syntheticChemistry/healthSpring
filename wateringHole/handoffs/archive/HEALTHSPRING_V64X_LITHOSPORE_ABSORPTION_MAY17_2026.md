# healthSpring V64x — lithoSpore Audit Absorption Handoff

**Date**: May 17, 2026 (PM)
**From**: healthSpring
**To**: primalSpring, lithoSpore, delta springs
**Version**: V64x
**Upstream Audit**: Wave 20 PM — lithoSpore Audit Absorption & Ecosystem Evolution

---

## What healthSpring Absorbed

### 1. Degradation Behavior Documented

`docs/DEGRADATION_BEHAVIOR.md` — Per-domain degradation table covering all
13 capability domains consumed by healthSpring.

**Pattern**: `NestComposition` tracks `steps_attempted`/`steps_succeeded`,
returns `NestStatus::{Complete, Partial, Unavailable}`. Each step degrades
independently. Science computation **never** fails due to primal unavailability.

Key degradation paths:
- rhizoCrypt down → `local-{experiment}` session ID (ephemeral-only)
- loamSpine down → DAG valid, unanchored (no permanence)
- sweetGrass down → provenance without attribution envelope
- BearDog down → unsigned provenance (still valid)
- NestGate down → local BLAKE3 fallback
- barraCuda down → direct Rust library call (no IPC needed for CPU)
- biomeOS down → manual multi-call chain instead of signal dispatch

All IPC paths return `Result` — callers pattern-match on `Err` and degrade.
Zero `unwrap()`/`panic!()` in production code (lint-enforced).

### 2. Stability Tier Awareness Absorbed

`docs/STABILITY_TIERS.md` — Two-part analysis:

**IPC Capabilities**: All routing strings in `composition/routing.rs` use
canonical stable names from `capability_registry.toml`. Zero local aliases.
No GAP-36 wire-name alias documentation needed.

**Niche Science Methods**: All 58 `science.*` methods classified:
- **15 stable** — consumed by lithoSpore Module 8, cross-spring validation,
  and experiment harnesses (e.g., `science.pkpd.hill_dose_response`,
  `science.microbiome.shannon_index`, `science.biosignal.pan_tompkins`)
- **41 evolving** — under active research, may change between major waves
- **2 internal** — helper methods with no downstream consumers

### 3. Cross-Tier Parity Proven for B5

`docs/CROSS_TIER_PARITY.md` + `control/ltee_symbiont_pkpd/parity_report.json`

**Result**: Python (Tier 0) and Rust (Tier 1) produce **bit-identical** IEEE
754 f64 results for all 8 B5 checks:

| Check | Python | Rust | Rel Error |
|-------|--------|------|-----------|
| CFU day 7 | 307,839,277.318 | 307,839,277.318 | 0.0 |
| Final CFU | 999,494,652.935 | 999,494,652.935 | 0.0 |
| Doubling time (h) | 13.863 | 13.863 | 0.0 |
| t_half_max (days) | 7.675 | 7.675 | 0.0 |
| SS molecule (ng) | 424.509 | 424.509 | 0.0 |
| Monotonic post-d1 | true | true | exact |
| Knockdown SS | 0.8525 | 0.8525 | 0.0 |
| PK half-life (h) | 8.318 | 8.318 | 0.0 |

Both tiers use identical algorithms: logistic growth, Euler ODE integration,
Hill dose-response. No floating-point divergence at any tolerance level.

### 4. Trio Transaction Semantics Confirmed

healthSpring's `NestComposition` already implements all 5 upstream rules
from `PROVENANCE_TRIO_INTEGRATION_GUIDE.md`:

- [x] DAG without braid = valid partial provenance
- [x] Braid without spine = valid attribution without permanence
- [x] No rollback — DAG sessions are append-only
- [x] Partial state reported (`NestStatus` + per-field presence)
- [x] Never error on partial provenance — domain logic always completes

No code changes needed — pre-existing implementation was already conformant.

---

## lithoSpore Module 8 Readiness

B5 Leonard PK/PD is ready for lithoSpore Module 8 coordination:

| Artifact | Path | Status |
|----------|------|--------|
| Expected values | `control/ltee_symbiont_pkpd/expected_values.json` | Complete |
| Python benchmark | `control/ltee_symbiont_pkpd/benchmark_ltee_symbiont.json` | Complete |
| Parity report | `control/ltee_symbiont_pkpd/parity_report.json` | Complete |
| Tolerances | `control/ltee_symbiont_pkpd/tolerances.toml` | Complete |
| Rust binary | `ecoPrimal/src/bin/validate_ltee_b5.rs` | 8/8 PASS |
| Python baseline | `control/ltee_symbiont_pkpd/ltee_symbiont_pkpd.py` | 8/8 PASS |

**Stable methods for Module 8**:
- `science.pkpd.hill_dose_response`
- `science.pkpd.one_compartment_pk`

**Next step**: Coordinate with lithoSpore on exact `expected_values.json`
schema version for Module 8 ingestion.

---

## Guidance for Other Springs

### Cross-Tier Parity Pattern

healthSpring's parity approach works for any spring with both Python and Rust:

1. Run Python → capture values to `benchmark_*.json`
2. Run Rust → compare against benchmark constants with documented tolerances
3. Generate `parity_report.json` with structured per-check comparison
4. Document stable method names consumed by the Rust binary

The `validate_ltee_b5.rs` binary is the reference implementation.

### Degradation Documentation Pattern

The `DEGRADATION_BEHAVIOR.md` template:
1. Table: domain → primal → unreachable behavior → return type → science impact
2. Code patterns: how each layer handles `Err` gracefully
3. Partial completion reporting: what metadata tracks success/failure
4. Alignment checklist against upstream trio guide

### Stability Tier Annotation

For niche methods (`science.*`, `game.*`, `ecology.*`, etc.):
- `stable`: any method consumed by lithoSpore or another spring — freeze name
- `evolving`: active research, name may change — notify downstream
- `internal`: test fixtures, helpers — no guarantee

---

## Posture

healthSpring V64x has absorbed all items from the Wave 20 PM lithoSpore
audit. Zero remaining debt. B5 is Module 8 ready. All 57 scenarios pass.
Cross-tier parity proven. Ready for upstream primal evolution phase
(biomeOS `nest.store` signal dispatch, `spore.instantiate`).
