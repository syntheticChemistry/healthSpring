// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 envelope helpers.
//!
//! Provides [`send`] for fire-and-forget IPC (returns `Option`) and
//! [`try_send`] for callers that need error context.

pub const PARSE_ERROR: i64 = -32700;
pub const METHOD_NOT_FOUND: i64 = -32601;

/// Build a JSON-RPC 2.0 success response.
#[must_use]
pub fn success(id: &serde_json::Value, result: &serde_json::Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id,
    })
    .to_string()
}

/// Build a JSON-RPC 2.0 error response.
#[must_use]
pub fn error(id: &serde_json::Value, code: i64, message: &str) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": code, "message": message },
        "id": id,
    })
    .to_string()
}

/// IPC send errors with structured context.
#[derive(Debug)]
pub enum SendError {
    Connect(std::io::Error),
    Write(std::io::Error),
    Read(std::io::Error),
    InvalidJson(serde_json::Error),
    NoResult,
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(e) => write!(f, "connect: {e}"),
            Self::Write(e) => write!(f, "write: {e}"),
            Self::Read(e) => write!(f, "read: {e}"),
            Self::InvalidJson(e) => write!(f, "parse: {e}"),
            Self::NoResult => write!(f, "response missing 'result' field"),
        }
    }
}

/// Send a JSON-RPC request with full error context.
///
/// Use this when the caller needs to know *why* a request failed (logging,
/// diagnostics, retry decisions). For fire-and-forget IPC, use [`send`].
///
/// # Errors
///
/// Returns [`SendError`] if the socket connection, write, read, or JSON
/// parsing fails, or if the response lacks a `result` field.
pub fn try_send(
    socket_path: &std::path::Path,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, SendError> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let mut stream = UnixStream::connect(socket_path).map_err(SendError::Connect)?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });

    let payload = serde_json::to_string(&request).map_err(SendError::InvalidJson)?;
    stream
        .write_all(payload.as_bytes())
        .map_err(SendError::Write)?;
    stream.write_all(b"\n").map_err(SendError::Write)?;
    stream.flush().map_err(SendError::Write)?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(SendError::Write)?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(SendError::Read)?;

    let parsed: serde_json::Value =
        serde_json::from_str(line.trim()).map_err(SendError::InvalidJson)?;
    parsed.get("result").cloned().ok_or(SendError::NoResult)
}

/// Send a JSON-RPC request to a Unix socket (fire-and-forget).
///
/// Returns `None` if the socket is unreachable or the response is malformed.
/// For error context, use [`try_send`].
#[must_use]
pub fn send(
    socket_path: &std::path::Path,
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
    try_send(socket_path, method, params).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_format() {
        let resp = success(&serde_json::json!(1), &serde_json::json!({"ok": true}));
        assert!(resp.contains("\"jsonrpc\":\"2.0\""));
        assert!(resp.contains("\"result\""));
    }

    #[test]
    fn error_response_format() {
        let resp = error(&serde_json::json!(1), PARSE_ERROR, "bad json");
        assert!(resp.contains("-32700"));
        assert!(resp.contains("bad json"));
    }

    #[test]
    fn try_send_connect_fails_gracefully() {
        let path = std::path::Path::new("/tmp/nonexistent_healthspring_rpc_test.sock");
        let result = try_send(path, "health.check", &serde_json::json!({}));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SendError::Connect(_)));
    }

    #[test]
    fn send_error_display() {
        let err = SendError::NoResult;
        assert_eq!(err.to_string(), "response missing 'result' field");
    }
}
