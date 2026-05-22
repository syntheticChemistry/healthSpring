# healthSpring V64z — Wave 38 IonicContractRegistry Absorption

**Date**: May 22, 2026
**From**: healthSpring
**To**: primalSpring, bearDog, nestGate, delta springs
**Upstream Audit**: primalSpring Wave 38
**Version**: V64z

---

## What healthSpring Absorbed

### 1. IonicContractRegistry `bonding.*` Protocol Wired

primalSpring Wave 38 shipped `bonding::ionic_runtime` — a full state machine
for ionic bond lifecycle management:

```text
IonicContractRegistry
  ├── propose()     → Proposed
  ├── accept()      → Active  (or Rejected)
  ├── record_call() → updates UsageMetrics
  ├── modify()      → Modifying → Active
  ├── terminate()   → Terminating → Sealed
  └── expire_stale()→ Expired  (TTL enforcement)
```

healthSpring wired four new methods in `TowerAtomic`:

| Method | Wire Name | Routes To | Purpose |
|--------|-----------|-----------|---------|
| `bonding_propose` | `bonding.propose` | coordination socket (primalSpring) | Create contract in Proposed state |
| `bonding_accept` | `bonding.accept` | coordination socket | Accept/reject with constraints |
| `bonding_terminate` | `bonding.terminate` | coordination socket | Seal with provenance |
| `bonding_status` | `bonding.status` | coordination socket | Query contract state |

The existing `crypto.contract.*` methods (Ed25519 signing layer via BearDog)
remain — the bonding protocol is a higher-level layer on top.

**Socket resolution**: `socket::coordination_socket()` resolves via
`PRIMALSPRING_SOCKET` env or `primalspring-{family}.sock` convention.

### 2. `storage.egress_fence` Phantom Reconciled

healthSpring had `storage.egress_fence` in `CONSUMED_CAPABILITIES` and
`capability_registry.toml`. Investigation confirmed:

- Not in primalSpring's canonical 445-method registry
- Not served by NestGate
- No upstream counterpart exists

**Action**: Removed from `CONSUMED_CAPABILITIES`, `capability_registry.toml`,
and PRIMAL_GAPS §2. When NestGate ships real egress policy (family-scoped
encryption at rest, time-series fence), the actual wire name (likely
`content.egress` or `storage.fence`) will be added.

### 3. Registry Reference Updated (452 → 445)

`niche.rs` comment updated from "primalSpring 452-method registry" to
"primalSpring 445-method registry" per Wave 38 audit confirmation.

### 4. Capability Domain Expansion

| Layer | Change |
|-------|--------|
| `routing.rs` | `bonding` domain added to `ALL_CAPS` + `capability_to_primal` (→ primalSpring) |
| `niche.rs` | 5 bonding methods in `CONSUMED_CAPABILITIES` |
| `capability_registry.toml` | `[bonding]` section with 5 methods |
| `DEGRADATION_BEHAVIOR.md` | `bonding` domain row (falls back to `crypto.contract.*` direct signing) |
| `STABILITY_TIERS.md` | `bonding` IPC alignment entry (**stable**) |

### 5. Tests Added

- `bonding_propose_fails_without_coordination`
- `bonding_terminate_fails_without_coordination`
- `bonding_status_fails_without_coordination`

---

## Wave 38 Ecosystem Situational Awareness

healthSpring's exposure to other Wave 38 items:

| Item | Spring/Primal | healthSpring Exposure |
|------|--------------|----------------------|
| BearDog S1 TLS shadow | bearDog | **LOW** — healthSpring uses UDS, not TLS. No action. |
| Songbird TURN relay / cross-gate TCP | songbird | **LOW** — single-gate topology. Cross-gate dispatch is a future concern. |
| toadStool `compute.fan_out` | toadStool | **LOW** — healthSpring's compute loads are per-experiment, not 590 GB batch. |
| biomeOS `nest.sync` / E2E | biomeOS | **MEDIUM** — unblocks WS-2 (cross-spring data exchange). healthSpring is ready as a Nest data tier consumer. |
| NestGate cellMembrane deploy | nestGate | **LOW** — healthSpring uses NestGate via standard `storage.*`/`content.*`. |
| wetSpring WS-11 re-measurement | wetSpring | **NONE** — healthSpring's WS-9 parity already proven for B5. |
| neuralSpring B3/B4 surrogates | neuralSpring | **LOW** — future ML surrogate consumption, not current. |

---

## PRIMAL_GAPS Status After V64z

### Substantially Resolved

- **Gap #2** (Ionic Bridge): `bonding.*` protocol + `crypto.contract.*` signing
  layer both wired. `storage.egress_fence` phantom removed. Remaining: NestGate
  real egress policy.

### Still Open (Upstream-Blocked)

- Gap #10 (BTSP server) — client ready, BearDog BTSP server pending
- Gap #20 (BTSP production mode) — `FAMILY_SEED` workaround
- Gap #22 (Socket discovery) — rhizocrypt/sweetgrass/squirrel capability sockets
- Gap #24 (Songbird crypto provider) — startup discovery failure
- Gap #42 (Foundation Thread 10) — sporeGarden structure pending
- Gap #47 (Signal dispatch live) — biomeOS signal.dispatch needed

---

## Posture

healthSpring V64z: WS-1 ionic bond protocol substantially resolved.
Both signing (Ed25519 via BearDog) and contract management
(IonicContractRegistry via primalSpring) layers wired. Phantom wire name
cleaned. Zero clippy, zero debt, 1,018 tests, 57 scenarios. Ready for
live E2E ionic bond validation when multi-tower NUCLEUS topology deploys.
