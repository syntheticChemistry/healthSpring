// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed client for biomeOS orchestrator `lifecycle.*` and `capability.*`
//! protocol.
//!
//! Handles primal registration, heartbeat, and capability announcement.
//! Discovery is socket-based via XDG conventions.

use super::rpc;
use super::socket;

/// Error from lifecycle/orchestrator operations.
#[derive(Debug)]
pub enum LifecycleError {
    /// Orchestrator socket not found in the standard discovery path.
    NoOrchestrator,
    /// RPC send failed (transport/codec).
    Send(rpc::SendError),
}

impl core::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoOrchestrator => write!(f, "no orchestrator socket found"),
            Self::Send(e) => write!(f, "lifecycle send: {e}"),
        }
    }
}

/// Register this primal with the orchestrator.
///
/// # Errors
///
/// Returns [`LifecycleError`] if the orchestrator is unreachable or rejects
/// the registration.
pub fn register(
    name: &str,
    socket_path: &std::path::Path,
    pid: u32,
) -> Result<serde_json::Value, LifecycleError> {
    let orch = resolve_orchestrator()?;
    rpc::try_send(
        &orch,
        "lifecycle.register",
        &serde_json::json!({
            "name": name,
            "socket_path": socket_path.to_string_lossy(),
            "pid": pid,
        }),
    )
    .map_err(LifecycleError::Send)
}

/// Register a set of capabilities with the orchestrator.
///
/// # Errors
///
/// Returns [`LifecycleError`] if the orchestrator is unreachable.
pub fn register_capabilities(
    domain: &str,
    capabilities: &[&str],
) -> Result<serde_json::Value, LifecycleError> {
    let orch = resolve_orchestrator()?;
    rpc::try_send(
        &orch,
        "capability.register",
        &serde_json::json!({
            "domain": domain,
            "capabilities": capabilities,
        }),
    )
    .map_err(LifecycleError::Send)
}

/// Send a heartbeat to the orchestrator.
///
/// # Errors
///
/// Returns [`LifecycleError`] if the orchestrator is unreachable.
pub fn heartbeat(name: &str) -> Result<serde_json::Value, LifecycleError> {
    let orch = resolve_orchestrator()?;
    rpc::try_send(
        &orch,
        "lifecycle.heartbeat",
        &serde_json::json!({ "name": name }),
    )
    .map_err(LifecycleError::Send)
}

/// Deregister this primal from the orchestrator.
///
/// # Errors
///
/// Returns [`LifecycleError`] if the orchestrator is unreachable.
pub fn deregister(name: &str) -> Result<serde_json::Value, LifecycleError> {
    let orch = resolve_orchestrator()?;
    rpc::try_send(
        &orch,
        "lifecycle.deregister",
        &serde_json::json!({ "name": name }),
    )
    .map_err(LifecycleError::Send)
}

fn resolve_orchestrator() -> Result<std::path::PathBuf, LifecycleError> {
    let orch_path = socket::orchestrator_socket();
    if orch_path.exists() {
        return Ok(orch_path);
    }
    if let Some(fallback_name) = socket::fallback_registration_primal() {
        if let Some(path) = socket::discover_primal(&fallback_name) {
            return Ok(path);
        }
    }
    Err(LifecycleError::NoOrchestrator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_fails_without_orchestrator() {
        let result = register("test", std::path::Path::new("/tmp/test.sock"), 1234);
        assert!(result.is_err());
    }

    #[test]
    fn heartbeat_fails_without_orchestrator() {
        let result = heartbeat("test");
        assert!(result.is_err());
    }

    #[test]
    fn lifecycle_error_display() {
        let err = LifecycleError::NoOrchestrator;
        assert_eq!(err.to_string(), "no orchestrator socket found");
    }
}
