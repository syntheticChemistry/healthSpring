// SPDX-License-Identifier: AGPL-3.0-or-later
//! Socket discovery for `biomeOS` and data providers.
//!
//! Pure capability-based discovery — no hardcoded primal names in
//! discovery logic. Environment overrides provided for testing.
//! Runtime discovery via XDG socket directory scanning.

use std::path::PathBuf;

/// Discover the `biomeOS` orchestrator socket.
///
/// Search order:
/// 1. `$BIOMEOS_SOCKET` — explicit override
/// 2. `$XDG_RUNTIME_DIR/{BIOMEOS_XDG_SUBDIR}/` — scan for orchestrator socket
/// 3. Delegate to `ipc::socket::orchestrator_socket()` for standard path
/// 4. `None` — `biomeOS` not available
#[must_use]
pub fn discover_biomeos_socket() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("BIOMEOS_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let orch = crate::ipc::socket::orchestrator_socket();
    if orch.exists() {
        return Some(orch);
    }

    None
}

/// Discover a data provider socket.
///
/// Search order:
/// 1. `$HEALTHSPRING_DATA_SOCKET` — explicit socket path override
/// 2. Capability-based discovery via `ipc::socket::discover_data_primal()`
/// 3. `None` — no data provider available
#[must_use]
pub fn discover_data_provider_socket() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("HEALTHSPRING_DATA_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    crate::ipc::socket::discover_data_primal()
}

/// Returns `true` if the data provider is explicitly enabled via environment.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("HEALTHSPRING_DATA_PROVIDER")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Discover NCBI API key from standard locations.
///
/// Search order:
/// 1. `$NCBI_API_KEY`
/// 2. `ecoPrimals/testing-secrets/api-keys.toml` (relative to workspace root)
/// 3. `~/.ncbi/api_key`
#[must_use]
pub fn discover_ncbi_api_key() -> Option<String> {
    if let Ok(key) = std::env::var("NCBI_API_KEY") {
        if !key.is_empty() {
            return Some(key);
        }
    }

    let home = std::env::var("HOME").ok().map(PathBuf::from)?;

    let ncbi_file = home.join(".ncbi").join("api_key");
    if ncbi_file.exists() {
        if let Ok(text) = std::fs::read_to_string(&ncbi_file) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_enabled_default_false() {
        if std::env::var("HEALTHSPRING_DATA_PROVIDER").is_err() {
            assert!(!is_enabled());
        }
    }

    #[test]
    fn discover_biomeos_no_panic() {
        let _ = discover_biomeos_socket();
    }

    #[test]
    fn discover_data_provider_no_panic() {
        let _ = discover_data_provider_socket();
    }

    #[test]
    fn discover_ncbi_api_key_no_panic() {
        let _ = discover_ncbi_api_key();
    }
}
