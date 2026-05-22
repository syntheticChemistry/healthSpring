# healthSpring Degradation Behavior

**Version**: 1.0
**Date**: May 17, 2026 (V64x)
**Reference**: `infra/wateringHole/PROVENANCE_TRIO_INTEGRATION_GUIDE.md` §Transaction Semantics

---

## Principle

Science computation MUST NOT fail because provenance, visualization, or
infrastructure primals are unavailable. healthSpring follows the pattern:
`has_capability()` before `call()` — provenance is enrichment, not a gate.

---

## Per-Domain Degradation Table

| Domain | Primal | When Unreachable | Return | Science Impact |
|--------|--------|-----------------|--------|----------------|
| `dag` | rhizoCrypt | `NestComposition` records `local-{experiment}` session ID | `NestStatus::Partial` or `Unavailable` | **None** — experiment runs, provenance unrecorded |
| `ledger` / `spine` | loamSpine | Commit step skipped; DAG session remains valid | `commit_id: ""` | **None** — DAG is valid ephemeral provenance |
| `braid` / `attribution` | sweetGrass | Attribution step skipped; DAG + spine still valid | `braid_id: ""` | **None** — provenance without attribution envelope |
| `storage` / `content` | NestGate | Content hash generation falls back to local BLAKE3 | Local hash | **None** — hash is computed locally |
| `security` / `crypto` | bearDog | Merkle signature skipped | `merkle_signature: ""` | **None** — unsigned provenance is still valid |
| `stats` / `tensor` | barraCuda | Direct Rust library call (no IPC needed for CPU path) | Direct result | **None** — IPC-first defaults, library fallback always available |
| `compute` | toadStool | Pipeline falls back to `execute_cpu` | CPU result | **None** — GPU acceleration unavailable, CPU path is the baseline |
| `shader` | coralReef | Not consumed at runtime (future) | N/A | **None** |
| `visualization` | petalTongue | Scenario JSON generated but not pushed | JSON on disk | **None** — visualization is enrichment |
| `discovery` | songBird | Socket discovery falls back to UDS convention paths | Convention path | **None** — 5-tier discovery escalation |
| `orchestration` | biomeOS | Signal dispatch falls back to manual multi-call chain | Manual chain | **None** — `NestComposition` has dual-path |
| `bonding` | primalSpring | `bonding.*` protocol unavailable; falls back to `crypto.contract.*` direct signing | Direct signing | **None** — Ed25519 layer works independently |
| `inference` | Squirrel | Not consumed (future ML surrogates) | N/A | **None** |
| `audit` | skunkBat | Audit log skipped | No log | **None** — audit is enrichment |

---

## Degradation Patterns in Code

### 1. NestComposition Facade (ipc/provenance/nest.rs)

The `NestComposition` tracks `steps_attempted` and `steps_succeeded`:

```rust
pub struct NestComposition<'a> {
    steps_attempted: u8,
    steps_succeeded: u8,
    // ... per-step Option<String> fields
}
```

Each step independently degrades:
- `begin_session()` → falls back to `local-{experiment}` ID
- `record_event()` → skips if no session ID
- `sign_merkle()` → skips if BearDog unreachable
- `commit()` → skips if loamSpine unreachable
- `attribute()` → skips if sweetGrass unreachable

Final `NestProvenanceChain` reports `NestStatus::{Complete, Partial, Unavailable}`.

### 2. Signal-First Dispatch (ctx.dispatch with fallback)

```rust
// Try biomeOS signal dispatch first
match ctx.dispatch("nest.store", params) {
    Ok(_) => { /* biomeOS orchestrated */ },
    Err(_) => {
        // Manual chain: individual IPC calls
        // Each individually degrades via NestComposition
    }
}
```

### 3. IPC Client (ipc/client.rs)

All `PrimalClient::call()` returns `Result<Value, IpcError>`. Callers
pattern-match on `Err` and degrade gracefully — never `unwrap()` or `panic!()`.

### 4. Experiment Binaries

Live IPC experiments (exp119-121) use:
```rust
match ctx.call("math", "stats.mean", params) {
    Ok(result) => v.check_bool("mean_result", true, "via barraCuda"),
    Err(_) => v.check_skip("mean_result", "barraCuda not available"),
}
```

`check_skip` is SKIP, not FAIL — the experiment catalog records primal availability.

---

## Partial Completion Reporting

healthSpring reports partial provenance through `NestProvenanceChain`:

| Field | Filled When | Empty When |
|-------|-------------|------------|
| `session_id` | rhizoCrypt responds | Unavailable (falls back to `local-*`) |
| `content_hash` | NestGate responds | Local BLAKE3 fallback |
| `merkle_signature` | BearDog responds | Unsigned |
| `commit_id` | loamSpine responds | Unanchored |
| `braid_id` | sweetGrass responds | No attribution |

The `status` field summarizes: `Complete` (all filled), `Partial` (some filled),
`Unavailable` (none filled). Consumers MUST check `status` before assuming
full provenance.

---

## Alignment with Upstream Standard

Per `PROVENANCE_TRIO_INTEGRATION_GUIDE.md`:

- [x] DAG without braid = valid partial provenance
- [x] Braid without spine = valid attribution without permanence
- [x] No rollback — DAG sessions are append-only
- [x] Partial state reported (`NestStatus` + per-field presence)
- [x] Never error on partial provenance — domain logic always completes
