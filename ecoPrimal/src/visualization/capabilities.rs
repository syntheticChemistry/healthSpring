// SPDX-License-Identifier: AGPL-3.0-or-later
//! Songbird capability announcement for healthSpring.
//!
//! Announces health-domain capabilities so that petalTongue (and other
//! primals) can discover healthSpring data sources at runtime without
//! hardcoded names or paths.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use crate::tolerances::IPC_RESPONSE_BUF;

/// Well-known Songbird socket paths (relative to `XDG_RUNTIME_DIR`).
///
/// Intentional exception to capability-based discovery: Songbird *is* the
/// discovery service, so we must know where to find it by convention.
/// Environment overrides (`HEALTHSPRING_SONGBIRD_SOCKET`, `SONGBIRD_SOCKET`)
/// are checked first for testing and non-standard deployments.
const SONGBIRD_PATHS: &[&str] = &["biomeos/songbird.sock", "songbird/songbird.sock"];

/// All capabilities that healthSpring can announce.
pub const CAPABILITIES: &[&str] = &[
    "health.metrics",
    "health.vitals",
    "health.pkpd",
    "health.pkpd.hill",
    "health.pkpd.population",
    "health.pkpd.pbpk",
    "health.microbiome",
    "health.microbiome.diversity",
    "health.microbiome.resistance",
    "health.biosignal",
    "health.biosignal.ecg",
    "health.biosignal.hrv",
    "health.biosignal.spo2",
    "health.endocrine",
    "health.endocrine.testosterone",
    "health.endocrine.trt",
    "health.diagnostic",
    "health.diagnostic.population",
];

/// Supported transport protocols for capability announcement.
pub const TRANSPORTS: &[&str] = &["jsonrpc"];

/// Error type for capability operations.
#[derive(Debug)]
pub enum CapabilityError {
    /// Songbird socket not found.
    NotFound(String),
    /// Connection to Songbird failed.
    ConnectionFailed(std::io::Error),
    /// JSON serialization error.
    SerializationError(String),
    /// Songbird returned an error response.
    RpcError { code: i64, message: String },
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Songbird not found: {msg}"),
            Self::ConnectionFailed(e) => write!(f, "Songbird connection failed: {e}"),
            Self::SerializationError(e) => write!(f, "serialization error: {e}"),
            Self::RpcError { code, message } => {
                write!(f, "Songbird RPC error {code}: {message}")
            }
        }
    }
}

impl std::error::Error for CapabilityError {}

type CapResult<T> = Result<T, CapabilityError>;

/// Announce healthSpring capabilities to Songbird.
///
/// This tells the local Songbird discovery service that healthSpring
/// offers `health.*` capabilities over JSON-RPC. petalTongue (and other
/// primals) can then query Songbird to find our socket.
///
/// # Errors
///
/// Returns `CapabilityError::NotFound` if no Songbird socket is found,
/// `CapabilityError::ConnectionFailed` if the connection fails, or
/// `CapabilityError::RpcError` if Songbird rejects the announcement.
pub fn announce(capabilities: &[&str], socket_path: &str) -> CapResult<()> {
    let songbird = discover_songbird()?;
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discovery.announce",
        "params": {
            "capabilities": capabilities,
            "transport": TRANSPORTS,
            "socket": socket_path,
        },
        "id": 1,
    });

    send_songbird_rpc(&songbird, &request)
}

/// Announce all healthSpring capabilities with default settings.
///
/// # Errors
///
/// Returns `CapabilityError` if announcement fails.
pub fn announce_all(socket_path: &str) -> CapResult<()> {
    announce(CAPABILITIES, socket_path)
}

/// Query Songbird for a capability and return the provider's socket path.
///
/// # Errors
///
/// Returns `CapabilityError::NotFound` if no Songbird socket is found or
/// no provider offers the requested capability.
pub fn query(capability: &str) -> CapResult<String> {
    let songbird = discover_songbird()?;
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discovery.query",
        "params": {
            "capability": capability,
        },
        "id": 1,
    });

    let payload = serde_json::to_vec(&request)
        .map_err(|e| CapabilityError::SerializationError(e.to_string()))?;

    let mut stream = UnixStream::connect(&songbird).map_err(CapabilityError::ConnectionFailed)?;
    stream
        .write_all(&payload)
        .map_err(CapabilityError::ConnectionFailed)?;
    stream
        .write_all(b"\n")
        .map_err(CapabilityError::ConnectionFailed)?;
    stream.flush().map_err(CapabilityError::ConnectionFailed)?;

    let mut buf = vec![0u8; IPC_RESPONSE_BUF];
    let n = stream
        .read(&mut buf)
        .map_err(CapabilityError::ConnectionFailed)?;

    let response: serde_json::Value = serde_json::from_slice(&buf[..n])
        .map_err(|e| CapabilityError::SerializationError(e.to_string()))?;

    if let Some(error) = response.get("error") {
        let (code, message) = crate::ipc::rpc::extract_rpc_error(error);
        return Err(CapabilityError::RpcError { code, message });
    }

    response
        .get("result")
        .and_then(|r| r.get("socket"))
        .and_then(|s| s.as_str())
        .map(String::from)
        .ok_or_else(|| CapabilityError::NotFound(format!("no provider for {capability}")))
}

/// Build the capability announcement payload without sending.
/// Useful for testing and inspection.
#[must_use]
pub fn build_announce_payload(capabilities: &[&str], socket_path: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discovery.announce",
        "params": {
            "capabilities": capabilities,
            "transport": TRANSPORTS,
            "socket": socket_path,
        },
        "id": 1,
    })
}

/// Build the capability query payload without sending.
#[must_use]
pub fn build_query_payload(capability: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discovery.query",
        "params": {
            "capability": capability,
        },
        "id": 1,
    })
}

fn discover_songbird() -> CapResult<PathBuf> {
    if let Ok(path) = std::env::var("HEALTHSPRING_SONGBIRD_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }
    if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let runtime = PathBuf::from(runtime);
        if let Some(p) = SONGBIRD_PATHS.iter().find_map(|rel| {
            let path = runtime.join(rel);
            path.exists().then_some(path)
        }) {
            return Ok(p);
        }
    }

    Err(CapabilityError::NotFound("no Songbird socket found".into()))
}

fn send_songbird_rpc(socket_path: &PathBuf, request: &serde_json::Value) -> CapResult<()> {
    let payload = serde_json::to_vec(request)
        .map_err(|e| CapabilityError::SerializationError(e.to_string()))?;

    let mut stream = UnixStream::connect(socket_path).map_err(CapabilityError::ConnectionFailed)?;
    stream
        .write_all(&payload)
        .map_err(CapabilityError::ConnectionFailed)?;
    stream
        .write_all(b"\n")
        .map_err(CapabilityError::ConnectionFailed)?;
    stream.flush().map_err(CapabilityError::ConnectionFailed)?;

    let mut buf = vec![0u8; IPC_RESPONSE_BUF];
    let n = stream
        .read(&mut buf)
        .map_err(CapabilityError::ConnectionFailed)?;

    let response: serde_json::Value = serde_json::from_slice(&buf[..n])
        .map_err(|e| CapabilityError::SerializationError(e.to_string()))?;

    if let Some(error) = response.get("error") {
        let (code, message) = crate::ipc::rpc::extract_rpc_error(error);
        return Err(CapabilityError::RpcError { code, message });
    }

    Ok(())
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn capabilities_not_empty() {
        assert!(!CAPABILITIES.is_empty());
    }

    #[test]
    fn capabilities_all_start_with_health() {
        for cap in CAPABILITIES {
            assert!(
                cap.starts_with("health."),
                "capability {cap} should start with 'health.'"
            );
        }
    }

    #[test]
    fn transports_includes_jsonrpc() {
        assert!(TRANSPORTS.contains(&"jsonrpc"));
    }

    #[test]
    fn build_announce_payload_structure() {
        let payload = build_announce_payload(
            &["health.metrics", "health.vitals"],
            "/tmp/healthspring.sock",
        );
        assert_eq!(payload["jsonrpc"], "2.0");
        assert_eq!(payload["method"], "discovery.announce");
        let caps = payload["params"]["capabilities"]
            .as_array()
            .expect("capabilities array");
        assert_eq!(caps.len(), 2);
        assert_eq!(payload["params"]["socket"], "/tmp/healthspring.sock");
        let transport = payload["params"]["transport"]
            .as_array()
            .expect("transport array");
        assert!(transport.iter().any(|t| t == "jsonrpc"));
    }

    #[test]
    fn build_query_payload_structure() {
        let payload = build_query_payload("health.pkpd");
        assert_eq!(payload["jsonrpc"], "2.0");
        assert_eq!(payload["method"], "discovery.query");
        assert_eq!(payload["params"]["capability"], "health.pkpd");
    }

    #[test]
    fn capability_error_display() {
        let e = CapabilityError::NotFound("no songbird".into());
        assert!(format!("{e}").contains("Songbird not found"));

        let e = CapabilityError::RpcError {
            code: -32600,
            message: "invalid".into(),
        };
        assert!(format!("{e}").contains("-32600"));
    }

    #[test]
    fn capability_error_is_error_trait() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<CapabilityError>();
    }

    #[test]
    fn discover_songbird_not_found() {
        let result = discover_songbird();
        if result.is_ok() {
            return;
        }
        assert!(matches!(result, Err(CapabilityError::NotFound(_))));
    }

    #[test]
    fn announce_fails_without_songbird() {
        let result = announce(&["health.metrics"], "/tmp/test.sock");
        if result.is_ok() {
            return;
        }
        assert!(matches!(
            result,
            Err(CapabilityError::NotFound(_) | CapabilityError::ConnectionFailed(_))
        ));
    }

    #[test]
    fn query_fails_without_songbird() {
        let result = query("health.metrics");
        if result.is_ok() {
            return;
        }
        assert!(matches!(
            result,
            Err(CapabilityError::NotFound(_) | CapabilityError::ConnectionFailed(_))
        ));
    }

    #[test]
    fn all_capabilities_list_complete() {
        assert!(CAPABILITIES.contains(&"health.metrics"));
        assert!(CAPABILITIES.contains(&"health.pkpd"));
        assert!(CAPABILITIES.contains(&"health.microbiome"));
        assert!(CAPABILITIES.contains(&"health.biosignal"));
        assert!(CAPABILITIES.contains(&"health.endocrine"));
        assert!(CAPABILITIES.contains(&"health.diagnostic"));
    }
}
