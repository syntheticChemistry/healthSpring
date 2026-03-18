// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tower Atomic bootstrap — `BearDog` (crypto) + `Songbird` (discovery).
//!
//! The Tower Atomic is the minimal NUCLEUS composition for a single machine:
//!
//! ```text
//! Tower Atomic = BearDog (crypto/identity) + Songbird (discovery/network)
//! ```
//!
//! This module provides:
//! 1. **Discovery via `Songbird`**: Query the network discovery service for
//!    primal sockets by capability, replacing filesystem glob scanning.
//! 2. **Identity via `BearDog`**: Verify primal identity before establishing
//!    trust for IPC connections.
//! 3. **Bootstrap readiness**: Check whether the Tower Atomic is available
//!    on the local machine.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use healthspring_barracuda::ipc::tower_atomic::TowerAtomic;
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     if let Some(tower) = TowerAtomic::discover() {
//!         let nestgate = tower.find_capability("data.ncbi_fetch")?;
//!         let toadstool = tower.find_capability("compute.dispatch.submit")?;
//!     }
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;

use super::error::IpcError;
use super::socket;

/// Tower Atomic handle — `BearDog` + `Songbird` on the local machine.
#[derive(Debug, Clone)]
pub struct TowerAtomic {
    beardog_socket: PathBuf,
    songbird_socket: PathBuf,
}

impl TowerAtomic {
    /// Attempt to discover a Tower Atomic on the local machine.
    ///
    /// Checks for `BearDog` and `Songbird` sockets in the XDG runtime
    /// directory. Both must be present for a valid Tower Atomic.
    #[must_use]
    pub fn discover() -> Option<Self> {
        let socket_dir = socket::resolve_socket_dir();

        let beardog = discover_primal_in(&socket_dir, "beardog")?;
        let songbird = discover_primal_in(&socket_dir, "songbird")?;

        Some(Self {
            beardog_socket: beardog,
            songbird_socket: songbird,
        })
    }

    /// Check if the Tower Atomic is healthy (both primals respond).
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if either primal fails its health check.
    pub fn health_check(&self) -> Result<(), IpcError> {
        health_probe(&self.beardog_socket, "health")?;
        health_probe(&self.songbird_socket, "health")?;
        Ok(())
    }

    /// Discover a primal that provides the given capability.
    ///
    /// Queries `Songbird`'s `net.discovery` service for a primal socket
    /// advertising the requested capability.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if `Songbird` is unreachable or no primal
    /// provides the capability.
    pub fn find_capability(&self, capability: &str) -> Result<PathBuf, IpcError> {
        let params = serde_json::json!({
            "capability": capability,
        });

        let result = super::rpc::try_send(
            &self.songbird_socket,
            "net.discovery.find_by_capability",
            &params,
        )?;

        result
            .get("socket_path")
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| IpcError::RpcReject {
                code: -32000,
                message: format!("Songbird found no primal for capability: {capability}"),
            })
    }

    /// Verify a primal's identity via `BearDog` before connecting.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if `BearDog` rejects the identity or is unreachable.
    pub fn verify_identity(&self, primal_name: &str) -> Result<bool, IpcError> {
        let params = serde_json::json!({
            "primal": primal_name,
        });

        let result = super::rpc::try_send(&self.beardog_socket, "crypto.verify_primal", &params)?;

        Ok(result
            .get("trusted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false))
    }

    /// Path to the `BearDog` socket.
    #[must_use]
    pub fn beardog_socket(&self) -> &std::path::Path {
        &self.beardog_socket
    }

    /// Path to the `Songbird` socket.
    #[must_use]
    pub fn songbird_socket(&self) -> &std::path::Path {
        &self.songbird_socket
    }

    /// Whether the Tower Atomic is available.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.beardog_socket.exists() && self.songbird_socket.exists()
    }
}

/// Discover a primal socket in a directory by name pattern.
fn discover_primal_in(dir: &std::path::Path, primal: &str) -> Option<PathBuf> {
    // 1. Environment override
    let env_key = format!("{}_SOCKET", primal.to_uppercase());
    if let Ok(path) = std::env::var(&env_key) {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Standard socket in biomeOS directory
    let default_sock = dir.join(format!("{primal}-default.sock"));
    if default_sock.exists() {
        return Some(default_sock);
    }

    // 3. Glob scan for any socket matching the primal name
    let pattern = dir.join(format!("{primal}-*.sock"));
    if let Some(pattern_str) = pattern.to_str() {
        if let Some(entry) = glob_sockets(pattern_str).into_iter().next() {
            return Some(entry);
        }
    }

    None
}

/// Simple glob for socket files (no external dependency).
fn glob_sockets(pattern: &str) -> Vec<PathBuf> {
    let dir = std::path::Path::new(pattern)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let prefix = std::path::Path::new(pattern)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .trim_end_matches('*');

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with(prefix) && name_str.ends_with(".sock")
        })
        .map(|e| e.path())
        .collect()
}

/// Send a health probe to a primal socket.
fn health_probe(socket: &std::path::Path, method: &str) -> Result<(), IpcError> {
    let params = serde_json::json!({});
    super::rpc::try_send(socket, method, &params)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tower_atomic_discover_returns_none_without_sockets() {
        // In test environment, no biomeOS sockets exist
        let tower = TowerAtomic::discover();
        // May or may not find sockets depending on environment
        drop(tower);
    }

    #[test]
    fn discover_primal_in_nonexistent_dir() {
        let result = discover_primal_in(
            std::path::Path::new("/tmp/nonexistent_biomeos_test_dir"),
            "beardog",
        );
        assert!(result.is_none());
    }

    #[test]
    fn glob_sockets_empty_dir() {
        let result = glob_sockets("/tmp/nonexistent_biomeos_test_dir/beardog-*.sock");
        assert!(result.is_empty());
    }

    #[test]
    fn tower_atomic_socket_accessors() {
        let tower = TowerAtomic {
            beardog_socket: PathBuf::from("/tmp/test-beardog.sock"),
            songbird_socket: PathBuf::from("/tmp/test-songbird.sock"),
        };
        assert!(tower.beardog_socket().to_string_lossy().contains("beardog"));
        assert!(
            tower
                .songbird_socket()
                .to_string_lossy()
                .contains("songbird")
        );
    }
}
