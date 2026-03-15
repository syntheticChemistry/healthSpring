<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V22 — biomeOS BYOB Niche Deployment: barraCuda/toadStool Handoff

**Date**: March 15, 2026
**From**: healthSpring
**To**: barraCuda team, toadStool team, biomeOS team
**License**: CC-BY-SA-4.0

---

## Executive Summary

healthSpring V22 evolves from experiment binaries to a **biomeOS BYOB niche deployment**. The `healthspring_primal` binary serves 55 capabilities via JSON-RPC 2.0 over Unix socket. 5 workflow graphs define the niche: patient assessment (ConditionalDag), TRT scenario (Sequential), microbiome analysis (Sequential), biosignal monitor (Continuous @ 250 Hz), and niche deploy. 414 tests pass, clippy pedantic clean.

---

## Part 1: For barraCuda

The healthSpring IPC dispatch module (`ecoPrimal/src/ipc/dispatch.rs`) maps 50+ JSON-RPC methods to barraCuda science primitives. Every call crosses the `barracuda::stats` / `barracuda::special` boundary. Key absorption opportunities:

- **dispatch.rs** contains JSON→Rust param extraction patterns that could be generalized into a `barracuda::ipc` macro/helper
- The **population_pk** dispatch shows how to wire `LognormalParam` + Monte Carlo through JSON-RPC
- **Anderson diagonalize** dispatched from JSON shows the eigenvalue → IPR → colonization pipeline end-to-end

---

## Part 2: For toadStool

The `biosignal_monitor.toml` graph defines a 250 Hz continuous niche. Pan-Tompkins at 250 Hz needs < 1.5ms per chunk. Current CPU path works but NPU offload via toadStool would unlock real-time clinical monitoring. The `patient_assessment` graph shows 4 parallel science tracks — toadStool could GPU-batch these when data scales up.

---

## Part 3: Niche Architecture

healthSpring is a **niche**, not a node. The primal provides capabilities; the graphs define composition. With primals + graphs, biomeOS recreates the entire diagnostic pipeline. This is the first health science niche in the ecosystem.

---

## Part 4: Quality Gates

| Gate | Status |
|------|--------|
| `cargo check` | pass |
| `cargo clippy --pedantic` | pass |
| `cargo test` | 414 pass |
| `cargo build --bin healthspring_primal` | pass |

---

## Part 5: Files Changed

**New:**
- `ecoPrimal/src/ipc/mod.rs`
- `ecoPrimal/src/ipc/dispatch.rs`
- `ecoPrimal/src/ipc/rpc.rs`
- `ecoPrimal/src/ipc/socket.rs`
- `ecoPrimal/src/bin/healthspring_primal.rs`
- `graphs/*.toml` (6 files)

**Modified:**
- `ecoPrimal/Cargo.toml`
- `ecoPrimal/src/lib.rs`
- `wateringHole/healthspring_deploy.toml`
- `specs/EVOLUTION_MAP.md`
- `specs/NUCLEUS_INTEGRATION.md`
