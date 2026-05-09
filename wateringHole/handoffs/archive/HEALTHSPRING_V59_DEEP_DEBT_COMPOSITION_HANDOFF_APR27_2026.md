<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V59 — Deep Debt Resolution + Composition Evolution Handoff

**From**: healthSpring V59 (April 27, 2026)
**To**: primalSpring, barraCuda, toadStool, petalTongue, songbird, all springs
**Status**: guideStone Level 5 (57/57), Phase 46 NUCLEUS composition (18/24), deep debt resolved

---

## What Changed (V57 → V58 → V59)

### V59: Deep Debt Resolution — Idiomatic Rust Evolution

| Change | Impact |
|--------|--------|
| **4 typed enums** | `NodeType`, `NodeStatus`, `EdgeType`, `ClinicalStatus` replace `String` fields across visualization dispatch. Eliminates stringly-typed matching, enables compile-time exhaustiveness. |
| **~45 clone eliminations** | `timeseries()` takes `&[f64]`, `bar()` takes `&[String]` — shared x-axis and category data passed by reference across 13+ scenario/clinical-node builders. |
| **`ValidationOutcome`** | `ValidationHarness::finish()` returns pass/fail/total without `process::exit()`. Library consumers can now validate without terminating. `exit()` delegates to `finish()` for binary use. |
| **Capability-first routing** | `handle_provenance_status()` uses capability domains (`dag`, `ledger`, `attribution`) instead of hardcoded primal names (`rhizocrypt`, `loamspine`, `sweetgrass`). |
| **`NicheDependency.name` clarified** | Doc comment updated: socket-prefix fallback hint, not primal identity. Capability domain is the primary discovery key. |
| **Clinical percentile optimization** | Single clone+sort for both 5th and 95th percentiles (was double clone+sort). |
| **892 tests pass, 0 clippy warnings** | All pedantic + nursery + doc-markdown enforced. |

### V58: Phase 46 NUCLEUS Composition

| Change | Impact |
|--------|--------|
| **Full 8-primal NUCLEUS** | Deployed via `composition_nucleus.sh` from primalSpring Phase 46 tooling. |
| **18/24 validation checks** | Capability discovery, liveness probes, barraCuda math IPC, petalTongue scene push, bearDog crypto sign, toadStool compute — all pass. |
| **4 failures** | Provenance trio (rhizoCrypt, loamSpine, sweetGrass) accept UDS but return empty (Gap 23). petalTongue proprioception offline in server mode (Gap 25). |
| **Composition tools** | `healthspring_composition.sh` (interactive), `healthspring_composition_headless.sh` (CI), `socat` shim (`nc -U`). |
| **Gaps 23–27** | Provenance trio empty responses, songbird crypto discovery, petalTongue server proprioception, nestgate default PRIMAL_LIST, socat dependency. |

---

## Patterns Discovered — For Upstream Absorption

### 1. Typed Enums for Closed Vocabularies (all springs)

healthSpring replaced `String` fields with `#[derive(Serialize)]` enums for
node types, statuses, and edge types. Pattern: define the enum with
`#[serde(rename_all = "snake_case")]`, implement `From<&str>` for backward
compatibility, use the enum everywhere. Compile-time exhaustiveness catches
missing match arms. Recommended for any primal or spring with closed
vocabularies in IPC payloads.

### 2. Borrow-Based Helpers Eliminate Clone Chains (all springs)

When multiple builders share the same x-axis or category data, passing
`&[f64]` instead of `Vec<f64>` eliminates N clones per shared dataset. The
`.to_vec()` happens once at the storage boundary. This pattern applies to any
scenario builder, test fixture, or IPC payload constructor that reuses data.

### 3. `ValidationOutcome` for Library Validation (primalSpring)

`process::exit()` in library code prevents composition — callers can't catch
failures or aggregate results. `ValidationOutcome` returns counts and a
suggested exit code. Binary wrappers call `exit()`, library consumers call
`finish()`. This pattern should be adopted in `primalspring::composition`
validation helpers.

### 4. Capability-Domain Discovery Is Complete (all primals)

healthSpring's routing no longer references any primal by name. Capability
domains (`dag`, `ledger`, `attribution`, `compute`, `storage`, `inference`)
are the discovery keys. `NicheDependency.name` is a socket-prefix fallback
hint, not a hard requirement. This pattern should be the standard for all
springs.

### 5. Provenance Trio Degrades Gracefully (primalSpring)

The provenance trio (rhizoCrypt, loamSpine, sweetGrass) accepts UDS
connections but returns empty JSON-RPC responses when started via
`composition_nucleus.sh`. This is likely a startup sequencing or
configuration issue. Compositions must degrade gracefully — provenance loss
is acceptable; domain logic must not block on it.

### 6. socat Shim Pattern (primalSpring)

`nc -q 1 -U` is a portable replacement for `socat - UNIX-CONNECT:path`.
The shim script in `tools/socat` translates the `socat` CLI syntax to `nc`.
This should be upstreamed into `nucleus_composition_lib.sh` as a fallback
when `socat` is not installed.

---

## Primal Evolution — What healthSpring Needs

### primalSpring

| Ask | Gap | Priority |
|-----|-----|----------|
| BTSP negotiation (probe before handshake) | Gap 20 | High — breaks all IPC when FAMILY_SEED set |
| Capability socket standardization | Gap 22 | Medium — 7/10 guideStone skips due to discovery |
| `ValidationOutcome` pattern in composition helpers | V59 | Low — library quality |
| `nestgate` in default PRIMAL_LIST | Gap 26 | Low — storage offline without override |
| socat fallback in composition lib | Gap 27 | Low — portability |

### barraCuda

All four generic math methods validated via IPC. No open wire gaps. Gap 19
resolved in Sprint 44. barraCuda is feature-complete for healthSpring Level 5.

### songbird

| Ask | Gap | Priority |
|-----|-----|----------|
| Document crypto provider discovery | Gap 24 | Medium — songbird fails to start |
| Canonical discovery method names | Gap 3 | Low — dual fallback in place |

### petalTongue

| Ask | Gap | Priority |
|-----|-----|----------|
| Synthetic proprioception in server mode | Gap 25 | Low — non-blocking |

### Provenance Trio (rhizoCrypt, loamSpine, sweetGrass)

| Ask | Gap | Priority |
|-----|-----|----------|
| Investigate empty UDS JSON-RPC responses | Gap 23 | High — all provenance offline |

---

## Composition Patterns for NUCLEUS Deployment

### Shell Composition via primalSpring Tooling

```
COMPOSITION_NAME=healthspring ./tools/composition_nucleus.sh start
PATH="$(pwd)/tools:$PATH" ./tools/healthspring_composition_headless.sh
```

The headless runner performs 24 automated checks across 8 capability domains.
Domain hooks (`domain_init`, `domain_render`, `domain_on_key`, etc.) are
defined in the composition script and exercised against the live NUCLEUS.

### Capability Alias Map (discovered in Phase 46)

| Capability Domain | Primal | Socket Pattern |
|-------------------|--------|----------------|
| `visualization` | petalTongue | `visualization-healthspring.sock` |
| `security` | bearDog | `security-healthspring.sock` |
| `compute` | toadStool | `compute-healthspring.sock` |
| `tensor` | barraCuda | `tensor-healthspring.sock` |
| `dag` | rhizoCrypt | `dag-healthspring.sock` |
| `ledger` | loamSpine | `ledger-healthspring.sock` |
| `attribution` | sweetGrass | `attribution-healthspring.sock` |
| `storage` | nestGate | (not in default PRIMAL_LIST) |

### NUCLEUS Deployment via biomeOS

The ecoBin (0.9.0, 3.2 MB static-PIE x86_64-musl) is harvested to
`infra/plasmidBin/healthspring/`. Deploy via:
1. Copy ecoBin to target machine
2. Start NUCLEUS primals via `composition_nucleus.sh`
3. Run `healthspring_primal serve` (UDS + optional TCP via `--port`)
4. biomeOS Neural API discovers capabilities and composes graphs

---

## Test Status

| Metric | Value |
|--------|-------|
| Library tests | 892 pass, 0 fail |
| Clippy | 0 warnings (pedantic + nursery) |
| Experiments | 94 (84 science + 11 composition) |
| guideStone | Level 5 — 57/57 pass, 10 skip |
| Phase 46 NUCLEUS | 18/24 pass, 4 fail, 2 skip |
| Python baselines | 54, 113/113 cross-validation |
| Unsafe blocks | 0 (`forbid(unsafe_code)`) |
| TODO/FIXME | 0 in production code |

---

## Files Modified (V59)

- `ecoPrimal/src/visualization/types.rs` — 4 typed enums
- `ecoPrimal/src/visualization/scenarios/mod.rs` — borrow-based helpers
- `ecoPrimal/src/visualization/scenarios/*.rs` — enum + borrow migration
- `ecoPrimal/src/visualization/clinical_nodes/*.rs` — enum + borrow migration
- `ecoPrimal/src/visualization/clinical.rs` — enum migration
- `ecoPrimal/src/validation/harness.rs` — `ValidationOutcome`
- `ecoPrimal/src/bin/healthspring_primal/server/routing.rs` — capability-first
- `ecoPrimal/src/niche.rs` — `NicheDependency` doc clarification
- `ecoPrimal/src/ipc/dispatch/handlers/clinical.rs` — percentile optimization
- `validation/CHECKSUMS` — regenerated
- All docs bumped to V59
