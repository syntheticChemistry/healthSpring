// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized math dispatch — routes barraCuda library calls through IPC
//! when the `primal-proof` feature is active.
//!
//! This module wraps every `barracuda::` call site in healthSpring. The
//! default path delegates to the library (Level 2 Rust proof). With
//! `--features primal-proof`, wire-ready methods route through the
//! barraCuda ecobin's JSON-RPC surface (Level 5 primal proof).
//!
//! ## Wire status
//!
//! | Method | Wire status | Feature-gated |
//! |--------|-------------|---------------|
//! | `stats.mean` | ON WIRE | Yes |
//! | `stats.std_dev` | ON WIRE | Yes |
//! | `rng.uniform` | ON WIRE (batch API) | No — architectural mismatch with step LCG |
//! | `stats.hill` | PENDING — library only | No |
//! | `stats.shannon_from_frequencies` | PENDING — library only | No |
//! | `stats.simpson` | PENDING — library only | No |
//! | `stats.chao1_classic` | PENDING — library only | No |
//! | `stats.bray_curtis` | PENDING — library only | No |
//! | `special.anderson_diagonalize` | PENDING — library only | No |
//! | `health.pkpd.mm_auc` | PENDING — library only | No |
//! | `health.microbiome.antibiotic_perturbation` | PENDING — library only | No |
//! | `health.biosignal.scr_rate` | PENDING — library only | No |
//!
//! When barraCuda adds JSON-RPC handlers for the PENDING methods, flip
//! each to IPC behind `primal-proof` — the call sites are already routed
//! through this module.

#[cfg(feature = "primal-proof")]
use std::sync::OnceLock;

#[cfg(feature = "primal-proof")]
use crate::ipc::barracuda_client::BarraCudaClient;

#[cfg(feature = "primal-proof")]
static BARRACUDA: OnceLock<Option<BarraCudaClient>> = OnceLock::new();

#[cfg(feature = "primal-proof")]
fn client() -> Option<&'static BarraCudaClient> {
    BARRACUDA
        .get_or_init(|| {
            let c = BarraCudaClient::discover();
            if c.is_some() {
                tracing::info!("math_dispatch: barraCuda ecobin discovered — routing via IPC");
            } else {
                tracing::warn!(
                    "math_dispatch: barraCuda ecobin not found — falling back to library"
                );
            }
            c
        })
        .as_ref()
}

// ── Wire-ready: IPC when primal-proof ────────────────────────────────────

/// Arithmetic mean. Routes to `stats.mean` IPC when `primal-proof` is active.
#[must_use]
pub fn mean(data: &[f64]) -> f64 {
    #[cfg(feature = "primal-proof")]
    if let Some(c) = client() {
        match c.stats_mean(data) {
            Ok(v) => return v,
            Err(e) => tracing::warn!("stats.mean IPC failed, falling back to library: {e}"),
        }
    }
    barracuda::stats::mean(data)
}

/// Standard deviation. Routes to `stats.std_dev` IPC when `primal-proof` is active.
///
/// Returns `None` only if the library call fails (e.g. empty data).
#[must_use]
pub fn std_dev(data: &[f64]) -> Option<f64> {
    #[cfg(feature = "primal-proof")]
    if let Some(c) = client() {
        match c.stats_std_dev(data) {
            Ok(v) => return Some(v),
            Err(e) => tracing::warn!("stats.std_dev IPC failed, falling back to library: {e}"),
        }
    }
    barracuda::stats::correlation::std_dev(data).ok()
}

// ── Wire-pending: library only (awaiting barraCuda wire handlers) ────────

/// Hill equation: `x^n / (k^n + x^n)`.
#[must_use]
pub fn hill(concentration: f64, ic50: f64, hill_n: f64) -> f64 {
    barracuda::stats::hill(concentration, ic50, hill_n)
}

/// Shannon entropy from frequency vector.
#[must_use]
pub fn shannon_from_frequencies(frequencies: &[f64]) -> f64 {
    barracuda::stats::shannon_from_frequencies(frequencies)
}

/// Simpson diversity: `1 - Σ p_i²`.
#[must_use]
pub fn simpson(abundances: &[f64]) -> f64 {
    barracuda::stats::simpson(abundances)
}

/// Chao1 richness estimator.
#[must_use]
pub fn chao1_classic(counts: &[u64]) -> f64 {
    barracuda::stats::chao1_classic(counts)
}

/// Bray-Curtis dissimilarity.
#[must_use]
pub fn bray_curtis(a: &[f64], b: &[f64]) -> f64 {
    barracuda::stats::bray_curtis(a, b)
}

/// Anderson lattice diagonalization via implicit QL.
#[must_use]
pub fn anderson_diagonalize(disorder: &[f64], t_hop: f64) -> (Vec<f64>, Vec<f64>) {
    barracuda::special::anderson_diagonalize(disorder, t_hop)
}

/// Michaelis-Menten AUC (numerical trapezoidal).
#[must_use]
pub fn mm_auc(concs: &[f64], dt: f64) -> f64 {
    barracuda::health::pkpd::mm_auc(concs, dt)
}

/// Species-level antibiotic perturbation.
#[must_use]
pub fn antibiotic_perturbation(
    abundances: &[f64],
    susceptibilities: &[f64],
    duration_h: f64,
) -> Vec<f64> {
    barracuda::health::microbiome::antibiotic_perturbation(abundances, susceptibilities, duration_h)
}

/// SCR rate (events per minute).
#[must_use]
pub fn scr_rate(n_scr_events: usize, duration_s: f64) -> f64 {
    barracuda::health::biosignal::scr_rate(n_scr_events, duration_s)
}

// ── Wire status introspection ────────────────────────────────────────────

/// Number of methods currently routed through IPC (when `primal-proof` is active).
pub const WIRE_READY_COUNT: usize = 2;

/// Total methods managed by this module.
pub const TOTAL_COUNT: usize = 11;

/// Methods not yet on barraCuda's JSON-RPC surface.
pub const WIRE_PENDING_COUNT: usize = TOTAL_COUNT - WIRE_READY_COUNT;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_matches_library() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let dispatch = mean(&data);
        let library = barracuda::stats::mean(&data);
        assert_eq!(dispatch.to_bits(), library.to_bits());
    }

    #[test]
    fn std_dev_matches_library() {
        let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let dispatch = std_dev(&data);
        let library = barracuda::stats::correlation::std_dev(&data).ok();
        assert_eq!(dispatch.map(|v| v.to_bits()), library.map(|v| v.to_bits()));
    }

    #[test]
    fn hill_matches_library() {
        let dispatch = hill(10.0, 10.0, 1.0);
        let library = barracuda::stats::hill(10.0, 10.0, 1.0);
        assert_eq!(dispatch.to_bits(), library.to_bits());
    }

    #[test]
    fn shannon_matches_library() {
        let data = [0.25, 0.25, 0.25, 0.25];
        let dispatch = shannon_from_frequencies(&data);
        let library = barracuda::stats::shannon_from_frequencies(&data);
        assert_eq!(dispatch.to_bits(), library.to_bits());
    }

    #[test]
    fn wire_counts_consistent() {
        assert_eq!(WIRE_READY_COUNT + WIRE_PENDING_COUNT, TOTAL_COUNT);
    }
}
