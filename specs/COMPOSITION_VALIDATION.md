<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Composition Validation — Tier 4

**Created**: April 10, 2026
**Status**: Active — 5 composition experiments (73 checks), 12 integration tests

---

## Evolution Path

```
Tier 0: Python baseline (analytical + numerical known-values)
Tier 1: Rust validation (direct Rust vs Python baseline, exit 0/1)
Tier 2: GPU parity (WGSL shader vs Rust CPU, tolerance-bounded)
Tier 3: metalForge validation (NUCLEUS dispatch, PCIe P2P)
Tier 4: Composition validation (IPC dispatch vs direct Rust)   ← THIS
Tier 5: Live primal round-trip (biomeOS ↔ healthspring_primal via Unix socket)
```

Tier 4 validates that calling science methods through the IPC dispatch
layer — the same pathway biomeOS uses via JSON-RPC — produces **identical**
results to direct Rust function calls. This proves the composition surface
(serialization, dispatch routing, parameter extraction, result packaging)
introduces zero numerical divergence.

---

## Experiments

| ID | Name | Domain | Checks | Status |
|----|------|--------|--------|--------|
| exp112 | `composition_pkpd` | PK/PD dispatch parity | 12 | PASS |
| exp113 | `composition_microbiome` | Microbiome dispatch parity | 10 | PASS |
| exp114 | `composition_health_triad` | Capability surface + domain coverage | 17 | PASS |
| exp115 | `composition_proto_nucleate` | Proto-nucleate alignment + socket resolution | 20 | PASS |
| exp116 | `composition_provenance` | Provenance lifecycle + session round-trip | 14 | PASS |

**Total**: 73 composition validation checks, all passing.

---

## What Each Experiment Validates

### exp112 — PK/PD Dispatch Parity

Calls `dispatch_science("science.pkpd.*", params)` and compares each
result field against the direct Rust function call. Tolerance:
`DETERMINISM` (1e-12). Tests Hill, IV bolus, AUC, allometric, MM, PopPK.

### exp113 — Microbiome Dispatch Parity

Same pattern for microbiome methods: Shannon, Simpson, Pielou, Chao1,
colonization resistance, Anderson gut (eigenvalues + IPR arrays).

### exp114 — Capability Surface

- All 58+ registered methods return `Some` from dispatch (no silently
  unroutable capabilities).
- Missing params always produce structured `missing_params` error (not panic).
- All 10 science domains have at least one handler.
- Proto-nucleate alias routing (health.pharmacology → science.pkpd.*).

### exp115 — Proto-Nucleate Alignment

- Socket bind path follows biomeos convention.
- Orchestrator socket resolves.
- All 10 proto-nucleate core capabilities are registered.
- Discovery helpers return `None` gracefully without live primals.
- `PRIMAL_NAME` and `PRIMAL_DOMAIN` constants are correct.

### exp116 — Provenance Lifecycle

- Provenance registry has 10+ tracks with well-formed records.
- Data session lifecycle: `begin → record → complete` round-trip.
- Record lookup by experiment and by track.
- Trio availability probe never panics.
- Distinct sessions get distinct IDs.

---

## Integration Tests

`ecoPrimal/tests/integration_composition.rs` — 12 tests covering:
- Hill, IV bolus, AUC, allometric, MM dispatch parity against direct Rust
- Shannon, Simpson, Chao1, colonization resistance dispatch parity
- All-methods-dispatch (every registry entry routes to a handler)
- All-domains-represented (10 science domains)
- Dispatch determinism (identical params → identical results)

---

## Tolerance

All deterministic composition checks use `tolerances::DETERMINISM` (1e-12).
The dispatch pathway is pure serialization + deserialization of `f64`
values through `serde_json`, which is lossless for IEEE 754 doubles.
Therefore zero divergence is expected and observed.

---

## CI

The `composition` job in `.github/workflows/ci.yml` runs both:
1. `cargo test --test integration_composition` (12 tests)
2. All 5 composition validation binaries with exit-code enforcement

---

## ecoBin Harvest

The ecoBin is built as a static-PIE `x86_64-unknown-linux-musl` binary,
stripped to 2.5 MB, and harvested to `infra/plasmidBin/healthspring/`.
SHA-256 checksum is recorded in `metadata.toml`. The cross-compile CI job
builds for both x86_64 and aarch64 musl targets and uploads artifacts.

---

## Next: Tier 5

Tier 5 requires a running healthspring_primal server and a biomeOS
orchestrator. Tests would send actual JSON-RPC requests over Unix sockets
and validate responses. This is blocked on biomeOS primal lifecycle
management being ready for CI (socket setup, process management, teardown).
