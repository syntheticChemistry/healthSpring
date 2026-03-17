// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance trio integration for data fetch sessions.
//!
//! Follows the `SPRING_PROVENANCE_TRIO_INTEGRATION_PATTERN.md` from
//! wateringHole. Each data fetch operation (NCBI, `UniProt`, KEGG, etc.)
//! is wrapped in a provenance session:
//!
//! ```text
//! begin_data_session("qs_gene_matrix")
//!   → record_fetch_step("ncbi_gene", { genera: 59, families: 6 })
//!   → record_fetch_step("uniprot", { genera: 59, hits: 15 })
//!   → record_merge_step({ total_hits: 63, density: "17.8%" })
//!   → complete_data_session()
//!       → dehydrate  (rhizoCrypt Merkle root)
//!       → commit     (loamSpine immutable record)
//!       → attribute  (sweetGrass braid)
//! ```
//!
//! Graceful degradation: if `biomeOS` is not running, data operations
//! succeed normally — provenance is best-effort, never blocking.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Result of a provenance operation — includes availability status.
#[derive(Debug, Clone)]
pub struct ProvenanceResult {
    /// Session or vertex ID (or local fallback ID).
    pub id: String,
    /// Whether the trio was actually reached.
    pub available: bool,
    /// Structured result data.
    pub data: serde_json::Value,
}

/// Completed provenance chain for a data fetch.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DataProvenanceChain {
    /// Status: `"complete"`, `"partial"`, or `"unavailable"`.
    pub status: String,
    /// Session ID in rhizoCrypt.
    pub session_id: String,
    /// Merkle root from dehydration (content integrity).
    pub merkle_root: String,
    /// Commit ID in loamSpine (immutable record).
    pub commit_id: String,
    /// Braid ID in sweetGrass (attribution).
    pub braid_id: String,
}

/// Well-known socket name for the data-provider primal's API endpoint.
///
/// The default `"neural-api"` reflects the historical naming. Overridable via
/// `DATA_PROVIDER_SOCK_PREFIX` env var — capability-based discovery makes the
/// provider identity irrelevant; any primal exposing `dag.*` capabilities works.
const DEFAULT_DATA_PROVIDER_PREFIX: &str = "neural-api";

/// Discover the data-provider API socket for trio communication.
///
/// Resolution order:
/// 1. `DATA_PROVIDER_SOCKET` (explicit override)
/// 2. `BIOMEOS_SOCKET_DIR/<prefix>-<family>.sock`
/// 3. `XDG_RUNTIME_DIR/biomeos/<prefix>-<family>.sock`
fn data_provider_socket_path() -> Option<PathBuf> {
    let family_id = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let prefix = std::env::var("DATA_PROVIDER_SOCK_PREFIX")
        .unwrap_or_else(|_| DEFAULT_DATA_PROVIDER_PREFIX.to_string());

    let sock_name = format!("{prefix}-{family_id}.sock");

    [
        std::env::var("DATA_PROVIDER_SOCKET")
            .ok()
            .map(PathBuf::from),
        std::env::var("BIOMEOS_SOCKET_DIR")
            .ok()
            .map(|d| PathBuf::from(d).join(&sock_name)),
        std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(|d| PathBuf::from(d).join("biomeos").join(&sock_name)),
    ]
    .into_iter()
    .flatten()
    .find(|candidate| candidate.exists())
}

/// Send a `capability.call` to the Neural API over Unix socket.
fn capability_call(
    socket_path: &Path,
    capability: &str,
    operation: &str,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capability.call",
        "params": {
            "capability": capability,
            "operation": operation,
            "args": args,
        },
        "id": 1,
    });

    let mut stream = std::os::unix::net::UnixStream::connect(socket_path)
        .map_err(|e| format!("connect: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let payload = serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("write newline: {e}"))?;
    stream.flush().map_err(|e| format!("flush: {e}"))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|e| format!("shutdown: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("read: {e}"))?;

    let parsed: serde_json::Value =
        serde_json::from_str(line.trim()).map_err(|e| format!("parse: {e}"))?;

    if let Some(err) = parsed.get("error") {
        let (code, message) = crate::ipc::rpc::extract_rpc_error(err);
        return Err(format!("rpc error {code}: {message}"));
    }

    parsed
        .get("result")
        .cloned()
        .ok_or_else(|| "no result in response".to_string())
}

fn unavailable_result(id: &str) -> ProvenanceResult {
    ProvenanceResult {
        id: id.to_owned(),
        available: false,
        data: serde_json::json!({ "provenance": "unavailable" }),
    }
}

/// Begin a provenance-tracked data fetch session.
///
/// Creates a rhizoCrypt DAG session via the Neural API. If the trio
/// is unavailable, returns a local fallback ID and the fetch proceeds
/// without provenance tracking.
#[must_use]
pub fn begin_data_session(dataset_name: &str) -> ProvenanceResult {
    let Some(socket) = data_provider_socket_path() else {
        return unavailable_result(&format!("local-{dataset_name}"));
    };

    let args = serde_json::json!({
        "metadata": {
            "type": "data_fetch",
            "dataset": dataset_name,
            "spring": "healthSpring",
            "scyborg.content_category": "Code",
        },
        "session_type": { "Experiment": { "spring_id": "healthSpring" } },
        "description": format!("Data fetch: {dataset_name}"),
    });

    capability_call(&socket, "dag", "create_session", &args).map_or_else(
        |_| unavailable_result(&format!("local-{dataset_name}")),
        |result| {
            let session_id = result
                .get("session_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_owned();
            ProvenanceResult {
                id: session_id.clone(),
                available: true,
                data: serde_json::json!({ "session_id": session_id }),
            }
        },
    )
}

/// Record a data fetch step (e.g., NCBI query, `UniProt` scan) in the DAG.
#[must_use]
pub fn record_fetch_step(session_id: &str, step: &serde_json::Value) -> ProvenanceResult {
    let Some(socket) = data_provider_socket_path() else {
        return unavailable_result("unavailable");
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "event": step,
    });

    capability_call(&socket, "dag", "append_event", &args).map_or_else(
        |_| unavailable_result("unavailable"),
        |result| {
            let vertex_id = result
                .get("vertex_id")
                .or_else(|| result.get("id"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_owned();
            ProvenanceResult {
                id: vertex_id.clone(),
                available: true,
                data: serde_json::json!({ "vertex_id": vertex_id }),
            }
        },
    )
}

/// Complete a data fetch session: dehydrate → commit → attribute.
///
/// Returns the full provenance chain. Each step degrades gracefully:
/// if dehydrate fails, we stop. If commit fails, dehydration is preserved.
/// If braid fails, commit is preserved.
pub fn complete_data_session(session_id: &str, license: &str) -> DataProvenanceChain {
    let unavailable = DataProvenanceChain {
        status: "unavailable".into(),
        session_id: session_id.into(),
        merkle_root: String::new(),
        commit_id: String::new(),
        braid_id: String::new(),
    };

    let Some(socket) = data_provider_socket_path() else {
        return unavailable;
    };

    // Step 1: Dehydrate (rhizoCrypt)
    let Ok(dehydration) = capability_call(
        &socket,
        "dag",
        "dehydrate",
        &serde_json::json!({ "session_id": session_id }),
    ) else {
        return unavailable;
    };

    let merkle_root = dehydration
        .get("merkle_root")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();

    // Step 2: Commit (loamSpine)
    let Ok(commit_result) = capability_call(
        &socket,
        "commit",
        "session",
        &serde_json::json!({
            "summary": dehydration,
            "content_hash": merkle_root,
            "metadata": {
                "scyborg.license": license,
                "scyborg.content_category": "Code",
            },
        }),
    ) else {
        return DataProvenanceChain {
            status: "partial".into(),
            session_id: session_id.into(),
            merkle_root,
            commit_id: String::new(),
            braid_id: String::new(),
        };
    };

    let commit_id = commit_result
        .get("commit_id")
        .or_else(|| commit_result.get("entry_id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();

    // Step 3: Attribute (sweetGrass) — best effort
    let braid_id = capability_call(
        &socket,
        "provenance",
        "create_braid",
        &serde_json::json!({
            "commit_ref": commit_id,
            "agents": [{
                "did": "did:key:healthSpring",
                "role": "author",
                "contribution": 1.0,
            }],
        }),
    )
    .ok()
    .and_then(|r| {
        r.get("braid_id")
            .or_else(|| r.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    })
    .unwrap_or_default();

    DataProvenanceChain {
        status: "complete".into(),
        session_id: session_id.into(),
        merkle_root,
        commit_id,
        braid_id,
    }
}

/// Check if the provenance trio is available (Neural API socket exists).
#[must_use]
pub fn trio_available() -> bool {
    data_provider_socket_path().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trio_degrades_gracefully() {
        let result = begin_data_session("test_dataset");
        // Without biomeOS running, should return unavailable but not panic
        if !result.available {
            assert!(result.id.starts_with("local-"));
        }
    }

    #[test]
    fn record_step_degrades_gracefully() {
        let result = record_fetch_step(
            "nonexistent-session",
            &serde_json::json!({ "source": "test" }),
        );
        if !result.available {
            assert_eq!(result.id, "unavailable");
        }
    }

    #[test]
    fn complete_session_degrades_gracefully() {
        let chain = complete_data_session("nonexistent-session", "AGPL-3.0-or-later");
        assert_eq!(chain.status, "unavailable");
        assert!(chain.merkle_root.is_empty());
    }

    #[test]
    fn trio_available_check() {
        let _available = trio_available();
    }

    #[test]
    fn provenance_chain_serializes() {
        let chain = DataProvenanceChain {
            status: "complete".into(),
            session_id: "test-123".into(),
            merkle_root: "abc123".into(),
            commit_id: "def456".into(),
            braid_id: "ghi789".into(),
        };
        let json = serde_json::to_string(&chain);
        assert!(json.is_ok());
        let s = json.unwrap_or_default();
        assert!(s.contains("merkle_root"));
        assert!(s.contains("complete"));
    }

    #[test]
    fn unavailable_result_has_local_prefix() {
        let r = unavailable_result("local-test");
        assert!(!r.available);
        assert_eq!(r.id, "local-test");
    }
}
