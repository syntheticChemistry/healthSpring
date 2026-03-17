// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed client for toadStool `compute.dispatch.*` protocol.
//!
//! Absorbed from ludoSpring V22 / toadStool S156 patterns. Provides typed
//! wrappers around `compute.dispatch.submit`, `compute.dispatch.result`,
//! and `compute.dispatch.capabilities` for real-time GPU dispatch via IPC.
//!
//! Discovery is capability-based — no hardcoded primal names.

use super::rpc;
use super::socket;

/// Result of a dispatch submission — contains the job ID for polling.
#[derive(Debug, Clone)]
pub struct DispatchHandle {
    pub job_id: String,
    pub compute_socket: std::path::PathBuf,
}

/// Error from dispatch operations.
#[derive(Debug)]
pub enum DispatchError {
    NoComputePrimal,
    Send(rpc::SendError),
    MissingJobId,
    JobFailed(String),
}

impl core::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoComputePrimal => write!(f, "no compute primal discovered"),
            Self::Send(e) => write!(f, "dispatch send: {e}"),
            Self::MissingJobId => write!(f, "response missing job_id"),
            Self::JobFailed(msg) => write!(f, "job failed: {msg}"),
        }
    }
}

/// Submit a GPU compute job to the discovered compute primal.
///
/// # Errors
///
/// Returns [`DispatchError`] if no compute primal is available, the RPC
/// call fails, or the response lacks a `job_id` field.
pub fn submit(
    workload_type: &str,
    params: &serde_json::Value,
) -> Result<DispatchHandle, DispatchError> {
    let compute_socket = socket::discover_compute_primal().ok_or(DispatchError::NoComputePrimal)?;

    let result = rpc::try_send(
        &compute_socket,
        "compute.dispatch.submit",
        &serde_json::json!({
            "workload": workload_type,
            "params": params,
        }),
    )
    .map_err(DispatchError::Send)?;

    let job_id = result
        .get("job_id")
        .and_then(serde_json::Value::as_str)
        .ok_or(DispatchError::MissingJobId)?
        .to_owned();

    Ok(DispatchHandle {
        job_id,
        compute_socket,
    })
}

/// Poll for the result of a previously submitted dispatch job.
///
/// # Errors
///
/// Returns [`DispatchError`] if the RPC call fails or the job errored.
pub fn result(handle: &DispatchHandle) -> Result<serde_json::Value, DispatchError> {
    let resp = rpc::try_send(
        &handle.compute_socket,
        "compute.dispatch.result",
        &serde_json::json!({ "job_id": handle.job_id }),
    )
    .map_err(DispatchError::Send)?;

    if let Some(err) = resp.get("error").and_then(serde_json::Value::as_str) {
        return Err(DispatchError::JobFailed(err.to_owned()));
    }

    Ok(resp)
}

/// Query compute capabilities from the discovered compute primal.
///
/// Returns the list of supported workload types, or an empty vec if
/// no compute primal is available.
#[must_use]
pub fn capabilities() -> Vec<String> {
    let Some(compute_socket) = socket::discover_compute_primal() else {
        return Vec::new();
    };
    let Ok(result) = rpc::try_send(
        &compute_socket,
        "compute.dispatch.capabilities",
        &serde_json::json!({}),
    ) else {
        return Vec::new();
    };
    result
        .get("workloads")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(clippy::unwrap_used, reason = "test code")]
    fn submit_fails_without_compute_primal() {
        let result = submit("hill_sweep", &serde_json::json!({"n": 1000}));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DispatchError::NoComputePrimal
        ));
    }

    #[test]
    fn capabilities_returns_empty_without_primal() {
        let caps = capabilities();
        assert!(caps.is_empty());
    }

    #[test]
    fn dispatch_error_display() {
        let err = DispatchError::NoComputePrimal;
        assert_eq!(err.to_string(), "no compute primal discovered");
        let err = DispatchError::JobFailed("timeout".to_owned());
        assert_eq!(err.to_string(), "job failed: timeout");
    }
}
