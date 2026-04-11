# healthSpring V45 — Capability Sync + IPC Fuzz Evolution Handoff

**Date:** April 7, 2026
**From:** healthSpring V45
**To:** primalSpring coordination, wetSpring, ecosystem
**License:** AGPL-3.0-or-later
**Supersedes:** HEALTHSPRING_V44_PRIMAL_SPRING_EVOLUTION_HANDOFF_MAR24_2026.md

---

## Executive Summary

V45 resolves the P1 capability registration desync identified in the V44 audit
and evolves the proptest IPC fuzz module per primalSpring Phase 12.1 guidance.
Three-way capability sync (ALL_CAPABILITIES ↔ dispatch REGISTRY ↔ niche YAML)
is now exact: 58 science + 14 infrastructure capabilities, all with live handlers.
Proptest coverage expands to include trio witness wire type fuzzing and
DispatchOutcome consistency checks.

---

## 1. Capability Registration Sync (P1 Resolution)

### Problem

Three sources of truth disagreed:
- `niches/healthspring-health.yaml`: 53 science (over-claimed comparative + discovery)
- `ALL_CAPABILITIES`: 46 science (missing comparative, discovery, toxicology, simulation)
- `dispatch/mod.rs` REGISTRY: 51 science (had toxicology + simulation, lacked comparative + discovery)

### Resolution

| Domain | Handlers Added | Methods |
|--------|---------------|---------|
| Comparative (3) | `handlers/comparative.rs` | `cross_species_pk`, `canine_il31`, `canine_jak1` |
| Discovery (4) | `handlers/discovery.rs` | `matrix_score`, `hts_analysis`, `compound_library`, `fibrosis_pathway` |
| Toxicology (3) | Already existed | Added to `ALL_CAPABILITIES` + niche YAML |
| Simulation (2) | Already existed | Added to `ALL_CAPABILITIES` + niche YAML |

**Final count: 58 science + 14 infrastructure = 72 total capabilities.**
All three sources now produce identical science capability sets (verified via diff).

### `build_semantic_mappings()` Completion

Was: 22 of 46 science capabilities mapped.
Now: All 58 science capabilities mapped.

### Niche YAML Version Alignment

`niches/healthspring-health.yaml` version updated from `"0.26.0"` to `"0.1.0"` to
match `ecoPrimal/Cargo.toml` package version.

---

## 2. Proptest IPC Fuzz Evolution

### Context

primalSpring absorbed healthSpring V42's proptest_ipc pattern (Phase 12.1). The
upstream audit identified two gaps:

1. No trio witness wire type fuzzing (new fields: kind, encoding, algorithm, tier, context)
2. No DispatchOutcome ↔ extract_rpc_result consistency checks

### New Property Tests (Section 6: DispatchOutcome Consistency)

| Test | Invariant |
|------|-----------|
| `dispatch_outcome_extract_consistency` | `extract_rpc_result` and `classify_response` agree on success vs failure |
| `dispatch_outcome_error_consistency` | Both paths agree on error (protocol + application codes) |
| `protocol_error_range_classified_correctly` | Codes -32700..-32600 → `ProtocolError` |
| `application_error_range_classified_correctly` | Codes outside protocol range → `ApplicationError` |

### New Property Tests (Section 7: Trio Witness Wire Type Fuzzing)

| Test | What it fuzzes |
|------|----------------|
| `trio_witness_wire_roundtrip` | All field combinations (kind × encoding × algorithm × tier × session_id × merkle_root × context) |
| `provenance_chain_wire_roundtrip` | `DataProvenanceChain` payloads (status × session_id × merkle_root × commit_id × braid_id) |
| `trio_witness_in_rpc_response_extracts` | Witness embedded in JSON-RPC response envelope extracts cleanly |
| `trio_witness_arbitrary_context_never_panics` | Arbitrary strings in context field → no panics via extract or classify |

**Total proptest property tests: ~30 (up from ~22).**

---

## 3. New IPC Handler Files

### `ecoPrimal/src/ipc/dispatch/handlers/comparative.rs`

- `dispatch_cross_species_pk`: Allometric CL/Vd/t½ scaling across species
- `dispatch_canine_il31`: IL-31 serum kinetics under oclacitinib/lokivetmab/untreated
- `dispatch_canine_jak1`: JAK1 selectivity panel vs oclacitinib reference

### `ecoPrimal/src/ipc/dispatch/handlers/discovery.rs`

- `dispatch_matrix_score`: Fajgenbaum MATRIX + Anderson geometry scoring
- `dispatch_hts_analysis`: Z'-factor + SSMD hit classification
- `dispatch_compound_library`: Batch Hill IC50 estimation
- `dispatch_fibrosis_pathway`: Rho/MRTF/SRF pathway scoring + fibrotic geometry

All follow existing handler conventions: param extraction via shared helpers,
`missing()` for absent params, domain module delegation.

---

## 4. Hormesis × wetSpring Alignment Status

healthSpring `toxicology/hormesis.rs` and wetSpring `bio/hormesis.rs` share the
same biphasic dose-response mathematics:

```
R(D) = baseline × (1 + stimulation(D)) × (1 - inhibition(D))
```

**healthSpring parameterization:** `(dose, baseline, s_max, k_stim, ic50, hill_n)`
**wetSpring parameterization:** `(dose, A, K_stim, n_s, K_inh, n_i)` (separate Hill coefficients)

Cross-validation path: When wetSpring runs Anderson QS pipeline with provenance
witnesses, healthSpring validates that hormesis-relevant parameters are consistent
via shared `biphasic_dose_response` math. The IPC handler
`science.toxicology.biphasic_dose_response` is now fully wired for cross-spring
validation calls.

Remaining: formal cross-spring experiment (exp379 × exp097/098 joint protocol)
documented in `WETSPRING_V130_ANDERSON_HORMESIS_FRAMEWORK_HANDOFF_MAR19_2026.md`.

---

## 5. Trio Witness Handoff Note

The `PRIMALSPRING_TRIO_WITNESS_HARVEST_HANDOFF_APR07_2026.md` referenced in the
upstream audit **does not yet exist** in `infra/wateringHole/`. healthSpring's
trio witness fuzzing is built against:

- Current trio types: `ProvenanceResult`, `DataProvenanceChain` (in `data/provenance.rs`)
- Field semantics from audit blurb: `kind`, `encoding`, `algorithm`, `tier`, `context`
- primalSpring `ipc/provenance.rs` patterns: `ProvenanceStatus`, `PipelineResult`

When the upstream handoff materializes, reconcile field names and add any new
variants to the proptest strategies.

---

## Files Changed

| File | Change |
|------|--------|
| `ecoPrimal/src/ipc/dispatch/handlers/comparative.rs` | **New** — 3 handlers |
| `ecoPrimal/src/ipc/dispatch/handlers/discovery.rs` | **New** — 4 handlers |
| `ecoPrimal/src/ipc/dispatch/handlers/mod.rs` | Added module declarations + 12 handler tests |
| `ecoPrimal/src/ipc/dispatch/mod.rs` | Added 7 REGISTRY entries (comparative + discovery) |
| `ecoPrimal/src/bin/healthspring_primal/capabilities.rs` | Synced ALL_CAPABILITIES (58 science) + complete semantic mappings |
| `ecoPrimal/src/ipc/proptest_ipc.rs` | Added sections 6 (DispatchOutcome) + 7 (trio witness) |
| `niches/healthspring-health.yaml` | Added toxicology + simulation; version → 0.1.0 |

---

## Metrics Delta

| Metric | V44 | V45 |
|--------|-----|-----|
| Science capabilities (registered) | 46 | **58** |
| Infrastructure capabilities | 14 | 14 |
| Dispatch handlers | 51 | **58** |
| Semantic mappings | 22 | **58** |
| Proptest property tests | ~22 | **~30** |
| Three-way capability diff | 12 mismatches | **0** |
