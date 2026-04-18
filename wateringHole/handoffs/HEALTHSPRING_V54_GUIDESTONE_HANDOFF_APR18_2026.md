# healthSpring V54 — guideStone Level 2 Handoff

**Date**: April 18, 2026
**From**: healthSpring V54
**To**: primalSpring, barraCuda, sibling springs
**Supersedes**: V53 composition parity narrative (§17 wire gap correction only)

---

## What Changed

### guideStone Binary Created

`healthspring_guidestone` — self-validating NUCLEUS node per
`GUIDESTONE_COMPOSITION_STANDARD` v1.0.0. Requires `--features guidestone`.

**Bare mode** (no primals):
- Property 1 (Deterministic): analytical math parity, wire count consistency
- Property 2 (Traceable): provenance records, cost estimates, tier coverage
- Property 4 (Env-agnostic): forbid(unsafe_code), relative paths, capability discovery
- Property 5 (Tolerance-documented): tolerance ordering, named constants
- Exit code 2 (bare only)

**NUCLEUS mode** (primals deployed):
- barraCuda math IPC: `stats.mean`, `stats.std_dev`, `stats.variance`,
  `stats.correlation` via `primalspring::composition::validate_parity`
- Manifest capabilities: `storage.store/retrieve`, `crypto.hash/sign`,
  `dag.session.create/event.append`, `inference.complete/embed`,
  `braid.create/commit`
- Exit code 0 (pass) / 1 (fail)

### V53 Wire Gap Narrative Corrected

The V53 handoff asked barraCuda to "ADD 9 JSON-RPC WIRE HANDLERS" for
`stats.hill`, `stats.shannon_from_frequencies`, `stats.simpson`, etc.
**This ask is withdrawn.**

These 9 methods are **domain-specific healthSpring science** — local
compositions of barraCuda's generic primitives. They are not generic math
operations that barraCuda should expose over IPC.

barraCuda's 32 IPC methods are correct and complete for healthSpring's needs:
- `stats.mean`, `stats.std_dev`, `stats.variance`, `stats.correlation` — validated
- `tensor.*`, `linalg.*`, `spectral.*` — available for future domain evolution

Domain functions (Hill equation, Shannon entropy, Simpson diversity, etc.)
belong to the spring. They compose FROM barraCuda primitives but are not
themselves primitives.

### Dependency Added

`primalspring` v0.9.15 as optional path dependency (behind `guidestone`
feature). Provides `CompositionContext`, `validate_parity`, `validate_liveness`,
`ValidationResult`, and `tolerances`.

---

## guideStone Readiness

| Property | Status | Blocker |
|----------|--------|---------|
| P1: Deterministic | ✓ | — |
| P2: Reference-Traceable | ✓ | — |
| P3: Self-Verifying | ✗ | CHECKSUMS file generation |
| P4: Environment-Agnostic | ✓ | — |
| P5: Tolerance-Documented | ✓ | — |
| **Level** | **2** | P3 → Level 3, NUCLEUS testing → Level 4 |

---

## Ecosystem Impact

### For primalSpring
- healthSpring now uses `primalspring::composition` — the guideStone pattern works
- `niche::GUIDESTONE_READINESS = 2`, `GUIDESTONE_BINARY = "healthspring_guidestone"`
- healthSpring's downstream manifest should add `guidestone_*` fields

### For barraCuda
- **V53 ask (9 wire handlers) is withdrawn** — no action needed
- healthSpring validates `stats.mean`, `stats.std_dev`, `stats.variance`,
  `stats.correlation` via IPC — these work

### For sibling springs
- The `math_dispatch` "validation window" pattern and its reframing may
  apply to neuralSpring's `IpcMathClient` (18 surface gaps → check if
  domain-specific vs generic)
- The guideStone binary structure (bare + NUCLEUS + exit codes) follows the
  standard pattern from `GUIDESTONE_COMPOSITION_STANDARD.md`

---

## Next Steps (Level 3–5)

1. **P3: CHECKSUMS** — generate SHA-256 checksums for binary integrity
2. **Bare guideStone testing** — run on clean machine, verify exit 2
3. **NUCLEUS guideStone testing** — deploy proto-nucleate, verify IPC parity
4. **Cross-substrate** — Docker/aarch64 validation
5. **Certification** — all 5 properties, cross-substrate parity

---

*This handoff supersedes V53's §17 wire gap narrative only. All other V53
content remains current.*
