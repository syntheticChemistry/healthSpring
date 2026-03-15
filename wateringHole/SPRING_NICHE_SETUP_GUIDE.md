<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Spring Niche Setup Guide

**ECOSYSTEM GUIDE**, v1.0.0, March 15 2026, wateringHole authority.

---

## 1. What is a Spring Niche?

A **niche** is what primals do together. The primal provides capabilities; graphs define composition. With primals registered and graphs loaded, biomeOS recreates the workflow through `capability.call` routing. The Neural API Pathway Learner optimizes over time.

---

## 2. Minimum Viable Niche

What you need:

a. **A primal binary** — JSON-RPC 2.0 server, Unix socket, capability registration  
b. **An IPC dispatch module** — method → science function routing  
c. **A niche manifest** — TOML: niche name, primals, graphs  
d. **At least one workflow graph** — TOML: nodes with capability refs, `depends_on`, `budget_ms`  
e. **A niche deploy graph** — TOML: startup order for all primals  

---

## 3. Reference: healthSpring

How healthSpring did it:

- **`ecoPrimal/src/bin/healthspring_primal.rs`**: 55 capabilities, XDG socket, biomeOS registration + heartbeat  
- **`ecoPrimal/src/ipc/dispatch.rs`**: Maps method strings to science functions, extracts JSON params  
- **`graphs/healthspring_niche.toml`**: Manifest with 5 workflow graphs  
- **`graphs/healthspring_patient_assessment.toml`**: ConditionalDag — 4 parallel science tracks  
- **`graphs/healthspring_biosignal_monitor.toml`**: Continuous @ 250 Hz  

---

## 4. Step-by-step

- **Step 1**: Create `src/ipc/` module (`dispatch.rs`, `rpc.rs`, `socket.rs`)  
- **Step 2**: Create `src/bin/<spring>_primal.rs` following the airSpring/healthSpring pattern  
- **Step 3**: Add `[[bin]]` to `Cargo.toml`  
- **Step 4**: Register capabilities with biomeOS on startup  
- **Step 5**: Create `graphs/<spring>_niche.toml` manifest  
- **Step 6**: Create workflow graphs (at least one transactional)  
- **Step 7**: Create niche deploy graph  

---

## 5. Compliance

Reference `SPRING_AS_NICHE_DEPLOYMENT_STANDARD.md` for mandatory requirements (UniBin, capability domain, `health.check`, `capability.list`). This guide is the practical "how-to" complement.

---

## 6. Current Springs with Niches

| Spring | Domain | Niche Version |
|--------|--------|---------------|
| airSpring | ecology | v0.8.0 |
| ludoSpring | game | reference |
| healthSpring | health | v0.1.0 |

**Others** (evolution path documented): wetSpring, hotSpring, groundSpring.
