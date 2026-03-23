// SPDX-License-Identifier: AGPL-3.0-or-later
//! Type definitions for the petalTongue-compatible scenario schema.

use serde::Serialize;

/// A typed data channel attached to a scenario node.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "channel_type")]
pub enum DataChannel {
    /// Uniformly sampled signal over time with labeled axes.
    #[serde(rename = "timeseries")]
    TimeSeries {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// Label for the horizontal (independent) axis.
        x_label: String,
        /// Label for the vertical (dependent) axis.
        y_label: String,
        /// Physical unit for the dependent variable (y).
        unit: String,
        /// Sample positions along the independent axis.
        x_values: Vec<f64>,
        /// Sample values at each x position.
        y_values: Vec<f64>,
    },
    /// Empirical distribution with summary statistics and a highlighted patient value.
    #[serde(rename = "distribution")]
    Distribution {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// Unit for displayed values and summary stats.
        unit: String,
        /// Raw draws or histogram bucket values.
        values: Vec<f64>,
        /// Mean over `values`.
        mean: f64,
        /// Standard deviation over `values`.
        std: f64,
        /// Value emphasized for the current patient or cohort.
        patient_value: f64,
    },
    /// Categorical comparison with one bar per category.
    #[serde(rename = "bar")]
    Bar {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// Category labels aligned with `values`.
        categories: Vec<String>,
        /// Bar heights or magnitudes per category.
        values: Vec<f64>,
        /// Physical unit for displayed magnitudes.
        unit: String,
    },
    /// Scalar reading with scale bounds and colored normal vs warning bands.
    #[serde(rename = "gauge")]
    Gauge {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// Current reading within the gauge range.
        value: f64,
        /// Lower bound of the gauge scale.
        min: f64,
        /// Upper bound of the gauge scale.
        max: f64,
        /// Physical unit for the reading.
        unit: String,
        /// Inclusive [low, high] band treated as clinically normal.
        normal_range: [f64; 2],
        /// Inclusive [low, high] band between normal and clearly out-of-range.
        warning_range: [f64; 2],
    },
    /// Frequency-domain spectrum (FFT, HRV power spectrum, noise analysis).
    #[serde(rename = "spectrum")]
    Spectrum {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// Frequency bin centers (Hz or normalized units per upstream convention).
        frequencies: Vec<f64>,
        /// Power or magnitude at each frequency bin.
        amplitudes: Vec<f64>,
        /// Physical unit for displayed magnitudes.
        unit: String,
    },
    /// 2D matrix visualization (correlation, dissimilarity, concentration grid).
    #[serde(rename = "heatmap")]
    Heatmap {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// Column labels (horizontal axis).
        x_labels: Vec<String>,
        /// Row labels (vertical axis).
        y_labels: Vec<String>,
        /// Row-major flattened grid aligned with `x_labels` × `y_labels`.
        values: Vec<f64>,
        /// Physical unit for cell values.
        unit: String,
    },
    /// 3D scatter plot (`PCoA` ordination, phase space, population PK).
    #[serde(rename = "scatter3d")]
    Scatter3D {
        /// Stable channel id for binding and updates.
        id: String,
        /// Human-readable title shown in the UI.
        label: String,
        /// First coordinate per sample (same length as `y` and `z`).
        x: Vec<f64>,
        /// Second coordinate per sample (same length as `x` and `z`).
        y: Vec<f64>,
        /// Third coordinate per sample (same length as `x` and `y`).
        z: Vec<f64>,
        /// Optional per-point labels for tooltips or legend entries.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        point_labels: Vec<String>,
        /// Physical unit for displayed coordinates when applicable.
        unit: String,
    },
}

/// Clinical reference range for threshold coloring.
///
/// `status` is `String` to match petalTongue's upstream `ClinicalRange`
/// (which needs `Deserialize`). healthSpring serializes, petalTongue deserializes.
#[derive(Debug, Clone, Serialize)]
pub struct ClinicalRange {
    /// Human-readable band name (e.g. "Normal", "Critical").
    pub label: String,
    /// Lower bound of the inclusive range.
    pub min: f64,
    /// Upper bound of the inclusive range.
    pub max: f64,
    /// petalTongue status token for styling (e.g. normal vs warning).
    pub status: String,
}

/// A node in the scenario graph.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioNode {
    /// Stable node id referenced by edges and updates.
    pub id: String,
    /// Display name for graph labels and panels.
    pub name: String,
    /// Semantic node kind string (e.g. compute, sensor) for petalTongue.
    #[serde(rename = "type")]
    pub node_type: String,
    /// Grouping or subsystem label for layout and filtering.
    pub family: String,
    /// Coarse health or lifecycle state as a string token.
    pub status: String,
    /// 0–100 aggregate health score for visualization.
    pub health: u8,
    /// 0–100 confidence score for the node's outputs or inference.
    pub confidence: u8,
    /// Optional manual layout coordinates when provided by the scenario.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Advertised capability strings for sensory negotiation.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    /// Attached visualization channels (time series, gauges, etc.).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data_channels: Vec<DataChannel>,
    /// Reference ranges used for threshold coloring on this node.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub clinical_ranges: Vec<ClinicalRange>,
}

/// 2D position for manual node placement in petalTongue layouts.
#[derive(Debug, Clone, Serialize)]
pub struct Position {
    /// Horizontal coordinate in layout space.
    pub x: f64,
    /// Vertical coordinate in layout space.
    pub y: f64,
}

/// An edge in the scenario graph.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioEdge {
    /// Source node id.
    pub from: String,
    /// Destination node id.
    pub to: String,
    /// Relationship kind (e.g. data flow, dependency).
    pub edge_type: String,
    /// Short caption or flow description for the link.
    pub label: String,
}

/// Complete scenario — petalTongue-compatible with extensions.
#[derive(Debug, Clone, Serialize)]
pub struct HealthScenario {
    /// Human-readable scenario title.
    pub name: String,
    /// Long-form summary for tooltips or intro panels.
    pub description: String,
    /// Schema or bundle version string for compatibility checks.
    pub version: String,
    /// Operating mode token (e.g. demo vs live) for the client.
    pub mode: String,
    /// Required and optional sensory capabilities for rendering.
    pub sensory_config: SensoryConfig,
    /// Theme, animation, performance, and panel preferences.
    pub ui_config: UiConfig,
    /// Graph nodes (primals) shown in the ecosystem view.
    pub ecosystem: Ecosystem,
    /// Whether natural-language graph queries are enabled in the UI.
    pub neural_api: NeuralApi,
    /// Directed links between nodes; omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ScenarioEdge>,
}

/// Container for scenario graph nodes (primals) in the petalTongue ecosystem view.
#[derive(Debug, Clone, Serialize)]
pub struct Ecosystem {
    /// Scenario nodes rendered as primals in the ecosystem graph.
    pub primals: Vec<ScenarioNode>,
}

/// Sensory requirements for petalTongue rendering (capability negotiation).
#[derive(Debug, Clone, Serialize)]
pub struct SensoryConfig {
    /// Capabilities the client must support to render this scenario.
    pub required_capabilities: CapReqs,
    /// Capabilities that improve the experience but may be skipped.
    pub optional_capabilities: CapReqs,
    /// Hint for how heavy or elaborate rendering should be.
    pub complexity_hint: String,
}

/// Capability requirements (inputs and outputs) for sensory negotiation.
#[derive(Debug, Clone, Serialize)]
pub struct CapReqs {
    /// Output channel or modality names required from the renderer.
    pub outputs: Vec<String>,
    /// Input channel or modality names the scenario may consume.
    pub inputs: Vec<String>,
}

/// UI configuration passed to petalTongue for theme, animation, and panel control.
#[derive(Debug, Clone, Serialize)]
pub struct UiConfig {
    /// Visual theme name understood by petalTongue.
    pub theme: String,
    /// Graph motion and transition toggles.
    pub animations: Animations,
    /// Frame rate and GPU-related preferences.
    pub performance: Performance,
    /// Optional per-panel visibility overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_panels: Option<ShowPanels>,
    /// Optional feature flag for awakening-related UI when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awakening_enabled: Option<bool>,
    /// Optional initial zoom preset or level string for the graph view.
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
    /// Show or hide the left sidebar region.
    pub left_sidebar: bool,
    /// Show or hide the right sidebar region.
    pub right_sidebar: bool,
    /// Show or hide the top menu bar.
    pub top_menu: bool,
    /// Show or hide the system status dashboard panel.
    pub system_dashboard: bool,
    /// Show or hide audio-related controls or visualization.
    pub audio_panel: bool,
    /// Show or hide trust / assurance dashboard content.
    pub trust_dashboard: bool,
    /// Show or hide proprioception-related panels.
    pub proprioception: bool,
    /// Show or hide graph statistics overlays or widgets.
    pub graph_stats: bool,
}

/// Animation settings for petalTongue graph rendering.
#[expect(clippy::struct_excessive_bools, reason = "matches petalTongue schema")]
#[derive(Debug, Clone, Serialize)]
pub struct Animations {
    /// Master switch for animated graph rendering.
    pub enabled: bool,
    /// Subtle scale or glow pulsing on nodes when enabled.
    pub breathing_nodes: bool,
    /// Animated pulses along edges to suggest data flow.
    pub connection_pulses: bool,
    /// Eased transitions when layout or selection changes.
    pub smooth_transitions: bool,
    /// Extra celebratory visuals for milestones or successes.
    pub celebration_effects: bool,
}

/// Performance constraints for petalTongue rendering.
#[derive(Debug, Clone, Serialize)]
pub struct Performance {
    /// Desired frame rate cap for the visualization loop.
    pub target_fps: u32,
    /// Whether vertical sync is requested from the compositor.
    pub vsync: bool,
    /// Whether GPU-backed rendering paths should be preferred when available.
    pub hardware_acceleration: bool,
}

/// Neural API toggle for petalTongue (enables natural-language graph queries).
#[derive(Debug, Clone, Serialize)]
pub struct NeuralApi {
    /// Enables natural-language graph query UI when true.
    pub enabled: bool,
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions use expect for clarity")]
mod tests {
    use super::*;
    use crate::tolerances;

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
        assert!((pos.x - 10.5).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert!((pos.y - 20.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
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
