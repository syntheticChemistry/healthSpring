<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# healthSpring V6.1 → petalTongue Absorption Complete / Lean Handoff

**Date**: March 9, 2026
**From**: healthSpring
**To**: petalTongue, biomeOS
**Status**: Absorption cycle complete — healthSpring leaned

---

## 1. Executive Summary

The V5 handoff prototyped `DataChannel`, `ClinicalRange`, chart renderers, and a clinical
theme inside healthSpring's `petaltongue-health` crate. petalTongue's commit `037caaa`
("deep debt evolution, healthSpring absorption, docs cleanup") confirms upstream absorption
of all V5 deliverables. healthSpring has now **leaned**: the `petaltongue-health` crate is
removed, and `ClinicalRange.status` is aligned to `String` to match petalTongue's upstream
schema.

---

## 2. What petalTongue Absorbed (Confirmed)

| V5 Deliverable | petalTongue Location | Status |
|----------------|----------------------|--------|
| `DataChannel` enum (4 variants) | `petal-tongue-core/src/data_channel.rs` | Absorbed — added `Deserialize` |
| `ClinicalRange` struct | same file | Absorbed — `status: String` (was `&'static str`) |
| `draw_timeseries` | `petal-tongue-graph/src/chart_renderer.rs` | Absorbed |
| `draw_distribution` | same file | Absorbed |
| `draw_bar_chart` | same file | Absorbed |
| `draw_gauge` | same file | Absorbed |
| `draw_node_detail` | same file | Absorbed (not yet wired to primal detail panel) |
| Clinical color theme | `petal-tongue-graph/src/clinical_theme.rs` | Absorbed |
| `healthspring-diagnostic.json` | `sandbox/scenarios/` | Absorbed — loadable scenario |
| Version parsing fix (string vs struct) | `dynamic_schema.rs` | Fixed |
| `#![forbid(unsafe_code)]` pattern | 16+ crates | Adopted |

### petalTongue Schema Delta

healthSpring serializes → petalTongue deserializes. Wire format is identical.

| Field | healthSpring (writer) | petalTongue (reader) |
|-------|----------------------|----------------------|
| `DataChannel` | `#[derive(Serialize)]` | `#[derive(Serialize, Deserialize)]` |
| `ClinicalRange.status` | `String` (aligned V6.1) | `String` |

---

## 3. What healthSpring Removed (Lean Phase)

| Removed | Reason |
|---------|--------|
| `petaltongue-health/` crate (render.rs, theme.rs, bin/) | Absorbed into petal-tongue-graph + petal-tongue-core |
| `petaltongue-health` workspace member | No longer needed |

### What healthSpring Keeps

| Kept | Reason |
|------|--------|
| `barracuda/src/visualization/types.rs` | `DataChannel`, `ClinicalRange` (Serialize-only side), `ScenarioNode`, `HealthScenario`, etc. |
| `barracuda/src/visualization/nodes.rs` | healthSpring-specific: `DiagnosticAssessment` → scenario nodes |
| `barracuda/src/visualization/mod.rs` | JSON export: `assessment_to_scenario`, `full_scenario_json` |
| `experiments/exp052_petaltongue_render/` | 31-check schema validation (validates JSON wire format) |

---

## 4. V5 Absorption Plan Status

| Phase | V5 Plan | Status |
|-------|---------|--------|
| Phase 1 — Bug Fixes | Version parsing, node positions, clippy | DONE (petalTongue `037caaa`) |
| Phase 2 — Schema Absorption | DataChannel, ClinicalRange → petal-tongue-core | DONE |
| Phase 3 — Renderer Absorption | draw_* → petal-tongue-graph | DONE |
| Phase 4 — healthSpring Goes Lean | Remove petaltongue-health | **DONE (this handoff)** |

---

## 5. Remaining Integration Gaps (petalTongue Side)

These are known gaps visible from healthSpring's perspective. They are NOT blocking
healthSpring but are opportunities for petalTongue:

1. **Primal detail panel doesn't render DataChannels** — `draw_channel` and `draw_node_detail`
   exist but are not called from the primal details panel. `DynamicScenarioProvider` puts
   `data_channels` into `properties`, but no adapter converts `PropertyValue` → `Vec<DataChannel>`
   → chart renderers.

2. **clinical_ranges not used for coloring** — parsed and stored but not applied to node or
   edge coloring.

3. **Node positions** — Bug 2 from V5 handoff: verify positions from scenario JSON are now
   used (petalTongue's `037caaa` addressed node positioning).

---

## 6. Metrics

| Metric | V5 | V6 | V6.1 |
|--------|----|----|------|
| Rust lib tests | 200 | 161 | 161 |
| Experiments | 27 | 30 | 30 |
| Binary checks | 346 | 371 | 371 |
| healthSpring crates | 8 | 8 | **7** (−petaltongue-health) |
| DataChannel types | 4 | 4 | 4 |
| ClinicalRange.status | `&'static str` | `&'static str` | **`String`** (aligned) |
| petaltongue-health | prototype | prototype | **removed** |

---

## 7. Files Changed in V6.1

| File | Change |
|------|--------|
| `petaltongue-health/` | **Removed** — entire crate deleted |
| `Cargo.toml` | Removed `petaltongue-health` from workspace members |
| `barracuda/src/visualization/types.rs` | `ClinicalRange.status`: `&'static str` → `String` |
| `barracuda/src/visualization/nodes.rs` | Added `.into()` to all `ClinicalRange` status literals |
| `README.md` | Updated directory structure, removed petaltongue-health references |
| `wateringHole/README.md` | Added V6.1 handoff, updated petalTongue status |
| This handoff | New |

---

## 8. Next Steps

healthSpring's petalTongue integration surface is now minimal and stable:

- **healthSpring writes** scenario JSON via `barracuda/src/visualization/`
- **petalTongue reads** scenario JSON via `DynamicScenarioProvider` + `DataChannel` types

Future evolution paths:

1. **New DataChannel types** (heatmap, network, 3D) — healthSpring prototypes → petalTongue absorbs
2. **Live streaming** — `toadStool` pipeline → petalTongue `ToadstoolDisplay` (V6 GPU pipeline ready)
3. **Field deployment** — petalTongue + healthSpring on edge device with GPU
