// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structured primal discovery with source tracking.
//!
//! Follows primalSpring's `discover.rs` pattern: every discovery result
//! carries the `DiscoverySource` so callers know *how* a peer was found
//! (env override, XDG convention, capability probe, etc.). Enables
//! better observability and debugging in composition validation.

use std::path::PathBuf;

use super::socket;

/// How a primal was discovered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoverySource {
    /// Explicit socket path from an environment variable.
    EnvOverride,
    /// Named primal found via env + socket directory scan.
    EnvPrimalName,
    /// Capability-based probe of sockets in the biomeOS directory.
    CapabilityProbe,
    /// XDG convention path (socket dir + primal name pattern).
    XdgConvention,
    /// Fallback `/tmp/biomeos` directory.
    TempFallback,
    /// Primal was not found.
    NotFound,
}

/// Result of a primal discovery attempt.
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Resolved socket path (if found).
    pub socket: Option<PathBuf>,
    /// How the primal was found (or why not).
    pub source: DiscoverySource,
    /// Domain or primal name that was searched for.
    pub query: String,
}

impl DiscoveryResult {
    /// Whether the primal was successfully discovered.
    #[must_use]
    pub const fn found(&self) -> bool {
        self.socket.is_some()
    }
}

/// Discover a compute primal with structured result.
#[must_use]
pub fn discover_compute() -> DiscoveryResult {
    discover_by_env_then_capability(
        "HEALTHSPRING_COMPUTE_SOCKET",
        "HEALTHSPRING_COMPUTE_PRIMAL",
        "compute",
    )
}

/// Discover a data primal with structured result.
#[must_use]
pub fn discover_data() -> DiscoveryResult {
    discover_by_env_then_capability(
        "HEALTHSPRING_DATA_SOCKET",
        "HEALTHSPRING_DATA_PRIMAL",
        "data",
    )
}

/// Discover a shader compiler primal with structured result.
#[must_use]
pub fn discover_shader() -> DiscoveryResult {
    discover_by_env_then_capability(
        "HEALTHSPRING_SHADER_SOCKET",
        "HEALTHSPRING_SHADER_PRIMAL",
        "shader",
    )
}

/// Discover an inference primal with structured result.
#[must_use]
pub fn discover_inference() -> DiscoveryResult {
    let mut result = discover_by_env_then_capability(
        "HEALTHSPRING_INFERENCE_SOCKET",
        "HEALTHSPRING_INFERENCE_PRIMAL",
        "model",
    );
    if !result.found() {
        if let Some(path) = socket::discover_by_capability_public("inference") {
            result.socket = Some(path);
            result.source = DiscoverySource::CapabilityProbe;
        }
    }
    result
}

fn discover_by_env_then_capability(
    socket_env: &str,
    primal_env: &str,
    domain: &str,
) -> DiscoveryResult {
    if let Ok(path_str) = std::env::var(socket_env) {
        let path = PathBuf::from(path_str);
        return DiscoveryResult {
            socket: Some(path),
            source: DiscoverySource::EnvOverride,
            query: domain.to_owned(),
        };
    }

    if let Some(path) = std::env::var(primal_env)
        .ok()
        .and_then(|name| socket::discover_primal(&name))
    {
        return DiscoveryResult {
            socket: Some(path),
            source: DiscoverySource::EnvPrimalName,
            query: domain.to_owned(),
        };
    }

    if let Some(path) = socket::discover_by_capability_public(domain) {
        return DiscoveryResult {
            socket: Some(path),
            source: DiscoverySource::CapabilityProbe,
            query: domain.to_owned(),
        };
    }

    DiscoveryResult {
        socket: None,
        source: DiscoverySource::NotFound,
        query: domain.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_result_found_returns_false_when_none() {
        let result = DiscoveryResult {
            socket: None,
            source: DiscoverySource::NotFound,
            query: "test".to_owned(),
        };
        assert!(!result.found());
    }

    #[test]
    fn discovery_result_found_returns_true_when_some() {
        let result = DiscoveryResult {
            socket: Some(PathBuf::from("/tmp/test.sock")),
            source: DiscoverySource::EnvOverride,
            query: "test".to_owned(),
        };
        assert!(result.found());
    }

    #[test]
    fn discover_compute_returns_result() {
        let result = discover_compute();
        assert_eq!(result.query, "compute");
    }

    #[test]
    fn discover_data_returns_result() {
        let result = discover_data();
        assert_eq!(result.query, "data");
    }

    #[test]
    fn discover_shader_returns_result() {
        let result = discover_shader();
        assert_eq!(result.query, "shader");
    }

    #[test]
    fn discover_inference_returns_result() {
        let result = discover_inference();
        assert!(result.query == "model");
    }
}
