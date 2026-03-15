<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# healthSpring V9 → barraCuda / toadStool / petalTongue Clinical Translation + SAME DAVE Handoff

**Date**: March 9, 2026
**From**: healthSpring V9
**To**: barraCuda, toadStool, petalTongue teams
**Covers**: V8 → V9 (patient-parameterized clinical scenarios, SAME DAVE motor channel, IPC bridge)
**License**: AGPL-3.0-only
**Status**: Complete — 37 experiments, 221 unit tests, 526+ binary checks

---

## Executive Summary

- **V8** delivered CPU vs GPU parity, mixed NUCLEUS dispatch, and PCIe P2P transfer validation
- **V9** closes the **per-person translation loop**: validated population models → patient-parameterized clinical scenarios → petalTongue visualization in clinical mode
- `PatientTrtProfile` (age, weight, testosterone, comorbidities) generates 8-node scenario graphs with edges, clinical ranges, and risk annotations
- 5 patient archetypes validated: young-athletic, middle-metabolic, older-cardiovascular, diabetic, post-FMT
- petalTongue SAME DAVE neuroanatomy integration: `MotorCommand` channel enables runtime UI control from scenarios and IPC
- Clinical mode preset: sidebars hidden, awakening skipped, graph fitted to view — clinician sees the patient, not the infrastructure
- IPC push via Unix socket JSON-RPC (`Exp064`): scenario → running petalTongue instance, no file intermediary needed
- Zero clippy warnings, zero unsafe blocks, 221 unit tests + 526+ binary checks all green

---

## Part 1: Per-Person Translation Pipeline

The core V9 contribution: closing the gap between population-level validated models and individual patient care.

```
PatientTrtProfile (age, weight, T level, comorbidities)
    → trt_clinical_scenario() → HealthScenario (8 nodes, 8 edges)
        → scenario JSON (mode: "clinical", ui_config: clinical preset)
            → petalTongue --scenario (or IPC push)
                → clinician sees THIS patient's trajectory
```

### 1.1 Patient Parameterization (Exp063)

| Component | Location | What It Does |
|-----------|----------|--------------|
| `PatientTrtProfile` | `barracuda/src/visualization/clinical.rs` | Age, weight, current T, protocol, comorbidities |
| `trt_clinical_scenario()` | `barracuda/src/visualization/clinical.rs` | Profile → 8-node scenario graph with edges |
| `scaffold_clinical()` | `barracuda/src/visualization/clinical.rs` | Sets mode="clinical", UI config, panel visibility |
| `dump_clinical_scenarios` | `experiments/exp063_clinical_trt_scenarios/` | Generates JSON for 5 archetypes |

### 1.2 Patient Archetypes

| Archetype | Age | Weight | T Level | Protocol | Key Risk |
|-----------|:---:|:------:|:-------:|----------|----------|
| Young athletic | 28 | 82 kg | 310 ng/dL | IM weekly | Minimal — baseline reference |
| Middle metabolic | 45 | 105 kg | 220 ng/dL | Pellet | Metabolic syndrome, weight |
| Older cardiovascular | 62 | 88 kg | 180 ng/dL | IM biweekly | Cardiovascular, lipids |
| Diabetic | 55 | 115 kg | 195 ng/dL | Pellet | T2DM, HbA1c, insulin resistance |
| Post-FMT gut recovery | 48 | 78 kg | 250 ng/dL | IM weekly | Gut dysbiosis, C. diff history |

### 1.3 Clinical Scenario Structure (per patient)

| Node | Content |
|------|---------|
| Patient | Demographics, current T level, risk profile |
| TRT Protocol | Dosing, route, frequency, expected steady-state |
| PK Trajectory | Predicted serum T curve (DataChannel: TimeSeries) |
| Metabolic Response | Weight/BMI/waist projections (DataChannel: TimeSeries) |
| Cardiovascular | Lipid panel, CRP, BP predictions (DataChannel: Bar) |
| Endocrine Status | HbA1c, HOMA-IR if diabetic (DataChannel: Gauge) |
| Gut Health | Diversity indices, Anderson ξ if applicable (DataChannel: Distribution) |
| Risk Summary | Composite risk score, clinical ranges |

---

## Part 2: SAME DAVE Integration (petalTongue)

The SAME DAVE neuroanatomy model (Sensory Afferent, Motor Efferent) was unified in petalTongue to enable runtime UI control from scenarios and external systems.

### 2.1 What healthSpring Drives

| Component | Purpose | healthSpring Use |
|-----------|---------|-----------------|
| `MotorCommand::SetMode { mode: "clinical" }` | Applies clinical mode preset | Scenario `mode` field |
| `MotorCommand::SetPanelVisibility` | Hide/show specific panels | `ui_config.show_panels` |
| `MotorCommand::SetAwakening { enabled: false }` | Skip startup animation | `ui_config.awakening_enabled` |
| `MotorCommand::FitToView` | Auto-zoom to fit patient graph | `ui_config.initial_zoom: "fit"` |

### 2.2 IPC Bridge (Exp064)

| Component | Location | Protocol |
|-----------|----------|----------|
| `PetalTonguePushClient` | `barracuda/src/visualization/ipc_push.rs` | Unix socket discovery + JSON-RPC |
| `visualization.render` | petalTongue RPC method | Full scenario replace |
| `visualization.render.stream` | petalTongue RPC method | Incremental append |
| `motor.set_panel` | petalTongue RPC method | Runtime panel control |
| `motor.set_mode` | petalTongue RPC method | Runtime mode switch |
| `motor.fit_to_view` | petalTongue RPC method | Runtime zoom adjustment |

Socket discovery: `PETALTONGUE_SOCKET` env → `XDG_RUNTIME_DIR/petaltongue/*.sock` → `/tmp/petaltongue-*.sock`

### 2.3 Scenario UiConfig (new fields)

```json
{
  "mode": "clinical",
  "ui_config": {
    "theme": "dark",
    "show_panels": {
      "left_sidebar": false,
      "right_sidebar": false,
      "top_menu": true,
      "graph_stats": true,
      "audio_panel": false,
      "trust_dashboard": false,
      "proprioception": false
    },
    "awakening_enabled": false,
    "initial_zoom": "fit"
  }
}
```

---

## Part 3: Absorption Tables

### For barraCuda Team

| healthSpring Source | What to Absorb | Why |
|---------------------|----------------|-----|
| `barracuda/src/visualization/clinical.rs` | `PatientTrtProfile`, `TrtProtocol` | Reusable patient parameterization pattern |
| `barracuda/src/visualization/ipc_push.rs` | `PetalTonguePushClient` | Generic petalTongue IPC client (works for any spring's scenarios) |
| `barracuda/src/visualization/types.rs` `UiConfig` extensions | `ShowPanels`, `awakening_enabled`, `initial_zoom` | Standard scenario config for petalTongue clinical mode |

### For toadStool Team

| healthSpring Source | What to Absorb | Why |
|---------------------|----------------|-----|
| `PatientTrtProfile → HealthScenario` pipeline | Pattern: domain struct → visualization graph | Any toadStool pipeline stage could generate scenario subgraphs |
| Clinical mode preset pattern | Mode → Vec<MotorCommand> | Reusable for any domain-specific petalTongue configuration |

### For petalTongue Team

| healthSpring Source | What Evolved Upstream | Status |
|---------------------|----------------------|--------|
| `channel.rs` — SAME DAVE channel model | `petal-tongue-core` new module | **Committed** |
| `MotorCommand` extensions (SetPanelVisibility, SetZoom, etc.) | `petal-tongue-core/rendering_awareness.rs` | **Committed** |
| `mode_presets.rs` (clinical, developer, presentation) | `petal-tongue-ui` new module | **Committed** |
| `IpcCommand` motor extensions | `petal-tongue-ipc/protocol.rs` | **Committed** |
| Motor command mpsc channel in PetalTongueApp | `petal-tongue-ui/app/mod.rs` | **Committed** |
| ProprioceptionPanel channel health display | `petal-tongue-ui/proprioception_panel.rs` | **Committed** |

---

## Part 4: New Experiments

| Experiment | What It Validates |
|------------|-------------------|
| Exp063 | Patient-parameterized TRT scenarios — 5 archetypes, 8 nodes + 8 edges each, clinical mode preset, UiConfig with panel visibility and awakening control |
| Exp064 | IPC push to petalTongue — Unix socket discovery, JSON-RPC `visualization.render`, fallback to file dump |

---

## Part 5: Key Design Decisions

1. **Per-person, not per-population** — Exp063 generates scenarios for individual patients, not cohort summaries. Population models (Exp036) inform the parameter distributions; clinical scenarios sample from them for a specific patient.

2. **Clinical mode is a motor command bundle** — not a separate code path. `SetMode { mode: "clinical" }` expands to a `Vec<MotorCommand>` that configures panel visibility, awakening, and zoom. New modes (e.g., "research", "patient-facing") are just new bundles.

3. **SAME DAVE is neuroanatomy, not AI** — the channel model (Sensory Afferent, Motor Efferent) maps specialized unidirectional pathways. Classification nodes (like Schwann cell nodes of Ranvier) allow smart signal routing. This is a proprioception model, not an acronym for AI capabilities.

4. **IPC is optional** — scenarios work as JSON files loaded via `--scenario` flag. IPC push (`Exp064`) is for live integration where petalTongue is already running and the clinician wants to update the view without restarting.

5. **UiConfig belongs in the scenario** — the scenario author (healthSpring) knows the clinical context and encodes the appropriate presentation. petalTongue respects these settings but allows override via IPC or local config.

---

## Part 6: Metrics

| Metric | V8 | V9 |
|--------|:--:|:--:|
| Experiments | 34 | **37** |
| Unit tests | 211 | **221** |
| Binary checks | 526 | **526+** |
| Forge tests | 33 | **33** |
| toadStool tests | 17 | **17** |
| Patient archetypes | 0 | **5** |
| petalTongue motor commands | 0 | **8 variants** |
| IPC motor methods | 0 | **6 methods** |
| Mode presets | 0 | **4 (clinical, developer, presentation, full)** |

---

This handoff is unidirectional: healthSpring → barraCuda / toadStool / petalTongue. No response expected.
