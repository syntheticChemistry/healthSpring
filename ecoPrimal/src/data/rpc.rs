// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 over Unix domain socket — `NestGate` and `biomeOS` protocol.
//!
//! Newline-delimited JSON-RPC matching wetSpring's transport exactly.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// RPC transport errors.
#[derive(Debug)]
pub enum RpcError {
    /// Socket connection failed.
    Connection(std::io::Error),

    /// JSON serialization/deserialization error.
    Json(serde_json::Error),

    /// Server returned an error response.
    Server {
        /// JSON-RPC error code.
        code: i64,
        /// Error message from server.
        message: String,
    },

    /// Unexpected response shape.
    Unexpected(String),
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(e) => write!(f, "connection failed: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Server { code, message } => write!(f, "server error {code}: {message}"),
            Self::Unexpected(msg) => write!(f, "unexpected response: {msg}"),
        }
    }
}

impl std::error::Error for RpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RpcError {
    fn from(err: std::io::Error) -> Self {
        Self::Connection(err)
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Send a JSON-RPC 2.0 request and receive the result.
///
/// # Errors
///
/// Returns `RpcError` on connection failure, protocol errors, or server errors.
pub fn rpc_call(
    socket_path: &Path,
    method: &str,
    params: &serde_json::Value,
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
        let (code, message) = crate::ipc::rpc::extract_rpc_error(error);
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
            &serde_json::json!({}),
        );
        assert!(result.is_err());
    }
}
