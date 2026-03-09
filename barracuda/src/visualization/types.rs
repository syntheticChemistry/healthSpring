// SPDX-License-Identifier: AGPL-3.0-or-later
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
    pub position: Position,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data_channels: Vec<DataChannel>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub clinical_ranges: Vec<ClinicalRange>,
}

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
}

#[derive(Debug, Clone, Serialize)]
pub struct Ecosystem {
    pub primals: Vec<ScenarioNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensoryConfig {
    pub required_capabilities: CapReqs,
    pub optional_capabilities: CapReqs,
    pub complexity_hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapReqs {
    pub outputs: Vec<String>,
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiConfig {
    pub theme: String,
    pub animations: Animations,
    pub performance: Performance,
}

#[expect(clippy::struct_excessive_bools, reason = "matches petalTongue schema")]
#[derive(Debug, Clone, Serialize)]
pub struct Animations {
    pub enabled: bool,
    pub breathing_nodes: bool,
    pub connection_pulses: bool,
    pub smooth_transitions: bool,
    pub celebration_effects: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Performance {
    pub target_fps: u32,
    pub vsync: bool,
    pub hardware_acceleration: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NeuralApi {
    pub enabled: bool,
}
