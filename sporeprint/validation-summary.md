+++
title = "healthSpring Validation Summary — V65a"
description = "PK/PD, gut microbiome, biosignal, drug discovery — 57 validation scenarios, B5 cross-tier parity proven, lithoSpore Module 8 ready, BLAKE3 provenance complete, zero debt"
date = 2026-05-25

[taxonomies]
primals = ["barracuda", "toadstool", "biomeos", "nestgate"]
springs = ["healthspring", "wetspring", "neuralspring", "groundspring"]
+++

## Status

- **1,021 Rust workspace tests** — 878 lib + 9 doc + 20 integration/composition + 12 integration_wfdb + 3 integration_registry + 5 forge + 6 parity + 1 experiment + 33 metalforge + 51 toadstool
- **113 Python cross-validation checks** (`control/pkpd/cross_validate.py`, Tracks 1–9)
- **7 clinical tracks**: PK/PD, microbiome, biosignal, endocrinology, NLME, comparative medicine, drug discovery
- **Sovereign NLME** (FOCE/SAEM) replaces proprietary NONMEM/Monolix
- **Species-agnostic PK** — same code for canine AD, feline hyperthyroid, human TRT
- **UniBin**: **`healthspring_unibin certify`** / **`validate`** / **`serve`** / **`status`** / **`version`** — certification + scenario validation without standalone fossil **`healthspring_guidestone`**

## Key validation binaries

- **`healthspring_unibin`** — `certify`, `validate`, `serve`, `status`, `version`
- **`validate_pk_models`** — Hill, 1-compartment PK, PopPK, Michaelis-Menten (`--format json` for projectNUCLEUS Tier 2)
- **`validate_ltee_b5`** — LTEE B5 symbiont PK/PD (Leonard 2024): colonization, production, gut-lumen PK, Hill efficacy (8/8 checks, `--format json`)
- **`healthspring_primal`** — biomeOS niche JSON-RPC server (`serve`, Unix socket + optional `--port` TCP)

**Legacy:** `healthspring_guidestone` remains a Cargo bin for compatibility; prefer **`healthspring_unibin certify`**. V61 absorbed certification logic into the **`certification/`** organelle (`fossilRecord/guidestone_prokaryotic_may2026/` documents the migration).

**Not shipped as standalone binaries:** `validate_gut_microbiome`, `validate_biosignal`, `validate_nlme` — those names never landed as separate `[[bin]]` targets; gut, biosignal, and NLME validation lives in **`experiments/exp*`** crates and **`control/`** scripts.

## Workload TOMLs

Skeleton available in `projectNUCLEUS/workloads/healthspring/`.

## See Also

- [healthSpring Science Hub](https://primals.eco/lab/springs/healthspring/) on primals.eco
- [baseCamp Paper 13](https://primals.eco/science/)
