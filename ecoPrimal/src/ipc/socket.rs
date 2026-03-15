// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix socket discovery and path resolution for `biomeOS` niche deployment.
//!
//! Follows XDG runtime conventions:
//! 1. `HEALTHSPRING_SOCKET` environment override
//! 2. `BIOMEOS_SOCKET_DIR` / `{PRIMAL_NAME}-{family_id}.sock`
//! 3. `$XDG_RUNTIME_DIR/biomeos/{PRIMAL_NAME}-{family_id}.sock`
//! 4. `/tmp/biomeos/{PRIMAL_NAME}-{family_id}.sock` (fallback)

use std::path::PathBuf;

/// This primal's socket name (healthSpring).
const PRIMAL_NAME: &str = "healthspring";

/// Default compute primal name when `HEALTHSPRING_COMPUTE_PRIMAL` is unset.
const COMPUTE_PRIMAL_DEFAULT: &str = "toadstool";

/// Default data primal name when `HEALTHSPRING_DATA_PRIMAL` is unset.
const DATA_PRIMAL_DEFAULT: &str = "nestgate";

/// Return the biomeOS socket directory, with XDG fallback.
#[must_use]
pub fn resolve_socket_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("biomeos");
    }
    PathBuf::from("/tmp/biomeos")
}

/// Stable family identifier, overridable via `HEALTHSPRING_FAMILY_ID`.
#[must_use]
pub fn get_family_id() -> String {
    std::env::var("HEALTHSPRING_FAMILY_ID").unwrap_or_else(|_| "default".to_owned())
}

/// Full socket path for this primal instance.
#[must_use]
pub fn resolve_bind_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("HEALTHSPRING_SOCKET") {
        return PathBuf::from(explicit);
    }
    let dir = resolve_socket_dir();
    let family = get_family_id();
    dir.join(format!("{PRIMAL_NAME}-{family}.sock"))
}

/// Discover the orchestrator socket.
#[must_use]
pub fn orchestrator_socket() -> PathBuf {
    let name =
        std::env::var("BIOMEOS_ORCHESTRATOR_SOCKET").unwrap_or_else(|_| "biomeOS.sock".to_owned());
    resolve_socket_dir().join(name)
}

/// Discover another primal by name — scans the socket dir for matching sockets.
#[must_use]
pub fn discover_primal(primal_name: &str) -> Option<PathBuf> {
    let dir = resolve_socket_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(primal_name) && name_str.ends_with(".sock") {
            return Some(entry.path());
        }
    }
    None
}

/// Discover all primals currently visible in the socket directory.
#[must_use]
pub fn discover_all_primals() -> Vec<String> {
    let dir = resolve_socket_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            std::path::Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                .then_some(name)
        })
        .collect()
}

/// Probe for a compute primal (toadStool / Node Atomic).
#[must_use]
pub fn discover_compute_primal() -> Option<PathBuf> {
    std::env::var("HEALTHSPRING_COMPUTE_PRIMAL")
        .ok()
        .and_then(|name| discover_primal(&name))
        .or_else(|| discover_primal(COMPUTE_PRIMAL_DEFAULT))
}

/// Probe for a data primal (`NestGate`).
#[must_use]
pub fn discover_data_primal() -> Option<PathBuf> {
    std::env::var("HEALTHSPRING_DATA_PRIMAL")
        .ok()
        .and_then(|name| discover_primal(&name))
        .or_else(|| discover_primal(DATA_PRIMAL_DEFAULT))
}

/// Probe for a fallback registration primal.
#[must_use]
pub fn fallback_registration_primal() -> Option<String> {
    std::env::var("BIOMEOS_FALLBACK_PRIMAL").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_path_contains_primal_name() {
        let path = resolve_bind_path();
        assert!(path.to_string_lossy().contains("healthspring"));
    }

    #[test]
    fn family_id_defaults_to_default() {
        if std::env::var("HEALTHSPRING_FAMILY_ID").is_err() {
            assert_eq!(get_family_id(), "default");
        }
    }
}
