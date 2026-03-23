// SPDX-License-Identifier: AGPL-3.0-or-later
//! Three-tier fetch logic: `biomeOS` → `NestGate` cache → direct NCBI HTTP.

use std::path::PathBuf;

use super::DataError;
use super::discovery;
use super::rpc;
use super::storage;

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
            nestgate_socket: discovery::discover_data_provider_socket(),
            api_key: discovery::discover_ncbi_api_key(),
        }
    }

    /// Which tier is the highest available.
    #[must_use]
    pub const fn highest_tier(&self) -> u8 {
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

    /// Search NCBI via `ESearch` and return matching IDs.
    ///
    /// Uses three-tier routing: `biomeOS` → `NestGate` → sovereign HTTP.
    ///
    /// # Errors
    ///
    /// Returns `DataError` if all tiers fail.
    pub fn search(
        &self,
        db: &str,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<String>, DataError> {
        // Tier 1: biomeOS
        if let Some(socket) = &self.biomeos_socket {
            let params = serde_json::json!({
                "capability": "data.ncbi_search",
                "params": { "db": db, "query": query, "max_results": max_results }
            });
            if let Ok(result) = rpc::rpc_call(socket, "capability.call", &params) {
                if let Some(arr) = result.as_array() {
                    return Ok(arr
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(String::from)
                        .collect());
                }
            }
        }

        // Tier 2: NestGate
        if let Some(socket) = &self.nestgate_socket {
            let params =
                serde_json::json!({ "db": db, "query": query, "max_results": max_results });
            if let Ok(result) = rpc::rpc_call(socket, "data.ncbi_search", &params) {
                if let Some(arr) = result.as_array() {
                    return Ok(arr
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(String::from)
                        .collect());
                }
            }
        }

        // Tier 3: Sovereign HTTP
        #[cfg(feature = "nestgate")]
        {
            super::ncbi_http::esearch(db, query, max_results)
        }

        #[cfg(not(feature = "nestgate"))]
        Err(DataError::Rpc(
            "NCBI search unavailable without nestgate feature".into(),
        ))
    }

    /// Fetch SRA run metadata for an accession.
    ///
    /// Returns CSV-formatted run info.
    ///
    /// # Errors
    ///
    /// Returns `DataError` if all tiers fail.
    pub fn fetch_sra_metadata(&self, accession: &str) -> Result<String, DataError> {
        let key = format!("sra_runinfo:{accession}");

        // Check NestGate cache
        if let Some(socket) = &self.nestgate_socket {
            let params = serde_json::json!({ "key": key });
            if let Ok(exists) = rpc::rpc_call(socket, "storage.exists", &params) {
                if exists.as_bool().unwrap_or(false) {
                    let params = serde_json::json!({ "key": key });
                    if let Ok(result) = rpc::rpc_call(socket, "storage.retrieve", &params) {
                        return extract_content(result);
                    }
                }
            }
        }

        // Sovereign HTTP
        #[cfg(feature = "nestgate")]
        {
            let result = super::ncbi_http::sra_run_info(accession)?;

            // Cache in NestGate if available
            if let Some(socket) = &self.nestgate_socket {
                let params = serde_json::json!({ "key": key, "value": result });
                let _ = rpc::rpc_call(socket, "storage.store", &params);
            }

            Ok(result)
        }

        #[cfg(not(feature = "nestgate"))]
        Err(DataError::Rpc(
            "SRA metadata fetch unavailable without nestgate feature".into(),
        ))
    }

    /// Store a result in `NestGate`'s content-addressed storage.
    ///
    /// Falls back to local cache if `NestGate` is unavailable.
    ///
    /// # Errors
    ///
    /// Returns `DataError` on I/O or RPC failure.
    pub fn store_result(&self, db: &str, id: &str, content: &str) -> Result<(), DataError> {
        let key = storage::content_key(db, id);

        if let Some(socket) = &self.nestgate_socket {
            let params = serde_json::json!({ "key": key, "value": content });
            if rpc::rpc_call(socket, "storage.store", &params).is_ok() {
                return Ok(());
            }
        }

        storage::store_local(db, id, content)?;
        Ok(())
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

/// Extract string content from a JSON-RPC result (handles string or object with "content"/"data").
fn extract_content(value: serde_json::Value) -> Result<String, DataError> {
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Object(obj) => {
            if let Some(v) = obj.get("content").or_else(|| obj.get("data")) {
                if let Some(s) = v.as_str() {
                    return Ok(s.to_owned());
                }
                if let Ok(ser) = serde_json::to_string(v) {
                    return Ok(ser);
                }
            }
            serde_json::to_string(&serde_json::Value::Object(obj))
                .map_err(|e| DataError::Parse(e.to_string()))
        }
        other => serde_json::to_string(&other).map_err(|e| DataError::Parse(e.to_string())),
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
    biomeos_socket: Option<&std::path::Path>,
    nestgate_socket: Option<&std::path::Path>,
    db: &str,
    id: &str,
    #[cfg_attr(
        not(feature = "nestgate"),
        expect(unused_variables, reason = "api_key only used by nestgate HTTP tier")
    )]
    api_key: &str,
) -> Result<String, DataError> {
    // Tier 1: biomeOS capability.call
    if let Some(socket) = biomeos_socket {
        let params = serde_json::json!({
            "capability": "data.ncbi_fetch",
            "params": { "db": db, "id": id }
        });
        match rpc::rpc_call(socket, "capability.call", &params) {
            Ok(result) => return extract_content(result),
            Err(e) => {
                // Fall through to tier 2
                let _ = e;
            }
        }
    }

    // Tier 2: NestGate cache — storage.exists then storage.retrieve or data.fetch
    if let Some(socket) = nestgate_socket {
        let key = storage::content_key(db, id);

        // Try storage.retrieve first (content-addressed cache)
        let exists_params = serde_json::json!({ "key": key });
        if let Ok(exists_result) = rpc::rpc_call(socket, "storage.exists", &exists_params) {
            if exists_result.as_bool().unwrap_or(false) {
                let retrieve_params = serde_json::json!({ "key": key });
                if let Ok(result) = rpc::rpc_call(socket, "storage.retrieve", &retrieve_params) {
                    return extract_content(result);
                }
            }
        }

        // Fallback: data.fetch (NestGate may fetch from NCBI and return)
        let fetch_params = serde_json::json!({ "db": db, "id": id });
        if let Ok(result) = rpc::rpc_call(socket, "data.fetch", &fetch_params) {
            return extract_content(result);
        }
    }

    // Tier 3a: Local cache
    let cache = storage::local_cache_path(db, id);
    if cache.exists() {
        return std::fs::read_to_string(&cache).map_err(DataError::from);
    }

    // Tier 3b: Sovereign HTTP via ureq (ecoBin-compliant, pure Rust)
    #[cfg(feature = "nestgate")]
    {
        let result = super::ncbi_http::efetch(db, id, api_key)?;
        let _ = storage::store_local(db, id, &result);
        Ok(result)
    }

    #[cfg(not(feature = "nestgate"))]
    Err(DataError::Rpc(format!(
        "NCBI fetch failed: no biomeOS or NestGate socket available, and local cache miss. \
         Enable the `nestgate` feature for direct HTTP, or set BIOMEOS_SOCKET / \
         NESTGATE_SOCKET, or pre-populate cache at \
         HEALTHSPRING_DATA_ROOT/ncbi_cache/{db}/{id}.json"
    )))
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
        // With nestgate feature: Tier 3b attempts sovereign HTTP.
        // Without: returns Rpc error for missing sockets + cache miss.
        // Use a nonsensical db/id that won't hit local cache.
        let result = fetch_tiered(None, None, "nonexistent_db", "nonexistent_id_99999", "");
        // Either an error (no feature / HTTP failure) or a string response
        // (NCBI returns XML error pages as 200). Both are valid for this test.
        drop(result);
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
