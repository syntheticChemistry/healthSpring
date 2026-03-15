// SPDX-License-Identifier: AGPL-3.0-only
//! Socket discovery for `biomeOS` and `NestGate`.
//!
//! Follows XDG conventions and capability-based discovery — no hardcoded
//! paths. Checks environment overrides first, then standard locations.

use std::path::PathBuf;

/// Discover the `biomeOS` orchestrator socket.
///
/// Search order:
/// 1. `$BIOMEOS_SOCKET` — explicit override
/// 2. `$XDG_RUNTIME_DIR/biomeos/biomeos-default.sock`
/// 3. `None` — `biomeOS` not available
#[must_use]
pub fn discover_biomeos_socket() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("BIOMEOS_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(xdg)
            .join("biomeos")
            .join("biomeos-default.sock");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Discover the `NestGate` data provider socket.
///
/// Search order:
/// 1. `$NESTGATE_SOCKET` — explicit override
/// 2. `$XDG_RUNTIME_DIR/biomeos/nestgate-default.sock`
/// 3. `None` — `NestGate` not available
#[must_use]
pub fn discover_nestgate_socket() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("NESTGATE_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(xdg)
            .join("biomeos")
            .join("nestgate-default.sock");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Returns `true` if the data provider is explicitly enabled.
///
/// Checks `HEALTHSPRING_DATA_PROVIDER=nestgate`.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("HEALTHSPRING_DATA_PROVIDER")
        .map(|v| v.eq_ignore_ascii_case("nestgate"))
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
        // Unless CI sets HEALTHSPRING_DATA_PROVIDER, should be false
        if std::env::var("HEALTHSPRING_DATA_PROVIDER").is_err() {
            assert!(!is_enabled());
        }
    }

    #[test]
    fn discover_biomeos_no_panic() {
        let _ = discover_biomeos_socket();
    }

    #[test]
    fn discover_nestgate_no_panic() {
        let _ = discover_nestgate_socket();
    }

    #[test]
    fn discover_ncbi_api_key_no_panic() {
        let _ = discover_ncbi_api_key();
    }
}
