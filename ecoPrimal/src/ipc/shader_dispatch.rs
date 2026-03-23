// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed client for shader compiler primal `shader.*` protocol.
//!
//! Discovery is capability-based — no hardcoded primal names.

use super::rpc;
use super::socket;

/// Error from shader dispatch operations.
#[derive(Debug)]
pub enum ShaderError {
    /// No shader compiler primal was discovered.
    NoShaderPrimal,
    /// RPC send failed (transport/codec).
    Send(rpc::SendError),
}

impl core::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoShaderPrimal => write!(f, "no shader compiler primal discovered"),
            Self::Send(e) => write!(f, "shader dispatch send: {e}"),
        }
    }
}

/// Compile a WGSL shader via the discovered shader compiler primal.
///
/// # Errors
///
/// Returns [`ShaderError`] if no shader primal is available or the RPC fails.
pub fn compile(params: &serde_json::Value) -> Result<serde_json::Value, ShaderError> {
    let shader_socket = socket::discover_shader_compiler().ok_or(ShaderError::NoShaderPrimal)?;
    rpc::try_send(&shader_socket, "shader.compile", params).map_err(ShaderError::Send)
}

/// Validate a WGSL shader without full compilation.
///
/// # Errors
///
/// Returns [`ShaderError`] if no shader primal is available or the RPC fails.
pub fn validate(wgsl_source: &str) -> Result<serde_json::Value, ShaderError> {
    let shader_socket = socket::discover_shader_compiler().ok_or(ShaderError::NoShaderPrimal)?;
    rpc::try_send(
        &shader_socket,
        "shader.validate",
        &serde_json::json!({ "source": wgsl_source }),
    )
    .map_err(ShaderError::Send)
}

/// Query shader capabilities from the discovered shader primal.
#[must_use]
pub fn capabilities() -> Vec<String> {
    let Some(shader_socket) = socket::discover_shader_compiler() else {
        return Vec::new();
    };
    let Ok(result) = rpc::try_send(&shader_socket, "capability.list", &serde_json::json!({}))
    else {
        return Vec::new();
    };
    socket::extract_capability_strings(&result)
        .into_iter()
        .filter(|s| s.starts_with("shader."))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_fails_without_shader_primal() {
        let result = compile(&serde_json::json!({"source": "@compute fn main() {}"}));
        assert!(matches!(result, Err(ShaderError::NoShaderPrimal)));
    }

    #[test]
    fn capabilities_returns_empty_without_primal() {
        assert!(capabilities().is_empty());
    }

    #[test]
    fn shader_error_display() {
        let err = ShaderError::NoShaderPrimal;
        assert_eq!(err.to_string(), "no shader compiler primal discovered");
    }
}
