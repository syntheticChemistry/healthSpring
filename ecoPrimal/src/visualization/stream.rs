// SPDX-License-Identifier: AGPL-3.0-only
//! Streaming session manager for live petalTongue visualization.
//!
//! Wraps `PetalTonguePushClient` with session lifecycle, typed push helpers
//! for clinical data (ECG, HRV, PK), and backpressure handling.

use std::time::{Duration, Instant};

use super::ipc_push::{PetalTonguePushClient, PushResult};
use super::types::HealthScenario;

/// Configuration for backpressure handling.
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum time to wait for a single push before considering it slow.
    pub push_timeout: Duration,
    /// How long to pause after a slow push to let petalTongue catch up.
    pub cooldown: Duration,
    /// Number of consecutive slow pushes before entering cooldown.
    pub slow_threshold: u32,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            push_timeout: Duration::from_millis(500),
            cooldown: Duration::from_millis(200),
            slow_threshold: 3,
        }
    }
}

/// Statistics for a streaming session.
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total frames pushed.
    pub frames_pushed: u64,
    /// Total push errors encountered.
    pub errors: u64,
    /// Number of times backpressure cooldown was triggered.
    pub cooldowns: u64,
    /// Total time spent in push operations.
    pub total_push_time: Duration,
}

impl StreamStats {
    /// Average push latency, or `None` if no frames pushed.
    #[must_use]
    pub fn avg_push_latency(&self) -> Option<Duration> {
        if self.frames_pushed == 0 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "avg_push_latency is informational; frame counts won't exceed u32::MAX in practice"
        )]
        Some(self.total_push_time / (self.frames_pushed as u32))
    }
}

/// A streaming session to petalTongue with backpressure management.
///
/// Use `StreamSession::discover()` or `StreamSession::new()` to create,
/// then call typed push methods for ECG, HRV, PK, and gauge data.
pub struct StreamSession {
    client: PetalTonguePushClient,
    session_id: String,
    config: BackpressureConfig,
    stats: StreamStats,
    consecutive_slow: u32,
}

impl StreamSession {
    /// Discover petalTongue and start a streaming session.
    ///
    /// # Errors
    ///
    /// Returns `PushError::NotFound` if no petalTongue socket is found.
    pub fn discover(session_id: &str) -> PushResult<Self> {
        let client = PetalTonguePushClient::discover()?;
        Ok(Self {
            client,
            session_id: session_id.into(),
            config: BackpressureConfig::default(),
            stats: StreamStats::default(),
            consecutive_slow: 0,
        })
    }

    /// Create a session with an explicit client.
    #[must_use]
    pub fn new(client: PetalTonguePushClient, session_id: &str) -> Self {
        Self {
            client,
            session_id: session_id.into(),
            config: BackpressureConfig::default(),
            stats: StreamStats::default(),
            consecutive_slow: 0,
        }
    }

    /// Override the default backpressure configuration.
    #[must_use]
    pub const fn with_backpressure(mut self, config: BackpressureConfig) -> Self {
        self.config = config;
        self
    }

    /// Push an initial full scenario render.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_initial_render(
        &mut self,
        title: &str,
        scenario: &HealthScenario,
    ) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_render(sid, title, scenario))
    }

    /// Push an ECG frame (time-amplitude samples).
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_ecg_frame(
        &mut self,
        binding_id: &str,
        times: &[f64],
        amplitudes: &[f64],
    ) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_append(sid, binding_id, times, amplitudes))
    }

    /// Push a PK concentration point.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_pk_point(
        &mut self,
        binding_id: &str,
        times: &[f64],
        concentrations: &[f64],
    ) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_append(sid, binding_id, times, concentrations))
    }

    /// Push rolling HRV gauge updates.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if any gauge push fails.
    pub fn push_hrv_update(&mut self, sdnn: f64, rmssd: f64, pnn50: f64) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_gauge_update(sid, "sdnn", sdnn))?;
        self.timed_push(|client, sid| client.push_gauge_update(sid, "rmssd", rmssd))?;
        self.timed_push(|client, sid| client.push_gauge_update(sid, "pnn50", pnn50))
    }

    /// Push a generic gauge update.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_gauge(&mut self, binding_id: &str, value: f64) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_gauge_update(sid, binding_id, value))
    }

    /// Push a generic timeseries append.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_timeseries(&mut self, binding_id: &str, x: &[f64], y: &[f64]) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_append(sid, binding_id, x, y))
    }

    /// Replace an entire binding in-place.
    ///
    /// Works with any `DataChannel` type — `Heatmap`, `Bar`, `Scatter3D`,
    /// `Distribution`, `Spectrum`, and also `TimeSeries`/`Gauge`. Use this when the
    /// entire dataset changes rather than appending incrementally.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_replace_binding(
        &mut self,
        binding_id: &str,
        binding: &super::types::DataChannel,
    ) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_replace(sid, binding_id, binding))
    }

    /// Push a full render with explicit domain and `UiConfig` passthrough.
    ///
    /// Use for clinical scenarios that carry panel/zoom/theme overrides.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the push fails.
    pub fn push_render_with_domain(
        &mut self,
        title: &str,
        scenario: &super::types::HealthScenario,
        domain: &str,
    ) -> PushResult<()> {
        self.timed_push(|client, sid| client.push_render_with_config(sid, title, scenario, domain))
    }

    /// Current session statistics.
    #[must_use]
    pub const fn stats(&self) -> &StreamStats {
        &self.stats
    }

    /// Query petalTongue's rendering capabilities.
    ///
    /// Returns the set of supported channel types, geometry, and features
    /// so healthSpring can adapt its data output to the available renderer.
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the RPC call fails.
    pub fn query_capabilities(&self) -> PushResult<serde_json::Value> {
        self.client.query_capabilities()
    }

    /// Subscribe to user interaction events from petalTongue.
    ///
    /// When a user selects, focuses, or filters data in petalTongue, events
    /// are delivered via JSON-RPC notification to `callback_method`.
    ///
    /// Returns the subscription response (contains `subscription_id`).
    ///
    /// # Errors
    ///
    /// Returns `PushError` if the subscription RPC fails.
    pub fn subscribe_interactions(
        &self,
        events: &[&str],
        callback_method: &str,
    ) -> PushResult<serde_json::Value> {
        self.client
            .subscribe_interactions(&self.session_id, events, callback_method)
    }

    /// Session ID.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    fn timed_push<F>(&mut self, op: F) -> PushResult<()>
    where
        F: FnOnce(&PetalTonguePushClient, &str) -> PushResult<()>,
    {
        let start = Instant::now();
        let result = op(&self.client, &self.session_id);
        let elapsed = start.elapsed();

        self.stats.total_push_time += elapsed;

        match &result {
            Ok(()) => {
                self.stats.frames_pushed += 1;
                if elapsed > self.config.push_timeout {
                    self.consecutive_slow += 1;
                    if self.consecutive_slow >= self.config.slow_threshold {
                        self.stats.cooldowns += 1;
                        std::thread::sleep(self.config.cooldown);
                        self.consecutive_slow = 0;
                    }
                } else {
                    self.consecutive_slow = 0;
                }
            }
            Err(_) => {
                self.stats.errors += 1;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::ipc_push::PushError;
    use std::path::PathBuf;

    #[test]
    fn backpressure_config_defaults() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.push_timeout, Duration::from_millis(500));
        assert_eq!(cfg.cooldown, Duration::from_millis(200));
        assert_eq!(cfg.slow_threshold, 3);
    }

    #[test]
    fn stream_stats_defaults() {
        let stats = StreamStats::default();
        assert_eq!(stats.frames_pushed, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.cooldowns, 0);
        assert_eq!(stats.total_push_time, Duration::ZERO);
    }

    #[test]
    fn stream_stats_avg_latency_empty() {
        let stats = StreamStats::default();
        assert!(stats.avg_push_latency().is_none());
    }

    #[test]
    #[expect(clippy::expect_used, reason = "test code")]
    fn stream_stats_avg_latency_computed() {
        let stats = StreamStats {
            frames_pushed: 10,
            total_push_time: Duration::from_millis(100),
            ..Default::default()
        };
        let avg = stats.avg_push_latency().expect("should have avg");
        assert_eq!(avg, Duration::from_millis(10));
    }

    #[test]
    fn session_new_stores_id() {
        let client = PetalTonguePushClient::new(PathBuf::from("/tmp/test.sock"));
        let session = StreamSession::new(client, "test-session");
        assert_eq!(session.session_id(), "test-session");
        assert_eq!(session.stats().frames_pushed, 0);
    }

    #[test]
    fn session_with_backpressure() {
        let client = PetalTonguePushClient::new(PathBuf::from("/tmp/test.sock"));
        let cfg = BackpressureConfig {
            push_timeout: Duration::from_millis(100),
            cooldown: Duration::from_millis(50),
            slow_threshold: 5,
        };
        let session = StreamSession::new(client, "bp-test").with_backpressure(cfg);
        assert_eq!(session.config.slow_threshold, 5);
        assert_eq!(session.config.cooldown, Duration::from_millis(50));
    }

    #[test]
    fn session_push_to_missing_socket_records_error() {
        let client = PetalTonguePushClient::new(PathBuf::from("/tmp/nonexistent_hs_stream.sock"));
        let mut session = StreamSession::new(client, "err-test");
        let result = session.push_gauge("test_gauge", 42.0);
        assert!(result.is_err());
        assert_eq!(session.stats().errors, 1);
        assert_eq!(session.stats().frames_pushed, 0);
    }

    #[test]
    fn session_discover_not_found() {
        let result = StreamSession::discover("test");
        if result.is_ok() {
            return;
        }
        assert!(matches!(result, Err(PushError::NotFound(_))));
    }
}
