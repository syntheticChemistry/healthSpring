// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-track petalTongue scenario builders.
//!
//! Each builder calls real healthSpring math and wraps the outputs in
//! `DataChannel` / `ScenarioNode` / `HealthScenario` so petalTongue can
//! render them directly.

mod biosignal;
pub mod compute;
mod endocrine;
mod microbiome;
mod nlme;
mod pkpd;
pub mod topology;
mod v16;

use super::types::{
    Animations, CapReqs, ClinicalRange, DataChannel, Ecosystem, HealthScenario, NeuralApi,
    Performance, ScenarioEdge, ScenarioNode, SensoryConfig, UiConfig,
};

pub use biosignal::biosignal_study;
pub use compute::{
    compute_pipeline_study, gpu_scaling_study, v16_dispatch_study, v16_topology_study,
};
pub use endocrine::endocrine_study;
pub use microbiome::microbiome_study;
pub use nlme::nlme_study;
pub use pkpd::pkpd_study;
pub use v16::v16_study;

fn scaffold(name: &str, description: &str) -> HealthScenario {
    HealthScenario {
        name: name.into(),
        description: description.into(),
        version: "2.0.0".into(),
        mode: "live-ecosystem".into(),
        sensory_config: SensoryConfig {
            required_capabilities: CapReqs {
                outputs: vec!["visual".into()],
                inputs: vec![],
            },
            optional_capabilities: CapReqs {
                outputs: vec!["audio".into()],
                inputs: vec!["pointer".into(), "keyboard".into()],
            },
            complexity_hint: "standard".into(),
        },
        ui_config: UiConfig {
            theme: "benchtop-dark".into(),
            animations: Animations {
                enabled: true,
                breathing_nodes: true,
                connection_pulses: true,
                smooth_transitions: true,
                celebration_effects: false,
            },
            performance: Performance {
                target_fps: 60,
                vsync: true,
                hardware_acceleration: true,
            },
            show_panels: None,
            awakening_enabled: None,
            initial_zoom: None,
        },
        ecosystem: Ecosystem { primals: vec![] },
        neural_api: NeuralApi { enabled: false },
        edges: Vec::new(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "internal helper — all args have clear roles"
)]
pub(crate) fn gauge(
    id: &str,
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    normal: [f64; 2],
    warn: [f64; 2],
) -> DataChannel {
    DataChannel::Gauge {
        id: id.into(),
        label: label.into(),
        value,
        min,
        max,
        unit: unit.into(),
        normal_range: normal,
        warning_range: warn,
    }
}

pub(crate) fn timeseries(
    id: &str,
    label: &str,
    x_label: &str,
    y_label: &str,
    unit: &str,
    xs: Vec<f64>,
    ys: Vec<f64>,
) -> DataChannel {
    DataChannel::TimeSeries {
        id: id.into(),
        label: label.into(),
        x_label: x_label.into(),
        y_label: y_label.into(),
        unit: unit.into(),
        x_values: xs,
        y_values: ys,
    }
}

pub(crate) fn bar(
    id: &str,
    label: &str,
    cats: Vec<String>,
    vals: Vec<f64>,
    unit: &str,
) -> DataChannel {
    DataChannel::Bar {
        id: id.into(),
        label: label.into(),
        categories: cats,
        values: vals,
        unit: unit.into(),
    }
}

pub(crate) fn spectrum(
    id: &str,
    label: &str,
    frequencies: Vec<f64>,
    amplitudes: Vec<f64>,
    unit: &str,
) -> DataChannel {
    DataChannel::Spectrum {
        id: id.into(),
        label: label.into(),
        frequencies,
        amplitudes,
        unit: unit.into(),
    }
}

pub(crate) fn heatmap(
    id: &str,
    label: &str,
    x_labels: Vec<String>,
    y_labels: Vec<String>,
    values: Vec<f64>,
    unit: &str,
) -> DataChannel {
    DataChannel::Heatmap {
        id: id.into(),
        label: label.into(),
        x_labels,
        y_labels,
        values,
        unit: unit.into(),
    }
}

pub(crate) fn scatter3d(
    id: &str,
    label: &str,
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    point_labels: Vec<String>,
    unit: &str,
) -> DataChannel {
    DataChannel::Scatter3D {
        id: id.into(),
        label: label.into(),
        x,
        y,
        z,
        point_labels,
        unit: unit.into(),
    }
}

pub(crate) fn node(
    id: &str,
    name: &str,
    node_type: &str,
    caps: &[&str],
    channels: Vec<DataChannel>,
    ranges: Vec<ClinicalRange>,
) -> ScenarioNode {
    ScenarioNode {
        id: id.into(),
        name: name.into(),
        node_type: node_type.into(),
        family: crate::PRIMAL_NAME.into(),
        status: "healthy".into(),
        health: 100,
        confidence: 95,
        position: None,
        capabilities: caps.iter().map(|s| (*s).into()).collect(),
        data_channels: channels,
        clinical_ranges: ranges,
    }
}

pub(crate) fn edge(from: &str, to: &str, label: &str) -> ScenarioEdge {
    ScenarioEdge {
        from: from.into(),
        to: to.into(),
        edge_type: "data-flow".into(),
        label: label.into(),
    }
}

// ---------------------------------------------------------------------------
// Full Study (all 4 tracks combined)
// ---------------------------------------------------------------------------

/// Build a combined all-tracks scenario for the complete healthSpring study.
#[must_use]
pub fn full_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let (pkpd, mut pkpd_edges) = pkpd_study();
    let (micro, mut micro_edges) = microbiome_study();
    let (bio, mut bio_edges) = biosignal_study();
    let (endo, mut endo_edges) = endocrine_study();
    let (nlme, mut nlme_edges) = nlme_study();
    let (v16, mut v16_edges) = v16_study();

    let mut s = scaffold(
        "healthSpring Complete Study",
        "All 6 tracks: PK/PD + Microbiome + Biosignal + Endocrinology + NLME + V16 Primitives — full pipeline",
    );

    for track in [pkpd, micro, bio, endo, nlme, v16] {
        for n in track.ecosystem.primals {
            s.ecosystem.primals.push(n);
        }
    }

    let mut all_edges = Vec::new();
    all_edges.append(&mut pkpd_edges);
    all_edges.append(&mut micro_edges);
    all_edges.append(&mut bio_edges);
    all_edges.append(&mut endo_edges);
    all_edges.append(&mut nlme_edges);
    all_edges.append(&mut v16_edges);

    // Cross-track links (original)
    all_edges.push(edge(
        "pop_pk",
        "diversity",
        "PK variability × gut diversity",
    ));
    all_edges.push(edge("diversity", "gut_axis", "microbiome → TRT metabolic"));
    all_edges.push(edge("hrv", "hrv_cardiac", "biosignal HRV → TRT cardiac"));
    all_edges.push(edge("one_comp", "t_im", "PK/PD → endocrine PK"));
    all_edges.push(edge("pop_pk", "nlme_population", "population PK → NLME"));

    // V16 cross-track links (bridging existing tracks to V16 nodes)
    all_edges.push(edge("one_comp", "mm_nonlinear_pk", "linear → nonlinear PK"));
    all_edges.push(edge("diversity", "abx_perturbation", "diversity source"));
    all_edges.push(edge("fusion", "eda_stress", "biosignal → EDA"));
    all_edges.push(edge("qrs", "arrhythmia_classify", "QRS → classification"));

    (s, all_edges)
}

/// Serialize a scenario + edges to pretty JSON.
///
/// Edges are merged into the scenario's `edges` field for a single clean JSON output.
/// All types implement `Serialize` with no dynamic content, so serialization
/// is infallible — the `expect` cannot fire.
///
/// # Panics
///
/// Cannot panic. `serde_json::to_string_pretty` only fails for recursive
/// structures or custom serializers that fail; `HealthScenario` has neither.
#[must_use]
pub fn scenario_with_edges_json(scenario: &HealthScenario, edges: &[ScenarioEdge]) -> String {
    let mut merged = scenario.clone();
    merged.edges.extend_from_slice(edges);
    #[expect(
        clippy::expect_used,
        reason = "Serialize impls on fixed structs cannot fail"
    )]
    serde_json::to_string_pretty(&merged).expect("serialization cannot fail")
}

#[cfg(test)]
mod tests;
