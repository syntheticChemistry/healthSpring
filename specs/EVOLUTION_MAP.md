<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring Evolution Map — Rust Module → WGSL Shader → Pipeline Stage

**Last Updated**: March 15, 2026
**Status**: V24 — audit-hardened biomeOS niche. 55+ wired JSON-RPC capabilities. `gpu/context.rs` smart refactor (968→350 LOC, per-op prep in `fused.rs`). toadStool Hill/AUC delegation. Tier A GPU rewire documented. Capability-based primal discovery.

---

## Evolution Path

```
Python baseline → Rust CPU (Tier 1) → barraCuda CPU parity (Exp040)
  → GPU via WGSL (Tier 2) → metalForge dispatch (Tier 3)

Species-specific model → Species-agnostic math → Cross-species parameter bridge
  → Comparative validation → Drug discovery pipeline → Preclinical
```

---

## Tier Assessment

### Tier A — Ready for GPU (Embarrassingly Parallel)

| Rust Module | Function | GpuOp | WGSL Shader | Pipeline Pattern | Priority |
|------------|----------|-------|-------------|-----------------|----------|
| `pkpd::population_pk_cpu` | Per-patient Bateman ODE | `PopulationPkBatch` | `population_pk_f64.wgsl` | 1 workgroup/patient, independent | **P0** |
| `pkpd::hill_dose_response` | Per-concentration Hill eq | `HillSweep` | `hill_dose_response_f64.wgsl` | Element-wise, f32 pow() intermediates | **P0 — LIVE** |
| `microbiome::shannon_index` | -Σ p·ln(p) batch | `DiversityBatch` | `diversity_f64.wgsl` | Workgroup reduction | **P0 — LIVE** |
| `endocrine::lognormal_params` | μ,σ from typical+CV | — | CPU-side utility | Parameter setup, not GPU | P2 |

### Tier B — Adapt (Decompose into Existing Primitives)

| Rust Module | Function | barraCuda Primitive | Notes |
|------------|----------|-------------------|-------|
| `microbiome::simpson_index` | 1 - Σ p² | `FusedMapReduceF64` (square→sum) | |
| `pkpd::auc_trapezoidal` | Σ (c_i+c_{i+1})/2 · Δt | `FusedMapReduceF64` (diff→mul→sum) | Parallel prefix |
| `endocrine::anderson_localization_length` | ξ(W) power-law | Extend hotSpring Anderson | Parameterize for gut substrate |
| `biosignal::ppg_r_value` | AC/DC ratio per channel | `batched_elementwise_f64.wgsl` | Element-wise division |
| `microbiome::bray_curtis` | Community dissimilarity | `FusedMapReduceF64` (abs_diff→sum) | Pairwise matrix |
| `pkpd::nlme::foce_estimate` | FOCE individual optimization | Batch-parallel per-subject gradient | Each subject's objective is independent |
| `pkpd::nlme::saem_estimate` | SAEM E-step Monte Carlo | Batched sampling + sufficient stats | E-step is embarrassingly parallel |
| `pkpd::diagnostics::vpc_simulate` | VPC Monte Carlo simulation | Embarrassingly parallel | 50+ simulations, each independent |

### Tier C — New Shader Required

| Rust Module | Function | Shader Design | Blocking |
|------------|----------|--------------|----------|
| `biosignal::pan_tompkins_qrs` | Streaming detect pipeline | Custom streaming shader or NPU | NPU dispatch path in toadStool |
| `biosignal::fuse_channels` | Multi-modal ECG+PPG+EDA | toadStool pipeline (3-stage) | Pipeline execution on GPU |
| `pkpd::pbpk_iv_simulate` | PBPK multi-compartment ODE | Euler/RK4 ODE shader | wetSpring ODE absorption status |
| `endocrine::hrv_trt_response` | Exponential saturation | `batched_elementwise_f64.wgsl` | Trivial once shader exists |

---

## Track 6: Comparative Medicine — Module Evolution (V21)

Species-agnostic mathematics validated on animal models for their own sake,
then translated to humans via parameter substitution.

### New Modules

| Rust Module (planned) | Function | Source | GPU Pattern |
|-----------------------|----------|--------|-------------|
| `comparative::species_params` | Species parameter registry (canine, human, feline, equine) | Gonzales G1–G6, FDA CVM | CPU lookup |
| `comparative::allometric_bridge` | Cross-species PK scaling (CL, Vd, t½) | Mahmood 2006, nS-601 | Element-wise |
| `comparative::tissue_lattice` | Species-parameterized tissue Anderson | nS-604, groundSpring | GPU eigensolve |
| `microbiome::cross_species_gut` | Comparative gut Anderson (dog/human/mouse) | NCBI SRA, HMP | Workgroup reduction |
| `pkpd::species_pk` | Species-generic compartment PK (parameterized) | Rowland & Tozer, Gonzales | Embarrassingly parallel |

### Cross-Spring Absorption

| Source Spring | Module | healthSpring Target |
|--------------|--------|-------------------|
| neuralSpring nS-601 | Canine Hill dose-response | `comparative::canine_ic50_panel` |
| neuralSpring nS-603 | Canine mAb PK (lokivetmab) | `comparative::canine_mab_pk` |
| neuralSpring nS-604 | Canine tissue lattice | `comparative::tissue_lattice` |
| wetSpring Track 6 | Soil QS/Anderson | `microbiome::cross_species_gut` (gut = anaerobic digester analogy) |
| groundSpring | Tissue Anderson localization | `comparative::tissue_lattice` |
| airSpring | Immunological Anderson | `comparative::immune_lattice` |

---

## Track 7: Drug Discovery — Module Evolution (V21)

MATRIX scoring + ADDRC HTS pipeline. Front-loaded for Gonzales meeting.

### New Modules

| Rust Module (planned) | Function | Source | GPU Pattern |
|-----------------------|----------|--------|-------------|
| `discovery::matrix_score` | Fajgenbaum MATRIX with Anderson geometry | nS-605, Fajgenbaum 2018 | Batch element-wise |
| `discovery::anderson_matrix` | Anderson-augmented drug scoring | Exp011 Anderson + MATRIX | Workgroup reduction |
| `discovery::hts_analysis` | HTS plate reader data analysis, Z'-factor, hit scoring | Lisabeth ADDRC | Element-wise |
| `discovery::compound_library` | ChEMBL/ADDRC compound IC50/EC50 batch computation | ChEMBL REST API | Embarrassingly parallel (Hill sweep) |
| `discovery::ipsc_protocol` | iPSC skin model readout → computational validation | Gonzales iPSC | CPU structured |
| `discovery::qs_drug_target` | QS gene profiling → MATRIX target identification | NCBI Gene, UniProt | Medium (matrix ops) |

### Pipeline Integration

```
discovery::compound_library   (8,000 compounds × IC50 sweep)
  → discovery::anderson_matrix (Anderson geometry scoring per compound)
  → discovery::matrix_score    (MATRIX rank + Anderson augmentation)
  → Lisabeth ADDRC HTS         (top N candidates → wet lab screening)
  → discovery::hts_analysis    (plate reader data → hit scoring)
  → Gonzales iPSC validation   (iPSC skin model functional readout)
  → discovery::ipsc_protocol   (readout → computational validation)
  → Ellsworth med chem          (lead optimization)
```

### DNA/Protein Integration Path (future)

```
neuralSpring protein analysis → drug target structure
wetSpring QS gene profiling   → microbial drug targets
                              ↓
         discovery::qs_drug_target (QS-informed MATRIX scoring)
                              ↓
         Cross-species genome comparison (ortholog mapping)
                              ↓
         Species-agnostic drug target → ADDRC screening
```

---

## GPU Dispatch Layer (`ecoPrimal/src/gpu/`) — LIVE (V6+)

`GpuContext` holds persistent `wgpu::Device`/`Queue`. `execute_fused()` dispatches multiple ops in a single encoder submission.

| GpuOp Variant | CPU Fallback | WGSL Shader | Status |
|--------------|-------------|-------------|--------|
| `HillSweep` | `pkpd::hill_dose_response` loop | `hill_dose_response_f64.wgsl` | **LIVE** — 17/17 parity, crossover at 100K |
| `PopulationPkBatch` | `pkpd::population_pk_cpu` | `population_pk_f64.wgsl` | **LIVE** — u32 xorshift32 PRNG, crossover at 5M |
| `DiversityBatch` | `microbiome::shannon_index` loop | `diversity_f64.wgsl` | **LIVE** — workgroup reduction |

Exp040 validates CPU parity (15 contracts). Exp053 validates GPU parity (17 checks). Exp054 validates fused pipeline (11 checks). Exp055 validates scaling to 10M elements.

---

## Absorption Candidates (healthSpring → barraCuda)

| Function | Current Location | Target | Status |
|----------|-----------------|--------|--------|
| `pk_multiple_dose` | `pkpd/compartment.rs` | `barraCuda::signal` | Ready |
| `hill_dose_response` | `pkpd/dose_response.rs` | `barraCuda::bio::pharmacology` | Ready |
| `allometric_scale` | `pkpd/allometry.rs` | `barraCuda::math::scale` | Ready |
| `lognormal_params` | `pkpd/population.rs` | `barraCuda::stats::distributions` | Ready |
| `population_pk_cpu` | `pkpd/population.rs` | `barraCuda::monte_carlo` | Ready |
| `hazard_ratio_model` | `endocrine.rs` | `barraCuda::epi` | Ready |
| `pan_tompkins_qrs` | `biosignal.rs` | `barraCuda::signal::detect` | Needs streaming design |
| `pbpk_iv_simulate` | `pkpd/pbpk.rs` | `barraCuda::bio::pbpk` | Ready (ODE integration) |
| `bray_curtis` | `microbiome.rs` | `barraCuda::bio::ecology` | Ready |
| `fmt_blend` | `microbiome.rs` | `barraCuda::bio::ecology` | Ready |
| `fuse_channels` | `biosignal.rs` | toadStool pipeline stage | Ready |
| `cardiac_risk_composite` | `endocrine.rs` | `barraCuda::epi::risk` | Ready |
| `foce_estimate` | `pkpd/nlme.rs` | `barraCuda::stats::nlme` | Ready — per-subject parallelizable |
| `saem_estimate` | `pkpd/nlme.rs` | `barraCuda::stats::nlme` | Ready — E-step parallelizable |
| `nca_analysis` | `pkpd/nca.rs` | `barraCuda::bio::nca` | Ready |
| `cwres_compute` | `pkpd/diagnostics.rs` | `barraCuda::stats::diagnostics` | Ready |
| `vpc_simulate` | `pkpd/diagnostics.rs` | `barraCuda::stats::diagnostics` | Ready — embarrassingly parallel |
| `gof_compute` | `pkpd/diagnostics.rs` | `barraCuda::stats::diagnostics` | Ready |
| `decode_format_212` | `wfdb.rs` | `barraCuda::signal::wfdb` | Ready — streaming parser |

---

## Absorption Candidates (healthSpring → toadStool/metalForge)

| Component | Current Location | Target | Status |
|-----------|-----------------|--------|--------|
| NUCLEUS atomics (Tower/Node/Nest) | `metalForge/forge/src/nucleus.rs` | toadStool core topology | Ready |
| PCIe P2P transfer planning | `metalForge/forge/src/transfer.rs` | toadStool scheduler | Ready |
| Pipeline/Stage/StageOp | `toadstool/src/` | toadStool core | Ready |
| Substrate/Workload dispatch | `metalForge/forge/src/lib.rs` | toadStool dispatch layer | Ready |

---

## metalForge Dispatch Targets

| Experiment | GPU (Server) | CPU (Desktop) | NPU (Wearable) |
|-----------|-------------|--------------|----------------|
| Exp005/036 Population PK | Batch 10K-1M patients | Single patient | — |
| Exp001 Hill sweep | 100K concentrations | 100 concentrations | — |
| Exp010/013 Diversity batch | 10K communities | 100 communities | — |
| Exp020 Pan-Tompkins | — | Offline analysis | Real-time ECG (Akida) |
| Exp023 Biosignal fusion | — | Multi-channel offline | Streaming 3-channel (Akida) |
| Exp006 PBPK | 10K patients × tissues | Single patient | — |

---

## petalTongue Evolution — LIVE (V6.1 lean, V7 visualization, V7.1 wiring)

petalTongue absorbed all healthSpring prototypes (commit `037caaa`). healthSpring leaned in V6.1 (petaltongue-health removed). V7 added per-track scenario builders. **V7.1**: Local petalTongue evolution wires data channel rendering end-to-end (3 additive changes, ready for absorption).

### Absorption Status

| Component | healthSpring Source | petalTongue Target | Status |
|-----------|--------------------|--------------------|--------|
| `DataChannel` enum | `visualization/types.rs` | `petal-tongue-core/data_channel.rs` | **Absorbed** |
| `ClinicalRange` struct | `visualization/types.rs` | `petal-tongue-core/data_channel.rs` | **Absorbed** (status: String) |
| Chart renderers | ~~petaltongue-health/render.rs~~ (removed) | `petal-tongue-graph/chart_renderer.rs` | **Absorbed** |
| Clinical theme | ~~petaltongue-health/theme.rs~~ (removed) | `petal-tongue-graph/clinical_theme.rs` | **Absorbed** |
| Version parsing fix | N/A | `dynamic_schema.rs` | **Fixed** |

### Per-Track Scenario Builders (V7 → V10)

| Track | Builder | Nodes | Channels | New in V10+ | Experiments |
|-------|---------|-------|----------|------------|-------------|
| PK/PD | `scenarios::pkpd_study()` | 6 | 19 | `Scatter3D` (PopPK CL/Vd/AUC) | Exp001-006 |
| Microbiome | `scenarios::microbiome_study()` | 4 | 11 | `Heatmap` (Bray-Curtis matrix) | Exp010-013 |
| Biosignal | `scenarios::biosignal_study()` | 5 | 16+ | `Spectrum` (HRV), WFDB ECG node (V14) | Exp020-023 |
| Endocrinology | `scenarios::endocrine_study()` | 8 | 19 | — | Exp030-038 |
| NLME | `scenarios::nlme_study()` | 5 | 41 | V14: Distribution, Scatter3D, TimeSeries, Bar, Gauge | Exp075 |
| Full Study | `scenarios::full_study()` | 28 | 121 | All 7 types, 5 tracks, cross-track edges | All 5 tracks |
| Diagnostic | `full_scenario_json()` | 8 | 12 | — | Exp050 |

### V10: Node Positions Optional

Node positions (`ScenarioNode.position`) changed from `Position { x, y }` to `Option<Position>`, defaulting to `None`. petalTongue handles graph layout via its force-directed layout engine. Hardcoded coordinates removed from all scenario builders.

### V10: Streaming + Capability Discovery

- `StreamSession` (`visualization/stream.rs`): backpressure-aware session manager for live data push to petalTongue.
- `capabilities` (`visualization/capabilities.rs`): Songbird capability announcement (20 `health.*` capabilities) and discovery protocol.
- `Exp065`: Live dashboard streamer pushing ECG, HRV, and PK data in real-time.

### Live + Storable Visualization (V7.1)

`dump_scenarios` binary writes 14 petalTongue-compatible JSON files to `sandbox/scenarios/`. Local petalTongue evolution (3 non-invasive changes) wires `PrimalDefinition.data_channels` through `PrimalInfo.properties` to `draw_node_detail()`. Loading `healthspring-full-study.json` renders 28-node topology with 121 data channels and 16 clinical ranges on node click.

| Local petalTongue Change | File | Effect |
|--------------------------|------|--------|
| Schema: `data_channels` + `clinical_ranges` fields | `ecosystem.rs` | Accept scenario channels during load |
| Convert: serialize to `properties["data_channels_json"]` | `convert.rs` | Flow channels through existing property system |
| Render: deserialize and call `draw_node_detail()` | `primal_details.rs` | Charts appear on node click |

---

## Tier 2+3 Status — LIVE

All previous blocking items resolved:

1. ~~metalForge routing logic empty~~ → NUCLEUS atomics + dispatch planning (33 tests) ✓
2. ~~No GPU dispatch abstraction~~ → `GpuContext` + `execute_fused` ✓
3. ~~`population_pk_f64.wgsl` not written~~ → LIVE with u32 xorshift32 PRNG ✓
4. ~~No fused-op chain~~ → `execute_fused()` single encoder submission ✓
5. ODE solver absorption status from wetSpring TBD
6. ~~NPU dispatch path in toadStool not production-ready~~ → `DispatchPlan` with NPU routing + PCIe P2P ✓
7. ~~coralReef `df64_core.wgsl` preamble~~ → `strip_f64_enable()` workaround ✓

### V8 additions

- **Exp060**: CPU vs GPU parity matrix — 3 kernels x 3 scales through toadStool pipeline (27/27)
- **Exp061**: Mixed hardware dispatch — NUCLEUS topology + `DispatchPlan` + PCIe P2P transfers (22/22)
- **Exp062**: PCIe transfer validation — Gen3/4/5 bandwidth, realistic workloads, overhead analysis (26/26)
- **toadStool**: `StageOp::PopulationPk` + `StageOp::DiversityReduce` → GPU-native dispatch
- **metalForge**: `dispatch.rs` module with `plan_dispatch()`, `StageAssignment`, `DispatchPlan`
- **Workload**: Added `BiosignalFusion` and `EndocrinePk` variants for NPU and CPU routing

### V11 additions

- **Exp072**: Compute dashboard — `toadStool::execute_streaming()` wired to `petalTongue::StreamSession` for live per-stage progress gauges, timing bar charts, and NUCLEUS topology visualization (8/8)
- **`dump_scenarios`**: Extended to 8 scenarios (6 clinical + topology + dispatch) with topology/dispatch JSON artifacts
- **`topology.rs`**: NUCLEUS topology + dispatch plan scenario builders for petalTongue (tower/node/nest hierarchy, `PCIe` edges, stage assignment bar charts)
- **`compute_dashboard.sh`**: Now includes exp072 in orchestration sequence

### V12 additions

- **petalTongue stream completeness**: Added `replace` stream operation to `ipc_push.rs` and `StreamSession` — enables live updates to all 7 channel types (Heatmap, Bar, Scatter3D, Distribution, Spectrum, TimeSeries, Gauge)
- **Domain theming + protocol alignment**: `push_render_with_config()` passes `UiConfig` (panel visibility, zoom, theme) and explicit `domain` field to petalTongue. IPC response buffer increased from 4KB to 64KB for large capability responses
- **Clinical TRT archetypes**: 5 patient-parameterized TRT scenarios (young hypogonadal, obese diabetic, senior sarcopenic, former athlete, metabolic syndrome) wired into `dump_scenarios` (13 total)
- **Exp073**: Live TRT clinical dashboard — streams weekly PK troughs, HRV improvement, HbA1c trajectory, and cardiac risk comparison via `StreamSession` with `replace` for Bar channels (7/7)
- **Interaction roundtrip**: `query_capabilities()` and `subscribe_interactions()` on both `PetalTonguePushClient` and `StreamSession` — enables bidirectional clinician↔healthSpring flow
- **Exp074**: Interaction roundtrip validation with mock petalTongue — tests render, append, replace, gauge, HRV, capabilities, and subscribe (12/12)
- **PBPK tissue profiles**: `pbpk_iv_tissue_profiles()` returns per-tissue concentration TimeSeries; PBPK scenario node now has 5 tissue concentration curves
- **Pan-Tompkins intermediates**: QRS detection node now includes derivative, squared, and MWI (moving window integration) TimeSeries — 5 processing stages visible
- **Anderson lattice spectra**: Eigenvalue spectrum and per-eigenstate IPR spectrum added to microbiome Anderson node

### V14 additions (NLME + full pipeline)

- **NLME population PK**: `pkpd/nlme.rs` — FOCE (150 iterations) + SAEM (200 iterations) estimation on 30 subjects. Sovereign NONMEM/Monolix replacement. Theta/omega/sigma recovery validated (Exp075).
- **NCA**: `pkpd/nca.rs` — Lambda-z terminal slope, AUC_inf, MRT, CL, Vss. Sovereign WinNonlin replacement.
- **NLME diagnostics**: `pkpd/diagnostics.rs` — CWRES (mean <2.0), VPC (50 simulations, 5th/50th/95th percentile bands), GOF (observed vs predicted, R²≥0).
- **WFDB parser**: `wfdb.rs` — PhysioNet Format 212/16 streaming decoder, beat annotation parsing. Biosignal scenario now includes `wfdb_ecg` node.
- **Kokkos-equivalent benchmarks**: `benches/kokkos_parity.rs` — reduction, scatter, Monte Carlo, ODE batch, NLME iteration. Validates GPU-portable patterns ahead of promotion.
- **Full petalTongue pipeline**: 28 nodes (was 22), 29 edges (was 22), 121 channels (was 65). NLME scenario builder (5 new nodes). All 7 DataChannel types exercised.
- **Exp075**: NLME cross-validation — 19 binary checks (FOCE/SAEM parameter recovery, NCA λz/AUC∞, CWRES, GOF).
- **Exp076**: Full pipeline validation — 197 binary checks across all 5 tracks + full study structure.
- **`dump_scenarios`**: Extended to 14 scenarios (was 13), includes NLME JSON artifact.
- **Industry benchmarks**: SnapGene, Chromeleon, NONMEM, Monolix, WinNonlin profiled. Sovereign replacements documented in `specs/PAPER_REVIEW_QUEUE.md`.

### V22 — biomeOS BYOB Niche Deployment

healthSpring is a **niche**, not a node. The primal provides capabilities; the **graphs** define what the primals do together. With the primals running and the graphs loaded, biomeOS recreates the entire diagnostic pipeline through Neural API `capability.call` routing.

- **`healthspring_primal` binary**: `ecoPrimal/src/bin/healthspring_primal.rs` — JSON-RPC 2.0 server over Unix domain socket. Capability provider for the health niche. Modeled on airSpring's production primal pattern.
- **IPC module** (`ecoPrimal/src/ipc/`): `dispatch.rs` (capability→science routing, 50+ methods), `rpc.rs` (envelope helpers + outbound send), `socket.rs` (XDG socket resolution, primal discovery, compute/data primal probing).
- **Dispatch coverage**: All 6 science domains wired: PK/PD (9 handlers), Microbiome (11), Biosignal (7), Endocrine (4), Diagnostic (2), plus infrastructure (provenance trio, cross-primal forward, compute offload, data fetch).
- **Niche manifest**: `graphs/healthspring_niche.toml` — declares healthSpring as a transactional+continuous niche with 5 workflow graphs.
- **Workflow graphs** (`graphs/`):
  - `healthspring_patient_assessment.toml` — `ConditionalDag`: 4 parallel science tracks → cross-track → composite risk → visualize + store. The Neural API discovers the parallelism.
  - `healthspring_trt_scenario.toml` — `Sequential`: testosterone PK → outcomes → HRV → cardiac risk → gut axis → visualize.
  - `healthspring_microbiome_analysis.toml` — `Sequential`: diversity (parallel) → Anderson → colonization → QS → SCFA → gut-brain.
  - `healthspring_biosignal_monitor.toml` — `Continuous` @ 250 Hz: ECG → QRS → HRV → fusion → render. First continuous health niche.
  - `healthspring_niche_deploy.toml` — `Sequential`: 8-node DAG for primal startup.
- **biomeOS registration**: On startup, registers `lifecycle.register` + `capability.register` with orchestrator. 30 s heartbeat thread. Capability semantic mappings for `health` domain.
- **Socket**: `$XDG_RUNTIME_DIR/biomeos/healthspring-{family_id}.sock`, overridable via `HEALTHSPRING_SOCKET`.

### V14.1 additions (deep debt)

- **biosignal.rs → biosignal/ submodules**: 953-line monolith split into 6 domain-coherent modules (`ecg.rs`, `hrv.rs`, `ppg.rs`, `eda.rs`, `fusion.rs`, `fft.rs`) with `mod.rs` re-exporting all public items for API compatibility
- **`#![deny(clippy::pedantic)]` promoted**: All three lib crates (`barracuda`, `toadstool`, `metalForge/forge`) now deny pedantic lints. 62+ warnings resolved: `must_use`, `mul_add`, `branches_sharing_code`, `option_if_let_else`, `significant_drop_tightening`, `while_float`, `too_long_first_doc_paragraph`
- **DFT deduplication**: `visualization/scenarios/biosignal.rs` HRV power spectrum now delegates to `biosignal::fft::rfft` instead of local DFT reimplementation
- **Dead code removal**: Unused `cpu_stages` vector in `toadstool/src/pipeline.rs`
- **Idiomatic Rust**: `if let Some/else` chains replaced with `filter().map()` in `metalForge/forge/src/dispatch.rs`. Shared code hoisted from if/else branches in experiments
- **Provenance fixes**: `exp023_biosignal_fusion.py` → `exp023_fusion.py`, `exp040_barracuda_cpu_parity.py` → `exp040_barracuda_cpu.py`

### V13 additions (deep audit)

- **Anderson eigensolver**: Implemented tridiagonal QL algorithm (`anderson_diagonalize` in `microbiome.rs`) for correct eigenvalue/eigenvector computation. Fixed IPR bug in `diagnostic.rs` and `scenarios/microbiome.rs` — was using Hamiltonian diagonal instead of true eigenvectors
- **Smart clinical.rs refactor**: 1177 → 374 + 819 lines. Extracted 8 node-building functions to `clinical_nodes.rs` by domain responsibility
- **LCG PRNG centralization**: New `rng.rs` module (37 lines) with `LCG_MULTIPLIER`, `lcg_step()`, `state_to_f64()`. Replaced hardcoded constant in `diagnostic.rs`, `gpu/mod.rs`, `biosignal.rs`, `toadstool/stage.rs`
- **Math deduplication**: `endocrine::evenness_to_disorder` → delegates to `microbiome::evenness_to_disorder`. `endocrine::lognormal_params` → delegates to `pkpd::LognormalParam::to_normal_params()`
- **Capability-based discovery**: `capabilities.rs` uses glob-based Songbird socket search instead of hardcoded `/tmp/songbird.sock`
- **Flaky IPC test fix**: `AtomicU64` unique socket paths + refactored test harness eliminates race conditions
- **Doc-tests**: 4 added (`shannon_index`, `hill_dose_response`, `auc_trapezoidal`, `state_to_f64`)
- **Tolerance registry**: Added `exp067` and `exp069` CPU parity class entries at `1e-10`
- **gpu.rs → gpu/**: Module split to `gpu/mod.rs`, `gpu/dispatch.rs`, `gpu/context.rs`
- **scenarios.rs → scenarios/**: Module split to `scenarios/mod.rs`, `scenarios/{biosignal,endocrine,microbiome,pkpd,topology}.rs`
