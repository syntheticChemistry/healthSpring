// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 protocol-level helpers.
//!
//! Provides [`DispatchOutcome`] for classifying RPC responses as success,
//! protocol error, or application error — absorbed from groundSpring V112
//! and biomeOS v2.46.
//!
//! Also provides generic primal discovery helpers that replace per-primal
//! env-var functions (absorbed from wetSpring V125 / groundSpring V112).

/// Structured outcome of a JSON-RPC dispatch.
///
/// Distinguishes protocol-level errors (invalid request, method not found,
/// timeout) from application-level errors (the target primal processed the
/// request but returned a domain error). Callers can retry protocol errors
/// but should propagate application errors.
///
/// Pattern source: biomeOS v2.46 / groundSpring V112.
#[derive(Debug, Clone)]
pub enum DispatchOutcome {
    /// The RPC succeeded and returned a result payload.
    Ok(serde_json::Value),
    /// JSON-RPC protocol-layer failure (parse, invalid request, method not found, etc.).
    ProtocolError {
        /// JSON-RPC error code.
        code: i64,
        /// Error message from the server.
        message: String,
    },
    /// Application-layer failure after the request was accepted.
    ApplicationError {
        /// Application-specific error code.
        code: i64,
        /// Error message from the domain handler.
        message: String,
    },
}

const JSONRPC_PROTOCOL_ERROR_MIN: i64 = -32700;
const JSONRPC_PROTOCOL_ERROR_MAX: i64 = -32600;

impl DispatchOutcome {
    /// Whether this is a `-32601 Method not found` protocol error.
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::ProtocolError { code: -32601, .. })
    }

    /// Whether this is a protocol error (likely transient / retryable).
    #[must_use]
    pub const fn is_protocol_error(&self) -> bool {
        matches!(self, Self::ProtocolError { .. })
    }
}

/// Parse a JSON-RPC 2.0 response string into a [`DispatchOutcome`].
///
/// # Errors
///
/// Returns `Err` if the response is not valid JSON.
pub fn parse_rpc_response(response: &str) -> Result<DispatchOutcome, serde_json::Error> {
    let parsed: serde_json::Value = serde_json::from_str(response)?;
    Ok(classify_response(&parsed))
}

/// Classify a parsed JSON-RPC response value.
#[must_use]
pub fn classify_response(response: &serde_json::Value) -> DispatchOutcome {
    if let Some(error) = response.get("error") {
        let code = error
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-32000);
        let message = error
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown RPC error")
            .to_owned();

        return if (JSONRPC_PROTOCOL_ERROR_MIN..=JSONRPC_PROTOCOL_ERROR_MAX).contains(&code) {
            DispatchOutcome::ProtocolError { code, message }
        } else {
            DispatchOutcome::ApplicationError { code, message }
        };
    }

    response.get("result").map_or_else(
        || DispatchOutcome::ApplicationError {
            code: -32000,
            message: "response missing both result and error".into(),
        },
        |result| DispatchOutcome::Ok(result.clone()),
    )
}

// ── Generic primal discovery helpers ────────────────────────────────────
// Absorbed from wetSpring V125 / groundSpring V112 / sweetGrass v0.7.19.
// Replace per-primal env-var functions with a single generic pattern.

/// Resolve a primal socket path from a named environment variable.
///
/// Returns `Some(path)` if the env var is set and the file exists.
#[must_use]
pub fn socket_from_env(env_var: &str) -> Option<std::path::PathBuf> {
    std::env::var(env_var)
        .ok()
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists())
}

/// Resolve a primal socket from env var override, then fall back to
/// scanning the socket directory for a matching name prefix.
///
/// This is the generic replacement for per-primal `discover_*` functions.
#[must_use]
pub fn discover_primal_socket(env_override: &str, name_prefix: &str) -> Option<std::path::PathBuf> {
    if let Some(path) = socket_from_env(env_override) {
        return Some(path);
    }

    let dir = super::socket::resolve_socket_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(name_prefix) && name_str.ends_with(".sock") {
            return Some(entry.path());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_success_response() {
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": {"ok": true}, "id": 1});
        let outcome = classify_response(&resp);
        assert!(matches!(outcome, DispatchOutcome::Ok(_)));
    }

    #[test]
    fn classify_protocol_error() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "method not found"},
            "id": 1
        });
        let outcome = classify_response(&resp);
        assert!(outcome.is_protocol_error());
        assert!(outcome.is_method_not_found());
    }

    #[test]
    fn classify_application_error() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -1, "message": "patient not found"},
            "id": 1
        });
        let outcome = classify_response(&resp);
        assert!(!outcome.is_protocol_error());
        assert!(matches!(outcome, DispatchOutcome::ApplicationError { .. }));
    }

    #[test]
    fn classify_missing_result_and_error() {
        let resp = serde_json::json!({"jsonrpc": "2.0", "id": 1});
        let outcome = classify_response(&resp);
        assert!(matches!(outcome, DispatchOutcome::ApplicationError { .. }));
    }

    #[test]
    fn parse_rpc_response_valid() {
        let json = r#"{"jsonrpc":"2.0","result":42,"id":1}"#;
        let outcome = parse_rpc_response(json);
        assert!(outcome.is_ok());
        assert!(matches!(
            outcome.unwrap_or(DispatchOutcome::ApplicationError {
                code: 0,
                message: String::new()
            }),
            DispatchOutcome::Ok(_)
        ));
    }

    #[test]
    fn parse_rpc_response_invalid_json() {
        let outcome = parse_rpc_response("not json");
        assert!(outcome.is_err());
    }

    #[test]
    fn socket_from_env_returns_none_for_unset() {
        assert!(socket_from_env("HEALTHSPRING_NONEXISTENT_TEST_VAR_XYZ").is_none());
    }

    #[test]
    fn discover_primal_socket_returns_none_for_missing() {
        assert!(
            discover_primal_socket(
                "HEALTHSPRING_NONEXISTENT_TEST_VAR_XYZ",
                "nonexistent_primal"
            )
            .is_none()
        );
    }
}
