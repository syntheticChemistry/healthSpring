// SPDX-License-Identifier: AGPL-3.0-or-later
//! `PetalTongue` push client: discovery, connection, and push methods.
//!
//! Springs discover petalTongue at runtime and push `DataChannel` payloads
//! without compile-time coupling.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use super::protocol::{
    build_append_params, build_gauge_params, build_render_params, build_render_with_config_params,
    build_replace_params,
};
use super::{DataChannel, PushError, PushResult};
use crate::visualization::types::HealthScenario;

const RPC_RESPONSE_BUF: usize = 65_536;

/// Client for pushing visualization data to petalTongue
pub struct PetalTonguePushClient {
    socket_path: PathBuf,
}

impl PetalTonguePushClient {
    /// Discover petalTongue socket at runtime.
    ///
    /// Resolution order (wateringHole Universal IPC v3.0):
    /// 1. `PETALTONGUE_SOCKET` env var (explicit override)
    /// 2. `$XDG_RUNTIME_DIR/biomeos/petaltongue-*.sock` (biomeOS standard)
    /// 3. `$XDG_RUNTIME_DIR/petaltongue/*.sock` (legacy)
    ///
    /// # Errors
    ///
    /// Returns `PushError::NotFound` if no petalTongue socket is found at
    /// any of the candidate paths.
    pub fn discover() -> PushResult<Self> {
        if let Ok(path) = std::env::var("PETALTONGUE_SOCKET") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(Self { socket_path: path });
            }
        }

        if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let runtime = PathBuf::from(runtime);

            // biomeOS standard path (wateringHole Universal IPC v3.0)
            let biomeos_dir = runtime.join("biomeos");
            if biomeos_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&biomeos_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.starts_with("petaltongue") && name.ends_with(".sock") {
                            return Ok(Self {
                                socket_path: entry.path(),
                            });
                        }
                    }
                }
            }

            // Legacy path
            let legacy_dir = runtime.join("petaltongue");
            if legacy_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&legacy_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().is_some_and(|e| e == "sock") {
                            return Ok(Self { socket_path: p });
                        }
                    }
                }
            }
        }

        Err(PushError::NotFound("no petalTongue socket found".into()))
    }

    /// Create client with explicit socket path
    #[must_use]
    pub const fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Socket path (for tests).
    #[cfg(test)]
    #[must_use]
    pub const fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Push a full visualization render request.
    ///
    /// # Errors
    ///
    /// Returns `PushError::ConnectionFailed` if the socket is unreachable,
    /// `PushError::SerializationError` if the payload cannot be encoded, or
    /// `PushError::RpcError` if petalTongue rejects the request.
    pub fn push_render(
        &self,
        session_id: &str,
        title: &str,
        scenario: &HealthScenario,
    ) -> PushResult<()> {
        let params = build_render_params(session_id, title, scenario);
        self.send_rpc("visualization.render", &params)?;
        Ok(())
    }

    /// Push a stream update (append data points to a `TimeSeries`).
    ///
    /// # Errors
    ///
    /// Returns `PushError::ConnectionFailed` if the socket is unreachable,
    /// `PushError::SerializationError` if the payload cannot be encoded, or
    /// `PushError::RpcError` if petalTongue rejects the request.
    pub fn push_append(
        &self,
        session_id: &str,
        binding_id: &str,
        x_values: &[f64],
        y_values: &[f64],
    ) -> PushResult<()> {
        let params = build_append_params(session_id, binding_id, x_values, y_values);
        self.send_rpc("visualization.render.stream", &params)?;
        Ok(())
    }

    /// Push a gauge value update.
    ///
    /// # Errors
    ///
    /// Returns `PushError::ConnectionFailed` if the socket is unreachable,
    /// `PushError::SerializationError` if the payload cannot be encoded, or
    /// `PushError::RpcError` if petalTongue rejects the request.
    pub fn push_gauge_update(
        &self,
        session_id: &str,
        binding_id: &str,
        value: f64,
    ) -> PushResult<()> {
        let params = build_gauge_params(session_id, binding_id, value);
        self.send_rpc("visualization.render.stream", &params)?;
        Ok(())
    }

    /// Replace an entire binding in-place.
    ///
    /// Unlike `append` (`TimeSeries` only) and `set_value` (`Gauge` only), `replace`
    /// works with any `DataChannel` type — enabling live updates to `Heatmap`,
    /// `Bar`, `Scatter3D`, `Distribution`, and `Spectrum` channels.
    ///
    /// # Errors
    ///
    /// Returns `PushError::ConnectionFailed` if the socket is unreachable,
    /// `PushError::SerializationError` if the payload cannot be encoded, or
    /// `PushError::RpcError` if petalTongue rejects the request.
    pub fn push_replace(
        &self,
        session_id: &str,
        binding_id: &str,
        binding: &DataChannel,
    ) -> PushResult<()> {
        let params = build_replace_params(session_id, binding_id, binding)?;
        self.send_rpc("visualization.render.stream", &params)?;
        Ok(())
    }

    /// Push a full visualization render with explicit domain and `UiConfig`.
    ///
    /// Use this variant when the scenario carries panel visibility, zoom, or
    /// theme settings that should override petalTongue defaults (e.g., clinical
    /// TRT scenarios that disable the system dashboard and audio panels).
    ///
    /// # Errors
    ///
    /// Returns `PushError::ConnectionFailed` if the socket is unreachable,
    /// `PushError::SerializationError` if the payload cannot be encoded, or
    /// `PushError::RpcError` if petalTongue rejects the request.
    pub fn push_render_with_config(
        &self,
        session_id: &str,
        title: &str,
        scenario: &HealthScenario,
        domain: &str,
    ) -> PushResult<()> {
        let params = build_render_with_config_params(session_id, title, scenario, domain);
        self.send_rpc("visualization.render", &params)?;
        Ok(())
    }

    /// Query petalTongue's rendering capabilities.
    ///
    /// Returns the set of supported channel types, geometry, and features
    /// so healthSpring can adapt its data output to the available renderer.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the RPC call fails.
    pub fn query_capabilities(&self) -> PushResult<serde_json::Value> {
        let params = serde_json::json!({});
        self.send_rpc("visualization.capabilities", &params)
    }

    /// Subscribe to interaction events from petalTongue.
    ///
    /// When a user selects, focuses, or filters data in petalTongue, the
    /// event will be delivered to the specified `callback_method` via
    /// JSON-RPC notification.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the subscription RPC fails.
    pub fn subscribe_interactions(
        &self,
        session_id: &str,
        events: &[&str],
        callback_method: &str,
    ) -> PushResult<serde_json::Value> {
        let params = serde_json::json!({
            "grammar_id": session_id,
            "events": events,
            "callback_method": callback_method,
        });
        self.send_rpc("visualization.interact.subscribe", &params)
    }

    fn send_rpc(&self, method: &str, params: &serde_json::Value) -> PushResult<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });

        let payload = serde_json::to_vec(&request)
            .map_err(|e| PushError::SerializationError(e.to_string()))?;

        let mut stream =
            UnixStream::connect(&self.socket_path).map_err(PushError::ConnectionFailed)?;
        stream
            .write_all(&payload)
            .map_err(PushError::ConnectionFailed)?;
        stream
            .write_all(b"\n")
            .map_err(PushError::ConnectionFailed)?;
        stream.flush().map_err(PushError::ConnectionFailed)?;

        let mut buf = vec![0u8; RPC_RESPONSE_BUF];
        let n = stream.read(&mut buf).map_err(PushError::ConnectionFailed)?;

        let response: serde_json::Value = serde_json::from_slice(&buf[..n])
            .map_err(|e| PushError::SerializationError(e.to_string()))?;

        if let Some(error) = response.get("error") {
            return Err(PushError::RpcError {
                code: error
                    .get("code")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(-1),
                message: error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            });
        }

        Ok(response)
    }
}
