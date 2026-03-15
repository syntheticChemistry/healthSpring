# healthSpring V12 — petalTongue Evolution Handoff

**Date**: March 9, 2026
**From**: healthSpring
**To**: barraCuda, toadStool, petalTongue, metalForge
**License**: AGPL-3.0-only
**Covers**: V10→V12 evolution — petalTongue stream completeness, domain theming, clinical TRT wiring, capability querying, interaction subscription, missing visualizations, validation

---

## Executive Summary

healthSpring V12 completes the petalTongue integration layer. Where V9 was push-only with append/set_value stream operations, V12 adds the `replace` operation (enabling live updates to **all** 7 DataChannel types), domain theming, `UiConfig` passthrough, capability querying, and interaction subscription. Five TRT patient archetypes are wired into the scenario dump pipeline. Three categories of computed data that were previously invisible are now surfaced as visualization nodes. The interaction roundtrip is validated end-to-end with a mock petalTongue server.

**Metrics**: 313 tests, 47 experiments, 630 binary checks, all green.

---

## Part 1: What Changed (V10→V12)

### 1.1 Stream Operation Completeness

**Gap closed**: healthSpring previously only implemented `append` (TimeSeries) and `set_value` (Gauge). The wateringHole spec defines a third: `replace`.

**Implementation**:
- `barracuda/src/visualization/ipc_push.rs`: `build_replace_params()` serializes the full `DataChannel` binding into the `replace` operation payload. `push_replace()` method on `PetalTonguePushClient`.
- `barracuda/src/visualization/stream.rs`: `push_replace_binding()` on `StreamSession` with backpressure.

**Why this matters**: `append` only works for TimeSeries. `set_value` only works for Gauge. But clinical dashboards need to update Heatmaps (evolving Bray-Curtis dissimilarity), Bar charts (risk comparisons at different time points), Scatter3D (population PK clusters), Distribution (trough level histograms), and Spectrum (HRV power evolving with treatment). `replace` enables all of these.

**Absorption target**: barraCuda should absorb `push_replace` into its core visualization client. toadStool should use `replace` when streaming pipeline results that produce non-TimeSeries output (e.g., diversity reduction → Bar, population stats → Distribution).

### 1.2 Domain Theming and Protocol Alignment

**Implementation**:
- `push_render_with_config()` on `PetalTonguePushClient`: passes `domain` field (e.g., `"health"`, `"clinical"`) and the full `UiConfig` (theme, `show_panels`, `awakening_enabled`, `initial_zoom`) through to petalTongue.
- `push_render_with_domain()` on `StreamSession`: convenience wrapper.
- IPC response buffer increased from 4KB to 64KB (capability responses can be large).

**Why this matters**: The wateringHole visualization guide specifies that `domain` triggers domain-appropriate color palettes in petalTongue. Clinical scenarios need to suppress system panels and skip the awakening animation. `UiConfig` passthrough gives the data producer control over the rendering context.

**Absorption target**: petalTongue should consume the `domain` and `ui_config` fields in `visualization.render` params. barraCuda should expose `domain` as a first-class concept in its visualization API.

### 1.3 Clinical TRT Archetype Wiring

Five patient-parameterized TRT scenarios are now generated and dumped via `dump_scenarios`:

| Archetype | Age | Weight | Baseline T | Protocol | Comorbidities |
|-----------|:---:|:------:|:----------:|----------|---------------|
| Young Hypogonadal | 32 | 175 lb | 180 ng/dL | IM Weekly | Low HRV |
| Obese Diabetic | 48 | 285 lb | 250 ng/dL | IM Biweekly | HbA1c 7.8, low diversity |
| Senior Sarcopenic | 68 | 155 lb | 220 ng/dL | Pellet | Very low HRV |
| Former Athlete | 42 | 210 lb | 310 ng/dL | IM Weekly | High HRV, high diversity |
| Metabolic Syndrome | 55 | 240 lb | 195 ng/dL | Pellet | HbA1c 6.9, low diversity |

Each generates an 8-node, 8-edge scenario with PK curves, population comparison, metabolic/cardiovascular/glycemic/cardiac/gut health outcomes.

**Exp073** streams a live TRT clinical session: weekly PK trough updates via `push_timeseries`, HRV improvement via `push_gauge`, HbA1c trajectory, and cardiac risk comparison via `push_replace_binding` on a Bar channel.

**Absorption target**: barraCuda should absorb `PatientTrtProfile` and `trt_clinical_scenario()` into its clinical module. petalTongue should use the `domain=clinical` + `UiConfig` passthrough to render these in clinical mode.

### 1.4 Capability Querying and Interaction Subscription

**Implementation**:
- `query_capabilities()` on both `PetalTonguePushClient` and `StreamSession`: sends `visualization.capabilities` RPC, returns supported channel types and features.
- `subscribe_interactions()`: sends `visualization.interact.subscribe` with event types (`select`, `focus`, `filter`) and callback method.

**Exp074** validates the full roundtrip with a mock petalTongue server: render → append → replace → gauge → HRV update → capabilities query → interaction subscription. 12/12 checks pass. The mock server responds with capability metadata (7 channel types, streaming=true, interaction=true) and subscription confirmation.

**Why this matters**: This enables bidirectional flow. Today healthSpring is push-only. With interaction subscription, a clinician can select a node in petalTongue and healthSpring can drill down into that node's data — e.g., clicking the "cardiovascular" node triggers a detailed cardiac risk breakdown.

**Absorption target**: petalTongue must implement `visualization.capabilities` and `visualization.interact.subscribe` server-side. barraCuda should absorb the client-side API. toadStool should be able to receive interaction events that trigger re-computation of specific pipeline stages.

### 1.5 Missing Visualizations — Computed but Previously Invisible

Three categories of data were computed by barracuda but had no petalTongue scenario representation:

**PBPK Tissue Profiles** (`pkpd/pbpk.rs`):
- New function `pbpk_iv_tissue_profiles()` runs the PBPK simulation and collects per-tissue concentration time series (liver, kidney, muscle, fat, rest) down-sampled to a configurable number of points.
- PBPK scenario node now has 9 channels: venous TimeSeries + 5 tissue concentration TimeSeries + tissue bar + AUC gauge + cardiac output gauge.

**Pan-Tompkins Intermediates** (`biosignal.rs`):
- `PanTompkinsResult` already contained `derivative`, `squared`, and `mwi` fields. These were internal-only.
- QRS detection scenario node now has 8 channels: raw ECG + bandpass + derivative + squared + MWI (5 TimeSeries showing all processing stages) + HR + sensitivity + PPV gauges.

**Anderson Lattice Spectra** (`microbiome.rs`):
- Hamiltonian diagonal elements exposed as eigenvalue Spectrum channel.
- Per-eigenstate IPR values exposed as IPR Spectrum channel.
- Anderson scenario node now has 6 channels: 2 spectra + 3 gauges + 1 bar.

**Absorption target**: barraCuda should absorb `pbpk_iv_tissue_profiles` into its PBPK API. The Pan-Tompkins intermediate exposure pattern (making internal computation stages visible) should be adopted by other springs for their own signal processing pipelines.

---

## Part 2: Absorption Tables

### barraCuda Absorption

| Module | What to absorb | Source file | Priority |
|--------|---------------|-------------|:--------:|
| `visualization::ipc_push` | `push_replace()`, `push_render_with_config()`, `query_capabilities()`, `subscribe_interactions()` | `barracuda/src/visualization/ipc_push.rs` | P0 |
| `visualization::stream` | `push_replace_binding()`, `push_render_with_domain()`, `query_capabilities()`, `subscribe_interactions()` | `barracuda/src/visualization/stream.rs` | P0 |
| `visualization::clinical` | `PatientTrtProfile`, `TrtProtocol`, `trt_clinical_scenario()`, `trt_clinical_json()` | `barracuda/src/visualization/clinical.rs` | P1 |
| `pkpd::pbpk` | `pbpk_iv_tissue_profiles()`, `PbpkTissueProfiles` | `barracuda/src/pkpd/pbpk.rs` | P1 |
| `visualization::scenarios` | PBPK 5-tissue node, Pan-Tompkins 5-stage node, Anderson spectra node | `barracuda/src/visualization/scenarios/{pkpd,biosignal,microbiome}.rs` | P2 |

### toadStool Absorption

| Module | What to absorb | Source | Priority |
|--------|---------------|--------|:--------:|
| `pipeline` | `execute_streaming()` callback pattern for per-stage progress | `toadstool/src/pipeline.rs` | P0 |
| `pipeline` | Wire `replace` stream op for non-TimeSeries stage outputs | healthSpring pattern | P1 |
| `pipeline` | Accept interaction events that trigger re-computation | Exp074 pattern | P2 |

### petalTongue Absorption

| Capability | What to implement | Source | Priority |
|-----------|------------------|--------|:--------:|
| `visualization.render` | Consume `domain` and `ui_config` fields | ipc_push.rs `build_render_with_config_params()` | P0 |
| `visualization.render.stream` | Handle `replace` operation type | ipc_push.rs `build_replace_params()` | P0 |
| `visualization.capabilities` | Server-side: return supported channels, streaming, interaction flags | Exp074 mock server response format | P1 |
| `visualization.interact.subscribe` | Server-side: accept subscription, emit events on user interaction | Exp074 mock server response format | P1 |
| Domain theming | `domain=health` and `domain=clinical` color palettes | wateringHole guide | P2 |

### metalForge Absorption

| Module | What to absorb | Source | Priority |
|--------|---------------|--------|:--------:|
| `dispatch` | Wire topology scenario to petalTongue via `push_render` | `barracuda/src/visualization/scenarios/topology.rs` | P1 |
| `nucleus` | biomeOS atomic graph → NUCLEUS topology mapping | `wateringHole/healthspring_deploy.toml` | P2 |

---

## Part 3: Learnings for the Team

### 3.1 Stream Operation Design

The `replace` operation is the missing piece for universal live visualization. Without it, only TimeSeries and Gauge can be updated incrementally. With it, any DataChannel type can be swapped in-place during a streaming session. The pattern is:

```json
{
    "session_id": "...",
    "binding_id": "risk_compare",
    "operation": {
        "type": "replace",
        "binding": { "channel_type": "bar", "id": "risk_compare", ... }
    }
}
```

The binding is serialized in full. This is intentional — `replace` is for complete swaps, not incremental edits. For incremental updates to TimeSeries, use `append`.

### 3.2 Domain Theming

The `domain` field in `visualization.render` is lightweight but powerful. It tells petalTongue "this is health data" without encoding any rendering details in the data producer. The data producer stays sovereign — it doesn't know what colors petalTongue will use. It just says "I'm health" and petalTongue applies the appropriate palette.

### 3.3 IPC Buffer Sizing

The 4KB response buffer was too small for capability responses (which enumerate all supported channel types and features). 64KB is sufficient for current payloads. Future: consider chunked reads for truly large responses.

### 3.4 PBPK Tissue Profiles

The `pbpk_iv_tissue_profiles()` function demonstrates a pattern: when a simulation computes intermediate state at each time step but only returns the final state, add a "profiles" variant that collects the intermediate states. This is pure instrumentation — no algorithmic change, just visibility.

### 3.5 Pan-Tompkins Intermediates

The derivative, squared, and MWI signals were already computed and stored in `PanTompkinsResult`. The only change was adding them to the scenario builder. This is a zero-cost visibility improvement — the data was always there, just not exposed.

### 3.6 Interaction Roundtrip

Exp074 proves the interaction API works end-to-end but uses a mock server. The next step is implementing server-side in petalTongue. The pattern is: healthSpring subscribes with a callback method, petalTongue stores the subscription, and when the user interacts, petalTongue sends a JSON-RPC notification to the callback method with the event data.

---

## Part 4: biomeOS Integration

### healthspring_deploy.toml

The deployment graph (`wateringHole/healthspring_deploy.toml`) defines the full healthSpring pipeline as a biomeOS atomic graph:

```
beardog → songbird → nestgate → toadstool → {pkpd, microbiome, biosignal, endocrine} → diagnostic → clinical → petaltongue → results
```

Each stage discovers its dependencies via Songbird capability announcements. healthSpring announces `health.metrics`, `health.pkpd`, `health.microbiome`, `health.biosignal`, `health.endocrine`, `health.diagnostic` capabilities.

### NUCLEUS Topology

metalForge's NUCLEUS atomics (Tower → Node → Nest) map directly to deployment:

- **Nest**: single compute device (CPU, GPU, or NPU)
- **Node**: PCIe topology (e.g., CPU + GPU + NPU connected via Gen4 bus)
- **Tower**: multi-node rack or cluster

healthSpring's Exp061-062 and Exp069-071 validate dispatch across these levels. The topology and dispatch scenarios generated by `dump_scenarios` visualize the assignment of pipeline stages to substrates.

### Next: Atomic Graphs

The current `healthspring_deploy.toml` is a static graph. The next evolution is dynamic orchestration via biomeOS atomic operations:

1. **Node deployment**: single machine with GPU — all stages on one node
2. **Tower deployment**: multiple nodes — diagnostic pipeline on CPU node, GPU Monte Carlo on GPU node, streaming results to petalTongue on UI node
3. **Mixed deployment**: NPU for biosignal streaming, GPU for population PK, CPU for diagnostic fusion — coordinated via NUCLEUS dispatch and PCIe P2P transfers

---

## Part 5: Open Data Controls

All 24 papers in the review queue use open data. All 47 experiments use open data or self-generated signals from published models. The validation chain is:

1. **Python control** (Tier 0): Reference implementation using published equations. Data sources cited with DOI.
2. **Rust CPU** (Tier 1): Validated against Python within documented tolerances.
3. **Rust GPU** (Tier 2): WGSL shader output validated against CPU for math parity. No new data.
4. **metalForge** (Tier 3): Cross-substrate dispatch validated for routing correctness. Identical results regardless of CPU/GPU/NPU.

No experiment requires proprietary data, commercial licenses, or restricted datasets.

---

## Action Items

1. **petalTongue team**: Implement `visualization.capabilities` and `visualization.interact.subscribe` server-side. Consume `domain` and `ui_config` in render params. Handle `replace` stream operation.
2. **barraCuda team**: Absorb `push_replace`, `push_render_with_config`, `query_capabilities`, `subscribe_interactions`, `pbpk_iv_tissue_profiles`.
3. **toadStool team**: Wire `execute_streaming()` to use `replace` for non-TimeSeries outputs. Accept interaction events for stage re-computation.
4. **metalForge team**: Map biomeOS atomic graphs to NUCLEUS topology for dynamic orchestration.
5. **biomeOS team**: Evolve `healthspring_deploy.toml` from static graph to dynamic atomic deployment.
