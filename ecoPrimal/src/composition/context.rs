// SPDX-License-Identifier: AGPL-3.0-or-later

//! Health-domain extensions to [`CompositionContext`].

use primalspring::composition::CompositionContext;
use primalspring::ipc::IpcError;

/// Health-domain wrapper around [`CompositionContext`] providing typed
/// accessors for barraCuda statistics, provenance operations, and
/// domain-specific parity validation.
pub struct HealthCompositionContext {
    inner: CompositionContext,
}

#[allow(
    clippy::missing_const_for_fn,
    reason = "wrapper mirrors CompositionContext constructors and accessors without forcing const API surface"
)]
impl HealthCompositionContext {
    /// Wrap an existing [`CompositionContext`].
    #[must_use]
    pub fn new(ctx: CompositionContext) -> Self {
        Self { inner: ctx }
    }

    /// Discover a full composition using primalSpring's escalation hierarchy.
    #[must_use]
    pub fn discover() -> Self {
        Self::new(CompositionContext::discover())
    }

    /// Discover with TCP fallback (no Songbird tier).
    #[must_use]
    pub fn from_live_discovery_with_fallback() -> Self {
        Self::new(CompositionContext::from_live_discovery_with_fallback())
    }

    /// Access the inner [`CompositionContext`].
    pub fn inner(&mut self) -> &mut CompositionContext {
        &mut self.inner
    }

    /// Compute mean via barraCuda IPC (`stats.mean`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if tensor capability unavailable or call fails.
    pub fn stats_mean(&mut self, data: &[f64]) -> Result<f64, IpcError> {
        self.inner.call_f64_flex(
            "tensor",
            "stats.mean",
            serde_json::json!({"data": data}),
            &["mean", "result"],
        )
    }

    /// Compute standard deviation via barraCuda IPC (`stats.std_dev`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if tensor capability unavailable or call fails.
    pub fn stats_std_dev(&mut self, data: &[f64]) -> Result<f64, IpcError> {
        self.inner.call_f64_flex(
            "tensor",
            "stats.std_dev",
            serde_json::json!({"data": data}),
            &["std_dev", "result"],
        )
    }

    /// Compute variance via barraCuda IPC (`stats.variance`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if tensor capability unavailable or call fails.
    pub fn stats_variance(&mut self, data: &[f64]) -> Result<f64, IpcError> {
        self.inner.call_f64_flex(
            "tensor",
            "stats.variance",
            serde_json::json!({"data": data}),
            &["variance", "result"],
        )
    }

    /// Compute correlation via barraCuda IPC (`stats.correlation`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if tensor capability unavailable or call fails.
    pub fn stats_correlation(&mut self, x: &[f64], y: &[f64]) -> Result<f64, IpcError> {
        self.inner.call_f64_flex(
            "tensor",
            "stats.correlation",
            serde_json::json!({"x": x, "y": y}),
            &["correlation", "result"],
        )
    }

    /// Health check for a specific capability.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if capability unavailable or health check fails.
    pub fn health_check(&mut self, capability: &str) -> Result<bool, IpcError> {
        self.inner.health_check(capability)
    }

    /// All available capabilities in this composition.
    #[must_use]
    pub fn available_capabilities(&self) -> Vec<&str> {
        self.inner.available_capabilities()
    }

    /// Whether a capability is available.
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.inner.has_capability(capability)
    }
}
