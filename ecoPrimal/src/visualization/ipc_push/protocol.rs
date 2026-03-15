// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC wire protocol helpers for petalTongue push visualization.
//!
//! Builds params for `visualization.render` and `visualization.render.stream`
//! methods. Testable without socket.

use super::{DataChannel, PushError, PushResult};
use crate::visualization::types::{ClinicalRange, HealthScenario};

/// Build JSON-RPC params for visualization.render (testable without socket).
pub fn build_render_params(
    session_id: &str,
    title: &str,
    scenario: &HealthScenario,
) -> serde_json::Value {
    let bindings: Vec<&DataChannel> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|p| p.data_channels.iter())
        .collect();
    let thresholds: Vec<&ClinicalRange> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|p| p.clinical_ranges.iter())
        .collect();

    serde_json::json!({
        "session_id": session_id,
        "title": title,
        "bindings": bindings,
        "thresholds": thresholds,
        "domain": "health",
    })
}

/// Build JSON-RPC params for visualization.render.stream append (testable without socket).
pub fn build_append_params(
    session_id: &str,
    binding_id: &str,
    x_values: &[f64],
    y_values: &[f64],
) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "binding_id": binding_id,
        "operation": {
            "type": "append",
            "x_values": x_values,
            "y_values": y_values,
        },
    })
}

/// Build JSON-RPC params for visualization.render.stream gauge update (testable without socket).
pub fn build_gauge_params(session_id: &str, binding_id: &str, value: f64) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "binding_id": binding_id,
        "operation": {
            "type": "set_value",
            "value": value,
        },
    })
}

/// Build JSON-RPC params for visualization.render.stream replace (testable without socket).
///
/// The `replace` operation swaps an entire binding in-place, enabling live
/// updates to channel types that `append`/`set_value` cannot modify
/// (`Heatmap`, `Bar`, `Scatter3D`, `Distribution`, `Spectrum`).
pub fn build_replace_params(
    session_id: &str,
    binding_id: &str,
    binding: &DataChannel,
) -> PushResult<serde_json::Value> {
    let binding_json =
        serde_json::to_value(binding).map_err(|e| PushError::SerializationError(e.to_string()))?;
    Ok(serde_json::json!({
        "session_id": session_id,
        "binding_id": binding_id,
        "operation": {
            "type": "replace",
            "binding": binding_json,
        },
    }))
}

/// Build JSON-RPC params for visualization.render with full `UiConfig`.
///
/// Includes `ui_config` for panel visibility, zoom, and theme control —
/// used by clinical scenarios that override default petalTongue layout.
pub fn build_render_with_config_params(
    session_id: &str,
    title: &str,
    scenario: &HealthScenario,
    domain: &str,
) -> serde_json::Value {
    let bindings: Vec<&DataChannel> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|p| p.data_channels.iter())
        .collect();
    let thresholds: Vec<&ClinicalRange> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|p| p.clinical_ranges.iter())
        .collect();

    serde_json::json!({
        "session_id": session_id,
        "title": title,
        "bindings": bindings,
        "thresholds": thresholds,
        "domain": domain,
        "ui_config": {
            "theme": scenario.ui_config.theme,
            "show_panels": scenario.ui_config.show_panels,
            "awakening_enabled": scenario.ui_config.awakening_enabled,
            "initial_zoom": scenario.ui_config.initial_zoom,
        },
    })
}
