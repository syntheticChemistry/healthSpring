// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed client for inference primal protocol.
//!
//! Supports both `model.*` and `inference.*` method namespaces for
//! compatibility with Squirrel / neuralSpring (proto-nucleate uses
//! `inference.*`; legacy callers use `model.*`).
//!
//! Discovery is capability-based — no hardcoded primal names.

use super::rpc;
use super::socket;

/// Error from inference dispatch operations.
#[derive(Debug)]
pub enum InferenceError {
    /// No inference primal socket was discovered.
    NoInferencePrimal,
    /// RPC send failed (transport/codec).
    Send(rpc::SendError),
}

impl core::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoInferencePrimal => write!(f, "no inference primal discovered"),
            Self::Send(e) => write!(f, "inference dispatch send: {e}"),
        }
    }
}

/// Discover an inference primal by probing both `model.*` and `inference.*`
/// capability namespaces.
fn discover_inference_socket() -> Option<std::path::PathBuf> {
    socket::discover_inference_primal()
        .or_else(|| socket::discover_by_capability_public("inference"))
}

/// Route a model inference request to the discovered inference primal.
///
/// # Errors
///
/// Returns [`InferenceError`] if no inference primal is available or the
/// RPC call fails.
pub fn infer(params: &serde_json::Value) -> Result<serde_json::Value, InferenceError> {
    let inference_socket = discover_inference_socket().ok_or(InferenceError::NoInferencePrimal)?;
    rpc::resilient_send(&inference_socket, "inference.complete", params)
        .or_else(|_| rpc::resilient_send(&inference_socket, "model.infer", params))
        .map_err(InferenceError::Send)
}

/// Route an inference operation by name to the discovered inference primal.
///
/// Tries the `inference.{operation}` namespace first (proto-nucleate
/// alignment), then falls back to `model.{operation}`.
///
/// # Errors
///
/// Returns [`InferenceError`] if no inference primal is available or the
/// RPC call fails.
pub fn route(
    operation: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, InferenceError> {
    let inference_socket = discover_inference_socket().ok_or(InferenceError::NoInferencePrimal)?;
    let inference_method = format!("inference.{operation}");
    let model_method = format!("model.{operation}");
    rpc::resilient_send(&inference_socket, &inference_method, params)
        .or_else(|e| {
            if e.is_method_not_found() {
                rpc::resilient_send(&inference_socket, &model_method, params)
            } else {
                Err(e)
            }
        })
        .map_err(InferenceError::Send)
}

/// Query inference capabilities from the discovered inference primal.
///
/// Returns capabilities matching both `model.*` and `inference.*` namespaces.
#[must_use]
pub fn capabilities() -> Vec<String> {
    let Some(inference_socket) = discover_inference_socket() else {
        return Vec::new();
    };
    let Ok(result) = rpc::try_send(&inference_socket, "capability.list", &serde_json::json!({}))
    else {
        return Vec::new();
    };
    socket::extract_capability_strings(&result)
        .into_iter()
        .filter(|s| s.starts_with("model.") || s.starts_with("inference."))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_fails_without_inference_primal() {
        let result = infer(&serde_json::json!({"input": [1.0, 2.0]}));
        assert!(matches!(result, Err(InferenceError::NoInferencePrimal)));
    }

    #[test]
    fn capabilities_returns_empty_without_primal() {
        assert!(capabilities().is_empty());
    }

    #[test]
    fn inference_error_display() {
        let err = InferenceError::NoInferencePrimal;
        assert_eq!(err.to_string(), "no inference primal discovered");
    }
}
