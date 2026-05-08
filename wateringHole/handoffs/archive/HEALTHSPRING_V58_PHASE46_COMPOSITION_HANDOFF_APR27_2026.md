# healthSpring V58 — Phase 46 Composition Template Handoff

**Date**: April 27, 2026
**From**: healthSpring V58
**For**: primalSpring, all primal teams, all spring teams

---

## Summary

healthSpring deployed and validated a full 8-primal NUCLEUS using
primalSpring's Phase 46 composition template tooling. The headless
composition runner exercised 24 automated checks across 8 capability
domains, achieving **18 pass, 4 fail, 2 skip**.

This is healthSpring's first shell-based composition using
`nucleus_composition_lib.sh`. It complements the Rust-based guideStone
(Level 5, 57/57) with a lightweight, scriptable validation surface.

---

## NUCLEUS Deployment

### Environment
- **Architecture**: x86_64
- **OS**: Linux 6.17.9 (Pop!_OS)
- **Binary source**: Local builds from primal repos (`target/release/`)
  — plasmidBin `primals/` directory not yet populated
- **Socket directory**: `/run/user/1000/biomeos/`
- **Family ID**: `healthspring`

### Primals Launched (7/8 healthy)

| Primal | Status | Socket | Notes |
|--------|--------|--------|-------|
| beardog | UP | `beardog-healthspring.sock` | crypto.sign works, health.liveness responds |
| songbird | FAILED | — | `Failed to discover crypto provider` despite beardog running |
| toadstool | UP | `toadstool-healthspring.sock` | 16 cores, 64GB, distributed coordinator |
| barracuda | UP | `math-healthspring.sock` (→ symlink) | All 4 math methods work |
| rhizocrypt | UP (partial) | `rhizocrypt-healthspring.sock` | Accepts UDS, returns empty JSON-RPC |
| loamspine | UP (partial) | `loamspine-healthspring.sock` | Accepts UDS, returns empty JSON-RPC |
| sweetgrass | UP (partial) | `sweetgrass-healthspring.sock` | Accepts UDS, returns empty JSON-RPC |
| petaltongue | UP | `petaltongue-healthspring.sock` | Scene push works, proprioception missing in server mode |

### Capability Alias Map (composition_nucleus.sh)

The script creates symlinks for capability-based discovery:
- `security-healthspring.sock` → `beardog-healthspring.sock`
- `tensor-healthspring.sock` → `barracuda-healthspring.sock`
- `compute-healthspring.sock` → `toadstool-healthspring.sock`
- `dag-healthspring.sock` → `rhizocrypt-healthspring.sock`
- `ledger-healthspring.sock` → `loamspine-healthspring.sock`
- `attribution-healthspring.sock` → `sweetgrass-healthspring.sock`
- `visualization-healthspring.sock` → `petaltongue-healthspring.sock`

This pattern works well and matches `nucleus_composition_lib.sh` expectations.

---

## Validation Results (18/24)

### PASS (18)
| Check | Detail |
|-------|--------|
| capability.discover: visualization | via symlink alias |
| capability.discover: security | via symlink alias |
| capability.discover: compute | via symlink alias |
| capability.discover: tensor | via symlink alias |
| capability.discover: dag | via symlink alias |
| capability.discover: ledger | via symlink alias |
| capability.discover: attribution | via symlink alias |
| visualization.liveness | petalTongue responds to health probe |
| security.liveness | beardog responds |
| compute.liveness | toadstool responds |
| tensor.liveness | barracuda responds |
| stats.mean IPC parity | ipc=5.5, diff=0.0 |
| stats.std_dev IPC parity | ipc=3.0276503540974917, diff=0.0 |
| stats.variance IPC parity | ipc=9.166666666666666, diff=1.78e-15 |
| stats.correlation self-parity | ipc=1.0, diff=0.0 |
| petalTongue.scene.push | scene_stored in server mode |
| beardog.crypto.sign | Ed25519 signature returned |
| toadstool.compute.capabilities | 16 cores, distributed coordinator |

### FAIL (4)
| Check | Detail | Gap |
|-------|--------|-----|
| rhizocrypt.dag.session.create | Empty response on UDS | Gap 23 (extends PG-45) |
| loamspine.spine.create | Empty response on UDS | Gap 23 |
| sweetgrass.braid.create | Empty response on UDS | Gap 23 |
| petalTongue.proprioception | No frame_rate in server mode | Gap 25 |

### SKIP (2)
| Check | Reason | Gap |
|-------|--------|-----|
| capability.discover: storage | nestgate not in default PRIMAL_LIST | Gap 26 |
| storage round-trip | storage offline | Gap 26 |

---

## Discovered Gaps (5 new)

### Gap 23: Provenance Trio Empty UDS Responses
rhizoCrypt, loamSpine, and sweetGrass all accept UDS connections but
return empty responses to JSON-RPC. This extends primalSpring's PG-45
(rhizoCrypt only) to the full trio. All three were started by
`composition_nucleus.sh` with standard env vars and `server` subcommand.

**Ask**: Investigate whether the provenance trio needs different startup
flags, BTSP negotiation, or a nestgate dependency for JSON-RPC to work
over UDS.

### Gap 24: Songbird Crypto Provider Discovery
Songbird exits immediately with `Failed to discover crypto provider`
despite beardog's socket being active and `--beardog-socket` being passed.
`SONGBIRD_SECURITY_PROVIDER` and `BTSP_PROVIDER_SOCKET` env vars also set.

**Ask**: Document songbird's expected startup sequence and crypto provider
discovery mechanism.

### Gap 25: petalTongue Proprioception in Server Mode
`proprioception.get` returns no `frame_rate` in server/headless mode.
Scene push and interaction subscribe/poll all work.

**Ask**: Return synthetic proprioception data in server mode for monitoring.

### Gap 26: NestGate Not in Default PRIMAL_LIST
`composition_nucleus.sh` defaults to 8 primals, excluding nestgate.
Storage-dependent compositions must override `PRIMAL_LIST`.

**Ask**: Add nestgate to defaults or document the override.

### Gap 27: socat Dependency Undocumented
`nucleus_composition_lib.sh` requires socat for JSON-RPC transport.
Not all systems have it. healthSpring created a `nc -q 1 -U` shim.

**Ask**: Document the dependency or add `nc -U` / `ncat` fallback.

---

## Patterns Discovered

### 1. `nc -U` as socat Substitute
`echo "$payload" | nc -q 1 -U "$socket"` works identically to
`echo "$payload" | socat - UNIX-CONNECT:$socket` for JSON-RPC.
The `-q 1` flag is critical — without it, `nc` hangs after receiving
the response. A 4-line shim script makes the lib work without socat.

### 2. barracuda Socket Naming
barracuda creates `math-{family}.sock` instead of
`barracuda-{family}.sock`. The `composition_nucleus.sh` script handles
this with a symlink. This is the correct pattern — `tensor` capability
alias is more useful than primal-name socket.

### 3. Headless Composition Pattern
Interactive compositions (petalTongue GUI loop) don't work in headless
environments. The solution is a non-interactive runner that calls the
same library functions and domain validation logic, replacing the event
loop with sequential checks and automatic exit.

### 4. Build-from-Source Path
With plasmidBin `primals/` directory empty, `composition_nucleus.sh`'s
`find_binary` successfully falls back to local builds. The CamelCase
directory → lowercase binary detection works for all primals. This
validates the "every environment you test strengthens the deployment
surface" principle from the downstream guide.

---

## Artifacts

| File | Purpose |
|------|---------|
| `tools/healthspring_composition.sh` | Interactive composition (petalTongue GUI) |
| `tools/healthspring_composition_headless.sh` | Headless/CI validation runner |
| `tools/socat` | `nc -U` shim for socat-less systems |
| `tools/nucleus_composition_lib.sh` | Copied from primalSpring |
| `tools/composition_nucleus.sh` | Copied from primalSpring |
| `tools/composition_template.sh` | Copied from primalSpring |
| `docs/PRIMAL_GAPS.md` | Updated to V58, gaps 23–27 |

---

**License**: AGPL-3.0-or-later
