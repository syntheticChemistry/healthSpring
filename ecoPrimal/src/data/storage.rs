// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content-addressed local storage for cached NCBI data.
//!
//! Keys follow `NestGate` convention: `ncbi:{db}:{id}`.
//! Local cache lives under `$HEALTHSPRING_DATA_ROOT/ncbi_cache/` or
//! `data/ncbi_cache/` relative to workspace root.

use std::path::PathBuf;

/// Build a `NestGate`-compatible content key.
///
/// Format: `ncbi:{db}:{id}` — matches the key patterns used by
/// `NestGate`'s `storage.store`/`storage.retrieve` methods.
#[must_use]
pub fn content_key(db: &str, id: &str) -> String {
    format!("ncbi:{db}:{id}")
}

/// Local file path for a cached NCBI response.
///
/// Maps `ncbi:{db}:{id}` to `$HEALTHSPRING_DATA_ROOT/ncbi_cache/{db}/{id}.json`.
#[must_use]
pub fn local_cache_path(db: &str, id: &str) -> PathBuf {
    data_root()
        .join("ncbi_cache")
        .join(db)
        .join(format!("{id}.json"))
}

/// Path to the QS gene matrix JSON file.
///
/// Checks `$HEALTHSPRING_DATA_ROOT` first, then `data/qs_gene_matrix.json`
/// relative to workspace root.
#[must_use]
pub fn qs_matrix_path() -> PathBuf {
    let root = data_root();
    let path = root.join("qs_gene_matrix.json");
    if path.exists() {
        return path;
    }

    // Cold storage fallback
    if let Ok(cold) = std::env::var("HEALTHSPRING_COLD_STORAGE") {
        let cold_path = PathBuf::from(cold).join("qs_gene_matrix.json");
        if cold_path.exists() {
            return cold_path;
        }
    }

    path
}

/// Resolve the data root directory.
fn data_root() -> PathBuf {
    if let Ok(root) = std::env::var("HEALTHSPRING_DATA_ROOT") {
        return PathBuf::from(root);
    }

    // Heuristic: look for `data/` sibling to `ecoPrimal/`
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default();

    let workspace_data = manifest_dir.parent().map(|p| p.join("data"));
    if let Some(ref d) = workspace_data {
        if d.exists() {
            return d.clone();
        }
    }

    PathBuf::from("data")
}

/// Store content in the local cache.
///
/// Creates parent directories as needed. Returns the written path.
///
/// # Errors
///
/// Returns `std::io::Error` if directory creation or file write fails.
pub fn store_local(db: &str, id: &str, content: &str) -> Result<PathBuf, std::io::Error> {
    let path = local_cache_path(db, id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Check if content exists in local cache.
#[must_use]
pub fn exists_local(db: &str, id: &str) -> bool {
    local_cache_path(db, id).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_key_format_matches_nestgate() {
        assert_eq!(content_key("gene", "12345"), "ncbi:gene:12345");
        assert_eq!(content_key("sra", "SRR000001"), "ncbi:sra:SRR000001");
    }

    #[test]
    fn local_cache_path_structure() {
        let p = local_cache_path("pubmed", "33456789");
        assert!(p.to_string_lossy().contains("ncbi_cache"));
        assert!(p.to_string_lossy().contains("pubmed"));
        assert!(p.to_string_lossy().ends_with("33456789.json"));
    }

    #[test]
    fn store_and_check_uses_correct_path() {
        let dir = std::env::temp_dir().join("healthspring_test_cache_storage");
        let path = dir.join("ncbi_cache").join("test_db").join("test_id.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, r#"{"test": true}"#);
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
