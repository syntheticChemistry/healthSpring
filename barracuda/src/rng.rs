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

/// Minimum value for Box-Muller `u1` to avoid `ln(0)`.
const BOX_MULLER_CLAMP: f64 = 1e-30;

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
