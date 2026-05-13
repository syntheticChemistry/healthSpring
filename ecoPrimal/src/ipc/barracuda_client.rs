// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    deprecated,
    reason = "BarraCudaClient wraps deprecated PrimalClient pending HealthCompositionContext migration"
)]
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
#[deprecated(
    since = "0.10.0",
    note = "use HealthCompositionContext::stats_mean() etc. instead"
)]
pub struct BarraCudaClient {
    inner: PrimalClient,
}

#[allow(deprecated, reason = "implementation of deprecated type")]
impl BarraCudaClient {
    /// Connect to a barraCuda ecobin at the given socket path.
    #[must_use]
    pub fn new(socket: PathBuf) -> Self {
        Self {
            inner: PrimalClient::new(socket, crate::primal_names::BARRACUDA),
        }
    }

    /// Discover barraCuda via capability-first then name fallback.
    ///
    /// Tries `stats` capability discovery (any primal advertising `stats.*`
    /// methods), then falls back to the `barracuda` name-based socket scan.
    #[must_use]
    pub fn discover() -> Option<Self> {
        super::socket::discover_by_capability_public("stats")
            .or_else(|| super::socket::discover_primal(crate::primal_names::BARRACUDA))
            .map(Self::new)
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

    /// `stats.variance` — sample variance of `data`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the call fails or the response is unparseable.
    pub fn stats_variance(&self, data: &[f64]) -> Result<f64, IpcError> {
        let params = serde_json::json!({ "data": data });
        let resp = self.inner.try_call("stats.variance", &params)?;
        extract_f64(&resp)
    }

    /// `stats.correlation` — Pearson correlation of `x` and `y`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the call fails or the response is unparseable.
    pub fn stats_correlation(&self, x: &[f64], y: &[f64]) -> Result<f64, IpcError> {
        let params = serde_json::json!({ "x": x, "y": y });
        let resp = self.inner.try_call("stats.correlation", &params)?;
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

    /// `rng.normal` — batch Gaussian random samples (mean, standard deviation).
    ///
    /// Wire contract matches barraCuda `rng.normal`: `n`, `mean`, `std_dev`, `seed`.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if the call fails or the response is unparseable.
    pub fn rng_normal(
        &self,
        n: usize,
        mean: f64,
        std_dev: f64,
        seed: u64,
    ) -> Result<Vec<f64>, IpcError> {
        let params = serde_json::json!({
            "n": n,
            "mean": mean,
            "std_dev": std_dev,
            "seed": seed,
        });
        let resp = self.inner.try_call("rng.normal", &params)?;
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

    /// `precision.route` — query recommended precision tier for a physics domain.
    ///
    /// Wraps `barracuda.precision.route` (Tier 2 Live Science API). Returns the
    /// recommended precision tier, hardware hint, and compiler requirements for
    /// a given domain (e.g. `"population_pk"`, `"eigensolve"`, `"bioinformatics"`).
    ///
    /// # Errors
    ///
    /// Returns `IpcError` if `barraCuda` is unreachable or rejects the query.
    pub fn precision_route(
        &self,
        domain: &str,
        hardware_hint: Option<&str>,
    ) -> Result<PrecisionAdvisory, IpcError> {
        let mut params = serde_json::json!({ "domain": domain });
        if let Some(hint) = hardware_hint {
            params["hardware_hint"] = serde_json::Value::String(hint.to_owned());
        }
        let resp = self.inner.try_call("precision.route", &params)?;
        let result = rpc::extract_rpc_result(&resp).unwrap_or(&resp);
        Ok(PrecisionAdvisory {
            recommended_tier: result
                .get("recommended_tier")
                .or_else(|| result.get("tier"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("FP64")
                .to_owned(),
            fma_safe: result
                .get("fma_safe")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            requires_compiler: result
                .get("requires_compiler")
                .or_else(|| result.get("compiler_required"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            hardware_hint: result
                .get("hardware_hint")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("compute")
                .to_owned(),
        })
    }
}

/// Precision routing advisory from `barracuda.precision.route`.
///
/// Field names match the canonical wire contract in
/// `primalSpring/docs/LIVE_SCIENCE_API.md`.
#[derive(Debug, Clone)]
pub struct PrecisionAdvisory {
    /// Recommended precision tier (e.g. "F32", "F64", "DF64").
    pub recommended_tier: String,
    /// Whether fused multiply-add is safe for this domain at the recommended tier.
    pub fma_safe: bool,
    /// Whether a shader compiler (`coralReef`) is required for this tier.
    pub requires_compiler: bool,
    /// Hardware hint (e.g. "compute", "`tensor_core`").
    pub hardware_hint: String,
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
#[allow(deprecated, reason = "tests exercise deprecated BarraCudaClient")]
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
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn extract_f64_from_result_wrapper() {
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": std::f64::consts::PI, "id": 1});
        assert!((extract_f64(&resp).unwrap() - std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn extract_f64_from_bare_number() {
        let resp = serde_json::json!(std::f64::consts::E);
        assert!((extract_f64(&resp).unwrap() - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn extract_f64_from_empty_object_returns_error() {
        let resp = serde_json::json!({});
        assert!(extract_f64(&resp).is_err());
    }

    #[test]
    fn precision_route_fails_without_barracuda() {
        let client = BarraCudaClient::new(PathBuf::from("/tmp/barracuda-nonexistent.sock"));
        let result = client.precision_route("population_pk", None);
        assert!(result.is_err());
    }
}
