# healthSpring V57 — Primal Proof (guideStone Level 5)

**Date**: April 20, 2026
**From**: healthSpring V57
**To**: primalSpring, barraCuda, BearDog, NestGate, Rhizocrypt, Sweetgrass, all springs, biomeOS integrators
**Supersedes**: V56 NUCLEUS Validated (Level 4→5; three-tier architecture unchanged)

---

## What Happened

healthSpring's `healthspring_guidestone` binary passes **57/57 checks** (10 skipped)
against a live NUCLEUS comprising barraCuda, beardog, and nestgate. Exit code 0.
This is **guideStone Level 5 — primal proof** per `GUIDESTONE_COMPOSITION_STANDARD`
v1.2.0 (primalSpring v0.9.17, Phase 45).

```
substrate: x86_64 linux
family:    healthspring-validation
engine:    cpu-native (pure Rust, NUCLEUS auto-detected)
standard:  GUIDESTONE_COMPOSITION_STANDARD v1.2.0

Tier 1 (LOCAL): 43/43 PASS — 5 bare properties + 17 BLAKE3 checksums + domain science
Tier 2 (IPC):   8/8  PASS — 4 math methods + 2 storage ops + 2 liveness probes
                       3   SKIP — crypto probes (schema mismatch, Gap 21)
                       7   SKIP — socket discovery miss (dag/ai/commit, Gap 22)
Tier 3 (PROOF): 6/6  PASS — 4 math parity + domain science local confirmation
```

---

## What This Proves

The full **Python → Rust → library → primal IPC** chain is validated for
healthSpring's core science:

| Stage | Verified By | Status |
|-------|------------|--------|
| L1 Python baseline | 54 control scripts in `control/` | 54/54 |
| L2 Rust proof | 948+ unit tests, 84 science experiments | ALL PASS |
| L3 barraCuda CPU | `math_dispatch` library calls | ALL PASS |
| L4 GPU | 6 WGSL shader ops via `GpuContext` | ALL PASS |
| **L5 guideStone (primal proof)** | **Tier 1+2+3: live IPC against 3-primal NUCLEUS** | **57/57 PASS** |

---

## Live IPC Evidence

NUCLEUS primals started from local source builds:

```
barraCuda:  tensor math via stats.* methods
beardog:    security liveness (crypto.* probes skipped — Gap 21)
nestgate:   storage.store + storage.retrieve round-trip
```

Parity results (Tier 2 + Tier 3 combined):

| Method | Local (Rust) | Composition (IPC) | Diff | Tolerance |
|--------|-------------|-------------------|------|-----------|
| `stats.mean` | 5.5 | 5.5 | 0.00e0 | 1.00e-10 |
| `stats.std_dev` | 3.0276503540974917 | 3.0276503540974917 | 0.00e0 | 1.00e-10 |
| `stats.variance` | 9.166666666666668 | 9.166666666666666 | 1.78e-15 | 1.00e-10 |
| `stats.correlation` | 1.0 | 1.0 | 0.00e0 | 1.00e-10 |
| `storage.store` | — | stored successfully | — | — |
| `storage.retrieve` | — | round-trip match | — | — |

Domain science (Hill, Shannon, Simpson, Bray-Curtis) validated locally in Tier 1
as compositions of the IPC-proven primitives above.

---

## What Changed Since V56

| Item | V56 | V57 |
|------|-----|-----|
| primalSpring | v0.9.16 | **v0.9.17** |
| Composition Standard | v1.1.0 | **v1.2.0** |
| Checks (pass/skip) | 49/14 | **57/10** |
| Math methods validated | 2 (mean, std_dev) | **4** (+ variance, correlation) |
| Storage validated | — | **nestgate round-trip** |
| Primals in NUCLEUS | 1 (barraCuda) | **3** (+ beardog, nestgate) |
| `GUIDESTONE_READINESS` | 4 | **5** |
| Gap 19 (variance/correlation) | open | **RESOLVED** (Sprint 44) |
| Gaps 20–22 | — | **NEW** (documented) |

---

## New Gaps Filed (docs/PRIMAL_GAPS.md)

### Gap 20: BTSP Production Mode Breaks IPC

**Affects**: primalSpring transport layer
**Symptom**: Setting `FAMILY_SEED` causes `Transport::connect` to attempt BTSP
handshake. Non-BTSP primals (barraCuda, nestgate, etc.) reject the handshake
silently, causing all IPC to fail.
**Workaround**: Unset `FAMILY_SEED` / `BEARDOG_FAMILY_SEED` /
`RHIZOCRYPT_FAMILY_SEED` before running `healthspring_guidestone`.
**Ask**: `Transport::connect` should negotiate BTSP capability, not attempt
unconditionally when `FAMILY_SEED` is present.

### Gap 21: Crypto Probe Schema Mismatch

**Affects**: BearDog JSON-RPC method signatures
**Symptom**: `crypto.hash` returns "Invalid base64 data"; `crypto.sign`
returns "Missing required parameter: message".
**Impact**: Crypto capabilities SKIPped in guideStone Tier 2.
**Ask**: Publish BearDog method parameter schemas in a wateringHole spec.

### Gap 22: Missing Socket Discovery for DAG/AI/Commit

**Affects**: Rhizocrypt, Squirrel, Sweetgrass socket registration
**Symptom**: `discover_by_capability` finds no sockets for `capability:dag`,
`capability:ai`, `capability:commit`. These primals either do not register
capability-keyed sockets or use a different naming convention.
**Impact**: 7 of 10 guideStone SKIPs trace to this.
**Ask**: Standardize capability-keyed socket registration across all primals,
or add primal-name-keyed fallback in `discover_by_capability`.

---

## What healthSpring Needs from Upstream

| Team | Ask | Priority |
|------|-----|----------|
| **primalSpring** | Fix BTSP negotiate (Gap 20) | High |
| **BearDog** | Publish `crypto.*` parameter schemas (Gap 21) | Medium |
| **Rhizocrypt** | Register `capability:dag` socket (Gap 22) | Medium |
| **Sweetgrass** | Register `capability:commit` socket (Gap 22) | Medium |
| **Squirrel** | Register `capability:ai` socket (Gap 22) | Low |
| **barraCuda** | Sprint 44 delivered — no outstanding asks | — |
| **NestGate** | Storage validated — no outstanding asks | — |

---

## How to Reproduce

```bash
cd healthSpring

# Build guideStone
cargo build --bin healthspring_guidestone --features guidestone

# Start NUCLEUS primals (separate terminals)
export FAMILY_ID=healthspring-validation
export BIOMEOS_SOCKET_DIR=/run/user/1000/biomeos

nohup barracuda server --unix $BIOMEOS_SOCKET_DIR/tensor-healthspring-validation.sock &
nohup beardog server --unix $BIOMEOS_SOCKET_DIR/security-healthspring-validation.sock &
nohup nestgate server --unix $BIOMEOS_SOCKET_DIR/storage-healthspring-validation.sock &

# Create capability symlinks
ln -sf tensor-healthspring-validation.sock $BIOMEOS_SOCKET_DIR/capability-tensor.sock
ln -sf security-healthspring-validation.sock $BIOMEOS_SOCKET_DIR/capability-security.sock
ln -sf storage-healthspring-validation.sock $BIOMEOS_SOCKET_DIR/capability-storage.sock

# Run guideStone (CRITICAL: unset FAMILY_SEED to avoid BTSP)
unset FAMILY_SEED BEARDOG_FAMILY_SEED RHIZOCRYPT_FAMILY_SEED
FAMILY_ID=healthspring-validation \
  target/debug/healthspring_guidestone

# Expected: 57/57 checks passed (10 skipped), exit code 0
```

---

## healthSpring Readiness Summary

| Property | Status |
|----------|--------|
| P1 Deterministic | All checks reproducible across runs |
| P2 Traceable | Every number traces to a paper or proof |
| P3 Self-Verifying | BLAKE3 checksums (17 files, `primalspring::checksums`) |
| P4 Env-Agnostic | Pure Rust, ecoBin, no network, no sudo |
| P5 Tolerance-Documented | All tolerances in `tolerances.rs` with provenance |
| **guideStone Level** | **5 — primal proof** |
| **Composition Standard** | **v1.2.0** |
| **primalSpring** | **v0.9.17** |
