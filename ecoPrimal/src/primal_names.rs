// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical primal name constants and socket environment helpers.
//!
//! Self-knowledge only: these constants name ecosystem primals for
//! **env-var derivation** and **socket-prefix fallback** when capability-based
//! discovery has not yet bootstrapped.  At runtime, primals are discovered
//! by the capabilities they advertise — never by compile-time identity.
//!
//! Follows the pattern established by airSpring, wetSpring, and neuralSpring
//! (`primal_names.rs`).

/// Conventional socket-name prefix for `barraCuda` (tensor / stats).
pub const BARRACUDA: &str = "barracuda";

/// Conventional socket-name prefix for `coralReef` (shader compile).
pub const CORALREEF: &str = "coralreef";

/// Conventional socket-name prefix for `toadStool` (compute dispatch).
pub const TOADSTOOL: &str = "toadstool";

/// Conventional socket-name prefix for `NestGate` (storage).
pub const NESTGATE: &str = "nestgate";

/// Conventional socket-name prefix for `rhizoCrypt` (DAG).
pub const RHIZOCRYPT: &str = "rhizocrypt";

/// Conventional socket-name prefix for `loamSpine` (ledger / spine).
pub const LOAMSPINE: &str = "loamspine";

/// Conventional socket-name prefix for `sweetGrass` (commit / braid).
pub const SWEETGRASS: &str = "sweetgrass";

/// Conventional socket-name prefix for `Squirrel` (inference).
pub const SQUIRREL: &str = "squirrel";

/// Conventional socket-name prefix for the crypto primal.
pub const BEARDOG: &str = "beardog";

/// Conventional socket-name prefix for the network/discovery primal.
pub const SONGBIRD: &str = "songbird";

/// Conventional socket-name prefix for the visualization primal.
pub const PETALTONGUE: &str = "petaltongue";

/// Conventional socket-name prefix for `skunkBat` (audit).
pub const SKUNKBAT: &str = "skunkbat";

/// Conventional socket-name prefix for the orchestrator.
pub const BIOMEOS: &str = "biomeOS";

/// Lowercase filesystem directory name for the orchestrator socket tree.
///
/// Filesystem conventions use lowercase (`/tmp/biomeos/`, `~/.cache/biomeos/`)
/// even though the display name is `biomeOS`.
pub const BIOMEOS_DIR_NAME: &str = "biomeos";

/// Default fallback socket directory (development convenience).
pub const FALLBACK_SOCKET_DIR: &str = "/tmp/biomeos";

/// Wire-protocol method prefixes used for JSON-RPC method normalization.
///
/// Legacy callers may send `barracuda.stats.mean` or `biomeos.lifecycle.health`;
/// these prefixes are stripped to produce the canonical bare form.
pub mod wire_prefix {
    /// Method prefix for healthSpring-originated JSON-RPC calls.
    pub const HEALTHSPRING: &str = "healthspring.";
    /// Method prefix for barraCuda-originated JSON-RPC calls.
    pub const BARRACUDA: &str = "barracuda.";
    /// Method prefix for biomeOS-originated JSON-RPC calls.
    pub const BIOMEOS: &str = "biomeos.";
}

/// Well-known Songbird socket paths relative to `XDG_RUNTIME_DIR`.
///
/// Songbird *is* the discovery service, so we locate it by convention
/// rather than capability probe.
pub const SONGBIRD_SOCKET_PATHS: &[&str] = &[
    "biomeos/songbird.sock",
    "songbird/songbird.sock",
];

/// Environment variable suffix convention: `{NAME}_SOCKET`.
///
/// ```
/// # use healthspring_barracuda::primal_names::socket_env_var;
/// assert_eq!(socket_env_var("BEARDOG"), "BEARDOG_SOCKET");
/// ```
#[must_use]
pub fn socket_env_var(primal_upper: &str) -> String {
    format!("{primal_upper}_SOCKET")
}

/// Environment variable for the socket-name prefix override.
///
/// ```
/// # use healthspring_barracuda::primal_names::prefix_env_var;
/// assert_eq!(prefix_env_var("CRYPTO"), "BIOMEOS_CRYPTO_PREFIX");
/// ```
#[must_use]
pub fn prefix_env_var(role_upper: &str) -> String {
    format!("BIOMEOS_{role_upper}_PREFIX")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_env_var_format() {
        assert_eq!(socket_env_var("BEARDOG"), "BEARDOG_SOCKET");
        assert_eq!(socket_env_var("SONGBIRD"), "SONGBIRD_SOCKET");
        assert_eq!(socket_env_var("PETALTONGUE"), "PETALTONGUE_SOCKET");
    }

    #[test]
    fn prefix_env_var_format() {
        assert_eq!(prefix_env_var("CRYPTO"), "BIOMEOS_CRYPTO_PREFIX");
        assert_eq!(prefix_env_var("DISCOVERY"), "BIOMEOS_DISCOVERY_PREFIX");
        assert_eq!(prefix_env_var("VIZ"), "BIOMEOS_VIZ_PREFIX");
    }
}
