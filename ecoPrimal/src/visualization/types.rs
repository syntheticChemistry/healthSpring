// SPDX-License-Identifier: AGPL-3.0-only
//! Type definitions for the petalTongue-compatible scenario schema.

use serde::Serialize;

/// A typed data channel attached to a scenario node.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "channel_type")]
pub enum DataChannel {
    #[serde(rename = "timeseries")]
    TimeSeries {
        id: String,
        label: String,
        x_label: String,
        y_label: String,
        unit: String,
        x_values: Vec<f64>,
        y_values: Vec<f64>,
    },
    #[serde(rename = "distribution")]
    Distribution {
        id: String,
        label: String,
        unit: String,
        values: Vec<f64>,
        mean: f64,
        std: f64,
        patient_value: f64,
    },
    #[serde(rename = "bar")]
    Bar {
        id: String,
        label: String,
        categories: Vec<String>,
        values: Vec<f64>,
        unit: String,
    },
    #[serde(rename = "gauge")]
    Gauge {
        id: String,
        label: String,
        value: f64,
        min: f64,
        max: f64,
        unit: String,
        normal_range: [f64; 2],
        warning_range: [f64; 2],
    },
    /// Frequency-domain spectrum (FFT, HRV power spectrum, noise analysis).
    #[serde(rename = "spectrum")]
    Spectrum {
        id: String,
        label: String,
        frequencies: Vec<f64>,
        amplitudes: Vec<f64>,
        unit: String,
    },
    /// 2D matrix visualization (correlation, dissimilarity, concentration grid).
    #[serde(rename = "heatmap")]
    Heatmap {
        id: String,
        label: String,
        x_labels: Vec<String>,
        y_labels: Vec<String>,
        values: Vec<f64>,
        unit: String,
    },
    /// 3D scatter plot (`PCoA` ordination, phase space, population PK).
    #[serde(rename = "scatter3d")]
    Scatter3D {
        id: String,
        label: String,
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<f64>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        point_labels: Vec<String>,
        unit: String,
    },
}

/// Clinical reference range for threshold coloring.
///
/// `status` is `String` to match petalTongue's upstream `ClinicalRange`
/// (which needs `Deserialize`). healthSpring serializes, petalTongue deserializes.
#[derive(Debug, Clone, Serialize)]
pub struct ClinicalRange {
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub status: String,
}

/// A node in the scenario graph.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub family: String,
    pub status: String,
    pub health: u8,
    pub confidence: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data_channels: Vec<DataChannel>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub clinical_ranges: Vec<ClinicalRange>,
}

/// 2D position for manual node placement in petalTongue layouts.
#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// An edge in the scenario graph.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub label: String,
}

/// Complete scenario — petalTongue-compatible with extensions.
#[derive(Debug, Clone, Serialize)]
pub struct HealthScenario {
    pub name: String,
    pub description: String,
    pub version: String,
    pub mode: String,
    pub sensory_config: SensoryConfig,
    pub ui_config: UiConfig,
    pub ecosystem: Ecosystem,
    pub neural_api: NeuralApi,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ScenarioEdge>,
}

/// Container for scenario graph nodes (primals) in the petalTongue ecosystem view.
#[derive(Debug, Clone, Serialize)]
pub struct Ecosystem {
    pub primals: Vec<ScenarioNode>,
}

/// Sensory requirements for petalTongue rendering (capability negotiation).
#[derive(Debug, Clone, Serialize)]
pub struct SensoryConfig {
    pub required_capabilities: CapReqs,
    pub optional_capabilities: CapReqs,
    pub complexity_hint: String,
}

/// Capability requirements (inputs and outputs) for sensory negotiation.
#[derive(Debug, Clone, Serialize)]
pub struct CapReqs {
    pub outputs: Vec<String>,
    pub inputs: Vec<String>,
}

/// UI configuration passed to petalTongue for theme, animation, and panel control.
#[derive(Debug, Clone, Serialize)]
pub struct UiConfig {
    pub theme: String,
    pub animations: Animations,
    pub performance: Performance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_panels: Option<ShowPanels>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awakening_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_zoom: Option<String>,
}

/// Panel visibility for petalTongue scenario config.
///
/// Uses individual `bool` fields rather than a bitfield because this struct
/// is serialized to JSON for petalTongue's upstream schema (which expects
/// named boolean keys). The schema is owned by petalTongue.
#[expect(
    clippy::struct_excessive_bools,
    reason = "matches petalTongue JSON schema — each field serializes as a named boolean key"
)]
#[derive(Debug, Clone, Serialize)]
pub struct ShowPanels {
    pub left_sidebar: bool,
    pub right_sidebar: bool,
    pub top_menu: bool,
    pub system_dashboard: bool,
    pub audio_panel: bool,
    pub trust_dashboard: bool,
    pub proprioception: bool,
    pub graph_stats: bool,
}

/// Animation settings for petalTongue graph rendering.
#[expect(clippy::struct_excessive_bools, reason = "matches petalTongue schema")]
#[derive(Debug, Clone, Serialize)]
pub struct Animations {
    pub enabled: bool,
    pub breathing_nodes: bool,
    pub connection_pulses: bool,
    pub smooth_transitions: bool,
    pub celebration_effects: bool,
}

/// Performance constraints for petalTongue rendering.
#[derive(Debug, Clone, Serialize)]
pub struct Performance {
    pub target_fps: u32,
    pub vsync: bool,
    pub hardware_acceleration: bool,
}

/// Neural API toggle for petalTongue (enables natural-language graph queries).
#[derive(Debug, Clone, Serialize)]
pub struct NeuralApi {
    pub enabled: bool,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn scenario_node_construction() {
        let node = ScenarioNode {
            id: "test-node".into(),
            name: "Test Node".into(),
            node_type: "compute".into(),
            family: "healthspring".into(),
            status: "healthy".into(),
            health: 100,
            confidence: 95,
            position: None,
            capabilities: vec!["cap.a".into()],
            data_channels: vec![],
            clinical_ranges: vec![],
        };
        assert_eq!(node.id, "test-node");
        assert_eq!(node.health, 100);
        assert_eq!(node.confidence, 95);
    }

    #[test]
    fn scenario_edge_construction() {
        let edge = ScenarioEdge {
            from: "a".into(),
            to: "b".into(),
            edge_type: "feeds".into(),
            label: "data".into(),
        };
        assert_eq!(edge.from, "a");
        assert_eq!(edge.to, "b");
    }

    #[test]
    fn position_construction() {
        let pos = Position { x: 10.5, y: 20.0 };
        assert!((pos.x - 10.5).abs() < 1e-10);
        assert!((pos.y - 20.0).abs() < 1e-10);
    }

    #[test]
    fn data_channel_gauge_serialization() {
        let ch = DataChannel::Gauge {
            id: "g1".into(),
            label: "Gauge 1".into(),
            value: 42.5,
            min: 0.0,
            max: 100.0,
            unit: "%".into(),
            normal_range: [20.0, 80.0],
            warning_range: [10.0, 90.0],
        };
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"gauge\""));
        assert!(json.contains("\"value\":42.5"));
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(parsed["channel_type"], "gauge");
    }

    #[test]
    fn data_channel_bar_serialization() {
        let ch = DataChannel::Bar {
            id: "bar1".into(),
            label: "Abundances".into(),
            categories: vec!["A".into(), "B".into()],
            values: vec![0.6, 0.4],
            unit: "relative".into(),
        };
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(json.contains("\"channel_type\":\"bar\""));
    }

    #[test]
    fn clinical_range_serialization() {
        let cr = ClinicalRange {
            label: "Normal".into(),
            min: 0.0,
            max: 10.0,
            status: "normal".into(),
        };
        let json = serde_json::to_string(&cr).expect("serialize");
        assert!(json.contains("\"label\":\"Normal\""));
    }

    #[test]
    fn performance_default_like_values() {
        let perf = Performance {
            target_fps: 60,
            vsync: true,
            hardware_acceleration: true,
        };
        assert_eq!(perf.target_fps, 60);
        assert!(perf.vsync);
    }

    #[test]
    fn animations_construction() {
        let anim = Animations {
            enabled: true,
            breathing_nodes: true,
            connection_pulses: true,
            smooth_transitions: true,
            celebration_effects: false,
        };
        let json = serde_json::to_string(&anim).expect("serialize");
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn scatter3d_skip_empty_point_labels() {
        let ch = DataChannel::Scatter3D {
            id: "pcoa".into(),
            label: "PCoA".into(),
            x: vec![1.0, 2.0],
            y: vec![3.0, 4.0],
            z: vec![5.0, 6.0],
            point_labels: vec![],
            unit: String::new(),
        };
        let json = serde_json::to_string(&ch).expect("serialize");
        assert!(!json.contains("point_labels"));
    }
}
