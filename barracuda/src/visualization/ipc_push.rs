// SPDX-License-Identifier: AGPL-3.0-or-later
//! Push visualization data to petalTongue via JSON-RPC IPC.
//!
//! Springs discover petalTongue at runtime and push DataChannel payloads
//! without compile-time coupling. Uses the visualization.render and
//! visualization.render.stream JSON-RPC methods.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use super::types::{ClinicalRange, DataChannel, HealthScenario};

/// Client for pushing visualization data to petalTongue
pub struct PetalTonguePushClient {
    socket_path: PathBuf,
}

/// Result type for push operations
pub type PushResult<T> = Result<T, PushError>;

/// Error type for push operations
#[derive(Debug)]
pub enum PushError {
    /// petalTongue socket not found
    NotFound(String),
    /// Connection failed
    ConnectionFailed(std::io::Error),
    /// JSON serialization error
    SerializationError(String),
    /// RPC error response
    RpcError { code: i64, message: String },
}

impl std::fmt::Display for PushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "petalTongue not found: {msg}"),
            Self::ConnectionFailed(e) => write!(f, "connection failed: {e}"),
            Self::SerializationError(e) => write!(f, "serialization error: {e}"),
            Self::RpcError { code, message } => write!(f, "RPC error {code}: {message}"),
        }
    }
}

impl std::error::Error for PushError {}

impl PetalTonguePushClient {
    /// Discover petalTongue socket. Checks:
    /// 1. PETALTONGUE_SOCKET env var
    /// 2. XDG_RUNTIME_DIR/petaltongue/*.sock
    /// 3. /tmp/petaltongue-*.sock
    pub fn discover() -> PushResult<Self> {
        // Try env var first
        if let Ok(path) = std::env::var("PETALTONGUE_SOCKET") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(Self { socket_path: path });
            }
        }
        // Try XDG_RUNTIME_DIR
        if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let dir = PathBuf::from(runtime).join("petaltongue");
            if dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |e| e == "sock") {
                            return Ok(Self { socket_path: p });
                        }
                    }
                }
            }
        }
        // Try /tmp
        if let Ok(entries) = std::fs::read_dir("/tmp") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("petaltongue") && name.ends_with(".sock") {
                    return Ok(Self { socket_path: entry.path() });
                }
            }
        }
        Err(PushError::NotFound("no petalTongue socket found".into()))
    }

    /// Create client with explicit socket path
    #[must_use]
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Push a full visualization render request
    pub fn push_render(
        &self,
        session_id: &str,
        title: &str,
        scenario: &HealthScenario,
    ) -> PushResult<()> {
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

        let params = serde_json::json!({
            "session_id": session_id,
            "title": title,
            "bindings": bindings,
            "thresholds": thresholds,
            "domain": "health",
        });

        self.send_rpc("visualization.render", params)?;
        Ok(())
    }

    /// Push a stream update (append data points to a TimeSeries)
    pub fn push_append(
        &self,
        session_id: &str,
        binding_id: &str,
        x_values: &[f64],
        y_values: &[f64],
    ) -> PushResult<()> {
        let params = serde_json::json!({
            "session_id": session_id,
            "binding_id": binding_id,
            "operation": {
                "type": "append",
                "x_values": x_values,
                "y_values": y_values,
            },
        });
        self.send_rpc("visualization.render.stream", params)?;
        Ok(())
    }

    /// Push a gauge value update
    pub fn push_gauge_update(
        &self,
        session_id: &str,
        binding_id: &str,
        value: f64,
    ) -> PushResult<()> {
        let params = serde_json::json!({
            "session_id": session_id,
            "binding_id": binding_id,
            "operation": {
                "type": "set_value",
                "value": value,
            },
        });
        self.send_rpc("visualization.render.stream", params)?;
        Ok(())
    }

    fn send_rpc(&self, method: &str, params: serde_json::Value) -> PushResult<serde_json::Value> {
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
        stream.write_all(&payload).map_err(PushError::ConnectionFailed)?;
        stream.write_all(b"\n").map_err(PushError::ConnectionFailed)?;
        stream.flush().map_err(PushError::ConnectionFailed)?;

        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).map_err(PushError::ConnectionFailed)?;

        let response: serde_json::Value =
            serde_json::from_slice(&buf[..n]).map_err(|e| PushError::SerializationError(e.to_string()))?;

        if let Some(error) = response.get("error") {
            return Err(PushError::RpcError {
                code: error
                    .get("code")
                    .and_then(|c| c.as_i64())
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
