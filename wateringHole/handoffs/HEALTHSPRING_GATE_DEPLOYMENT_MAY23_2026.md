# healthSpring Gate Deployment — southGate

**Date**: May 23, 2026
**From**: healthSpring V65a
**Gate**: southGate (5800X3D, 128GB DDR4)
**Co-tenant**: wetSpring
**NUCLEUS Composition**: `healthspring_enclave_proto_nucleate` (dual-tower ionic bond)
**Status**: **DEPLOYMENT-READY** — 14/14 primal binaries available, tooling wired

---

## Gate Assignment Confirmed

healthSpring deploys on **southGate** alongside **wetSpring**. This is a
Nest-heavy gate (128GB DDR4) suitable for healthSpring's dual-tower patient
data enclave and wetSpring's fermentation data pipelines.

| Hardware | Spec |
|----------|------|
| CPU | AMD 5800X3D (8C/16T, 96MB V-Cache) |
| RAM | 128GB DDR4 |
| GPU | N/A for healthSpring (CPU-only science + IPC) |
| Storage | TBD (NestGate storage volumes) |
| OS | Linux (x86_64-unknown-linux-musl static primals) |

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
| `tools/southgate_nucleus.sh start` | Launch dual-tower NUCLEUS | **WIRED** |
| `tools/southgate_nucleus.sh status` | Health check all 14 instances | **WIRED** |
| `tools/southgate_nucleus.sh stop` | Graceful shutdown | **WIRED** |
| `tools/southgate_nucleus.sh validate` | Run `healthspring validate` against live NUCLEUS | **WIRED** |
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

## Gaps Found (for upstream primal teams)

### Gap: Dual-Tower Socket Namespacing

**Problem**: The standard `nucleus_launcher.sh` uses a single FAMILY_ID for all primals.
healthSpring's dual-tower graph needs **two FAMILY_IDs** with isolated socket directories.
This required a custom `southgate_nucleus.sh` rather than using the plasmidBin launcher directly.

**Ask for biomeOS**: Support `--graph` flag that reads a proto-nucleate TOML and auto-deploys
with the specified family topology (per-node family assignment). This would make dual-tower
deployment declarative rather than scripted.

**Ask for primalspring_primal**: Confirm that the ionic bridge binary can discover both
families' songbird instances when passed SOCKET_DIR_B as an env var. Current assumption
is that the bridge needs visibility into both socket directories.

### Gap: primalspring_primal Not in plasmidBin

**Problem**: The ionic bridge coordinator (`primalspring_primal`) is not in the 13-primal
plasmidBin set. healthSpring's proto-nucleate requires it as the bonding mediator.

**Ask for plasmidBin**: Consider adding `primalspring_primal` as a 14th binary in plasmidBin
for springs that use ionic bond compositions. Current workaround: build from source.

### Gap: Multi-Spring Socket Contention (L4 validation)

**Problem**: southGate hosts both healthSpring and wetSpring. If both springs' NUCLEUS
compositions use overlapping FAMILY_IDs or socket paths, we'll see contention.

**Ask for wetSpring**: Coordinate FAMILY_ID naming convention. Proposed:
- healthSpring: `healthspring-tower-a`, `healthspring-tower-b`
- wetSpring: `wetspring-{family}` (single-family NUCLEUS)

### Gap: Live Guidestone Certification (L5)

**Problem**: Current guideStone Level 5 was certified against mock composition.
Post-primordial standard requires live NUCLEUS validation.

**Ask for primalSpring**: Confirm whether `healthspring_unibin certify --max-tier 5`
should be run against live IPC (via env pointing at southGate sockets) or if the
current in-process model remains valid.

---

## Multi-Domain Composition (southGate shared)

southGate will host both healthSpring and wetSpring NUCLEUS compositions. Expected
resource profile:

| Resource | healthSpring | wetSpring | Total |
|----------|-------------|-----------|-------|
| Primal instances | 14 (dual-tower) | ~9 (single NUCLEUS) | ~23 |
| UDS sockets | ~14 | ~9 | ~23 |
| RAM estimate | ~2–4 GB | ~2–4 GB | ~4–8 GB of 128 GB |
| CPU (idle) | Minimal | Minimal | < 5% |

No contention expected — 128GB DDR4 is ample for 23 static-musl primal instances.
CPU contention only during concurrent validation runs.

---

## Posture

healthSpring V65a is **deployment-ready** for southGate. All primal binaries verified.
Dual-tower launcher (`southgate_nucleus.sh`) wired. Validation pipeline ready to run
against live NUCLEUS. L3 live IPC validation is the next milestone — blocked only on
physical deployment to southGate hardware and coordination with wetSpring for L4
multi-domain testing.
