<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V64g — Provenance Elevation Handoff

**From**: healthSpring
**To**: primalSpring, lithoSpore, projectNUCLEUS, foundation
**Date**: 2026-05-13
**Version**: V64g (1,014+ tests, 17 scenarios, 88 capabilities, zero clippy)
**Focus**: Auditable data chains — provenance strengthening, IPC unification, Nest Atomic composition

---

## What healthSpring Shipped

### Phase 1: Python Baseline Provenance (all 53 scripts, 7 tracks)

| Track | Scripts | `expected_values.json` | `tolerances.toml` | DOIs Added |
|-------|---------|----------------------|-------------------|------------|
| pkpd | 7 | Created | Created | Hill 1910, Mould & Upton 2013, Ludden 1977, Kabashima 2020, Mahmood 1996 |
| endocrine | 9 | Created | Created | Harman 2001, Feldman 2002, Saad 2016, Kapoor 2006, Kleiger 1987, Tremellen 2012, Ridaura 2013 |
| microbiome | 7 | Created | Created | Anderson 1958, Simpson 1949, Dethlefsen 2011, den Besten 2013, Yano 2015, van Nood 2013, Buffie 2015, Jenior 2017 |
| comparative | 7 | Created | Created | Gonzales 2014, Mahmood 1996, Michels 2016, Trepanier 2006 |
| biosignal | 6 | Created | — | Pan & Tompkins 1985, PhysioNet/Goldberger 2000 |
| discovery | 6 | Created | — | Zhang 1999 (Z-prime) |
| toxicology | 2 | Created | — | Calabrese 2003 (hormesis) |
| simulation | 1 | Created | — | — (internal analytical) |

18 `ProvenanceRecord` entries in `records_science.rs` updated with explicit DOIs.

### Phase 2: Provenance IPC Wire Shape Unified

`data/provenance.rs` refactored to use canonical JSON-RPC method names:

| Old (`capability.call` envelope) | New (canonical wire name) |
|----------------------------------|--------------------------|
| `capability: "dag", operation: "create_session"` | `dag.session.create` |
| `capability: "dag", operation: "append_event"` | `dag.event.append` |
| `capability: "dag", operation: "dehydrate"` | `dag.dehydrate` |
| `capability: "commit", operation: "session"` | `commit.create` |
| `capability: "provenance", operation: "create_braid"` | `braid.create` |

All 8 provenance tests pass. Wire shape now matches `ipc/provenance/*.rs` and `LIVE_SCIENCE_API.md`.

### Phase 3: NestComposition Facade

New `ipc/provenance/nest.rs` — orchestrates the full Nest Atomic chain:

```text
begin_session()    → rhizoCrypt  dag.session.create
record_event(data) → NestGate    storage.store (BLAKE3)
                   + rhizoCrypt  dag.event.append
sign_merkle()      → BearDog     crypto.sign (Ed25519)
commit()           → loamSpine   commit.create
attribute(contribs)→ sweetGrass  braid.create + braid.commit
```

Builder-pattern API with graceful degradation at each step. 4 new tests pass.

---

## New Gaps Surfaced

| # | Gap | Upstream Action |
|---|-----|-----------------|
| 32 | `NestComposition` end-to-end testing blocked by trio empty UDS responses (Gap #23) | rhizoCrypt/loamSpine/sweetGrass: return non-empty JSON-RPC results |
| 33 | Dataset SHA256 checksums still empty in `data/manifest.toml` | healthSpring: populate when datasets fetched |

---

## For Downstream Consumers

- **lithoSpore**: All 7 tracks now have `expected_values.json` + `tolerances.toml` matching the LTEE B5 pattern. Any track can be ingested as a lithoSpore module candidate.
- **foundation**: DOI-linked provenance enables automated citation tracking. Thread 3 + 8 expressions can reference DOIs from `expected_values.json` directly.
- **projectNUCLEUS**: Unified wire shape means NUCLEUS workload routing can use the same JSON-RPC methods regardless of whether the call originates from `data/provenance.rs` or `ipc/provenance/*.rs`.
