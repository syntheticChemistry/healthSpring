// SPDX-License-Identifier: AGPL-3.0-or-later

//! Connection handling — newline-delimited JSON-RPC over Unix and TCP streams.
//!
//! Each accepted connection is served in a dedicated thread. Requests
//! are parsed, dispatched, and responses written back line-by-line.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use healthspring_barracuda::ipc::rpc;

use super::routing::{PrimalState, dispatch_request};

const READ_TIMEOUT_SECS: u64 = 60;
const WRITE_TIMEOUT_SECS: u64 = 10;

/// Handles a Unix domain socket client connection.
#[expect(
    clippy::needless_pass_by_value,
    reason = "stream is consumed by BufReader and shared via reference"
)]
pub fn handle_unix_connection(stream: UnixStream, state: &PrimalState) {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT_SECS)))
        .ok();

    let reader = BufReader::new(&stream);
    let writer: &UnixStream = &stream;
    handle_lines(reader, writer, state);
}

/// Handles a TCP client connection (newline JSON-RPC per Deployment
/// Validation Standard).
#[expect(
    clippy::needless_pass_by_value,
    reason = "stream is consumed by BufReader and shared via reference"
)]
pub fn handle_tcp_connection(stream: TcpStream, state: &PrimalState) {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT_SECS)))
        .ok();

    let reader = BufReader::new(&stream);
    let writer: &TcpStream = &stream;
    handle_lines(reader, writer, state);
}

fn handle_lines<R: Read, W: Write>(reader: BufReader<R>, mut writer: W, state: &PrimalState) {
    for line_result in reader.lines() {
        let Ok(line) = line_result else { break };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = rpc::error(
                    &serde_json::Value::Null,
                    rpc::PARSE_ERROR,
                    &format!("Parse error: {e}"),
                );
                let _ = writeln!(writer, "{resp}");
                let _ = writer.flush();
                continue;
            }
        };

        let id = parsed.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = parsed.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = parsed
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        state
            .requests_served
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result = dispatch_request(method, &params, state);
        let response = rpc::success(&id, &result);

        let _ = writeln!(writer, "{response}");
        let _ = writer.flush();
    }
}
