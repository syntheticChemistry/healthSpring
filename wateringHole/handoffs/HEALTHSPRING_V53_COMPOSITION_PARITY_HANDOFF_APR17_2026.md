<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# HEALTHSPRING V53 — Composition Parity & Primal Evolution Handoff

**Date**: 2026-04-17
**From**: healthSpring V53
**To**: primalSpring, biomeOS, barraCuda, toadStool, BearDog, Songbird, Squirrel, neuralSpring, coralReef, all springs
**barraCuda**: v0.3.12 (workspace current)
**Previous**: V50–V52 (archived)

---

## Summary

V53 completes the composition evolution spiral. Where V48–V52 validated
in-process dispatch parity (JSON-RPC method routing = direct Rust), V53
validates the **live NUCLEUS wire path**: Unix socket IPC to a running
primal server produces identical science results to direct Rust calls.

The four-layer validation ladder is now complete:

```
Layer 0: Python control     → peer-reviewed science (DOI-cited baselines)
Layer 1: Rust CPU            → faithful port (f64-canonical, tolerance-documented)
Layer 2: In-process dispatch → JSON-RPC dispatch = direct Rust (exp112–118)
Layer 3: Live IPC            → Unix socket wire path = direct Rust (exp119–121)
```

**Python was the validation target for Rust. Now Rust and Python are both
validation targets for the primal composition.** This handoff captures what
we learned and what the ecosystem should absorb.

---

## V53 Changes

### Live IPC composition experiments

- **exp119** (`composition_live_parity`): Discovers healthspring primal via
  capability probe → name scan → bind path fallback. Calls Hill dose-response,
  one-compartment PK, AUC trapezoidal, Shannon diversity, Anderson eigenvalues
  via `PrimalClient.try_call()` over Unix socket. Compares IPC results against
  direct Rust function calls within `tolerances::DETERMINISM`. Skips gracefully
  when primal is offline.

- **exp120** (`composition_live_provenance`): Provenance trio session lifecycle
  over IPC: `begin_data_session` → `record_fetch_step` → `complete_data_session`.
  Validates Merkle root, commit ID, braid ID when trio is available; skips
  gracefully otherwise. Determinism check: distinct sessions get distinct IDs.

- **exp121** (`composition_live_health`): NUCLEUS health contract over IPC:
  `health.liveness`, `health.readiness`, `capability.list` (validates all
  `niche::CAPABILITIES` are advertised), `identity.get` (name = healthspring,
  domain = health), niche science dispatch (Hill via IPC returns numeric result).

### Stadial zero-dyn compliance

- `Box<dyn ValidationSink>` replaced with enum dispatch:
  `ValidationSink::{Tracing, Silent, Collecting}`. Zero `dyn` in application code.

### Typed error enums

- `ServerError` replaces `Result<_, String>` in `cmd_serve`.
- `TrioError` replaces `Result<_, String>` in `capability_call` / `resilient_capability_call`.

### Capability routing by domain

- `ROUTED_CAPABILITIES` maps to `by_capability` domains (`compute`, `shader`,
  `storage`, `inference`) — not hardcoded primal names. Discovery resolves the
  domain to whichever primal currently advertises it.

### Niche composition registry

- `niche::COMPOSITION_EXPERIMENTS` maps all 10 composition experiments to their
  validation tier (tier3_dispatch_parity through tier4_live_health_probes).

### ecoBin 0.9.0

- Static-PIE x86_64-musl, 3.2 MB, harvested to `infra/plasmidBin/healthspring/`.
- SHA256: `ce71f129a36b0574d5ee417962762a2dae2d62141312ebc73b4292e59b599c22`.
- barraCuda v0.3.12.

---

## What We Learned (for all teams)

### 1. Live IPC parity is trivial when dispatch parity is proven

Once in-process dispatch (exp112–118) was validated, adding live IPC experiments
was straightforward — the wire path (socket connect, JSON serialize, dispatch,
JSON deserialize) introduces no numerical divergence. The test pattern is:
discover primal → `PrimalClient.try_call(method, params)` → extract result
field → `check_abs(label, ipc_val, local_val, DETERMINISM)`.

### 2. Graceful degradation is essential for composition experiments

Live IPC experiments must skip cleanly when the primal server is offline.
Otherwise CI breaks every run. Pattern: check `IpcError::is_connection_error()`
→ `check_bool("label [SKIP: primal offline]", true)`.

### 3. Capability routing by domain makes composition forward-compatible

When `ROUTED_CAPABILITIES` maps to capability domains instead of primal names,
the composition survives primal identity changes. If `toadStool` is replaced by
a different compute provider, healthSpring routes to it automatically via
`by_capability = "compute"`.

### 4. `dyn` dispatch was the last stadial blocker

Replacing `Box<dyn ValidationSink>` with an enum was the only code change
needed for stadial zero-dyn compliance. The pattern is always: trait with
N known implementors → enum with N variants → match dispatch.

### 5. `Result<_, String>` is composition debt

Typed error enums (`ServerError`, `TrioError`) enable match-based error
handling and structured reporting. `String` errors are opaque to callers and
create false retries in resilient IPC paths.

---

## Primal Usage (Current State)

### LOCAL capabilities (62 methods, served in-process)

All `science.*` capabilities across 10 domains: PK/PD (15), microbiome (12),
biosignal (10), endocrine (8), diagnostic (5), clinical (3), comparative (4),
discovery (5), toxicology (3), simulation (2).

### ROUTED capabilities (22 methods, forwarded by domain)

| Capability domain | Methods | Discovery |
|-------------------|---------|-----------|
| `compute` | `compute.offload`, `compute.shader_compile` | `by_capability = "compute"` |
| `storage` | `data.fetch` | `by_capability = "storage"` |
| `inference` | `inference.complete`, `inference.embed`, `inference.models`, `inference.route`, `model.inference_route` | `by_capability = "inference"` |

### Primals consumed

| Primal | Status | Via |
|--------|--------|-----|
| BearDog | Active (awaiting ionic bond runtime) | env override → well-known path |
| Songbird | Active | socket dir scan |
| toadStool | Active | typed `compute_dispatch` client |
| barraCuda | Active (compile-time) | path dependency |
| coralReef | Ready (awaiting device) | feature flag |
| NestGate | Active | capability probe |
| rhizoCrypt/loamSpine/sweetGrass | Active | capability probe |
| Squirrel | Active (Ollama fallback) | optional node, auto-discover |
| petalTongue | Active | IPC push client |
| biomeOS | Ready | well-known socket |

---

## Remaining Gaps (docs/PRIMAL_GAPS.md)

| # | Gap | Status | Blocker |
|---|-----|--------|---------|
| 2 | Ionic bridge enforcement | Blocked | BearDog `crypto.ionic_bond` |
| 4 | Inference canonical namespace | Partial | primalSpring/Squirrel alignment |
| 10 | BTSP server endpoint | Blocked | BearDog BTSP server |
| 17 | barraCuda lib→IPC (Level 5) | Documented | healthSpring wiring (12 call sites) |

Gaps §1, §3, §5–§9, §11–§16 are resolved.

---

## Level 5 Primal Proof — Gap Analysis

### The gap

healthSpring currently links barraCuda as a Rust library dependency and calls
12 functions in-process (`barracuda::stats::mean`, `barracuda::stats::hill`,
`barracuda::special::anderson_diagonalize`, etc.). For the Level 5 primal
proof, these must route through barraCuda's 32 JSON-RPC methods over UDS.
The IPC surface already exists in the barraCuda ecobin — the gap is in
healthSpring's wiring.

`niche::BARRACUDA_IPC_MIGRATION` inventories all 12 library→IPC mappings.
`niche::PROTO_NUCLEATE_VALIDATION_CAPABILITIES` mirrors the 10 capabilities
from the proto-nucleate manifest (`healthspring_enclave_proto_nucleate.toml`).

### Validation ladder status

```
Level 1: Python baseline        — DONE (all 93 experiments cite DOI/SRA provenance)
Level 2: Rust validation        — DONE (93 experiments, 936 tests)
Level 3: barraCuda CPU          — DONE (exp040 CPU parity, exp066 bench)
Level 4: barraCuda GPU          — DONE (exp041+ GPU, feature-gated)
Level 5: Primal composition     — IN PROGRESS (exp119–121 live IPC; barraCuda lib→IPC pending)
Level 6: NUCLEUS deployment     — READY (plasmidBin ecobin 0.9.0 harvested)
```

### Migration plan (§17)

1. Add `BarraCudaClient` typed IPC client (like `PrimalClient`, targets
   barraCuda ecobin at `stats.mean`, `stats.hill`, etc.)
2. Feature-gate: `--features primal-proof` routes math via IPC; default
   keeps library dep for Level 2 comparison
3. Three-tier validation (following hotSpring pattern):
   - Tier 1: local dispatch (library, always green in CI)
   - Tier 2: IPC-wired (live barraCuda ecobin)
   - Tier 3: full NUCLEUS from plasmidBin (clean machine)
4. Validate: IPC result == library result == Python baseline

### Sibling spring patterns adopted

- **hotSpring V0.6.32**: Three-tier composition probes, named tolerances
  in `tolerances/physics.rs`, `HOTSPRING_NO_NUCLEUS=1` for standalone skip,
  `NucleusContext` for parity calls.
- **neuralSpring V1.32**: `PROTO_NUCLEATE_VALIDATION_CAPABILITIES` constant
  mirroring manifest, strict proto-nucleate vs spring-deploy separation,
  `depends_on` alignment verification.

---

## Ecosystem Asks

### To primalSpring
- **Audit this push**: healthSpring V53 is ready for composition audit.
  `niche.rs` carries full self-knowledge (PRIMAL_ID, NICHE_DOMAIN, FRAGMENTS,
  BOND_TYPE, TRUST_MODEL, DEPENDENCIES, CAPABILITIES, CONSUMED_CAPABILITIES,
  COMPOSITION_EXPERIMENTS, COST_ESTIMATES, OPERATION_DEPENDENCIES,
  PROTO_NUCLEATE_VALIDATION_CAPABILITIES, BARRACUDA_IPC_MIGRATION).
- **Absorb composition validation pattern**: exp119–121 pattern (discover →
  call → compare → skip-on-offline) should become a reusable crate.
- **Validate proto-nucleate alignment**: deploy graph trust_model
  (`dual_tower_enclave`) vs proto-nucleate trust_model (`btsp_enforced`).

### To barraCuda
- **IPC parity priority**: healthSpring has 12 library calls to migrate to
  barraCuda's 32 JSON-RPC methods. Confirm these map cleanly: `stats.mean`,
  `stats.std_dev`, `stats.hill`, `stats.shannon_from_frequencies`,
  `stats.simpson`, `stats.chao1_classic`, `stats.bray_curtis`,
  `special.anderson_diagonalize`, `rng.uniform`, `health.pkpd.mm_auc`,
  `health.microbiome.antibiotic_perturbation`, `health.biosignal.scr_rate`.
- **TensorSession API**: Enables local shader removal (Write → Absorb → Lean).
  healthSpring has 6 fused pipeline patterns waiting.
- **Anderson eigensolve GPU**: Gut lattice localization is a shader candidate.

### To BearDog
- **Ionic bond runtime**: `crypto.ionic_bond`, `crypto.verify_family` for
  dual-tower enforcement. Deploy graph declares it; runtime enforcement awaiting.
- **BTSP server endpoint**: `ipc/btsp.rs` has pure-Rust client handshake ready.
- **`crypto.hash` ownership**: healthSpring routes `crypto.hash` to BearDog
  (not barraCuda). Confirm this is the canonical provider.

### To biomeOS
- **Deploy graph validated**: `healthspring_niche_deploy.toml` structurally
  validated (exp118, 99 checks). Four workflow graphs ready for Neural API.
- **ecoBin 0.9.0**: Static-PIE binary at `infra/plasmidBin/healthspring/`.
- **Clean-machine test**: healthSpring Tier 3 harness should discover NUCLEUS
  sockets via `$BIOMEOS_ORCHESTRATOR_SOCKET` or socket dir scan — not
  hardcoded paths. Confirm biomeOS socket layout.

### To Squirrel / neuralSpring
- **Inference namespace**: healthSpring routes `inference.*` to Squirrel.
  Confirm or evolve the canonical namespace.

### To all springs
- **Adopt `PROTO_NUCLEATE_VALIDATION_CAPABILITIES`**: Each spring's `niche.rs`
  should mirror its manifest's `validation_capabilities` as a const array.
  Enables machine-readable composition readiness checks.
- **Adopt `BARRACUDA_IPC_MIGRATION`**: Inventory your `barracuda::` library
  imports and their IPC targets. This is the Level 5 gap for every spring.

---

## Validation Evidence

```
cargo test --lib                → 936 tests, 0 failures (852 + 33 + 51)
cargo clippy --workspace -- -D warnings  → 0 warnings
cargo fmt --check               → clean
exp119 (offline)                → 6/6 PASS (skip-on-offline)
exp120 (offline)                → 4/4 PASS (skip-on-offline)
exp121 (offline)                → 6/6 PASS (skip-on-offline)
```
