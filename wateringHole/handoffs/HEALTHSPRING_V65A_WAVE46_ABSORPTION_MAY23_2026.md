# healthSpring V65a — Wave 46 Absorption

**Date**: May 23, 2026
**From**: healthSpring
**To**: primalSpring, delta springs
**Upstream Audit**: primalSpring v0.9.27 — Wave 46
**Version**: V65a

---

## What healthSpring Absorbed

### 1. BLAKE3 Provenance Backfill (FN-1)

`tools/blake3_backfill.sh` — idempotent script that hashes all JSON files in
`control/` and injects `_provenance.blake3` into each file's provenance block.

**Result**: 62 of 63 JSON files backfilled (1 was already current). Covers:
- 47 experiment baselines (`*_baseline.json`)
- 8 expected values (`expected_values.json`)
- 6 benchmark timing results (`bench_results_*.json`)
- 1 LTEE benchmark (`benchmark_ltee_symbiont.json`)

### 2. sporePrint Sovereign Publish Pipeline (SP-4)

`tools/publish_sporeprint.sh` — pushes `sporeprint/*.md` to NestGate via
`content.put` JSON-RPC call over UDS. Includes:
- Base64-encoded content
- BLAKE3 integrity hash
- Metadata: source, pipeline version, content type
- Dry-run mode for validation

Mirrors the pattern established by primalSpring (Wave 37) and absorbed by
airSpring (Wave 46).

### 3. Registry Reference Updated (445 → 458)

primalSpring v0.9.27 shipped 458 methods (13 new since Wave 38). healthSpring's
`niche.rs` comment and 6 documentation files updated to reflect the new count.

---

## Wave 46 Exposure Assessment

| Item | healthSpring Exposure | Action |
|------|----------------------|--------|
| 458-method registry | **LOW** — consume subset; reference count updated | Done |
| 49 validation scenarios (primalSpring) | **NONE** — healthSpring has its own 57 scenarios | Noted |
| IonicContractRegistry | **DONE** — wired in V64z (Wave 38) | No change |
| NeuralBridge observatory | **NONE** — primalSpring-internal pattern, not consumed | Noted |
| Deploy graph library (94 TOMLs) | **LOW** — healthSpring uses 4 deploy graphs locally | No change |
| sporePrint publishing | **DONE** — `publish_sporeprint.sh` added | Done |
| BLAKE3 backfill (FN-1) | **DONE** — 62 files hashed | Done |
| Tier 4 guidestone rewiring (G column) | **PENDING** — awaiting primalSpring spec for new guidestone protocol | Noted |
| Cross-tier L3 parity (WS-9) | **B5 PROVEN** — 8/8 bit-identical Python↔Rust | No new action |
| LTEE enrichments | **B5 COMPLETE** — Module 8 ready | No new action |

---

## Focus Areas (per PRIMAL_GAPS.md)

### Tier 4 Guidestone Rewiring (G Column)

healthSpring's `healthspring_unibin certify` is the current guidestone
entrypoint. When primalSpring ships the G-column spec, healthSpring will
rewire to the new protocol. Current status: PENDING upstream spec.

### Cross-Tier L3 Parity (WS-9)

B5 Leonard PK/PD has full L1/L2 parity proven (8/8 bit-identical).
L3 requires deployed Nest (live IPC). healthSpring is L3-ready once
westGate NUCLEUS deploys with Nest data tier.

### BLAKE3 Backfill (FN-1)

**COMPLETE** for healthSpring — 62/63 control JSON files now have
`_provenance.blake3` fields. Source TOMLs (deploy graphs, capability
registry) already use BLAKE3 via the workspace `blake3` crate.

---

## Posture

healthSpring V65a: Wave 46 absorbed. BLAKE3 provenance complete (FN-1).
sporePrint pipeline ready for sovereign publish (SP-4). Registry at 458.
Zero clippy, zero debt, 1,021 tests, 57 scenarios. Awaiting Tier 4
guidestone G-column spec and westGate L3 deployment for next evolution.
