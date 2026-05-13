<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V64f — Tier 2 Convergence Handoff

**From**: healthSpring
**To**: primalSpring, lithoSpore, projectNUCLEUS, plasmidBin
**Date**: 2026-05-13
**Version**: V64f (1,014 tests, 17 scenarios, 88 capabilities, zero clippy)
**Audit response**: ecoPrimals Delta Spring Evolution — Tier 2 Convergence Wave (May 13, 2026)

---

## What healthSpring Shipped

### Tier 2 IPC Wiring (highest leverage)

| Upstream Method | healthSpring Module | Function | Status |
|-----------------|-------------------|----------|--------|
| `toadstool.validate` | `ipc::compute_dispatch` | `validate_workload()` | WIRED |
| `toadstool.list_workloads` | `ipc::compute_dispatch` | `list_workloads()` | WIRED |
| `barracuda.precision.route` | `ipc::barracuda_client` | `precision_route()` | WIRED |
| `crypto.contract.propose` | `ipc::tower_atomic` | `ionic_propose()` | WIRED |
| `crypto.contract.countersign` | `ipc::tower_atomic` | `ionic_countersign()` | WIRED |
| `crypto.contract.verify` | `ipc::tower_atomic` | `ionic_verify()` | WIRED |

`barracuda.precision.route` response fields aligned to canonical `LIVE_SCIENCE_API.md`
contract: `recommended_tier`, `fma_safe`, `requires_compiler`, `hardware_hint`.

### Domain → Precision Mapping

`docs/PRIMAL_PROOF_IPC_MAPPING.md` created — maps all 17 healthSpring domain
operations (PK/PD, microbiome, biosignal, toxicology, simulation) to
`barracuda.precision.route` queries with expected tiers and FMA safety.

### plasmidBin Deployment Readiness

- **musl static binary**: `healthspring_unibin` builds and runs standalone on
  `x86_64-unknown-linux-musl`
- **`--list` flag**: Added to `validate` subcommand — `healthspring_unibin validate --list`
  lists all 17 scenarios without executing
- **`--format json`**: Produces structured output for Tier 2 projectNUCLEUS ingestion
- **Cell TOML updated**: `healthspring_cell.toml` now includes compute trio nodes
  (toadStool, barraCuda, coralReef) and updated validation targets
- **Niche promoted**: `healthspring` niche in `manifest.toml` updated from `nest`
  to `full` composition with all 12 NUCLEUS primals

### LTEE B5 lithoSpore Module Candidate

Module `ltee-symbiont-pk` is fully packaged:

```
control/ltee_symbiont_pkpd/
├── expected_values.json        # Ground-truth parameters
├── tolerances.toml             # Machine-readable CI tolerances
├── ltee_symbiont_pkpd.py       # Python Tier 0 baseline (deterministic)
├── LITHO_MODULE_README.md      # Exact reproduction commands
├── benchmark_ltee_symbiont.json
└── __init__.py
```

Rust Tier 1 binary: `validate_ltee_b5` (8/8 checks pass, `--format json`)

**Ready for lithoSpore BLAKE3 ingestion.**

---

## Gaps Surfaced Upstream

### New (discovered during convergence wiring)

| # | Gap | For |
|---|-----|-----|
| 30 | `precision.route` blurb contract diverges from `LIVE_SCIENCE_API.md` — healthSpring wires to `LIVE_SCIENCE_API.md` as canonical | primalSpring: reconcile |
| 31 | lithoSpore module ingestion for B5 pending | lithoSpore: accept `ltee-symbiont-pk` candidate |

### Pre-existing (unchanged, documented in `docs/PRIMAL_GAPS.md`)

| # | Gap | Blocked On |
|---|-----|------------|
| 2 | NestGate egress fence for ionic bridge | NestGate evolution |
| 10 | BTSP server endpoint | BearDog BTSP server |
| 20 | BTSP production mode breaks IPC | primalSpring transport negotiation |
| 22 | Socket discovery (DAG/AI/commit) | Ecosystem socket standardization |
| 23 | Provenance trio empty UDS responses | Trio startup config |
| 24 | Songbird crypto provider discovery | Songbird startup docs |

---

## Test & Capability Summary

| Metric | Value |
|--------|-------|
| Tests | 1,014 (874 lib + 9 doc + 131 integration) |
| Scenarios | 17 |
| Capabilities | 88 (58 science + 30 infra) |
| Clippy warnings | 0 |
| Unsafe code | 0 |
| LTEE reproductions | B5 COMPLETE (Python + Rust) |
| Foundation threads | T3 active, T5 active, T8 active |
| GuideStone level | 5 |
| IPC-first default | yes (`default = []`) |

---

## Not Done (per audit guidance)

- **Did not touch compute trio internals** — toadStool/barraCuda/coralReef sovereignty belongs to hotSpring on biomeGate
- **Did not wait for ionic runtime** — wired what's available now per audit guidance
- **Did not remove library fallback** — `barracuda-lib` feature flag preserved for toggle
