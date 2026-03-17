// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC resilience: circuit breaker and retry policy.
//!
//! Follows the airSpring pattern for cross-ecosystem consistency.
//! Circuit breaker prevents cascading failures; retry policy provides
//! exponential backoff for transient errors.

use std::time::{Duration, Instant};

/// Circuit breaker states.
///
/// - **`Closed`**: Normal operation; requests are allowed.
/// - **`Open`**: Too many failures; requests are rejected until cooldown expires.
/// - **`HalfOpen`**: Cooldown expired; one probe request allowed to test recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests allowed.
    Closed,
    /// Failures exceeded threshold — requests rejected.
    Open,
    /// Cooldown expired — one probe allowed.
    HalfOpen,
}

/// Circuit breaker for IPC connections.
///
/// Tracks consecutive failures and opens the circuit when the threshold
/// is exceeded. After a cooldown period, transitions to half-open to
/// allow a probe request.
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with the given failure threshold and cooldown.
    ///
    /// # Arguments
    ///
    /// * `failure_threshold` - Number of consecutive failures before opening.
    /// * `cooldown` - Duration to wait before allowing a probe (half-open).
    #[must_use]
    pub const fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            last_failure: None,
            cooldown,
        }
    }

    /// Records a successful request.
    ///
    /// Resets failure count when closed; transitions from half-open to closed.
    #[expect(clippy::missing_const_for_fn, reason = "requires &mut self")]
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Closed;
                self.failure_count = 0;
            }
            CircuitState::Open => {
                // No change — must wait for cooldown to transition to half-open
            }
        }
    }

    /// Records a failed request.
    ///
    /// Increments failure count when closed; opens when threshold reached.
    /// Transitions from half-open back to open.
    pub fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.failure_count = self.failure_threshold;
            }
            CircuitState::Open => {
                // Reset cooldown on additional failure
                self.last_failure = Some(Instant::now());
            }
        }
    }

    /// Returns whether a request may be attempted.
    ///
    /// - `Closed`: always `true`
    /// - `Open`: `false` until cooldown expires
    /// - `HalfOpen`: `true` (one probe allowed)
    #[must_use]
    pub fn can_attempt(&self) -> bool {
        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                let Some(last) = self.last_failure else {
                    return true;
                };
                last.elapsed() >= self.cooldown
            }
        }
    }

    /// Updates state when cooldown expires (open → half-open).
    ///
    /// Call this before `can_attempt` if you need to transition open→half-open.
    /// Alternatively, `can_attempt` will return true once cooldown has passed,
    /// but the state remains Open until the next `record_success` or
    /// `record_failure`. For strict state accuracy, call this periodically.
    pub fn tick(&mut self) {
        if self.state == CircuitState::Open {
            if let Some(last) = self.last_failure {
                if last.elapsed() >= self.cooldown {
                    self.state = CircuitState::HalfOpen;
                }
            }
        }
    }

    /// Returns the current circuit state.
    #[must_use]
    pub const fn state(&self) -> CircuitState {
        self.state
    }
}

/// Retry policy with exponential backoff.
///
/// Delays are capped by `max_delay` to avoid excessive wait times.
pub struct RetryPolicy {
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
}

impl RetryPolicy {
    /// Creates a new retry policy.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of retry attempts (0 = no retries).
    /// * `initial_delay` - Delay before first retry.
    /// * `max_delay` - Cap on delay (avoids unbounded backoff).
    #[must_use]
    pub const fn new(max_retries: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
        }
    }

    /// Returns the delay for the given attempt number.
    ///
    /// Attempt 0 is the first retry. Returns `None` if attempt exceeds
    /// `max_retries` (no more retries).
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }
        let factor = 2_u32.saturating_pow(attempt);
        let delay = self
            .initial_delay
            .saturating_mul(factor)
            .min(self.max_delay);
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_attempt());
    }

    #[test]
    fn circuit_breaker_opens_after_threshold() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_attempt());
    }

    #[test]
    fn circuit_breaker_success_resets_count() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        // Count was reset — need 3 more failures to open
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn circuit_breaker_half_open_after_cooldown() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(20));
        cb.tick();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.can_attempt());
    }

    #[test]
    fn circuit_breaker_half_open_success_closes() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));
        cb.tick();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn circuit_breaker_half_open_failure_reopens() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));
        cb.tick();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn retry_policy_first_attempt() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100), Duration::from_secs(5));
        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
    }

    #[test]
    fn retry_policy_exponential_backoff() {
        let policy = RetryPolicy::new(5, Duration::from_millis(100), Duration::from_secs(10));
        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(1),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(400))
        );
    }

    #[test]
    fn retry_policy_caps_at_max_delay() {
        let policy = RetryPolicy::new(10, Duration::from_secs(1), Duration::from_secs(5));
        let delay = policy.delay_for_attempt(5);
        assert_eq!(delay, Some(Duration::from_secs(5)));
    }

    #[test]
    fn retry_policy_returns_none_when_exhausted() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100), Duration::from_secs(5));
        assert!(policy.delay_for_attempt(3).is_none());
        assert!(policy.delay_for_attempt(4).is_none());
    }

    #[test]
    fn retry_policy_zero_retries() {
        let policy = RetryPolicy::new(0, Duration::from_millis(100), Duration::from_secs(5));
        assert!(policy.delay_for_attempt(0).is_none());
    }
}
