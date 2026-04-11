// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed IPC clients for primal and inference coordination.
//!
//! Follows primalSpring's `PrimalClient` / `InferenceClient` pattern:
//! typed wrappers over JSON-RPC with method fallback chains and structured
//! error handling. These replace raw `rpc::send` / `rpc::try_send` calls
//! with a discoverable, resilient API.

use std::path::{Path, PathBuf};

use super::error::IpcError;
use super::rpc;

/// Typed JSON-RPC client for peer primal communication.
///
/// Wraps `rpc::try_send` with health probe fallback chains and method
/// normalization per primalSpring's `PrimalClient` pattern.
pub struct PrimalClient {
    socket: PathBuf,
    name: String,
}

impl PrimalClient {
    /// Connect to a primal at the given socket path.
    #[must_use]
    pub fn new(socket: PathBuf, name: impl Into<String>) -> Self {
        Self {
            socket,
            name: name.into(),
        }
    }

    /// The logical primal name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The socket path.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Send a JSON-RPC call with retry (resilient default).
    ///
    /// Uses exponential backoff for transient failures. For single-attempt
    /// calls, use [`try_call`](Self::try_call).
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if all retry attempts are exhausted or a
    /// non-retriable error is encountered.
    pub fn call(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, IpcError> {
        rpc::resilient_send(&self.socket, method, params)
    }

    /// Send a JSON-RPC call without retry (single attempt).
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the socket connection or JSON-RPC exchange fails.
    pub fn try_call(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, IpcError> {
        rpc::try_send(&self.socket, method, params)
    }

    /// Health liveness probe with fallback chain:
    /// `health.liveness` -> `health.check` -> `health` -> `{name}.health`
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if all probe methods fail at the transport level.
    pub fn health_liveness(&self) -> Result<serde_json::Value, IpcError> {
        let methods = [
            "health.liveness",
            "health.check",
            "health",
            &format!("{}.health", self.name),
        ];
        for method in &methods {
            match self.call(method, &serde_json::json!({})) {
                Ok(result) => {
                    if !is_method_not_found(&result) {
                        return Ok(result);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        self.call("health.liveness", &serde_json::json!({}))
    }

    /// Health readiness probe with fallback.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the readiness probe fails at the transport level.
    pub fn health_readiness(&self) -> Result<serde_json::Value, IpcError> {
        let result = self.call("health.readiness", &serde_json::json!({}));
        if let Ok(ref v) = result {
            if is_method_not_found(v) {
                return self.call("health.check", &serde_json::json!({}));
            }
        }
        result
    }

    /// List capabilities with fallback chain:
    /// `capabilities.list` -> `capability.list` -> `primal.capabilities`
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if all capability list methods fail at the transport level.
    pub fn capabilities(&self) -> Result<serde_json::Value, IpcError> {
        let methods = [
            "capabilities.list",
            "capability.list",
            "primal.capabilities",
        ];
        for method in &methods {
            match self.call(method, &serde_json::json!({})) {
                Ok(result) => {
                    if !is_method_not_found(&result) {
                        return Ok(result);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        self.call("capabilities.list", &serde_json::json!({}))
    }
}

/// Typed inference client for Squirrel / neuralSpring coordination.
///
/// Follows primalSpring's `InferenceClient` pattern with discovery and
/// method routing for `inference.*` capabilities.
pub struct InferenceClient {
    inner: PrimalClient,
}

impl InferenceClient {
    /// Create an inference client from a discovered socket.
    #[must_use]
    pub fn new(socket: PathBuf) -> Self {
        Self {
            inner: PrimalClient::new(socket, "inference"),
        }
    }

    /// Discover and connect to an inference primal.
    #[must_use]
    pub fn discover() -> Option<Self> {
        let result = super::discover::discover_inference();
        result.socket.map(Self::new)
    }

    /// The socket path.
    #[must_use]
    pub fn socket(&self) -> &Path {
        self.inner.socket()
    }

    /// Complete a prompt via `inference.complete`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the inference call fails.
    pub fn complete(&self, params: &serde_json::Value) -> Result<serde_json::Value, IpcError> {
        self.inner.call("inference.complete", params)
    }

    /// Generate embeddings via `inference.embed`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the embedding call fails.
    pub fn embed(&self, params: &serde_json::Value) -> Result<serde_json::Value, IpcError> {
        self.inner.call("inference.embed", params)
    }

    /// List available models via `inference.models`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the models listing call fails.
    pub fn models(&self) -> Result<serde_json::Value, IpcError> {
        self.inner.call("inference.models", &serde_json::json!({}))
    }
}

fn is_method_not_found(result: &serde_json::Value) -> bool {
    result
        .get("error")
        .and_then(|e| e.as_str())
        .is_some_and(|s| s == "method_not_found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primal_client_name_and_socket() {
        let client = PrimalClient::new(PathBuf::from("/tmp/test.sock"), "test_primal");
        assert_eq!(client.name(), "test_primal");
        assert_eq!(client.socket(), Path::new("/tmp/test.sock"));
    }

    #[test]
    fn inference_client_discover_returns_option() {
        let _client = InferenceClient::discover();
    }

    #[test]
    fn is_method_not_found_detects_error() {
        let error = serde_json::json!({"error": "method_not_found"});
        assert!(is_method_not_found(&error));

        let ok = serde_json::json!({"status": "healthy"});
        assert!(!is_method_not_found(&ok));
    }
}
