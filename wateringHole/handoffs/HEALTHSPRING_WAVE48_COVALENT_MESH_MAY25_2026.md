# healthSpring Wave 48 — Covalent Mesh Sound Off

**Date**: May 25, 2026
**From**: healthSpring V65a
**Gate**: ironGate (i9-14900K, RTX 5070, 96GB DDR5)
**Co-tenants**: primalSpring, ludoSpring
**Status**: **OPERATIONAL** — NUCLEUS + cell deployed, Songbird federation active

---

## Sound Off

| Field | Value |
|-------|-------|
| **Gate** | ironGate |
| **Hardware** | i9-14900K (24C/32T), RTX 5070, 96GB DDR5 |
| **Composition** | Full NUCLEUS (13 primals) + healthspring_primal cell |
| **NUCLEUS status** | **operational** |
| **Songbird federation** | port 7700 (bound 0.0.0.0) |
| **LAN mesh** | active — awaiting peer connections |
| **Cell graph** | `plasmidBin/cells/healthspring_cell.toml` |

---

## Deployment Results

### NUCLEUS: 23 UDS Sockets Alive

Deployed via `plasmidBin/nucleus_launcher.sh` with `FAMILY_ID=healthspring`.

| Primal | UDS Socket | Status |
|--------|-----------|--------|
| beardog-irongate | ALIVE | `"status":"alive"` |
| barracuda-irongate | ALIVE | `"status":"alive"` |
| barracuda | ALIVE | `"status":"alive"` |
| biomeos-irongate | ALIVE | `"status":"healthy"` |
| btsp | ALIVE | `"status":"alive"` |
| compute-irongate | ALIVE | `"status":"alive"` |
| coralreef-core-default | ALIVE | `"alive":true` |
| crypto | ALIVE | `"status":"alive"` |
| ed25519 | ALIVE | `"status":"alive"` |
| health | ALIVE | `"alive":true` |
| loamspine-healthspring | ALIVE | `"status":"alive"` |
| network-irongate | ALIVE | `"status":"alive"` |
| network | ALIVE | `"status":"alive"` |
| provenance | ALIVE | `"alive":true` |
| rhizocrypt-irongate | ALIVE | `"status":"alive"` |
| security | ALIVE | `"status":"alive"` |
| shader | ALIVE | `"alive":true` |
| songbird-irongate | ALIVE | `"status":"alive"` |
| songbird | ALIVE | `"status":"alive"` |
| sweetgrass-healthspring | ALIVE | `"alive":true` |
| sweetgrass-irongate | ALIVE | `"alive":true` |
| x25519 | ALIVE | `"status":"alive"` |
| **healthspring-default** | **ALIVE** | `"alive":true` |

### healthspring_primal Cell

Built from source (`cargo build --release --bin healthspring_primal`), symlinked
to `plasmidBin/primals/healthspring_primal`. Started manually on TCP 9800 +
UDS `/run/user/1000/biomeos/healthspring-default.sock`.

All 4 domain capabilities responding:

| Capability | Status | Response |
|-----------|--------|----------|
| `health.pharmacology` | LIVE | Requires `concentration, ic50, hill_n, e_max` params |
| `health.genomics` | LIVE | Requires `abundances` param |
| `health.clinical` | LIVE | Returns biosignal, endocrine, microbiome, PK data |
| `health.aggregate` | LIVE | Returns 100-patient composite risk simulation |

### Songbird TCP Federation

```
Port 7700:  LISTENING on 0.0.0.0 (all interfaces)
Port 8091:  HTTP API (Songbird orchestrator)
Port 7701:  Internal (localhost only)
UDS:        songbird-irongate.sock — ALIVE

discovery.peers: {"peers":[], "total_count":0}
```

Peers empty — no other gates on local LAN segment. Ready for mesh when
eastGate/southGate/biomeGate are reachable.

---

## Deployment Gaps Found (for upstream)

### 1. `nucleus_launcher.sh` syntax error (line 304)

**Problem**: The launcher has a bash syntax error (`unexpected token 'then'`)
in its Phase 5 seeding logic. Deployment proceeds through Phase 4 but exits
with code 2.

**Impact**: NUCLEUS starts successfully but the script reports failure.

**Ask**: Fix the syntax error in `nucleus_launcher.sh` line 304.

### 2. Primals reject `--socket` CLI flag

**Problem**: The Wave 48 announcement states "All 13 primals accept `--socket`
+ `--port`" but the current plasmidBin binaries for nestgate, rhizocrypt,
barracuda, and coralreef reject `--socket` as an unexpected argument. They
use `$XDG_RUNTIME_DIR/biomeos/` for socket placement.

**Impact**: `nucleus_launcher.sh` reports these as UNREACHABLE during Phase 4
health sweep (they're actually alive on UDS, just not on the TCP port the
launcher probes).

**Ask**: Update these primal binaries to accept `--socket` as documented,
or update `nucleus_launcher.sh` to probe UDS instead of TCP for health.

### 3. Songbird federation requires manual restart

**Problem**: `SONGBIRD_FEDERATION_PORT=7700` env var is not passed through
by `nucleus_launcher.sh` to the Songbird start. Songbird also requires
`SONGBIRD_SECURITY_PROVIDER` pointing to the BearDog security socket.
After launcher-started Songbird, federation is `enabled: false`.

**Workaround**: Kill Songbird, set env vars, restart with `--federation-port 7700`:
```bash
SONGBIRD_SECURITY_PROVIDER=/run/user/1000/biomeos/security.sock \
  songbird server --socket /run/user/1000/biomeos/songbird-irongate.sock \
  --federation-port 7700
```

**Ask**: Have `nucleus_launcher.sh` pass `SONGBIRD_FEDERATION_PORT` through
to Songbird when the env var is set. Consider auto-discovering the security
socket from the same runtime directory.

### 4. `healthspring_primal` not in plasmidBin

**Problem**: The cell graph references `healthspring_primal` but this binary
is not in the plasmidBin 13-primal set. Springs must build from source and
symlink into `plasmidBin/primals/`.

**Workaround**: `cargo build --release --bin healthspring_primal` then
`ln -sf target/release/healthspring_primal plasmidBin/primals/`.

**Ask**: Document that spring-specific primal binaries need to be built
and placed into `plasmidBin/primals/` before `cell_launcher.sh` can deploy
the cell. Alternatively, have `cell_launcher.sh` search the spring's
`target/release/` directory as a fallback.

---

## Updated Roster (healthSpring's view)

| Spring | Gate | Hardware | Status |
|--------|------|----------|--------|
| primalSpring | eastGate | i9-12900, RTX 4070 + Akida, 32GB | **operational** |
| primalSpring | ironGate | i9-14900K, RTX 5070, 96GB | **operational** |
| wetSpring | southGate | 5800X3D, RTX 4060 + 3090s, 128GB | **operational** |
| ludoSpring | ironGate | i9-14900K, RTX 5070, 96GB | **operational** |
| hotSpring | biomeGate | TR 3970X, RTX 3090 + Titan V, 256GB | **operational** |
| **healthSpring** | **ironGate** | **i9-14900K, RTX 5070, 96GB** | **operational** |
| neuralSpring | ? | ? | sound off |
| airSpring | ? | ? | sound off |
| groundSpring | ? | ? | sound off |

---

## Posture

healthSpring V65a is **operational on ironGate**. Full NUCLEUS deployed (23
alive UDS sockets). Songbird TCP federation listening on port 7700 for
cross-gate LAN discovery. `healthspring_primal` cell live with all 4 domain
capabilities responding. 4 deployment gaps documented for upstream.

Ready for covalent mesh when 3+ gates are networked.
