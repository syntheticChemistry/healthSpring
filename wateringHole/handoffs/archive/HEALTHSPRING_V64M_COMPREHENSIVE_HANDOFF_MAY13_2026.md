<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V64m — Comprehensive Upstream Handoff

**Date**: May 13, 2026
**From**: healthSpring (Nest Atomic Specialist)
**To**: primalSpring, primal teams (bearDog, skunkBat, nestGate, rhizoCrypt, loamSpine, sweetGrass, songBird), spring teams
**Scope**: Wire contract learnings, Nest Atomic composition patterns, provenance pipeline architecture, plasmidBin deployment, Foundation Thread 10

---

## 1. Wire Contract Learnings

These are hard-won wire fixes discovered during live Nest Atomic validation (V64h–V64l).
Other springs will hit the same issues unless they absorb these findings.

### 1.1 BearDog `crypto.sign` — base64 `message` (not `payload`)

**What broke**: healthSpring sent `{ "payload": "<raw string>", "algorithm": "ed25519" }`.
BearDog silently returned an error or empty signature.

**Fix**: BearDog expects `{ "message": "<base64-encoded>", "purpose": "<string>" }`.

```json
{
  "method": "crypto.sign",
  "params": {
    "message": "bmVzdC1hdG9taWMtdmFsaWRhdGlvbi1mYWxsYmFjaw==",
    "purpose": "provenance_commit"
  }
}
```

**Impact**: Any spring calling `crypto.sign` must base64-encode the message payload.
The `algorithm` parameter is not used — BearDog selects signing algorithm internally.

### 1.2 SkunkBat Audit — `security.audit_log` (not `defense.audit`)

**What broke**: healthSpring called `defense.audit` on a socket discovered via the `"defense"` domain.
SkunkBat returned `-32601 MethodNotFound`.

**Fix**: The canonical method is `security.audit_log`. Socket discovery uses the `"audit"` domain.

```json
{
  "method": "security.audit_log",
  "params": {
    "event": "nest_atomic_validation_complete",
    "session_id": "...",
    "result": { ... }
  }
}
```

**Impact**: Any spring routing to skunkBat for audit trails must use `security.audit_log` on wire
and `"audit"` as the capability domain for socket discovery (not `"defense"` or `"security"`).

### 1.3 LoamSpine — `spine.create` / `entry.append` (not `commit.create` / `ledger.append`)

**What broke**: healthSpring called `commit.create` and `ledger.append`. LoamSpine returned
`-32601 MethodNotFound` until upstream shipped `normalize_method()` alias tables.

**Fix**: Canonical wire names are `spine.create` and `entry.append`. Upstream loamSpine now has
alias tables that accept both old and new names, but new code should use canonical names.

### 1.4 NestGate — `content.*` vs `storage.*` is By-Design

`content.*` methods (`content.store`, `content.retrieve`) are **content-addressed storage** (CAS) —
immutable, BLAKE3-hashed, used for provenance. `storage.*` methods (`storage.store`, `storage.retrieve`)
are **keyed/blob storage** — mutable, used for general data. This distinction is confirmed by
biomeOS v3.53 and is intentional.

**For other springs**: Use `content.*` when you need immutable provenance-tracked storage.
Use `storage.*` when you need mutable keyed storage. Do not conflate them.

### 1.5 RhizoCrypt — `dag.create_node` / `dag.query` canonical

RhizoCrypt accepts aliases (`dag.add`, `dag.node`, etc.) via its `normalize_method()` table,
but canonical wire names are `dag.create_node` and `dag.query`.

### 1.6 SweetGrass — `braid.create` / `braid.query` canonical

SweetGrass aliases exist for `attribution.create` and `attribution.query`, but canonical
wire names are `braid.create` and `braid.query`.

---

## 2. Nest Atomic Composition Pattern

healthSpring proved the Nest Atomic (data lineage and provenance) end-to-end via a 9-phase
validation scenario. This is the reference pattern for any spring implementing provenance.

### 2.1 Nine-Phase Validation

```
Phase 1: NestGate CAS      → content.store → content_hash
Phase 2: rhizoCrypt DAG     → dag.create_node(content_hash) → dag_node_id
Phase 3: loamSpine ledger   → spine.create + entry.append → merkle_root + entry_id
Phase 4: sweetGrass braid   → braid.create(dag_node_id, entry_id) → braid_id
Phase 5: BearDog crypto     → crypto.sign(base64(merkle_root)) → signature
Phase 6: Cross-references   → verify chain integrity across all primal outputs
Phase 7: sweetGrass query   → braid.query for attribution metadata
Phase 8: skunkBat audit     → security.audit_log of validation result
Phase 9: Summary            → verify all phases connected end-to-end
```

### 2.2 NestComposition Facade

The `NestComposition` struct (`ipc/provenance/nest.rs`) orchestrates the pipeline:

```rust
let mut nest = NestComposition::new(ctx);
nest.store(content);          // Phase 1: NestGate CAS
nest.dag_node(content_hash);  // Phase 2: rhizoCrypt DAG
nest.commit(experiment, data); // Phase 3: loamSpine ledger
nest.braid(metadata);         // Phase 4: sweetGrass attribution
nest.sign(merkle_root);       // Phase 5: BearDog crypto
```

Each method is independent and skips gracefully when the primal is unavailable.

### 2.3 Deploy Graph Structure

Three deploy graph files define the composition:

| File | Purpose |
|------|---------|
| `healthspring_nest_atomic.toml` | 7-node Nest Atomic validation graph (nestGate + rhizoCrypt + loamSpine + sweetGrass + bearDog + skunkBat + healthSpring) |
| `healthspring_niche_deploy.toml` | Full niche deployment (all primals including songBird discovery) |
| `healthspring_cell.toml` | plasmidBin cellular deployment (Tower + Node + Nest + Meta primals + validation node) |

### 2.4 Capability Routing Architecture

```
capability domain → socket discovery → wire method
─────────────────────────────────────────────────────
"content"  → nestGate    → content.store / content.retrieve
"storage"  → nestGate    → storage.store / storage.retrieve
"dag"      → rhizoCrypt  → dag.create_node / dag.query
"commit"   → loamSpine   → spine.create / entry.append
"braid"    → sweetGrass  → braid.create / braid.query
"crypto"   → bearDog     → crypto.sign / crypto.verify
"audit"    → skunkBat    → security.audit_log
"discovery"→ songBird    → discovery.find / discovery.register
```

The domain used for socket discovery is **not** always the same as the wire method prefix.
For example, skunkBat's socket is found via domain `"audit"` but the wire method is
`security.audit_log`. This is by-design for backward compatibility.

---

## 3. Provenance Pipeline — What Each Primal Does

| Primal | Role | Wire Methods | Output |
|--------|------|-------------|--------|
| **NestGate** | Content-addressed storage | `content.store`, `content.retrieve` | Content hash (BLAKE3) |
| **rhizoCrypt** | Directed acyclic graph | `dag.create_node`, `dag.query` | DAG node ID |
| **loamSpine** | Immutable ledger | `spine.create`, `entry.append` | Merkle root, entry ID |
| **sweetGrass** | Semantic attribution | `braid.create`, `braid.query` | Braid ID |
| **BearDog** | Cryptographic signing | `crypto.sign`, `crypto.verify` | Ed25519 signature |
| **skunkBat** | Audit trail | `security.audit_log` | Audit event ID |

The pipeline forms a chain: content → DAG → ledger → attribution → signature → audit.
Each link references the output of the previous link, creating an auditable provenance chain.

---

## 4. plasmidBin Deployment — cell.toml Pattern

The `healthspring_cell.toml` follows ludoSpring's `[[nodes]]` pattern:

```toml
[cell]
name = "healthspring_health"
version = "0.2.0"
domain = "health"

[[nodes]]
name = "tower_atomic"
primals = ["beardog", "songbird", "skunkbat"]

[[nodes]]
name = "node_atomic"
primals = ["barracuda", "toadstool", "metalforge"]

[[nodes]]
name = "nest_atomic"
primals = ["nestgate", "rhizocrypt", "loamspine", "sweetgrass"]

[[nodes]]
name = "meta"
primals = ["petaltongue", "biome_neural_api"]

[[nodes]]
name = "validation"
primals = ["healthspring"]
```

Each `[[nodes]]` group maps to a NUCLEUS atomic type. The spring itself appears as a
`validation` node that consumes all other atomics.

---

## 5. Foundation Thread 10 — Provenance Expression

**Gap #42** (documented in `docs/PRIMAL_GAPS.md`): Foundation Thread 10 asks each spring to
express how it represents provenance in its domain.

healthSpring's provenance expression for clinical data:
- **Content**: Clinical dataset hashes (NestGate CAS)
- **Lineage**: Experiment → analysis → result DAG (rhizoCrypt)
- **Audit**: Immutable ledger of all data transformations (loamSpine)
- **Attribution**: Semantic metadata — which paper, which experiment, which patient cohort (sweetGrass)
- **Integrity**: Merkle root signatures for tamper detection (BearDog)

This completes healthSpring's provenance narrative. Other springs should define their own
domain-specific provenance expressions following this pattern.

---

## 6. Metrics at V64m

| Metric | Value |
|--------|-------|
| Tests (workspace) | 1,018 |
| Experiments | 95 |
| JSON-RPC capabilities | 88 |
| Validation scenarios | 17 |
| Deploy graphs | 7 (4 workflow + nest_atomic + niche_deploy + cell) |
| Clippy warnings | 0 (pedantic + nursery) |
| Unsafe blocks | 0 |
| TODO/FIXME | 0 |
| Deep debt categories at zero | 7/7 |
| Open gaps | 1 (GAP-42: Foundation Thread 10 pending upstream expression template) |

---

## 7. Recommendations for Upstream Teams

### For primal teams (bearDog, skunkBat, loamSpine, etc.)

1. **Document wire contracts explicitly** — parameter names, encoding (base64 vs raw), and
   which domain to use for socket discovery. The implicit contracts cost springs significant
   debugging time.
2. **Ship `normalize_method()` alias tables** — loamSpine's approach of accepting both old
   and new method names during transition is the right pattern.
3. **Standardize error messages** — `-32601 MethodNotFound` should include the attempted
   method name and suggest the canonical alternative.

### For other springs

1. **Use canonical wire names from day one** — see Section 1 above.
2. **Base64-encode payloads for BearDog** — this is not documented anywhere upstream yet.
3. **Use `"audit"` domain for skunkBat discovery** — not `"defense"` or `"security"`.
4. **Test provenance with `NestComposition` pattern** — the facade + 9-phase validation
   is reusable across springs.

### For primalSpring composition framework

1. **Consider a wire contract registry** — a machine-readable mapping of primal → domain → methods
   → parameter schemas would prevent the discovery process healthSpring went through.
2. **`CompositionContext` could validate wire names** — at composition time, not just at call time.
