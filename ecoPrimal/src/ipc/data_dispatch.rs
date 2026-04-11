// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed client for data primal `data.*` protocol.
//!
//! Mirrors `compute_dispatch.rs` for consistency. Discovery is
//! capability-based — no hardcoded primal names.

use super::rpc;
use super::socket;

/// Error from data dispatch operations.
#[derive(Debug)]
pub enum DataError {
    /// No data primal socket was discovered.
    NoDataPrimal,
    /// RPC send failed (transport/codec).
    Send(rpc::SendError),
    /// Response body was missing or malformed for this operation.
    InvalidResponse(String),
}

impl core::fmt::Display for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoDataPrimal => write!(f, "no data primal discovered"),
            Self::Send(e) => write!(f, "data dispatch send: {e}"),
            Self::InvalidResponse(msg) => write!(f, "invalid response: {msg}"),
        }
    }
}

/// Fetch data from the discovered data primal.
///
/// # Errors
///
/// Returns [`DataError`] if no data primal is available or the RPC fails.
pub fn fetch(source: &str, query: &serde_json::Value) -> Result<serde_json::Value, DataError> {
    let data_socket = socket::discover_data_primal().ok_or(DataError::NoDataPrimal)?;
    let method = format!("data.{source}_fetch");
    rpc::resilient_send(&data_socket, &method, query).map_err(DataError::Send)
}

/// Store data via the discovered data primal.
///
/// # Errors
///
/// Returns [`DataError`] if no data primal is available or the RPC fails.
pub fn store(key: &str, value: &serde_json::Value) -> Result<serde_json::Value, DataError> {
    let data_socket = socket::discover_data_primal().ok_or(DataError::NoDataPrimal)?;
    rpc::resilient_send(
        &data_socket,
        "data.storage.store",
        &serde_json::json!({ "key": key, "value": value }),
    )
    .map_err(DataError::Send)
}

/// Query data capabilities from the discovered data primal.
#[must_use]
pub fn capabilities() -> Vec<String> {
    let Some(data_socket) = socket::discover_data_primal() else {
        return Vec::new();
    };
    let Ok(result) = rpc::try_send(&data_socket, "capability.list", &serde_json::json!({})) else {
        return Vec::new();
    };
    socket::extract_capability_strings(&result)
        .into_iter()
        .filter(|s| s.starts_with("data."))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_fails_without_data_primal() {
        let result = fetch("ncbi", &serde_json::json!({"query": "test"}));
        assert!(matches!(result, Err(DataError::NoDataPrimal)));
    }

    #[test]
    fn capabilities_returns_empty_without_primal() {
        assert!(capabilities().is_empty());
    }

    #[test]
    fn data_error_display() {
        let err = DataError::NoDataPrimal;
        assert_eq!(err.to_string(), "no data primal discovered");
    }
}
