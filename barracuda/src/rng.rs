// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic LCG PRNG for reproducible simulations.
//!
//! This is a Knuth LCG with 64-bit state. All modules that need
//! deterministic pseudo-random sequences should use this constant
//! and step function rather than hardcoding the multiplier.

/// Knuth LCG multiplier (64-bit).
pub const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;

/// Advance the LCG state by one step.
#[must_use]
#[inline]
pub const fn lcg_step(state: u64) -> u64 {
    state.wrapping_mul(LCG_MULTIPLIER).wrapping_add(1)
}

/// Extract a `[0, 1)` f64 from the upper 31 bits of a 64-bit state.
///
/// # Example
///
/// ```
/// use healthspring_barracuda::rng::{lcg_step, state_to_f64};
///
/// let state = lcg_step(42);
/// let value = state_to_f64(state);
/// assert!((0.0..1.0).contains(&value));
/// ```
#[must_use]
#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "upper-31-bit extraction fits f64"
)]
pub fn state_to_f64(state: u64) -> f64 {
    (state >> 33) as f64 / f64::from(u32::MAX)
}

/// Box-Muller transform: two uniform `[0,1)` samples to one standard normal.
///
/// Clamps `u1` away from zero to avoid `ln(0)`.
#[must_use]
#[inline]
pub fn box_muller(u1: f64, u2: f64) -> f64 {
    (-2.0 * u1.max(1e-30).ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
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
