// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    deprecated,
    reason = "compute dispatch discovers sockets via legacy helpers until CompositionContext integration"
)]
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
    /// Opaque job identifier returned by `compute.dispatch.submit`.
    pub job_id: String,
    /// Socket path of the compute primal handling this job.
    pub compute_socket: std::path::PathBuf,
}

/// Error from dispatch operations.
#[derive(Debug)]
pub enum DispatchError {
    /// No compute primal was discovered on this host.
    NoComputePrimal,
    /// RPC send failed (transport/codec).
    Send(rpc::SendError),
    /// Response omitted `job_id`.
    MissingJobId,
    /// Remote reported `error` for the job.
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

    let result = rpc::resilient_send(
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
    let resp = rpc::resilient_send(
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

/// Pre-flight validation report from `toadstool.validate`.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether the workload is compatible with the current environment.
    pub valid: bool,
    /// Whether a GPU is available for dispatch.
    pub gpu_available: bool,
    /// Recommended precision tier (e.g. "DF64", "FP32").
    pub precision_tier: String,
    /// Estimated dispatch time in milliseconds.
    pub estimated_dispatch_time_ms: u64,
    /// Any warnings about the workload configuration.
    pub warnings: Vec<String>,
    /// Capabilities required by the workload.
    pub required_capabilities: Vec<String>,
}

/// Validate a workload TOML against the current compute environment
/// without executing it. Wraps `toadstool.validate` (Tier 2 Live Science API).
///
/// # Errors
///
/// Returns [`DispatchError`] if no compute primal is available or the RPC
/// call fails.
pub fn validate_workload(workload_path: &str) -> Result<ValidationReport, DispatchError> {
    let compute_socket = socket::discover_compute_primal().ok_or(DispatchError::NoComputePrimal)?;

    let result = rpc::try_send(
        &compute_socket,
        "toadstool.validate",
        &serde_json::json!({
            "workload_path": workload_path,
            "dry_run": true,
        }),
    )
    .map_err(DispatchError::Send)?;

    Ok(ValidationReport {
        valid: result.get("valid").and_then(serde_json::Value::as_bool).unwrap_or(false),
        gpu_available: result
            .get("gpu_available")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        precision_tier: result
            .get("precision_tier")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_owned(),
        estimated_dispatch_time_ms: result
            .get("estimated_dispatch_time_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        warnings: result
            .get("warnings")
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
            .unwrap_or_default(),
        required_capabilities: result
            .get("required_capabilities")
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
            .unwrap_or_default(),
    })
}

/// Query available workloads from the compute primal.
/// Wraps `toadstool.list_workloads` (Tier 2 Live Science API).
///
/// Returns an empty vec if no compute primal is discovered.
#[must_use]
pub fn list_workloads() -> Vec<String> {
    let Some(compute_socket) = socket::discover_compute_primal() else {
        return Vec::new();
    };
    let Ok(result) = rpc::try_send(
        &compute_socket,
        "toadstool.list_workloads",
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
    fn submit_fails_without_compute_primal() {
        let result = submit("hill_sweep", &serde_json::json!({"n": 1000}));
        assert!(matches!(result, Err(DispatchError::NoComputePrimal)));
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

    #[test]
    fn validate_workload_fails_without_compute_primal() {
        let result = validate_workload("/path/to/workload.toml");
        assert!(matches!(result, Err(DispatchError::NoComputePrimal)));
    }

    #[test]
    fn list_workloads_returns_empty_without_primal() {
        let wl = list_workloads();
        assert!(wl.is_empty());
    }
}
