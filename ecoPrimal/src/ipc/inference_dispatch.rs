// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed client for inference primal `model.*` protocol.
//!
//! Discovery is capability-based — no hardcoded primal names.

use super::rpc;
use super::socket;

/// Error from inference dispatch operations.
#[derive(Debug)]
pub enum InferenceError {
    NoInferencePrimal,
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

/// Route a model inference request to the discovered inference primal.
///
/// # Errors
///
/// Returns [`InferenceError`] if no inference primal is available or the
/// RPC call fails.
pub fn infer(params: &serde_json::Value) -> Result<serde_json::Value, InferenceError> {
    let inference_socket =
        socket::discover_inference_primal().ok_or(InferenceError::NoInferencePrimal)?;
    rpc::try_send(&inference_socket, "model.infer", params).map_err(InferenceError::Send)
}

/// Route a model operation by name to the discovered inference primal.
///
/// # Errors
///
/// Returns [`InferenceError`] if no inference primal is available or the
/// RPC call fails.
pub fn route(
    operation: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, InferenceError> {
    let inference_socket =
        socket::discover_inference_primal().ok_or(InferenceError::NoInferencePrimal)?;
    let method = format!("model.{operation}");
    rpc::try_send(&inference_socket, &method, params).map_err(InferenceError::Send)
}

/// Query model capabilities from the discovered inference primal.
#[must_use]
pub fn capabilities() -> Vec<String> {
    let Some(inference_socket) = socket::discover_inference_primal() else {
        return Vec::new();
    };
    let Ok(result) = rpc::try_send(&inference_socket, "capability.list", &serde_json::json!({}))
    else {
        return Vec::new();
    };
    socket::extract_capability_strings(&result)
        .into_iter()
        .filter(|s| s.starts_with("model."))
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
