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

/// Probe for a compute primal by capability.
///
/// Resolution order:
/// 1. Explicit env `HEALTHSPRING_COMPUTE_PRIMAL` (name override for testing)
/// 2. Scan socket dir for any primal advertising `compute.*` capability
///
/// No hardcoded primal names — self-knowledge only. If the compute primal
/// changes its name, capability discovery still works.
#[must_use]
pub fn discover_compute_primal() -> Option<PathBuf> {
    if let Some(path) = std::env::var("HEALTHSPRING_COMPUTE_PRIMAL")
        .ok()
        .and_then(|name| discover_primal(&name))
    {
        return Some(path);
    }
    discover_by_capability("compute")
}

/// Probe for a data primal by capability.
///
/// Resolution order:
/// 1. Explicit env `HEALTHSPRING_DATA_PRIMAL` (name override for testing)
/// 2. Scan socket dir for any primal advertising `data.*` capability
///
/// No hardcoded primal names — self-knowledge only.
#[must_use]
pub fn discover_data_primal() -> Option<PathBuf> {
    if let Some(path) = std::env::var("HEALTHSPRING_DATA_PRIMAL")
        .ok()
        .and_then(|name| discover_primal(&name))
    {
        return Some(path);
    }
    discover_by_capability("data")
}

/// Discover a primal by capability domain: query each visible socket with
/// `capability.list` and check for matching domain prefix.
fn discover_by_capability(domain: &str) -> Option<PathBuf> {
    let dir = resolve_socket_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.ends_with(".sock") || name_str.starts_with(PRIMAL_NAME) {
            continue;
        }
        if probe_capability(&path, domain) {
            return Some(path);
        }
    }
    None
}

/// Send a `capability.list` probe and check if any capability starts with the domain.
fn probe_capability(socket_path: &std::path::Path, domain: &str) -> bool {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let Ok(mut stream) = UnixStream::connect(socket_path) else {
        return false;
    };
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_millis(500)))
        .ok();

    let req = "{\"jsonrpc\":\"2.0\",\"method\":\"capability.list\",\"params\":{},\"id\":1}\n";
    if stream.write_all(req.as_bytes()).is_err() || stream.flush().is_err() {
        return false;
    }

    let mut buf = vec![0u8; 8192];
    let Ok(n) = stream.read(&mut buf) else {
        return false;
    };
    let Ok(resp) = serde_json::from_slice::<serde_json::Value>(&buf[..n]) else {
        return false;
    };

    let prefix = format!("{domain}.");
    resp.get("result")
        .and_then(|r| r.get("science"))
        .and_then(|s| s.as_array())
        .is_some_and(|caps| {
            caps.iter()
                .any(|c| c.as_str().is_some_and(|s| s.starts_with(&prefix)))
        })
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
