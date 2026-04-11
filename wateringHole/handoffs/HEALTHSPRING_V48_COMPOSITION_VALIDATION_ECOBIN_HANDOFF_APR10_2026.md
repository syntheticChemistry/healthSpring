<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V48 — Composition Validation & ecoBin Harvest

**Date**: April 10, 2026
**From**: healthSpring
**To**: primalSpring, barraCuda, toadStool, biomeOS, all sibling springs
**Status**: V48 (0.8.0)

---

## Summary

healthSpring V48 introduces **Tier 4 composition validation** — proving that
the IPC dispatch surface (what biomeOS calls via JSON-RPC) produces
bit-identical results to direct Rust function calls. This closes the gap
between "validated science" and "validated primal composition."

The ecoBin has been built (static-PIE, 2.5 MB, x86_64-musl) and harvested
to `infra/plasmidBin/healthspring/`.

---

## What Changed

### Tier 4 Composition Validation (NEW)

Five experiments (exp112–116) + 12 integration tests validate the full
dispatch layer:

| Experiment | Checks | What it proves |
|---|---|---|
| exp112 — PK/PD dispatch | 12 | Hill, IV, AUC, allometric, MM, PopPK — IPC = direct Rust |
| exp113 — Microbiome dispatch | 10 | Shannon, Simpson, Pielou, Chao1, Anderson — IPC = direct Rust |
| exp114 — Capability surface | 17 | All 58+ methods dispatch, 10 domains, structured errors |
| exp115 — Proto-nucleate alignment | 20 | Socket resolution, discovery, PRIMAL_NAME/DOMAIN constants |
| exp116 — Provenance lifecycle | 14 | Registry, data sessions (begin/record/complete), trio probe |

**Tolerance**: `DETERMINISM` (1e-12). Zero divergence observed. The dispatch
pathway is pure `serde_json` serialization of IEEE 754 doubles — lossless.

### ecoBin Harvest

- **Binary**: `healthspring_primal` (static-PIE, x86_64-unknown-linux-musl)
- **Size**: 2.5 MB stripped
- **Location**: `infra/plasmidBin/healthspring/healthspring_primal`
- **SHA-256**: `eb510065cf550030a4897294ae4336897cd706635affe3c7c5d62e222691b435`
- **Capabilities**: 80+ (58 science + 22 infrastructure)
- **Subcommands**: `serve`, `version`, `capabilities`

### IPC Resilience

`resilient_send` wired into all cross-primal RPC paths (compute, data,
shader, inference). Exponential backoff retry for retriable errors (connect
failures, timeouts). Circuit breaker remains available for persistent
connections.

### Proto-Nucleate Aliases

- `health.pharmacology` → `science.pkpd.hill_dose_response`
- `health.clinical` → `science.diagnostic.assess_patient`
- `health.de_identify` → `science.clinical.patient_parameterize`
- `health.aggregate` → `science.diagnostic.population_montecarlo`
- `inference.*` (`complete`, `embed`, `models`, `route`) alongside `model.*`

### CI Evolution

- New `composition` job: runs all Tier 4 experiments + integration tests
- Cross-compile uploads ecoBin artifacts (x86_64 + aarch64)
- Static linkage verified in CI
- Weekly GPU tests on schedule

---

## Metrics

| Metric | V44 | V48 |
|---|---|---|
| Tests | 928 | 940+ |
| Experiments | 83 | 88 |
| Composition checks | 0 | 73 |
| Integration tests | 20 | 32 |
| Capabilities | 59 | 80+ |
| ecoBin | not built | 2.5 MB static-PIE |
| barraCuda | v0.3.7 | v0.3.11 |

---

## For primalSpring

1. **Proto-nucleate namespace decision needed**: healthSpring now serves
   `health.*` aliases alongside `science.*`. The proto-nucleate graph
   declares `health.pharmacology` etc. — should the canonical namespace
   be `science.*` (healthSpring's native) or `health.*` (proto-nucleate's
   declared)? See `docs/PRIMAL_GAPS.md` gap #1.

2. **Ionic bridge blocked**: BearDog lacks `crypto.ionic_bond` and NestGate
   lacks `storage.egress_fence`. The dual-tower enclave pattern cannot
   enforce its security model until these evolve. See gap #2.

3. **Songbird naming**: healthSpring calls `net.discovery.find_by_capability`;
   the proto-nucleate declares `discovery.find_primals`. Need alignment.
   See gap #3.

4. **Inference namespace**: healthSpring now supports both `model.*` and
   `inference.*`. Canonical namespace decision needed. See gap #4.

## For barraCuda

1. **v0.3.11 validated**: All CPU primitives pass. 6 WGSL shaders validated.
   Composition validation confirms zero divergence through dispatch layer.

2. **TensorSession**: Still evaluated as "ready when upstream ships." The
   MM batch + fused pipeline patterns are documented in
   `specs/EVOLUTION_MAP.md` as adoption candidates.

3. **Shader absorption candidates**: `hill_dose_response_f64.wgsl`,
   `population_pk_f64.wgsl`, `diversity_f64.wgsl` — all validated, all
   following Write → Absorb → Lean cycle.

## For toadStool

1. **Dispatch matrix validated**: exp086 (24/24) + exp087 (35/35) still pass.
2. **Streaming callbacks**: Live dashboard pipeline (exp072) confirms
   toadStool streaming dispatch works for real-time clinical scenarios.

## For biomeOS

1. **ecoBin ready**: Static binary at `infra/plasmidBin/healthspring/`.
   Responds to `--version`, `capabilities`, `serve`. Checksum in metadata.
2. **Socket convention**: Binds to `$XDG_RUNTIME_DIR/biomeos/healthspring-default.sock`.
3. **Readiness probe**: `health.readiness` now gates on science dispatch status.
4. **Capability discovery**: 80+ capabilities via `capability.list`.

## For sibling springs

This composition validation pattern (Tier 4: dispatch parity) is reusable.
The pattern:
1. Call `dispatch_science(method, params)` from the dispatch registry
2. Call the direct Rust function with the same arguments
3. Assert results match within `tolerances::DETERMINISM`
4. Test the full capability surface (all methods dispatch, structured errors)
5. Test proto-nucleate alignment (socket resolution, discovery, constants)

Any spring with a dispatch layer can adopt this pattern to prove its
composition surface is numerically faithful.

---

## Files Changed

- 5 new experiments: `experiments/exp112–exp116`
- New integration test: `ecoPrimal/tests/integration_composition.rs`
- New spec: `specs/COMPOSITION_VALIDATION.md`
- Updated: `CHANGELOG.md`, `README.md`, `specs/README.md`
- Updated: `.github/workflows/ci.yml` (composition + cross-compile jobs)
- Updated: `ecoPrimal/src/bin/healthspring_primal/capabilities.rs`
  (programmatic map, no recursion limit)
- Fixed: proptest edition 2024 compatibility, unused imports
- Harvested: `infra/plasmidBin/healthspring/` (binary + metadata)
