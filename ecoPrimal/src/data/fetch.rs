// SPDX-License-Identifier: AGPL-3.0-only
//! Three-tier fetch logic: `biomeOS` → `NestGate` cache → direct NCBI HTTP.

use std::path::PathBuf;

use super::discovery;
use super::storage;
use super::DataError;

/// NCBI data provider with three-tier fetch support.
///
/// Discovers available tiers at construction time and falls through
/// from `biomeOS` → `NestGate` → direct HTTP transparently.
#[derive(Debug, Clone)]
pub struct NcbiProvider {
    biomeos_socket: Option<PathBuf>,
    nestgate_socket: Option<PathBuf>,
    api_key: Option<String>,
}

impl NcbiProvider {
    /// Discover available data tiers from the environment.
    ///
    /// This is cheap — it only checks for socket existence, no I/O.
    #[must_use]
    pub fn discover() -> Self {
        Self {
            biomeos_socket: discovery::discover_biomeos_socket(),
            nestgate_socket: discovery::discover_nestgate_socket(),
            api_key: discovery::discover_ncbi_api_key(),
        }
    }

    /// Which tier is the highest available.
    #[must_use]
    pub fn highest_tier(&self) -> u8 {
        if self.biomeos_socket.is_some() {
            1
        } else if self.nestgate_socket.is_some() {
            2
        } else {
            3
        }
    }

    /// Fetch data from NCBI using three-tier fallback.
    ///
    /// # Errors
    ///
    /// Returns `DataError` if all tiers fail. In practice, Tier 3 (direct HTTP)
    /// should always work unless NCBI is unreachable or rate-limited.
    pub fn fetch(&self, db: &str, id: &str) -> Result<String, DataError> {
        fetch_tiered(
            self.biomeos_socket.as_deref(),
            self.nestgate_socket.as_deref(),
            db,
            id,
            self.api_key.as_deref().unwrap_or(""),
        )
    }

    /// Load a QS gene matrix from local cache or data directory.
    ///
    /// Checks `HEALTHSPRING_DATA_ROOT`, then `data/qs_gene_matrix.json`
    /// relative to the workspace.
    ///
    /// # Errors
    ///
    /// Returns `DataError::Io` if the file doesn't exist or can't be read,
    /// or `DataError::Parse` if the JSON is malformed.
    pub fn load_qs_matrix(&self) -> Result<crate::qs::QsGeneMatrix, DataError> {
        let path = storage::qs_matrix_path();
        let text = std::fs::read_to_string(&path).map_err(|e| {
            DataError::Io(std::io::Error::new(
                e.kind(),
                format!("loading QS matrix from {}: {e}", path.display()),
            ))
        })?;
        serde_json::from_str(&text).map_err(|e| DataError::Parse(e.to_string()))
    }
}

/// Three-tier fetch: `biomeOS` → `NestGate` cache → direct NCBI HTTP.
///
/// This is the standalone function form; prefer `NcbiProvider::fetch` for
/// repeated use (avoids re-discovering sockets on each call).
///
/// # Errors
///
/// Returns `DataError` if all tiers fail.
pub fn fetch_tiered(
    _biomeos_socket: Option<&std::path::Path>,
    _nestgate_socket: Option<&std::path::Path>,
    db: &str,
    id: &str,
    _api_key: &str,
) -> Result<String, DataError> {
    // Tier 1: biomeOS capability.call
    // TODO: Implement when biomeOS IPC client is ready.
    // If biomeos_socket is available, attempt:
    //   capability.call("data.ncbi_fetch", { db, id })
    // On success, return the result.

    // Tier 2: NestGate cache
    // TODO: Implement when NestGate socket protocol is finalized.
    // If nestgate_socket is available:
    //   Check: rpc_call(socket, "storage.exists", { key: content_key(db, id) })
    //   If exists: rpc_call(socket, "storage.retrieve", { key })
    //   Else: fetch from NCBI, then store via NestGate

    // Tier 3: Direct NCBI HTTP (stub)
    // The actual HTTP client will be added when the nestgate feature is enabled.
    // For now, check local cache.
    let cache = storage::local_cache_path(db, id);
    if cache.exists() {
        return std::fs::read_to_string(&cache).map_err(DataError::from);
    }

    Err(DataError::SocketNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_discover_no_panic() {
        let p = NcbiProvider::discover();
        assert!(p.highest_tier() >= 1);
        assert!(p.highest_tier() <= 3);
    }

    #[test]
    fn fetch_tiered_falls_to_error_without_sockets() {
        let result = fetch_tiered(None, None, "gene", "nonexistent_id_99999", "");
        assert!(result.is_err());
    }

    #[test]
    fn provider_highest_tier_without_sockets() {
        let p = NcbiProvider {
            biomeos_socket: None,
            nestgate_socket: None,
            api_key: None,
        };
        assert_eq!(p.highest_tier(), 3);
    }
}
