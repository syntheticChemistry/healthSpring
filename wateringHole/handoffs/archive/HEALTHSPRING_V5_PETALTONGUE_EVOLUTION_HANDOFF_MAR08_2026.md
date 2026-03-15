<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# healthSpring V5 → petalTongue Evolution Handoff

**Date**: March 8, 2026
**From**: healthSpring
**To**: petalTongue, biomeOS
**Status**: Active — prototypes built, ready for absorption

---

## 1. Executive Summary

healthSpring V5 pivots from compute pipeline validation to **driving petalTongue's evolution into a universal data visualization UI**. We built working prototypes of the schema extensions, rendering components, and clinical domain features that petalTongue needs to become "any user, any data."

### What We Built

| Component | Location | Purpose |
|-----------|----------|---------|
| **Enriched Scenario JSON** | `barracuda/src/visualization.rs` | Native petalTongue v2.0 schema with `DataChannel` extensions |
| **Data Channel Types** | `DataChannel` enum (serde) | TimeSeries, Distribution, Bar, Gauge — typed charting |
| **Clinical Ranges** | `ClinicalRange` struct | Normal/warning/critical threshold coloring |
| **Rich Diagnostic Data** | `barracuda/src/diagnostic.rs` | Full PK curves, Hill sweeps, RR tachograms, gut abundances |
| **Interactive UI Prototype** | `petaltongue-health/` crate | egui dashboard: topology, charts, gauges, click-to-inspect |
| **Schema Validation** | `experiments/exp052_petaltongue_render/` | 31-check schema round-trip validation |

---

## 2. Bugs Found in petalTongue (Fixes Required)

### Bug 1: `DynamicData` version field expects struct, JSON has string

**Location**: `petal-tongue-core/src/dynamic_schema.rs`
**Impact**: `DynamicScenarioProvider` fails on any scenario with `"version": "2.0.0"`
**Root Cause**: `SchemaVersion { major, minor, patch }` can't deserialize from a JSON string
**Fix**: Add `#[serde(deserialize_with = "...")]` for semver string parsing, or accept both

### Bug 2: Scenario node positions ignored

**Location**: `petal-tongue-ui/src/app.rs`, `graph_engine.rs`
**Impact**: All nodes render at (0,0) regardless of scenario `position` field
**Root Cause**: `graph.add_node()` uses `Node::new()` which defaults position; should use `Node::with_position()`
**Fix**: Pass `(primal.position.x, primal.position.y)` to `Node::with_position`

### Bug 3: ~96 clippy warnings

**Location**: Across crates
**Impact**: Code quality; potential real bugs masked
**Fix**: Address or suppress with documented reasons

---

## 3. Schema Extensions — What petalTongue Absorbs

### 3.1 DataChannel (New)

Typed data channels attached to each primal (node). This is the key extension — it transforms petalTongue from a topology viewer into a universal data dashboard.

```json
{
  "data_channels": [
    {
      "channel_type": "timeseries",
      "id": "pk_curve",
      "label": "Oral PK Concentration",
      "x_label": "Time (hr)",
      "y_label": "Concentration (mg/L)",
      "unit": "mg/L",
      "x_values": [0.0, 0.24, ...],
      "y_values": [0.0, 0.0012, ...]
    },
    {
      "channel_type": "distribution",
      "id": "risk_distribution",
      "label": "Population Risk Distribution",
      "values": [0.35, 0.36, ...],
      "mean": 0.363,
      "std": 0.011,
      "patient_value": 0.43
    },
    {
      "channel_type": "bar",
      "id": "gut_abundances",
      "categories": ["Genus 1", "Genus 2", ...],
      "values": [0.30, 0.25, ...]
    },
    {
      "channel_type": "gauge",
      "id": "heart_rate",
      "label": "Heart Rate",
      "value": 72.0,
      "min": 40.0,
      "max": 140.0,
      "unit": "bpm",
      "normal_range": [60.0, 100.0],
      "warning_range": [40.0, 60.0]
    }
  ]
}
```

### 3.2 ClinicalRange (New)

Per-node clinical reference ranges for threshold-based coloring:

```json
{
  "clinical_ranges": [
    { "label": "Shannon healthy", "min": 2.5, "max": 4.0, "status": "normal" },
    { "label": "Shannon dysbiotic", "min": 0.0, "max": 1.5, "status": "critical" }
  ]
}
```

### 3.3 Enhanced PrimalDefinition

healthSpring nodes now carry all fields petalTongue expects:

```json
{
  "id": "pk",
  "name": "PK/PD Engine",
  "type": "compute",
  "family": "healthspring",
  "status": "healthy",
  "health": 100,
  "confidence": 100,
  "position": { "x": 160.0, "y": 280.0 },
  "capabilities": ["science.pkpd.one_compartment_pk"],
  "data_channels": [...],
  "clinical_ranges": [...]
}
```

---

## 4. Rendering Prototypes (petaltongue-health crate)

The `petaltongue-health` crate contains working egui prototypes for each `DataChannel` type:

| Renderer | DataChannel | egui Widget | Lines |
|----------|------------|-------------|-------|
| `draw_timeseries` | TimeSeries | `egui_plot::Line` | Time-series with hover |
| `draw_distribution` | Distribution | `egui_plot::BarChart` + VLine | Histogram + mean/SD/patient markers |
| `draw_bar_chart` | Bar | `egui_plot::BarChart` | Categorical bar chart |
| `draw_gauge` | Gauge | Custom painter + rect_filled | Progress bar with normal/warning ranges |
| `draw_node_detail` | All | ScrollArea panel | Header + capabilities + all channels |
| `draw_topology` | — | Custom painter | Interactive node graph with click selection |

### Dependencies

- `eframe 0.29` (matches petalTongue's version)
- `egui_plot 0.29`
- `healthspring-barracuda` (data source)

### Architecture for Absorption

```
petaltongue-health/src/
├── render.rs    → absorb into petal-tongue-graph/src/chart_renderer.rs
├── theme.rs     → absorb into petal-tongue-ui/src/theme/clinical.rs
└── bin/healthspring_ui.rs → reference impl, not absorbed
```

---

## 5. Metrics

| Metric | V4 | V5 |
|--------|----|----|
| Rust lib tests | 185 | 200 |
| Experiments | 24 | 27 |
| Binary checks | 280 | 346 |
| New crates | 0 | 1 (petaltongue-health) |
| petalTongue schema coverage | partial | full native v2.0 |
| DataChannel types | 0 | 4 (timeseries, distribution, bar, gauge) |
| Interactive UI | none | egui prototype with topology + charts |

---

## 6. Absorption Plan

### Phase 1 — petalTongue Bug Fixes (Blocking)

1. Fix `DynamicData` version parsing (string → struct)
2. Fix `add_node` to use scenario positions
3. Clean clippy warnings

### Phase 2 — Schema Absorption

1. Add `DataChannel` enum to `petal-tongue-core`
2. Add `ClinicalRange` to `petal-tongue-core`
3. Extend `PrimalDefinition` with `data_channels` and `clinical_ranges`
4. Update `ScenarioVisualizationProvider` to parse new fields

### Phase 3 — Renderer Absorption

1. Port `draw_timeseries` → `petal-tongue-graph` chart module
2. Port `draw_distribution` → histogram renderer
3. Port `draw_gauge` → gauge widget
4. Port `draw_node_detail` → detail panel system
5. Port clinical theme → theme system

### Phase 4 — healthSpring Goes Lean

After petalTongue absorbs, healthSpring:
- Removes `petaltongue-health` crate (absorbed)
- Keeps `visualization.rs` for JSON export
- Scenario JSON works directly in evolved petalTongue
- Iterate on new data types (heatmaps, 3D, networks)

---

## 7. Files Changed in V5

| File | Change |
|------|--------|
| `barracuda/src/diagnostic.rs` | Added `curve_times_hr`, `curve_concs_mg_l`, `hill_concs`, `hill_responses` to PkAssessment; `abundances` to MicrobiomeAssessment; `rr_intervals_ms` to BiosignalAssessment |
| `barracuda/src/visualization.rs` | Complete rewrite: serde-based, native petalTongue schema, DataChannel, ClinicalRange, full_scenario_json |
| `petaltongue-health/` | New crate: render, theme, standalone UI binary |
| `experiments/exp050_*/src/main.rs` | Updated for new visualization API |
| `experiments/exp050_*/src/dump_scenario.rs` | Uses `full_scenario_json` |
| `experiments/exp051_*/src/main.rs` | Updated for new visualization API |
| `experiments/exp052_petaltongue_render/` | New: 31-check schema validation |
| `Cargo.toml` | Added exp052, petaltongue-health to workspace |
