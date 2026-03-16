// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic LCG PRNG for reproducible simulations.
//!
//! Delegates to upstream `barracuda::rng` for the canonical LCG
//! implementation. healthSpring adds domain-specific wrappers
//! (Box-Muller normal sampling) used by PK/PD Monte Carlo and
//! endocrine models.
//!
//! ## Cross-spring provenance
//!
//! The LCG constants and step function originated in healthSpring V3
//! and were absorbed into barraCuda during the absorption sprint.
//! This module now consumes the canonical upstream version.

pub use barracuda::rng::{LCG_MULTIPLIER, lcg_step, state_to_f64, uniform_f64_sequence};

use crate::tolerances::BOX_MULLER_CLAMP;

/// Box-Muller transform: two uniform `[0,1)` samples to one standard normal.
///
/// Clamps `u1` away from zero to avoid `ln(0)`.
#[must_use]
#[inline]
pub fn box_muller(u1: f64, u2: f64) -> f64 {
    (-2.0 * u1.max(BOX_MULLER_CLAMP).ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Draw one standard normal variate, advancing LCG state twice.
///
/// Returns `(z, new_state)` where `z ~ N(0,1)`.
#[must_use]
pub fn normal_sample(state: u64) -> (f64, u64) {
    let s1 = lcg_step(state);
    let u1 = state_to_f64(s1);
    let s2 = lcg_step(s1);
    let u2 = state_to_f64(s2);
    (box_muller(u1, u2), s2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_same_seed_same_sequence() {
        let seed = 42_u64;
        let s1a = lcg_step(seed);
        let s1b = lcg_step(seed);
        assert_eq!(s1a, s1b, "same seed must produce same next state");

        let u1a = state_to_f64(s1a);
        let u1b = state_to_f64(s1b);
        assert_eq!(u1a.to_bits(), u1b.to_bits(), "same state → same uniform");

        let s2a = lcg_step(s1a);
        let s2b = lcg_step(s1b);
        assert_eq!(s2a, s2b, "sequence continues identically");
    }

    #[test]
    fn lcg_different_seeds_different_sequences() {
        let s1 = lcg_step(42_u64);
        let s2 = lcg_step(43_u64);
        assert_ne!(s1, s2, "different seeds must produce different states");

        let u1 = state_to_f64(s1);
        let u2 = state_to_f64(s2);
        assert_ne!(
            u1.to_bits(),
            u2.to_bits(),
            "different states → different uniforms"
        );
    }

    #[test]
    fn normal_sample_deterministic() {
        let seed = 123_456_u64;
        let (z1, state1) = normal_sample(seed);
        let (z2, state2) = normal_sample(seed);
        assert_eq!(z1.to_bits(), z2.to_bits(), "same seed → same normal sample");
        assert_eq!(state1, state2, "same seed → same next state");

        // Second sample from same chain
        let (z1b, _) = normal_sample(state1);
        let (z2b, _) = normal_sample(state2);
        assert_eq!(z1b.to_bits(), z2b.to_bits(), "chain continues identically");
    }
}
