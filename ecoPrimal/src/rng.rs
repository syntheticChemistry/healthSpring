// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic LCG PRNG for reproducible simulations.
//!
//! Delegates to upstream `barracuda::rng` for the canonical LCG
//! implementation. healthSpring adds domain-specific wrappers
//! (Box-Muller normal sampling) used by PK/PD Monte Carlo and
//! endocrine models.
//!
//! ## Seed Strategy
//!
//! **All** stochastic computations in healthSpring use this module's
//! deterministic LCG, seeded with an explicit `u64` value. Seeds are:
//!
//! - **Hardcoded per experiment** — each experiment binary documents its
//!   seed alongside its provenance (e.g. `seed = 42` in exp005, exp036).
//! - **Fixed for validation** — rerunning with the same seed produces
//!   bitwise-identical results (verified by determinism tests).
//! - **Documented in `specs/TOLERANCE_REGISTRY.md`** — stochastic checks
//!   use population/statistical tolerances that assume a known seed and
//!   sample size (√(1/N) convergence).
//!
//! **GPU parity**: The WGSL Monte Carlo shaders use a `u32` Wang hash
//! (not this LCG) because WGSL lacks `u64` on some backends. GPU seeds
//! derive from the same experiment seed via truncation. See
//! `gpu::wang_hash_uniform`.
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

        let (z1b, _) = normal_sample(state1);
        let (z2b, _) = normal_sample(state2);
        assert_eq!(z1b.to_bits(), z2b.to_bits(), "chain continues identically");
    }

    /// Determinism: run the same 1000-element LCG sequence twice and assert
    /// bitwise identity across all values.
    #[test]
    fn lcg_sequence_bitwise_determinism() {
        let seed = 7_u64;
        let run = |s: u64| -> Vec<u64> {
            let mut state = s;
            (0..1000)
                .map(|_| {
                    state = lcg_step(state);
                    state
                })
                .collect()
        };
        let a = run(seed);
        let b = run(seed);
        assert_eq!(a, b, "1000-step LCG must be bitwise identical across runs");
    }

    /// Determinism: Box-Muller chain of 500 normal samples must match
    /// bitwise across two runs from the same seed.
    #[test]
    fn normal_chain_bitwise_determinism() {
        let seed = 999_u64;
        let run = |s: u64| -> (Vec<u64>, Vec<u64>) {
            let mut state = s;
            let mut bits = Vec::with_capacity(500);
            let mut states = Vec::with_capacity(500);
            for _ in 0..500 {
                let (z, next) = normal_sample(state);
                bits.push(z.to_bits());
                states.push(next);
                state = next;
            }
            (bits, states)
        };
        let (bits_a, states_a) = run(seed);
        let (bits_b, states_b) = run(seed);
        assert_eq!(bits_a, bits_b, "normal samples must be bitwise identical");
        assert_eq!(states_a, states_b, "RNG states must match across runs");
    }

    /// Determinism: Hill dose-response over a seeded population sweep must
    /// produce bitwise-identical AUC sequences on rerun.
    #[test]
    fn population_pk_determinism() {
        let seed = 42_u64;
        let run = |s: u64| -> Vec<u64> {
            let mut state = s;
            let mut aucs = Vec::with_capacity(200);
            for _ in 0..200 {
                state = lcg_step(state);
                let u = state_to_f64(state);
                let cl = 10.0 * (0.5 + u);
                let auc = 0.8 * 100.0 / cl;
                aucs.push(auc.to_bits());
            }
            aucs
        };
        let a = run(seed);
        let b = run(seed);
        assert_eq!(a, b, "population PK AUCs must be bitwise identical");
    }
}
