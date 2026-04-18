// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed IPC client for the barraCuda ecobin primal.
//!
//! barraCuda exposes 32 JSON-RPC methods over UDS. This client provides
//! typed wrappers for the `stats.*` and `rng.*` methods that healthSpring
//! consumes. Used by [`crate::math_dispatch`] behind the `primal-proof`
//! feature to route math through IPC instead of library imports.

use std::path::PathBuf;

use super::client::PrimalClient;
use super::error::IpcError;
use super::rpc;

/// Typed client for barraCuda's JSON-RPC surface.
pub struct BarraCudaClient {
    inner: PrimalClient,
}

impl BarraCudaClient {
    /// Connect to a barraCuda ecobin at the given socket path.
    #[must_use]
    pub fn new(socket: PathBuf) -> Self {
        Self {
            inner: PrimalClient::new(socket, "barracuda"),
        }
    }

    /// Discover barraCuda via socket dir scan.
    #[must_use]
    pub fn discover() -> Option<Self> {
        super::socket::discover_primal("barracuda").map(Self::new)
    }

    /// The underlying socket path.
    #[must_use]
    pub fn socket(&self) -> &std::path::Path {
        self.inner.socket()
    }

    /// `stats.mean` — arithmetic mean of `data`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the call fails or the response is unparseable.
    pub fn stats_mean(&self, data: &[f64]) -> Result<f64, IpcError> {
        let params = serde_json::json!({ "data": data });
        let resp = self.inner.try_call("stats.mean", &params)?;
        extract_f64(&resp)
    }

    /// `stats.std_dev` — standard deviation of `data`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the call fails or the response is unparseable.
    pub fn stats_std_dev(&self, data: &[f64]) -> Result<f64, IpcError> {
        let params = serde_json::json!({ "data": data });
        let resp = self.inner.try_call("stats.std_dev", &params)?;
        extract_f64(&resp)
    }

    /// `rng.uniform` — batch uniform random samples.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the call fails or the response is unparseable.
    pub fn rng_uniform(
        &self,
        n: usize,
        min: f64,
        max: f64,
        seed: u64,
    ) -> Result<Vec<f64>, IpcError> {
        let params = serde_json::json!({
            "n": n,
            "min": min,
            "max": max,
            "seed": seed,
        });
        let resp = self.inner.try_call("rng.uniform", &params)?;
        let result = rpc::extract_rpc_result(&resp).unwrap_or(&resp);
        result
            .as_array()
            .map(|arr| arr.iter().filter_map(serde_json::Value::as_f64).collect())
            .or_else(|| {
                result
                    .get("result")
                    .and_then(serde_json::Value::as_array)
                    .map(|arr| arr.iter().filter_map(serde_json::Value::as_f64).collect())
            })
            .ok_or(IpcError::EmptyResponse)
    }

    /// `health.liveness` probe on the barraCuda ecobin.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the probe fails.
    pub fn health_liveness(&self) -> Result<serde_json::Value, IpcError> {
        self.inner.health_liveness()
    }
}

fn extract_f64(resp: &serde_json::Value) -> Result<f64, IpcError> {
    rpc::extract_rpc_result(resp)
        .and_then(|r| {
            r.as_f64()
                .or_else(|| r.get("result").and_then(serde_json::Value::as_f64))
        })
        .or_else(|| resp.as_f64())
        .ok_or(IpcError::EmptyResponse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_option() {
        let _client = BarraCudaClient::discover();
    }

    #[test]
    fn new_sets_socket_and_name() {
        let client = BarraCudaClient::new(PathBuf::from("/tmp/barracuda-test.sock"));
        assert_eq!(
            client.socket(),
            std::path::Path::new("/tmp/barracuda-test.sock")
        );
    }

    #[test]
    fn extract_f64_from_result_wrapper() {
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": 3.14, "id": 1});
        assert!((extract_f64(&resp).unwrap() - 3.14).abs() < 1e-15);
    }

    #[test]
    fn extract_f64_from_bare_number() {
        let resp = serde_json::json!(2.718);
        assert!((extract_f64(&resp).unwrap() - 2.718).abs() < 1e-15);
    }

    #[test]
    fn extract_f64_from_empty_object_returns_error() {
        let resp = serde_json::json!({});
        assert!(extract_f64(&resp).is_err());
    }
}
