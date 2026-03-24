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

/// Conventional socket-name prefix for the crypto primal.
pub const BEARDOG: &str = "beardog";

/// Conventional socket-name prefix for the network/discovery primal.
pub const SONGBIRD: &str = "songbird";

/// Conventional socket-name prefix for the visualization primal.
pub const PETALTONGUE: &str = "petaltongue";

/// Conventional socket-name prefix for the orchestrator.
pub const BIOMEOS: &str = "biomeOS";

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
