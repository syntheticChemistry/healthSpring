// SPDX-License-Identifier: AGPL-3.0-or-later
//! Clinical attention state machine with hysteresis.
//!
//! Absorbed from wetSpring V118 `AttentionState` pattern. A three-level
//! state machine (Healthy/Alert/Critical) with configurable escalation
//! and de-escalation thresholds prevents oscillation at decision boundaries.

use serde::{Deserialize, Serialize};

/// Clinical attention level for biosignal monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionState {
    Healthy,
    Alert,
    Critical,
}

/// Thresholds for state transitions with hysteresis.
#[derive(Debug, Clone)]
pub struct AttentionThresholds {
    /// Stress score above which we escalate Healthy → Alert.
    pub escalate_to_alert: f64,
    /// Stress score above which we escalate Alert → Critical.
    pub escalate_to_critical: f64,
    /// Stress score below which we de-escalate Critical → Alert.
    pub deescalate_from_critical: f64,
    /// Stress score below which we de-escalate Alert → Healthy.
    pub deescalate_to_healthy: f64,
}

impl Default for AttentionThresholds {
    fn default() -> Self {
        Self {
            escalate_to_alert: 0.6,
            escalate_to_critical: 0.8,
            deescalate_from_critical: 0.5,
            deescalate_to_healthy: 0.3,
        }
    }
}

impl AttentionThresholds {
    /// Compute the next state given current state and a normalized stress score [0, 1].
    #[must_use]
    pub fn next_state(&self, current: AttentionState, stress_score: f64) -> AttentionState {
        match current {
            AttentionState::Healthy => {
                if stress_score >= self.escalate_to_critical {
                    AttentionState::Critical
                } else if stress_score >= self.escalate_to_alert {
                    AttentionState::Alert
                } else {
                    AttentionState::Healthy
                }
            }
            AttentionState::Alert => {
                if stress_score >= self.escalate_to_critical {
                    AttentionState::Critical
                } else if stress_score <= self.deescalate_to_healthy {
                    AttentionState::Healthy
                } else {
                    AttentionState::Alert
                }
            }
            AttentionState::Critical => {
                if stress_score <= self.deescalate_to_healthy {
                    AttentionState::Healthy
                } else if stress_score <= self.deescalate_from_critical {
                    AttentionState::Alert
                } else {
                    AttentionState::Critical
                }
            }
        }
    }
}

/// Run the attention state machine over a sequence of stress scores.
#[must_use]
pub fn attention_trajectory(
    scores: &[f64],
    thresholds: &AttentionThresholds,
) -> Vec<AttentionState> {
    let mut state = AttentionState::Healthy;
    scores
        .iter()
        .map(|&s| {
            state = thresholds.next_state(state, s);
            state
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_stays_below_threshold() {
        let t = AttentionThresholds::default();
        assert_eq!(
            t.next_state(AttentionState::Healthy, 0.3),
            AttentionState::Healthy
        );
    }

    #[test]
    fn escalate_healthy_to_alert() {
        let t = AttentionThresholds::default();
        assert_eq!(
            t.next_state(AttentionState::Healthy, 0.7),
            AttentionState::Alert
        );
    }

    #[test]
    fn escalate_healthy_to_critical() {
        let t = AttentionThresholds::default();
        assert_eq!(
            t.next_state(AttentionState::Healthy, 0.9),
            AttentionState::Critical
        );
    }

    #[test]
    fn hysteresis_prevents_oscillation() {
        let t = AttentionThresholds::default();
        let state = t.next_state(AttentionState::Alert, 0.55);
        assert_eq!(state, AttentionState::Alert);
    }

    #[test]
    fn deescalate_critical_to_alert() {
        let t = AttentionThresholds::default();
        assert_eq!(
            t.next_state(AttentionState::Critical, 0.4),
            AttentionState::Alert
        );
    }

    #[test]
    fn deescalate_alert_to_healthy() {
        let t = AttentionThresholds::default();
        assert_eq!(
            t.next_state(AttentionState::Alert, 0.2),
            AttentionState::Healthy
        );
    }

    #[test]
    fn trajectory_transitions() {
        let t = AttentionThresholds::default();
        let scores = [0.1, 0.3, 0.7, 0.9, 0.6, 0.4, 0.2];
        let traj = attention_trajectory(&scores, &t);
        assert_eq!(traj[0], AttentionState::Healthy);
        assert_eq!(traj[3], AttentionState::Critical);
        assert_eq!(traj[6], AttentionState::Healthy);
    }
}
