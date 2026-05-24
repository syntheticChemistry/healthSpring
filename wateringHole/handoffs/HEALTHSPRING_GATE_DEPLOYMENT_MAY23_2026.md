# healthSpring Gate Deployment — ironGate

**Date**: May 23, 2026
**From**: healthSpring V65a
**Gate**: ironGate (i9-14900K, RTX 5070, 96GB DDR5)
**Co-tenants**: primalSpring, ludoSpring, groundSpring
**NUCLEUS Composition**: `healthspring_enclave_proto_nucleate` (dual-tower ionic bond)
**Status**: **DEPLOYED** — Tower A 7/7 healthy, Tower B partial (socket path gaps)

---

## Gate Assignment Confirmed

healthSpring deploys on **ironGate** alongside primalSpring, ludoSpring, and
groundSpring. This is the primary development gate with the most compute headroom.

| Hardware | Spec |
|----------|------|
| CPU | Intel i9-14900K (24C/32T) |
| RAM | 96GB DDR5 |
| GPU | NVIDIA RTX 5070 |
| OS | Linux 6.12.10 (Pop!_OS / x86_64) |

---

## NUCLEUS Composition: Dual-Tower Ionic Bond

healthSpring's proto-nucleate is unique in the ecosystem — a **dual-tower**
architecture with ionic bridge enforcing HIPAA-grade data separation:

### Tower A — Patient Data Enclave (FAMILY_A)

| Node | Binary | Role |
|------|--------|------|
| biomeos_tower_a | biomeos | Graph coordinator |
| beardog_a | beardog | Crypto spine (identity, encryption) |
| songbird_a | songbird | Discovery within trust boundary |
| nestgate_a | nestgate | Patient record storage (egress fence) |
| rhizocrypt_a | rhizocrypt | Regulatory audit trail (DAG) |
| loamspine_a | loamspine | Permanent ledger |
| sweetgrass_a | sweetgrass | Attribution |

### Tower B — Analytics/Inference (FAMILY_B)

| Node | Binary | Role |
|------|--------|------|
| beardog_b | beardog | Separate trust domain |
| songbird_b | songbird | Discovery |
| squirrel_b | squirrel | Clinical AI inference |
| nestgate_b | nestgate | Model weights, inference cache |
| rhizocrypt_b | rhizocrypt | Inference audit trail |
| sweetgrass_b | sweetgrass | Inference attribution |

### Ionic Bridge

| Node | Binary | Role |
|------|--------|------|
| ionic_bridge | primalspring_primal | `bonding.propose` / `bonding.accept` mediation |

**Total**: 14 primal instances (7 in Tower A + 6 in Tower B + 1 bridge)

### Ionic Policy

```toml
from_family = "${FAMILY_B}"
to_family = "${FAMILY_A}"
bond_type = "ionic"
capabilities_requested = ["inference.complete", "inference.models", "health.aggregate"]
capabilities_denied = ["storage.*", "dag.*"]
metered = true
time_window = "session"
```

---

## Deployment Tooling

| Tool | Purpose | Status |
|------|---------|--------|
| `tools/fetch_primals.sh` | Verify/fetch all 14 binaries | **WIRED** |
| `tools/gate_nucleus.sh start` | Launch dual-tower NUCLEUS | **WIRED** |
| `tools/gate_nucleus.sh status` | Health check all 14 instances | **WIRED** |
| `tools/gate_nucleus.sh stop` | Graceful shutdown | **WIRED** |
| `tools/gate_nucleus.sh validate` | Run `healthspring validate` against live NUCLEUS | **WIRED** |
| `tools/composition_nucleus.sh` | Single-family fallback launcher (existing) | Available |
| `tools/healthspring_composition.sh` | IPC math validation (existing) | Available |

### Deployment Flow

```bash
# 1. Verify primals (13 from plasmidBin + 1 primalspring_primal)
./tools/fetch_primals.sh --check

# 2. Launch dual-tower NUCLEUS
./tools/southgate_nucleus.sh start

# 3. Validate composition
./tools/southgate_nucleus.sh validate

# 4. Run full scenario suite against live primals
healthspring_unibin validate --format json

# 5. Stop when done
./tools/southgate_nucleus.sh stop
```

---

## Primal Binary Availability

```
13/13 NUCLEUS primals: plasmidBin/primals/ (musl-static)
 1/1  primalspring_primal: primalSpring/target/release/ (built v0.9.27)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
14/14 TOTAL — ALL PRESENT
```

---

## Validation Posture

| Tier | Description | Status |
|------|-------------|--------|
| L1 | Python vs Rust (local, no IPC) | **57/57 scenarios PASS** |
| L2 | Rust vs barraCuda CPU/GPU | **42/42 parity PASS** |
| L3 | Live IPC via NUCLEUS composition | **READY** — tooling wired, awaiting live deployment |
| L4 | Multi-gate mesh (southGate + wetSpring contention test) | **PENDING** — requires wetSpring co-deployment |
| L5 | guideStone certification against live NUCLEUS | **Level 5** (against mock; live TBD) |

---

## Live Deployment Results (May 23, 2026 — ironGate)

**Deployment method**: `tools/gate_nucleus.sh start` via `plasmidBin/start_primal.sh`

### Tower A — Patient Data Enclave: **7/7 HEALTHY**

| Primal | Status | Socket Path | Notes |
|--------|--------|-------------|-------|
| beardog | **HEALTHY** | `/tmp/biomeos/{family}/beardog-{family}.sock` | Responds `"alive"` |
| songbird | **HEALTHY** | `/run/user/1000/biomeos/songbird-{family}.sock` | Needs `BEARDOG_SOCKET` env |
| nestgate | **HEALTHY** | `/run/user/1000/biomeos/nestgate-{family}.sock` | Ignores `--socket`, uses XDG |
| rhizocrypt | **HEALTHY** | `/run/user/1000/biomeos/rhizocrypt-{family}.sock` | Needs `FAMILY_SEED` env |
| loamspine | **HEALTHY** | `/tmp/biomeos/{family}/loamspine-{family}.sock` | 15s infant discovery timeout; needs `DISCOVERY_ENDPOINT` |
| sweetgrass | **HEALTHY** | varies | Responds `"alive"` |
| biomeos | **HEALTHY** | varies | Coordinator ready |

### Tower B — Analytics/Inference: **3/6 HEALTHY** (socket path gap)

| Primal | Status | Notes |
|--------|--------|-------|
| beardog | **HEALTHY** | Same pattern as Tower A |
| songbird | **HEALTHY** | Same pattern |
| squirrel | **RUNNING but unreachable** | Socket at XDG path; empty health response (format issue) |
| nestgate | Not started (blocked by squirrel) | Would succeed (same pattern as Tower A) |
| rhizocrypt | Not started | Would succeed |
| sweetgrass | Not started | Would succeed |

### Deployment Issues Discovered

1. **Socket path disagreement**: NestGate and Squirrel ignore `--socket` CLI flag, always place socket at `$XDG_RUNTIME_DIR/biomeos/{name}-{family}.sock`. BearDog and LoamSpine respect the passed path. This creates discovery confusion.

2. **`FAMILY_SEED` vs `BEARDOG_FAMILY_SEED`**: RhizoCrypt requires `FAMILY_SEED` env for BTSP. BearDog uses `BEARDOG_FAMILY_SEED`. Both must be set for dual-tower operation.

3. **Songbird requires explicit `BEARDOG_SOCKET`**: Without it, Songbird fails with "No security provider configured."

4. **LoamSpine infant discovery timeout**: 15s DNS SRV scan before fallback. Set `DISCOVERY_ENDPOINT` to songbird socket to skip.

5. **Health response format inconsistency**: BearDog returns `{"status":"alive"}`, others return `{"status":"alive"}` or nothing. Squirrel returns empty on `health.liveness`.

6. **`socat` required**: Not installed by default on some systems. Required for UDS health probes.

---

## Gaps Found (for upstream primal teams)

### Gap: Socket Path Inconsistency (CRITICAL for multi-tower)

**Problem**: Primals disagree on socket placement. BearDog and LoamSpine respect
the `--socket` CLI flag and use the passed directory. NestGate and Squirrel ignore
`--socket` and always use `$XDG_RUNTIME_DIR/biomeos/{name}-{family}.sock`.

**Impact**: Multi-tower compositions cannot reliably isolate socket namespaces.
Discovery breaks when the health checker looks in the wrong directory.

**Ask for NestGate/Squirrel teams**: Honor the `--socket` CLI flag passed by
`start_primal.sh`. Alternatively, document the canonical socket resolution order
so launchers can predict the actual path.

### Gap: Dual-Tower Socket Namespacing

**Problem**: The standard `nucleus_launcher.sh` uses a single FAMILY_ID for all primals.
healthSpring's dual-tower graph needs **two FAMILY_IDs** with isolated socket directories.
This required a custom `gate_nucleus.sh` rather than using the plasmidBin launcher directly.

**Ask for biomeOS**: Support `--graph` flag that reads a proto-nucleate TOML and auto-deploys
with the specified family topology (per-node family assignment). This would make dual-tower
deployment declarative rather than scripted.

### Gap: FAMILY_SEED Environment Inconsistency

**Problem**: BearDog uses `BEARDOG_FAMILY_SEED`. RhizoCrypt uses `FAMILY_SEED`. If only
one is set, rhizoCrypt rejects all connections with "BTSP: no family seed." Both must
be exported for a family to function.

**Ask for ecosystem**: Standardize on a single env var (`FAMILY_SEED`) with BearDog
accepting it as an alias, or document that launchers must export both.

### Gap: Songbird Security Provider Discovery

**Problem**: Songbird refuses to start without `BEARDOG_SOCKET` or `SONGBIRD_SECURITY_PROVIDER`
explicitly set. It cannot auto-discover BearDog from the same socket directory.

**Ask for Songbird**: Auto-discover BearDog by scanning `$SOCKET_DIR/beardog-*.sock` or
`$SOCKET_DIR/security.sock` before requiring explicit env configuration.

### Gap: primalspring_primal Not in plasmidBin

**Problem**: The ionic bridge coordinator (`primalspring_primal`) is not in the 13-primal
plasmidBin set. healthSpring's proto-nucleate requires it as the bonding mediator.

**Ask for plasmidBin**: Consider adding `primalspring_primal` as a 14th binary in plasmidBin
for springs that use ionic bond compositions. Current workaround: build from source.

### Gap: Health Response Format Inconsistency

**Problem**: BearDog returns `{"status":"alive"}`. Squirrel returns empty on `health.liveness`.
The Deployment Validation Standard says primals MUST return `{"status":"healthy"}`.

**Ask for ecosystem**: Enforce the standard. All primals should return
`{"status":"healthy"}` or `{"alive":true}` per DEPLOYMENT_VALIDATION_STANDARD.md.

### Gap: LoamSpine 15s Infant Discovery Timeout

**Problem**: Without `DISCOVERY_ENDPOINT`, LoamSpine does a 15-second DNS SRV scan
before starting. This exceeds reasonable health check timeouts.

**Ask for LoamSpine**: Reduce timeout to 3s or skip DNS when `FAMILY_ID` is set
(indicating a composed NUCLEUS, not standalone).

---

## Multi-Domain Composition (ironGate shared)

ironGate hosts healthSpring, primalSpring, ludoSpring, and groundSpring. Expected
resource profile:

| Resource | healthSpring | Others (3 springs) | Total |
|----------|-------------|-------------------|-------|
| Primal instances | 14 (dual-tower) | ~9 each | ~41 |
| UDS sockets | ~14 | ~27 | ~41 |
| RAM estimate | ~2–4 GB | ~6–12 GB | ~8–16 GB of 96 GB |
| CPU (idle) | Minimal | Minimal | < 10% |

No contention expected — 96GB DDR5 and 24C/32T i9-14900K have ample headroom.

---

## Posture

healthSpring V65a is **live on ironGate**. Tower A (7/7 primals healthy) deployed
successfully. Tower B blocked by socket path inconsistency (Squirrel/NestGate ignore
`--socket` flag). Dual-tower launcher (`gate_nucleus.sh`) wired and functional.
**7 deployment gaps** documented for upstream primal teams. L3 live IPC validation
achievable once socket path standardization resolves.
