# healthSpring Wave 49 — Post-Primordial Cleanup + Covalent Mesh

**Date**: May 25, 2026
**From**: healthSpring V65a (ironGate)
**Audit**: primalSpring Wave 49 — Post-Primordial Deployment + Covalent Mesh

---

## 1. Primordial Patterns Cut

All primal binary resolution now goes through plasmidBin only. No `target/release/`
fallbacks, no `cargo build` for deploying primals, no stale symlinks.

### Changes Made

| File | What Was Cut | Replacement |
|------|-------------|-------------|
| `tools/gate_nucleus.sh` | `find_binary()` fell back to `primals/$name/target/release/` and `primalSpring/target/release/primalspring_primal` | plasmidBin triple-target dir only (`primals/x86_64-unknown-linux-musl/`) |
| `tools/fetch_primals.sh` | Checked `primalSpring/target/release/primalspring_primal`; error message suggested "build from source" | Rewritten: plasmidBin-only verification, no fallbacks, hard error if missing |
| `tools/composition_nucleus.sh` | 389-line launcher with `find_binary()` scanning `target/release/` + CamelCase variants | Archived to `fossilRecord/` (superseded by `nucleus_launcher.sh` + `cell_launcher.sh`) |
| `scripts/live_dashboard.sh` | `PETALTONGUE_ROOT/target/release/petaltongue`, `cargo build --release` for petalTongue | plasmidBin triple-target or flat fallback |
| `scripts/visualize.sh` | Same petalTongue `target/release/` build pattern | Same fix as live_dashboard.sh |
| `plasmidBin/primals/healthspring_primal` | Symlink → `springs/healthSpring/target/release/healthspring_primal` | Removed (spring-specific binary, not a NUCLEUS primal) |

### Verification

```bash
# No primals in PATH outside plasmidBin:
for p in beardog songbird biomeos toadstool barracuda coralreef nestgate \
         squirrel rhizocrypt loamspine sweetgrass skunkbat petaltongue; do
    w=$(which $p 2>/dev/null) && echo "STALE: $p -> $w"
done
# Result: empty (clean)

# Sole binary source:
ls infra/plasmidBin/primals/x86_64-unknown-linux-musl/
# 13 primals present
```

### Not Cut (correct behavior)

- `scripts/compute_dashboard.sh` — builds our own experiment binaries (`exp066_*`, `exp069_*`, etc.) from source. These are healthSpring's own code, not primals.
- `scripts/visualize.sh` — still builds `dump_scenarios` / `dump_clinical_scenarios` from source (our own experiment binaries).
- `scripts/sync_scenarios.sh` — references `petalTongue/sandbox/scenarios` for file sync (source directory, not binary).
- `healthspring_primal` — built from source (it's our spring's primal, not a NUCLEUS primal). Started manually after NUCLEUS.

---

## 2. NUCLEUS Deployed from plasmidBin

Restarted using the standard primalSpring launcher:

```bash
cd springs/primalSpring
SONGBIRD_FEDERATION_PORT=7700 ./tools/nucleus_launcher.sh start
```

### Status: 19 UDS Sockets

| Primal | Socket | Status |
|--------|--------|--------|
| biomeOS | neural-api-nucleus01 | ALIVE |
| BearDog | beardog-nucleus01 | ALIVE |
| Songbird | songbird-nucleus01 | ALIVE |
| ToadStool | toadstool-nucleus01 | ALIVE |
| barraCuda | barracuda-nucleus01 | ALIVE |
| coralReef | coralreef-core-nucleus01 | ALIVE |
| NestGate | nestgate-nucleus01 | ALIVE |
| Squirrel | squirrel-nucleus01 | BTSP-protected |
| rhizoCrypt | rhizocrypt-nucleus01 | ALIVE |
| loamSpine | loamspine-nucleus01 | ALIVE |
| sweetGrass | sweetgrass-nucleus01 | ALIVE |
| petalTongue | petaltongue-nucleus01 | BTSP-protected |
| healthspring_primal | healthspring-default | ALIVE |

Plus domain aliases: math, network, storage, ledger, permanence, shader, visualization.

### Songbird Federation

```
Port 7700:  LISTENING on *:7700 (all interfaces — NOT loopback)
LAN IP:     192.168.1.238
eastGate:   192.168.1.144:7700 — reachable (HTTP 400 = federation port alive)
Peers:      [] (auto-discovery pending — no explicit peer seeding yet)
```

---

## 3. Known Pipeline Debt (from Wave 49 audit)

| Issue | Status | Action Taken |
|-------|--------|-------------|
| petalTongue stale socket (EADDRINUSE) | **RESOLVED** | Cleaned stale socket, restarted manually |
| loamSpine Tokio runtime-in-runtime | **NOT HIT** | loamSpine started successfully this run |
| Songbird sled DB corruption | **PREEMPTIVE** | Cleaned `task_lifecycle*` before restart |
| petalTongue rejects `--family-id` | **BYPASSED** | Passed via `FAMILY_ID=nucleus01` env (launcher handles) |

---

## 4. Remaining Gaps (for upstream)

### Federation peer discovery not automatic

**Problem**: After starting NUCLEUS with `SONGBIRD_FEDERATION_PORT=7700`, the
`discovery.peers` endpoint returns empty even though eastGate is reachable at
192.168.1.144:7700. Federation port is bound on both gates but no peers appear.

**Ask**: Document whether peer discovery requires explicit seeding
(e.g., `SONGBIRD_PEERS=192.168.1.144:7700`) or if multicast/mDNS is needed.
The Wave 49 audit says "verify you can see other gates" but doesn't specify
how peers are bootstrapped.

### Spring-specific primal binary deployment

**Problem**: `cell_launcher.sh` looks for `healthspring_primal` in
`plasmidBin/primals/` but spring-specific binaries aren't part of the 13-primal
NUCLEUS set. Springs must build from source and manually place/link.

**Ask**: Document the pattern for spring-specific cell binaries. Either:
1. Have `cell_launcher.sh` search `springs/<spring>/target/release/` as fallback
2. Add a `plasmidBin/cells/bin/` directory for spring-built binaries
3. Document that springs must `cp` their binary into plasmidBin after building

---

## Posture

healthSpring V65a is **post-primordial** on ironGate. All primal binaries
sourced exclusively from plasmidBin. NUCLEUS running with 19 UDS sockets.
Songbird TCP federation on `*:7700`. Cross-gate reachability confirmed to
eastGate. `healthspring_primal` cell live with all 4 domain capabilities.
Ready for covalent mesh once peer discovery bootstrapping is documented.
