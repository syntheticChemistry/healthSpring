// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC 2.0 over Unix domain socket — `NestGate` protocol.
//!
//! Newline-delimited JSON-RPC matching wetSpring's transport exactly.
//! Only compiled when the `nestgate` feature is enabled.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// RPC transport errors.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Socket connection failed.
    #[error("connection failed: {0}")]
    Connection(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Server returned an error response.
    #[error("server error {code}: {message}")]
    Server {
        /// JSON-RPC error code.
        code: i64,
        /// Error message from server.
        message: String,
    },

    /// Unexpected response shape.
    #[error("unexpected response: {0}")]
    Unexpected(String),
}

/// Send a JSON-RPC 2.0 request and receive the result.
///
/// # Errors
///
/// Returns `RpcError` on connection failure, protocol errors, or server errors.
pub fn rpc_call(
    socket_path: &Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, RpcError> {
    let mut stream = UnixStream::connect(socket_path)?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });

    let mut payload = serde_json::to_string(&request)?;
    payload.push('\n');
    stream.write_all(payload.as_bytes())?;
    stream.flush()?;

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let resp: serde_json::Value = serde_json::from_str(line.trim())?;

    if let Some(error) = resp.get("error") {
        let code = error.get("code").and_then(serde_json::Value::as_i64).unwrap_or(-1);
        let message = error
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        return Err(RpcError::Server { code, message });
    }

    resp.get("result")
        .cloned()
        .ok_or_else(|| RpcError::Unexpected("missing 'result' field".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_error_display() {
        let e = RpcError::Server {
            code: -32601,
            message: "method not found".into(),
        };
        assert!(e.to_string().contains("-32601"));
        assert!(e.to_string().contains("method not found"));
    }

    #[test]
    fn rpc_call_nonexistent_socket() {
        let result = rpc_call(
            Path::new("/tmp/nonexistent_healthspring_test.sock"),
            "test.ping",
            serde_json::json!({}),
        );
        assert!(result.is_err());
    }
}
