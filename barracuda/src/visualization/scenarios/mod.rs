// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-track petalTongue scenario builders.
//!
//! Each builder calls real healthSpring math and wraps the outputs in
//! `DataChannel` / `ScenarioNode` / `HealthScenario` so petalTongue can
//! render them directly.

mod biosignal;
mod endocrine;
mod microbiome;
mod pkpd;
pub mod topology;

use super::types::{
    Animations, CapReqs, ClinicalRange, DataChannel, Ecosystem, HealthScenario, NeuralApi,
    Performance, ScenarioEdge, ScenarioNode, SensoryConfig, UiConfig,
};

pub use biosignal::biosignal_study;
pub use endocrine::endocrine_study;
pub use microbiome::microbiome_study;
pub use pkpd::pkpd_study;

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
        family: "healthspring".into(),
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

    let mut s = scaffold(
        "healthSpring Complete Study",
        "All 4 tracks: PK/PD + Microbiome + Biosignal + Endocrinology — 30 experiments",
    );

    for track in [pkpd, micro, bio, endo] {
        for n in track.ecosystem.primals {
            s.ecosystem.primals.push(n);
        }
    }

    let mut all_edges = Vec::new();
    all_edges.append(&mut pkpd_edges);
    all_edges.append(&mut micro_edges);
    all_edges.append(&mut bio_edges);
    all_edges.append(&mut endo_edges);

    // Cross-track links
    all_edges.push(edge(
        "pop_pk",
        "diversity",
        "PK variability × gut diversity",
    ));
    all_edges.push(edge("diversity", "gut_axis", "microbiome → TRT metabolic"));
    all_edges.push(edge("hrv", "hrv_cardiac", "biosignal HRV → TRT cardiac"));
    all_edges.push(edge("one_comp", "t_im", "PK/PD → endocrine PK"));

    (s, all_edges)
}

/// Serialize a scenario + edges to pretty JSON.
///
/// Edges are merged into the scenario's `edges` field for a single clean JSON output.
///
/// # Panics
/// Cannot panic — all types are `Serialize`.
#[must_use]
pub fn scenario_with_edges_json(scenario: &HealthScenario, edges: &[ScenarioEdge]) -> String {
    let mut merged = scenario.clone();
    merged.edges.extend_from_slice(edges);
    serde_json::to_string_pretty(&merged).expect("serialization cannot fail")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::{DataChannel, HealthScenario, ScenarioEdge};

    fn assert_study_invariants(
        scenario: &HealthScenario,
        edges: &[ScenarioEdge],
        expected_node_ids: &[&str],
        expected_edge_count: usize,
    ) {
        let nodes = &scenario.ecosystem.primals;
        assert_eq!(nodes.len(), expected_node_ids.len(), "node count mismatch");
        for node in nodes {
            assert!(
                node.health <= 100,
                "node {} health {} > 100",
                node.id,
                node.health
            );
        }
        let ids: std::collections::HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(
            ids.len(),
            nodes.len(),
            "duplicate node IDs: {:?}",
            nodes.iter().map(|n| &n.id).collect::<Vec<_>>()
        );
        for id in expected_node_ids {
            assert!(
                ids.contains(id),
                "expected node id {} not found in {:?}",
                id,
                ids
            );
        }
        assert_eq!(edges.len(), expected_edge_count, "edge count mismatch");
        let node_ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();
        for e in edges {
            assert!(
                node_ids.contains(e.from.as_str()),
                "edge from {} references unknown node",
                e.from
            );
            assert!(
                node_ids.contains(e.to.as_str()),
                "edge to {} references unknown node",
                e.to
            );
        }
    }

    fn assert_json_roundtrips(scenario: &HealthScenario, edges: &[ScenarioEdge]) {
        let json = scenario_with_edges_json(scenario, edges);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON must be valid");
        assert!(parsed.get("name").is_some());
        assert!(parsed.get("ecosystem").is_some());
        assert!(parsed.get("edges").is_some());
        assert!(parsed["edges"].is_array());
    }

    #[test]
    fn pkpd_study_structure() {
        let (scenario, edges) = pkpd_study();
        assert_study_invariants(
            &scenario,
            &edges,
            &["hill", "one_comp", "two_comp", "mab", "pop_pk", "pbpk"],
            5,
        );
    }

    #[test]
    fn pkpd_study_capabilities() {
        let (scenario, _) = pkpd_study();
        let caps: std::collections::HashSet<String> = scenario
            .ecosystem
            .primals
            .iter()
            .flat_map(|n| n.capabilities.clone())
            .collect();
        assert!(caps.contains("science.pkpd.hill_dose_response"));
        assert!(caps.contains("science.pkpd.one_compartment_pk"));
        assert!(caps.contains("science.pkpd.two_compartment_pk"));
        assert!(caps.contains("science.pkpd.allometric_scaling"));
        assert!(caps.contains("science.pkpd.population_pk"));
        assert!(caps.contains("science.pkpd.pbpk"));
    }

    #[test]
    fn pkpd_study_json_roundtrips() {
        let (scenario, edges) = pkpd_study();
        assert_json_roundtrips(&scenario, &edges);
    }

    #[test]
    fn microbiome_study_structure() {
        let (scenario, edges) = microbiome_study();
        assert_study_invariants(
            &scenario,
            &edges,
            &["diversity", "anderson", "cdiff", "fmt"],
            3,
        );
    }

    #[test]
    fn microbiome_study_capabilities() {
        let (scenario, _) = microbiome_study();
        let caps: std::collections::HashSet<String> = scenario
            .ecosystem
            .primals
            .iter()
            .flat_map(|n| n.capabilities.clone())
            .collect();
        assert!(caps.contains("science.microbiome.diversity"));
        assert!(caps.contains("science.microbiome.anderson_lattice"));
        assert!(caps.contains("science.microbiome.cdiff_resistance"));
        assert!(caps.contains("science.microbiome.fmt"));
    }

    #[test]
    fn microbiome_study_json_roundtrips() {
        let (scenario, edges) = microbiome_study();
        assert_json_roundtrips(&scenario, &edges);
    }

    #[test]
    fn biosignal_study_structure() {
        let (scenario, edges) = biosignal_study();
        assert_study_invariants(&scenario, &edges, &["qrs", "hrv", "spo2", "fusion"], 3);
    }

    #[test]
    fn biosignal_study_capabilities() {
        let (scenario, _) = biosignal_study();
        let caps: std::collections::HashSet<String> = scenario
            .ecosystem
            .primals
            .iter()
            .flat_map(|n| n.capabilities.clone())
            .collect();
        assert!(caps.contains("science.biosignal.pan_tompkins"));
        assert!(caps.contains("science.biosignal.hrv"));
        assert!(caps.contains("science.biosignal.ppg_spo2"));
        assert!(caps.contains("science.biosignal.fusion"));
    }

    #[test]
    fn biosignal_study_json_roundtrips() {
        let (scenario, edges) = biosignal_study();
        assert_json_roundtrips(&scenario, &edges);
    }

    #[test]
    fn endocrine_study_structure() {
        let (scenario, edges) = endocrine_study();
        assert_study_invariants(
            &scenario,
            &edges,
            &[
                "t_im",
                "t_pellet",
                "age_decline",
                "trt_weight",
                "trt_cardio",
                "trt_diabetes",
                "gut_axis",
                "hrv_cardiac",
            ],
            7,
        );
    }

    #[test]
    fn endocrine_study_capabilities() {
        let (scenario, _) = endocrine_study();
        let caps: std::collections::HashSet<String> = scenario
            .ecosystem
            .primals
            .iter()
            .flat_map(|n| n.capabilities.clone())
            .collect();
        assert!(caps.contains("science.endocrine.testosterone_im"));
        assert!(caps.contains("science.endocrine.testosterone_pellet"));
        assert!(caps.contains("science.endocrine.testosterone_decline"));
        assert!(caps.contains("science.endocrine.trt_weight"));
        assert!(caps.contains("science.endocrine.trt_cardiovascular"));
        assert!(caps.contains("science.endocrine.trt_diabetes"));
        assert!(caps.contains("science.endocrine.gut_trt_axis"));
        assert!(caps.contains("science.endocrine.hrv_trt"));
    }

    #[test]
    fn endocrine_study_json_roundtrips() {
        let (scenario, edges) = endocrine_study();
        assert_json_roundtrips(&scenario, &edges);
    }

    #[test]
    fn full_study_all_nodes_and_edges() {
        let (scenario, edges) = full_study();
        assert_eq!(
            scenario.ecosystem.primals.len(),
            22,
            "full_study must have 22 nodes"
        );
        assert_eq!(
            edges.len(),
            22,
            "full_study must have 18 domain + 4 cross = 22 edges"
        );
        let ids: std::collections::HashSet<&str> = scenario
            .ecosystem
            .primals
            .iter()
            .map(|n| n.id.as_str())
            .collect();
        assert_eq!(ids.len(), 22, "all node IDs must be unique");
        for node in &scenario.ecosystem.primals {
            assert!(
                node.health <= 100,
                "node {} health {} > 100",
                node.id,
                node.health
            );
        }
        let node_ids: std::collections::HashSet<&str> = scenario
            .ecosystem
            .primals
            .iter()
            .map(|n| n.id.as_str())
            .collect();
        for e in &edges {
            assert!(
                node_ids.contains(e.from.as_str()),
                "edge from {} references unknown node",
                e.from
            );
            assert!(
                node_ids.contains(e.to.as_str()),
                "edge to {} references unknown node",
                e.to
            );
        }
        assert!(ids.contains("pop_pk"));
        assert!(ids.contains("diversity"));
        assert!(ids.contains("gut_axis"));
        assert!(ids.contains("hrv"));
        assert!(ids.contains("hrv_cardiac"));
        assert!(ids.contains("one_comp"));
        assert!(ids.contains("t_im"));
    }

    #[test]
    fn full_study_cross_track_edges() {
        let (_, edges) = full_study();
        let edge_pairs: std::collections::HashSet<(String, String)> = edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone()))
            .collect();
        assert!(
            edge_pairs.contains(&("pop_pk".into(), "diversity".into())),
            "cross-track: pop_pk -> diversity"
        );
        assert!(
            edge_pairs.contains(&("diversity".into(), "gut_axis".into())),
            "cross-track: diversity -> gut_axis"
        );
        assert!(
            edge_pairs.contains(&("hrv".into(), "hrv_cardiac".into())),
            "cross-track: hrv -> hrv_cardiac"
        );
        assert!(
            edge_pairs.contains(&("one_comp".into(), "t_im".into())),
            "cross-track: one_comp -> t_im"
        );
    }

    #[test]
    fn scenario_with_edges_json_valid() {
        let (scenario, edges) = pkpd_study();
        let json = scenario_with_edges_json(&scenario, &edges);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["name"].as_str().is_some());
        assert!(parsed["ecosystem"]["primals"].is_array());
        assert!(parsed["edges"].is_array());
        assert_eq!(parsed["edges"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn scaffold_structure_via_study() {
        let (scenario, _) = pkpd_study();
        assert!(!scenario.name.is_empty());
        assert!(!scenario.description.is_empty());
        assert_eq!(scenario.version, "2.0.0");
        assert_eq!(scenario.mode, "live-ecosystem");
        assert_eq!(
            scenario.sensory_config.required_capabilities.outputs,
            vec!["visual"]
        );
        assert!(scenario.ui_config.theme.contains("benchtop"));
        assert!(!scenario.neural_api.enabled);
    }

    #[test]
    fn gauge_produces_gauge_channel() {
        let ch = super::gauge(
            "g1",
            "Test Gauge",
            50.0,
            0.0,
            100.0,
            "unit",
            [20.0, 80.0],
            [10.0, 20.0],
        );
        match &ch {
            DataChannel::Gauge {
                id,
                label,
                value,
                unit,
                ..
            } => {
                assert_eq!(id, "g1");
                assert_eq!(label, "Test Gauge");
                assert!((*value - 50.0).abs() < 1e-9);
                assert_eq!(unit, "unit");
            }
            _ => panic!("expected Gauge, got {:?}", ch),
        }
    }

    #[test]
    fn timeseries_produces_timeseries_channel() {
        let ch = super::timeseries(
            "ts1",
            "Test TS",
            "X",
            "Y",
            "u",
            vec![1.0, 2.0],
            vec![10.0, 20.0],
        );
        match &ch {
            DataChannel::TimeSeries {
                id,
                label,
                x_label,
                y_label,
                x_values,
                y_values,
                ..
            } => {
                assert_eq!(id, "ts1");
                assert_eq!(label, "Test TS");
                assert_eq!(x_label, "X");
                assert_eq!(y_label, "Y");
                assert_eq!(x_values, &[1.0, 2.0]);
                assert_eq!(y_values, &[10.0, 20.0]);
            }
            _ => panic!("expected TimeSeries, got {:?}", ch),
        }
    }

    #[test]
    fn bar_produces_bar_channel() {
        let ch = super::bar(
            "b1",
            "Test Bar",
            vec!["A".into(), "B".into()],
            vec![1.0, 2.0],
            "u",
        );
        match &ch {
            DataChannel::Bar {
                id,
                label,
                categories,
                values,
                unit,
                ..
            } => {
                assert_eq!(id, "b1");
                assert_eq!(label, "Test Bar");
                assert_eq!(categories, &["A", "B"]);
                assert_eq!(values, &[1.0, 2.0]);
                assert_eq!(unit, "u");
            }
            _ => panic!("expected Bar, got {:?}", ch),
        }
    }

    #[test]
    fn node_produces_scenario_node() {
        let n = super::node("n1", "Node Name", "compute", &["cap1"], vec![], vec![]);
        assert_eq!(n.id, "n1");
        assert_eq!(n.name, "Node Name");
        assert_eq!(n.node_type, "compute");
        assert_eq!(n.family, "healthspring");
        assert_eq!(n.status, "healthy");
        assert_eq!(n.health, 100);
        assert_eq!(n.confidence, 95);
        assert!(
            n.position.is_none(),
            "position should be None for graph layout"
        );
        assert_eq!(n.capabilities, vec!["cap1"]);
    }

    #[test]
    fn edge_produces_scenario_edge() {
        let e = super::edge("a", "b", "a to b");
        assert_eq!(e.from, "a");
        assert_eq!(e.to, "b");
        assert_eq!(e.edge_type, "data-flow");
        assert_eq!(e.label, "a to b");
    }

    #[test]
    fn spectrum_produces_spectrum_channel() {
        let ch = super::spectrum(
            "sp1",
            "Test Spectrum",
            vec![0.1, 0.2, 0.3],
            vec![10.0, 20.0, 5.0],
            "ms²/Hz",
        );
        match &ch {
            DataChannel::Spectrum {
                id,
                label,
                frequencies,
                amplitudes,
                unit,
            } => {
                assert_eq!(id, "sp1");
                assert_eq!(label, "Test Spectrum");
                assert_eq!(frequencies, &[0.1, 0.2, 0.3]);
                assert_eq!(amplitudes, &[10.0, 20.0, 5.0]);
                assert_eq!(unit, "ms²/Hz");
            }
            _ => panic!("expected Spectrum, got {ch:?}"),
        }
    }

    #[test]
    fn heatmap_produces_heatmap_channel() {
        let ch = super::heatmap(
            "hm1",
            "Test Heatmap",
            vec!["A".into(), "B".into()],
            vec!["X".into(), "Y".into()],
            vec![1.0, 2.0, 3.0, 4.0],
            "BC",
        );
        match &ch {
            DataChannel::Heatmap {
                id,
                label,
                x_labels,
                y_labels,
                values,
                unit,
            } => {
                assert_eq!(id, "hm1");
                assert_eq!(label, "Test Heatmap");
                assert_eq!(x_labels, &["A", "B"]);
                assert_eq!(y_labels, &["X", "Y"]);
                assert_eq!(values, &[1.0, 2.0, 3.0, 4.0]);
                assert_eq!(unit, "BC");
            }
            _ => panic!("expected Heatmap, got {ch:?}"),
        }
    }

    #[test]
    fn scatter3d_produces_scatter3d_channel() {
        let ch = super::scatter3d(
            "s3d",
            "Test 3D",
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
            vec!["P1".into(), "P2".into()],
            "mixed",
        );
        match &ch {
            DataChannel::Scatter3D {
                id,
                label,
                x,
                y,
                z,
                point_labels,
                unit,
            } => {
                assert_eq!(id, "s3d");
                assert_eq!(label, "Test 3D");
                assert_eq!(x, &[1.0, 2.0]);
                assert_eq!(y, &[3.0, 4.0]);
                assert_eq!(z, &[5.0, 6.0]);
                assert_eq!(point_labels, &["P1", "P2"]);
                assert_eq!(unit, "mixed");
            }
            _ => panic!("expected Scatter3D, got {ch:?}"),
        }
    }

    #[test]
    fn biosignal_study_has_spectrum_channel() {
        let (scenario, _) = biosignal_study();
        let hrv_node = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "hrv")
            .expect("hrv node");
        let has_spectrum = hrv_node
            .data_channels
            .iter()
            .any(|ch| matches!(ch, DataChannel::Spectrum { id, .. } if id == "hrv_psd"));
        assert!(has_spectrum, "HRV node should have a Spectrum channel");
    }

    #[test]
    fn microbiome_study_has_heatmap_channel() {
        let (scenario, _) = microbiome_study();
        let div_node = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "diversity")
            .expect("diversity node");
        let has_heatmap = div_node
            .data_channels
            .iter()
            .any(|ch| matches!(ch, DataChannel::Heatmap { id, .. } if id == "bray_curtis"));
        assert!(has_heatmap, "Diversity node should have a Heatmap channel");
    }

    #[test]
    fn pkpd_study_has_scatter3d_channel() {
        let (scenario, _) = pkpd_study();
        let pop_node = scenario
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "pop_pk")
            .expect("pop_pk node");
        let has_scatter = pop_node
            .data_channels
            .iter()
            .any(|ch| matches!(ch, DataChannel::Scatter3D { id, .. } if id == "pop_pk_3d"));
        assert!(has_scatter, "Pop PK node should have a Scatter3D channel");
    }

    #[test]
    fn spectrum_serializes_correctly() {
        let ch = super::spectrum("sp", "Spec", vec![0.1], vec![1.0], "dB");
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"spectrum\""));
        assert!(json.contains("\"frequencies\""));
        assert!(json.contains("\"amplitudes\""));
    }

    #[test]
    fn heatmap_serializes_correctly() {
        let ch = super::heatmap(
            "hm",
            "Heat",
            vec!["A".into()],
            vec!["B".into()],
            vec![1.0],
            "u",
        );
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"heatmap\""));
        assert!(json.contains("\"x_labels\""));
        assert!(json.contains("\"y_labels\""));
    }

    #[test]
    fn scatter3d_serializes_correctly() {
        let ch = super::scatter3d("s", "3D", vec![1.0], vec![2.0], vec![3.0], vec![], "u");
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"scatter3d\""));
        assert!(!json.contains("\"point_labels\""));
    }

    #[test]
    fn scatter3d_with_labels_serializes() {
        let ch = super::scatter3d(
            "s",
            "3D",
            vec![1.0],
            vec![2.0],
            vec![3.0],
            vec!["P1".into()],
            "u",
        );
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"point_labels\""));
    }
}
