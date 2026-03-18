<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# healthSpring V37 — Cross-Ecosystem Absorption Sprint

**Date**: 2026-03-18
**From**: healthSpring V37
**To**: barraCuda team, toadStool team, coralReef team, Squirrel team, All Springs
**License**: AGPL-3.0-or-later
**Covers**: V36 → V37 (FMA sweep, centralized RPC extraction, proptest IPC fuzz, provenance registry, MCP tools, deny.toml hardening, leverage guide)
**Pins**: barraCuda v0.3.5 (rev a60819c3), toadStool S158+, coralReef Phase 10 Iteration 55+
**Supersedes**: HEALTHSPRING_V36_DEEP_DEBT_ECOSYSTEM_MATURITY_HANDOFF_MAR18_2026.md

---

## Executive Summary

- **`mul_add()` FMA sweep** — 8 sites across PK/ODE/biosignal/QS modules converted from `a * b + c` to `a.mul_add(b, c)` for IEEE 754 fused multiply-add (single rounding, potential hardware FMA instruction). Absorbed from neuralSpring S165 / airSpring V090 pattern.
- **Centralized `extract_rpc_result()`** — new `rpc::extract_rpc_result()` and `rpc::extract_rpc_result_owned()` replace 6 ad-hoc `val.get("result")` extraction sites. Absorbed from wetSpring V127 `extract_rpc_result()` pattern.
- **18 proptest IPC fuzz tests** — property-based fuzz testing for `extract_rpc_result`, `classify_response`, `extract_capability_strings`, and JSON-RPC 2.0 structural invariants. Absorbed from ludoSpring V24 / airSpring V090 proptest pattern.
- **Structured provenance registry** — `PROVENANCE_REGISTRY` with 49 `ProvenanceRecord` entries for all Python control scripts. Completeness test enforces that new scripts must have registry entries. Absorbed from wetSpring V127 provenance pattern.
- **MCP tool definitions** — 23 typed MCP tool schemas for Squirrel AI coordination, exposed via `mcp.tools.list` JSON-RPC method. Absorbed from wetSpring V127 MCP pattern.
- **`deny.toml` hardened** — 14-crate C-dep ban list (ecoBin compliance), `ring` wrapper exception for `rustls`, `all-features = true`, `confidence-threshold = 0.8`, `unknown-git = "deny"`. Absorbed from groundSpring V115.
- **Leverage guide published** — `HEALTHSPRING_LEVERAGE_GUIDE.md` covering standalone, trio, and composition usage patterns.
- **706 tests**, 79 experiments, 80 capabilities, zero clippy, zero unsafe, zero `#[allow()]`

---

## Part 1: What Changed (V36 → V37)

### FMA Sweep (8 sites)

| File | Expression | FMA Form |
|------|-----------|----------|
| `pkpd/util.rs` | `0.5 * (c[i-1] + c[i]) * dt` trapezoidal AUC | `(0.5 * dt).mul_add(sum, auc)` |
| `pkpd/nca.rs` | `0.5 * (tc_prev + tc_curr) * dt` AUMC | `(0.5 * dt).mul_add(sum, aumc)` |
| `biosignal/ppg.rs` | `dc_red + ac_red * pulse + noise` | `ac_red.mul_add(pulse, dc_red + noise)` |
| `biosignal/ppg.rs` | `dc_ir + ac_ir * pulse + noise_ir` | `ac_ir.mul_add(pulse, dc_ir + noise_ir)` |
| `comparative/canine.rs` | `7.0 * dose / 0.5 + 7.0` | `14.0_f64.mul_add(dose, 7.0)` |
| `qs.rs` | `alpha * w_struct + (1-alpha) * w_func` | `alpha.mul_add(w_struct, rest)` |
| `exp031` | trapezoidal AUC | `(0.5 * dt).mul_add(sum, auc)` |
| `exp107` | QS blending | `ALPHA.mul_add(w_struct, rest)` |

### Centralized RPC Result Extraction

New functions in `ipc::rpc`:
- `extract_rpc_result(response) → Option<&Value>` — borrows
- `extract_rpc_result_owned(response) → Option<Value>` — clones

Migrated 6 call sites:
- `ipc::rpc::try_send` — was `parsed.get("result").cloned()`
- `ipc::socket::extract_capability_strings` — was `result.get("result")`
- `ipc::socket::probe_capability` — was `resp.get("result")`
- `visualization::capabilities::query` — was `response.get("result")`
- `data::rpc::rpc_call` — was `resp.get("result").cloned()`
- `data::provenance::capability_call` — was `parsed.get("result").cloned()`

### Proptest IPC Fuzz (18 tests)

| Category | Tests | What's Fuzzed |
|----------|-------|--------------|
| `extract_rpc_result` round-trip | 4 | Error→None, Result→Some, Neither→None, Both→None |
| `extract_rpc_result_owned` parity | 2 | Owned matches borrowed.cloned() |
| `classify_response` consistency | 3 | Ok→Some, Error→None, Neither→None |
| `extract_capability_strings` | 6 | Format A–E plus result wrappers |
| JSON-RPC 2.0 invariants | 3 | success/error constructors, structure |

### Provenance Registry

`PROVENANCE_REGISTRY` in `provenance.rs` — 49 `ProvenanceRecord` entries:

| Track | Scripts | Key Experiments |
|-------|---------|----------------|
| pkpd | 8 | exp001–006, exp077, cross_validate |
| microbiome | 7 | exp010–013, exp078–080 |
| biosignal | 6 | exp020–023, exp081–082 |
| endocrine | 9 | exp030–038 |
| comparative | 7 | exp100–106 |
| discovery | 5 | exp090–094 |
| validation | 1 | exp040 |
| scripts | 6 | benchmarks, tolerances, provenance |

Completeness test walks `control/` at runtime and asserts count matches.

### MCP Tool Definitions (23 tools)

| Domain | Tools | Methods |
|--------|-------|---------|
| PK/PD | 6 | hill, 1-cpt, 2-cpt, PBPK, popPK, MM |
| Microbiome | 5 | shannon, anderson, cdiff, SCFA, QS |
| Biosignal | 4 | QRS, HRV, SpO2, arrhythmia |
| Endocrine | 3 | T-PK, TRT outcomes, serotonin |
| Compute | 2 | offload, shader_compile |
| Model | 1 | inference_route |
| Health | 2 | liveness, readiness |

Exposed via `mcp.tools.list` JSON-RPC method.

### deny.toml Hardening

Added from groundSpring V115:
- `[graph] all-features = true`
- `confidence-threshold = 0.8`
- `[[licenses.clarify]]` for `ring` (MIT AND ISC AND OpenSSL)
- 14-crate ban list: openssl-sys, openssl, native-tls, aws-lc-sys, aws-lc-rs, ring (wrapper: rustls), libz-sys, bzip2-sys, curl-sys, libsqlite3-sys, cmake, cc (wrapper: ring), pkg-config, vcpkg
- `unknown-git = "deny"`, `allow-git = []`

---

## Part 2: Absorption Sources

| Pattern | Source | healthSpring Implementation |
|---------|--------|----------------------------|
| `mul_add()` FMA sweep | neuralSpring S165, airSpring V090 | 8 sites in lib + experiments |
| `extract_rpc_result()` | wetSpring V127 | `ipc::rpc` module, 6 migrated sites |
| Proptest IPC fuzz | ludoSpring V24, airSpring V090, neuralSpring S163 | `ipc::proptest_ipc` 18 tests |
| Provenance registry | wetSpring V127 `provenance.rs` | 49 records with completeness test |
| MCP tool definitions | wetSpring V127 MCP pattern | 23 tools, `mcp.tools.list` method |
| `deny.toml` 14-crate ban | groundSpring V115 | Adopted with `ring` wrapper exception |
| Leverage guide | wetSpring V127, ludoSpring V24 | `HEALTHSPRING_LEVERAGE_GUIDE.md` |

---

## Part 3: Patterns Worth Adopting (For All Springs)

1. **Provenance completeness test** — healthSpring's `registry_complete` test walks `control/` and asserts every `.py` file has a registry entry. Catches undocumented baselines at compile time. Recommend all springs adopt.

2. **`extract_rpc_result()` centralization** — any spring doing ad-hoc `val.get("result")` should centralize. The function also handles the `"error"` precedence rule (error present → None).

3. **MCP tool definitions** — typed schemas so Squirrel can discover and invoke capabilities. The `mcp.tools.list` method is a simple way to expose them.

4. **FMA audit** — `mul_add()` is a one-pass audit with measurable accuracy improvement. No API changes, no breakage risk.

---

## Part 4: Recommended Upstream Actions

### For barraCuda

| Priority | Action | Context |
|----------|--------|---------|
| **P1** | Add `sample_variance(data) → f64` to `barracuda::stats` | healthSpring `uncertainty.rs` still computes manually |
| **P2** | Stabilize `TensorSession` for fused pipelines | healthSpring `gpu/fused.rs` is local implementation |
| **P2** | Add `mul_add` to CPU reference implementations | Follow neuralSpring/healthSpring pattern |

### For Squirrel

| Priority | Action | Context |
|----------|--------|---------|
| **P1** | Consume `mcp.tools.list` from healthSpring | 23 typed tool schemas now available |
| **P2** | Add `tool.invoke` coordination flow | Route AI requests to healthSpring science methods |

### For All Springs

| Priority | Action | Context |
|----------|--------|---------|
| **P1** | Adopt `extract_rpc_result()` pattern | Prevents silent data loss on RPC error responses |
| **P2** | Provenance completeness test | Catches undocumented baselines |
| **P2** | FMA sweep | Accuracy improvement with zero risk |

---

## Part 5: Quality Metrics

| Metric | V36 | V37 | Δ |
|--------|-----|-----|---|
| Tests | 617 | **706** | +89 |
| Proptest IPC fuzz | 0 | **18** | +18 |
| Experiments | 79 | 79 | — |
| JSON-RPC capabilities | 79 | **80** | +1 (`mcp.tools.list`) |
| MCP tool schemas | 0 | **23** | +23 |
| Provenance records | (undocumented) | **49** | +49 |
| Ad-hoc `val.get("result")` | 6 | **0** | -6 (centralized) |
| FMA `mul_add()` sites (lib) | 15 | **23** | +8 |
| C-dep crates banned | 0 | **14** | +14 |
| `deny.toml` features | basic | **hardened** | Upgraded |
| Unsafe blocks | 0 | 0 | — |
| Clippy warnings | 0 | 0 | — |

---

## Verification

```bash
cargo fmt --check --all          # 0 diffs
cargo clippy --workspace --all-targets  # 0 warnings
cargo test --workspace           # 706 passed, 0 failed
cargo doc --workspace --no-deps  # 0 warnings
```

---

**healthSpring V37 | 706 tests | 79 experiments | 80 capabilities | 23 MCP tools | 49 provenance records | AGPL-3.0-or-later**
