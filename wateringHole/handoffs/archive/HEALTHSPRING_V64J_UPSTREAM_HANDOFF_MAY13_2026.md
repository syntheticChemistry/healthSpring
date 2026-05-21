# healthSpring V64j — Delta Spring Evolution, Niche Atomic Convergence

**Date**: May 13, 2026
**Sprint**: V64j
**Upstream audit**: Delta Spring Evolution — Upstream Clear, Niche Atomic Convergence

---

## Summary

GAP-36 (provenance trio wire aliases) is **RESOLVED** on both sides — upstream primals shipped `normalize_method()` alias tables, and healthSpring fixed local non-canonical method names. Five gaps closed. Nest Atomic is now live-ready pending primal deployment.

---

## Gaps Resolved (5)

| # | Gap | Resolution |
|---|-----|-----------|
| 23 | Provenance trio `-32601 MethodNotFound` | Root cause: healthSpring sent non-canonical method names (`commit.create`, `ledger.append`). Fixed locally to canonical (`spine.create`, `entry.append`) + upstream shipped 37+ aliases. |
| 32 | NestComposition testing blocked by trio | Transitive from #23. |
| 34 | `content.*` vs `storage.*` wire naming | Confirmed **by design** per biomeOS `capability_registry.toml`: `content.*` = CAS (BLAKE3, immutable), `storage.*` = keyed blob (mutable). Both route to nestGate. |
| 35 | `ledger.entry.append` vs `entry.append` | `entry.append` is native loamSpine. `spine.create` is canonical for ledger creation. loamSpine v0.9.16 documents full vocabulary + 6 aliases. |
| 36 | Nest Atomic live exercises blocked | Upstream alias tables + local canonical names resolve all dispatch failures. |

---

## Code Changes

### `ecoPrimal/src/ipc/provenance/loamspine.rs`
- New canonical functions: `spine_create()` → `spine.create`, `entry_append()` → `entry.append`
- Backward-compatible wrappers: `commit_create()` and `ledger_append()` delegate to canonical fns
- Doc header references loamSpine v0.9.16 GAP-36 reconciliation canonical names

### `ecoPrimal/src/data/provenance.rs`
- `commit.create` → `spine.create` in resilient trio call

---

## Upstream Wire Alias Summary

### rhizoCrypt S68 (21 aliases)
- `provenance.session.create` → `dag.session.create`
- `provenance.event.append` → `dag.event.append`
- Full `provenance.*` → `dag.*` mirror via `PROVENANCE_ALIASES` in `niche.rs`

### loamSpine v0.9.16 (6 aliases)
- `session.create` → `spine.create`
- `session.state` → `spine.get`
- `ledger.create` → `spine.create`
- `ledger.get` → `spine.get`
- `session.get` → `spine.get`
- `ledger.state` → `spine.get`

### sweetGrass v0.7.35 (10 aliases)
- `braid.attribution.create` → `braid.create`
- `attribution.create_braid` → `braid.create`
- `attribution.add_contribution` → `contribution.record`
- `attribution.calculate` → `attribution.calculate_rewards`
- `attribution.seal` → `braid.commit`
- `attribution.export_prov` → `provenance.export_provo`
- `provenance.lineage` → `attribution.chain`
- `attribution.anchor` → `anchoring.anchor`
- `provenance.create_braid` → `braid.create`
- `attribution.braid` → `braid.create`

---

## healthSpring Canonical Wire Names (current)

| Capability | Method | Target Primal | Status |
|-----------|--------|--------------|--------|
| `storage.store` | blob put | nestGate | canonical |
| `storage.retrieve` | blob get | nestGate | canonical |
| `content.put` | CAS put | nestGate | canonical (different domain) |
| `content.get` | CAS get | nestGate | canonical (different domain) |
| `dag.session.create` | DAG session | rhizoCrypt | canonical |
| `dag.event.append` | DAG event | rhizoCrypt | canonical |
| `spine.create` | ledger create | loamSpine | canonical (was `commit.create`) |
| `entry.append` | ledger entry | loamSpine | canonical |
| `braid.create` | attribution | sweetGrass | canonical |
| `braid.commit` | seal braid | sweetGrass | canonical |
| `crypto.sign` | Ed25519 sig | bearDog | canonical |
| `discovery.peers` | mesh peers | songbird | canonical |
| `defense.audit` | audit trail | skunkBat | canonical |

---

## Remaining Blockers (not resolved by GAP-36)

| # | Gap | Blocked On |
|---|-----|-----------|
| 2 | Ionic bridge NestGate side | NestGate egress fence |
| 10 | BTSP server endpoint | BearDog BTSP server |
| 22 | Socket discovery (DAG/AI/commit) | Ecosystem socket standardization |
| 24 | Songbird crypto provider | Songbird startup docs |

---

## Build Status

- `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery`: **0 warnings, 0 errors**
- `cargo test`: **all pass**
- Nest Atomic scenario: structural routing verified, live execution ready pending primal deployment

---

## Next Steps

1. Deploy provenance trio locally and run `validate --scenario nest-atomic` live
2. Confirm full pipeline: nestGate CAS → rhizoCrypt DAG → loamSpine ledger → sweetGrass attribution
3. Once live validation passes, healthSpring owns the Nest validation pattern for all springs
4. Report results back to primalSpring
