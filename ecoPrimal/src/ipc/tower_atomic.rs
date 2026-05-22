// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    deprecated,
    reason = "tower atomic bootstrap probes sockets via legacy discovery until CompositionContext integration"
)]
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
//! ## Bonding Policy (healthSpring proto-nucleate)
//!
//! | Boundary | Bond type | Trust model | Encryption |
//! |----------|-----------|-------------|------------|
//! | Tower A (patient enclave) | Ionic | `BearDog` A enforces family-scoped trust | `BearDog`-encrypted at rest + in transit |
//! | Tower B (analytics) | Ionic | `BearDog` B verifies Squirrel + analytics primals | `BearDog`-encrypted at rest + in transit |
//! | Ionic bridge (A ↔ B) | Ionic | De-identified aggregates only; `BearDog` cross-family bond | No PII crosses; aggregates signed by `BearDog` A |
//! | Within-tower IPC | Covalent | Same-family trust; UDS-only | Platform socket permissions |
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
use crate::primal_names;

/// Capability domains used for Tower Atomic primal discovery.
/// These are the semantic capabilities, not primal names — any primal
/// advertising these capabilities satisfies the Tower Atomic contract.
const CRYPTO_CAPABILITY: &str = "crypto";
const DISCOVERY_CAPABILITY: &str = "net.discovery";

/// Env-driven or conventional socket-name prefixes for filesystem scan.
/// Only used when capability-based discovery has not yet bootstrapped.
fn crypto_socket_prefix() -> String {
    std::env::var(primal_names::prefix_env_var("CRYPTO"))
        .unwrap_or_else(|_| primal_names::BEARDOG.into())
}
fn discovery_socket_prefix() -> String {
    std::env::var(primal_names::prefix_env_var("DISCOVERY"))
        .unwrap_or_else(|_| primal_names::SONGBIRD.into())
}

/// Tower Atomic handle — crypto + discovery primals on the local machine.
///
/// The Tower Atomic is a composed system: a crypto primal (identity/trust)
/// plus a discovery primal (network presence). The specific primal names
/// are not assumed — discovery is capability-based with name-prefix fallback.
#[derive(Debug, Clone)]
pub struct TowerAtomic {
    crypto_socket: PathBuf,
    discovery_socket: PathBuf,
}

impl TowerAtomic {
    /// Attempt to discover a Tower Atomic on the local machine.
    ///
    /// Resolution (per primal role):
    /// 1. Environment override (`BEARDOG_SOCKET` / `SONGBIRD_SOCKET`)
    /// 2. Socket-dir scan by default prefix
    /// 3. Capability probe: `crypto.*` / `net.discovery.*`
    #[must_use]
    pub fn discover() -> Option<Self> {
        let socket_dir = socket::resolve_socket_dir();

        let crypto = discover_primal_in(&socket_dir, &crypto_socket_prefix())
            .or_else(|| socket::discover_by_capability_public(CRYPTO_CAPABILITY))?;
        let discovery = discover_primal_in(&socket_dir, &discovery_socket_prefix())
            .or_else(|| socket::discover_by_capability_public(DISCOVERY_CAPABILITY))?;

        Some(Self {
            crypto_socket: crypto,
            discovery_socket: discovery,
        })
    }

    /// Check if the Tower Atomic is healthy (both primals respond).
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if either primal fails its health check.
    pub fn health_check(&self) -> Result<(), IpcError> {
        health_probe(&self.crypto_socket, "health")?;
        health_probe(&self.discovery_socket, "health")?;
        Ok(())
    }

    /// Discover a primal that provides the given capability.
    ///
    /// Queries the discovery primal's Songbird service for a primal socket
    /// advertising the requested capability. Tries the canonical
    /// `discovery.find_by_capability` method first, falling back to the
    /// legacy `net.discovery.find_by_capability` for backward compat
    /// (see `docs/PRIMAL_GAPS.md` §3).
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the discovery primal is unreachable or no
    /// primal provides the capability.
    pub fn find_capability(&self, capability: &str) -> Result<PathBuf, IpcError> {
        let params = serde_json::json!({
            "capability": capability,
        });

        let result = super::rpc::try_send(
            &self.discovery_socket,
            "discovery.find_by_capability",
            &params,
        )
        .or_else(|_| {
            super::rpc::try_send(
                &self.discovery_socket,
                "net.discovery.find_by_capability",
                &params,
            )
        })?;

        result
            .get("socket_path")
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| IpcError::RpcReject {
                code: -32000,
                message: format!("discovery primal found no provider for: {capability}"),
            })
    }

    /// Verify a primal's identity via the crypto primal before connecting.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the crypto primal rejects the identity or is
    /// unreachable.
    pub fn verify_identity(&self, primal_name: &str) -> Result<bool, IpcError> {
        let params = serde_json::json!({
            "primal": primal_name,
        });

        let result = super::rpc::try_send(&self.crypto_socket, "crypto.verify_primal", &params)?;

        Ok(result
            .get("trusted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false))
    }

    /// Path to the crypto primal socket.
    #[must_use]
    pub fn crypto_socket(&self) -> &std::path::Path {
        &self.crypto_socket
    }

    /// Path to the discovery primal socket.
    #[must_use]
    pub fn discovery_socket(&self) -> &std::path::Path {
        &self.discovery_socket
    }

    /// Whether the Tower Atomic is available.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.crypto_socket.exists() && self.discovery_socket.exists()
    }

    /// Propose an ionic bond contract for cross-tower trust establishment.
    ///
    /// Calls `BearDog`'s `crypto.contract.propose` — the first step in the
    /// propose → countersign → verify lifecycle for ionic bridge bonds.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if `BearDog` is unreachable or rejects the proposal.
    pub fn ionic_propose(
        &self,
        family_a: &str,
        family_b: &str,
        scope: &str,
    ) -> Result<serde_json::Value, IpcError> {
        let params = serde_json::json!({
            "family_a": family_a,
            "family_b": family_b,
            "scope": scope,
        });
        super::rpc::try_send(&self.crypto_socket, "crypto.contract.propose", &params)
    }

    /// Countersign an ionic bond contract (second party approval).
    ///
    /// Calls `BearDog`'s `crypto.contract.countersign` with the contract ID
    /// returned from a prior `ionic_propose` call.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if `BearDog` is unreachable or rejects the countersign.
    pub fn ionic_countersign(&self, contract_id: &str) -> Result<serde_json::Value, IpcError> {
        let params = serde_json::json!({
            "contract_id": contract_id,
        });
        super::rpc::try_send(
            &self.crypto_socket,
            "crypto.contract.countersign",
            &params,
        )
    }

    /// Verify an ionic bond contract (dual-signature check).
    ///
    /// Calls `BearDog`'s `crypto.contract.verify` to confirm both parties
    /// have signed the cross-family trust bond.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if `BearDog` is unreachable or the contract is invalid.
    pub fn ionic_verify(&self, contract_id: &str) -> Result<bool, IpcError> {
        let params = serde_json::json!({
            "contract_id": contract_id,
        });
        let result =
            super::rpc::try_send(&self.crypto_socket, "crypto.contract.verify", &params)?;
        Ok(result
            .get("valid")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false))
    }

    // ── Wave 38 bonding.* protocol (IonicContractRegistry) ──────────
    //
    // These methods call the primalSpring coordination endpoint
    // (bonding.propose / bonding.accept / bonding.terminate) which
    // manages the IonicContractRegistry state machine. The
    // crypto.contract.* methods above remain for the BearDog Ed25519
    // signing layer underneath.

    /// Propose an ionic bond via the `bonding.propose` protocol.
    ///
    /// Routes through the coordination endpoint (primalSpring) to the
    /// `IonicContractRegistry` state machine. The registry validates
    /// the proposal and creates a contract in `Proposed` state.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the coordination endpoint is unreachable.
    pub fn bonding_propose(
        &self,
        proposer_identity: &str,
        capabilities: &[&str],
        duration_secs: u64,
        rate_limit_rps: u32,
    ) -> Result<serde_json::Value, IpcError> {
        let params = serde_json::json!({
            "proposer_identity": proposer_identity,
            "requested_capabilities": capabilities,
            "duration_secs": duration_secs,
            "trust_model": "Contractual",
            "rate_limit_rps": rate_limit_rps,
        });
        let socket = socket::coordination_socket();
        super::rpc::try_send(&socket, "bonding.propose", &params)
    }

    /// Accept or reject a proposed ionic bond via `bonding.accept`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the coordination endpoint is unreachable.
    pub fn bonding_accept(
        &self,
        contract_id: &str,
        accept: bool,
        capability_allow: &[&str],
    ) -> Result<serde_json::Value, IpcError> {
        let params = serde_json::json!({
            "contract_id": contract_id,
            "accept": accept,
            "constraints": {
                "capability_allow": capability_allow,
            },
        });
        let socket = socket::coordination_socket();
        super::rpc::try_send(&socket, "bonding.accept", &params)
    }

    /// Terminate an active ionic bond via `bonding.terminate`.
    ///
    /// Returns the provenance seal from the registry.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the coordination endpoint is unreachable.
    pub fn bonding_terminate(
        &self,
        contract_id: &str,
        reason: &str,
    ) -> Result<serde_json::Value, IpcError> {
        let params = serde_json::json!({
            "contract_id": contract_id,
            "reason": reason,
        });
        let socket = socket::coordination_socket();
        super::rpc::try_send(&socket, "bonding.terminate", &params)
    }

    /// Query the status of an ionic bond via `bonding.status`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the coordination endpoint is unreachable.
    pub fn bonding_status(&self, contract_id: &str) -> Result<serde_json::Value, IpcError> {
        let params = serde_json::json!({
            "contract_id": contract_id,
        });
        let socket = socket::coordination_socket();
        super::rpc::try_send(&socket, "bonding.status", &params)
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
            &crypto_socket_prefix(),
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
            crypto_socket: PathBuf::from("/tmp/test-crypto.sock"),
            discovery_socket: PathBuf::from("/tmp/test-discovery.sock"),
        };
        assert!(tower.crypto_socket().to_string_lossy().contains("crypto"));
        assert!(
            tower
                .discovery_socket()
                .to_string_lossy()
                .contains("discovery")
        );
    }

    #[test]
    fn ionic_propose_fails_without_beardog() {
        let tower = TowerAtomic {
            crypto_socket: PathBuf::from("/tmp/nonexistent-beardog.sock"),
            discovery_socket: PathBuf::from("/tmp/nonexistent-songbird.sock"),
        };
        let result = tower.ionic_propose("patient_enclave", "analytics", "aggregate");
        assert!(result.is_err());
    }

    #[test]
    fn ionic_countersign_fails_without_beardog() {
        let tower = TowerAtomic {
            crypto_socket: PathBuf::from("/tmp/nonexistent-beardog.sock"),
            discovery_socket: PathBuf::from("/tmp/nonexistent-songbird.sock"),
        };
        let result = tower.ionic_countersign("test-contract-id");
        assert!(result.is_err());
    }

    #[test]
    fn ionic_verify_fails_without_beardog() {
        let tower = TowerAtomic {
            crypto_socket: PathBuf::from("/tmp/nonexistent-beardog.sock"),
            discovery_socket: PathBuf::from("/tmp/nonexistent-songbird.sock"),
        };
        let result = tower.ionic_verify("test-contract-id");
        assert!(result.is_err());
    }

    #[test]
    fn bonding_propose_fails_without_coordination() {
        let tower = TowerAtomic {
            crypto_socket: PathBuf::from("/tmp/nonexistent-beardog.sock"),
            discovery_socket: PathBuf::from("/tmp/nonexistent-songbird.sock"),
        };
        let result = tower.bonding_propose(
            "did:key:healthspring",
            &["compute.submit", "storage.retrieve"],
            3600,
            50,
        );
        assert!(result.is_err());
    }

    #[test]
    fn bonding_terminate_fails_without_coordination() {
        let tower = TowerAtomic {
            crypto_socket: PathBuf::from("/tmp/nonexistent-beardog.sock"),
            discovery_socket: PathBuf::from("/tmp/nonexistent-songbird.sock"),
        };
        let result = tower.bonding_terminate("ionic-0001-test", "complete");
        assert!(result.is_err());
    }

    #[test]
    fn bonding_status_fails_without_coordination() {
        let tower = TowerAtomic {
            crypto_socket: PathBuf::from("/tmp/nonexistent-beardog.sock"),
            discovery_socket: PathBuf::from("/tmp/nonexistent-songbird.sock"),
        };
        let result = tower.bonding_status("ionic-0001-test");
        assert!(result.is_err());
    }
}
