// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 envelope helpers.

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

/// Send a JSON-RPC request to a Unix socket and return the parsed result.
///
/// Returns `None` if the socket is unreachable or the response is malformed.
#[must_use]
pub fn send(
    socket_path: &std::path::Path,
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });

    let payload = serde_json::to_string(&request).ok()?;
    stream.write_all(payload.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;
    stream.flush().ok()?;
    stream.shutdown(std::net::Shutdown::Write).ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;

    let parsed: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    parsed.get("result").cloned()
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
}
