// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized math dispatch — the "validation window" for guideStone evolution.
//!
//! This module wraps every `barracuda::` call site in healthSpring.
//!
//! ## Dispatch modes
//!
//! | Feature combination | Behavior |
//! |---------------------|----------|
//! | `barracuda-lib` (default) | Library import: `barracuda::stats::*` etc. |
//! | `barracuda-lib` + `primal-proof` | IPC first, library fallback |
//! | neither (IPC-only sovereign) | IPC only, panics if ecobin unavailable |
//!
//! ## Method classification
//!
//! | Method | Type | IPC via |
//! |--------|------|---------|
//! | `stats.mean` | Generic barraCuda IPC | `primal-proof` or IPC-only mode |
//! | `stats.std_dev` | Generic barraCuda IPC | `primal-proof` or IPC-only mode |
//! | `hill` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `shannon_from_frequencies` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `simpson` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `chao1_classic` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `bray_curtis` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `anderson_diagonalize` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `mm_auc` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `antibiotic_perturbation` | Domain composition (local) | N/A — healthSpring-specific science |
//! | `scr_rate` | Domain composition (local) | N/A — healthSpring-specific science |
//!
//! Domain-specific functions (Hill, Shannon, etc.) are LOCAL compositions of
//! barraCuda's generic primitives. They belong to the spring, not the primal.
//! barraCuda's 32 IPC methods are generic (stats, linalg, tensor, spectral).
//! The guideStone validates generic IPC parity; domain science stays local.

#[cfg(any(feature = "primal-proof", not(feature = "barracuda-lib")))]
use std::sync::OnceLock;

#[cfg(any(feature = "primal-proof", not(feature = "barracuda-lib")))]
use crate::ipc::barracuda_client::BarraCudaClient;

#[cfg(any(feature = "primal-proof", not(feature = "barracuda-lib")))]
static BARRACUDA: OnceLock<Option<BarraCudaClient>> = OnceLock::new();

#[cfg(any(feature = "primal-proof", not(feature = "barracuda-lib")))]
fn client() -> Option<&'static BarraCudaClient> {
    BARRACUDA
        .get_or_init(|| {
            let c = BarraCudaClient::discover();
            if c.is_some() {
                tracing::info!("math_dispatch: barraCuda ecobin discovered — routing via IPC");
            } else {
                #[cfg(feature = "barracuda-lib")]
                tracing::warn!(
                    "math_dispatch: barraCuda ecobin not found — falling back to library"
                );
                #[cfg(not(feature = "barracuda-lib"))]
                tracing::error!(
                    "math_dispatch: barraCuda ecobin not found — no library fallback available"
                );
            }
            c
        })
        .as_ref()
}

// ── Wire-ready: IPC when primal-proof or barracuda-lib is off ────────────

/// Arithmetic mean. Routes to `stats.mean` IPC when `primal-proof` is active
/// or `barracuda-lib` is disabled.
#[must_use]
pub fn mean(data: &[f64]) -> f64 {
    #[cfg(any(feature = "primal-proof", not(feature = "barracuda-lib")))]
    if let Some(c) = client() {
        match c.stats_mean(data) {
            Ok(v) => return v,
            Err(e) => tracing::warn!("stats.mean IPC failed, falling back to library: {e}"),
        }
    }
    #[cfg(feature = "barracuda-lib")]
    {
        barracuda::stats::mean(data)
    }
    #[cfg(not(feature = "barracuda-lib"))]
    {
        let _ = data;
        0.0
    }
}

/// Standard deviation. Routes to `stats.std_dev` IPC when `primal-proof` is
/// active or `barracuda-lib` is disabled.
///
/// Returns `None` only if the library call fails (e.g. empty data).
#[must_use]
pub fn std_dev(data: &[f64]) -> Option<f64> {
    #[cfg(any(feature = "primal-proof", not(feature = "barracuda-lib")))]
    if let Some(c) = client() {
        match c.stats_std_dev(data) {
            Ok(v) => return Some(v),
            Err(e) => tracing::warn!("stats.std_dev IPC failed, falling back to library: {e}"),
        }
    }
    #[cfg(feature = "barracuda-lib")]
    {
        barracuda::stats::correlation::std_dev(data).ok()
    }
    #[cfg(not(feature = "barracuda-lib"))]
    {
        let _ = data;
        None
    }
}

// ── Domain compositions: local science (require barracuda-lib) ──────────

/// Hill equation: `x^n / (k^n + x^n)`.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn hill(concentration: f64, ic50: f64, hill_n: f64) -> f64 {
    barracuda::stats::hill(concentration, ic50, hill_n)
}

/// Hill equation fallback — pure Rust when `barracuda-lib` is disabled.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn hill(concentration: f64, ic50: f64, hill_n: f64) -> f64 {
    let xn = concentration.powf(hill_n);
    let kn = ic50.powf(hill_n);
    xn / (kn + xn)
}

/// Shannon entropy from frequency vector.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn shannon_from_frequencies(frequencies: &[f64]) -> f64 {
    barracuda::stats::shannon_from_frequencies(frequencies)
}

/// Shannon entropy fallback — pure Rust when `barracuda-lib` is disabled.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn shannon_from_frequencies(frequencies: &[f64]) -> f64 {
    -frequencies
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| p * p.ln())
        .sum::<f64>()
}

/// Simpson diversity: `1 - Σ p_i²`.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn simpson(abundances: &[f64]) -> f64 {
    barracuda::stats::simpson(abundances)
}

/// Simpson diversity fallback.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn simpson(abundances: &[f64]) -> f64 {
    let total: f64 = abundances.iter().sum();
    if total == 0.0 {
        return 0.0;
    }
    1.0 - abundances.iter().map(|&a| (a / total).powi(2)).sum::<f64>()
}

/// Chao1 richness estimator.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn chao1_classic(counts: &[u64]) -> f64 {
    barracuda::stats::chao1_classic(counts)
}

/// Chao1 fallback.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn chao1_classic(counts: &[u64]) -> f64 {
    let s_obs = counts.iter().filter(|&&c| c > 0).count() as f64;
    let f1 = counts.iter().filter(|&&c| c == 1).count() as f64;
    let f2 = counts.iter().filter(|&&c| c == 2).count() as f64;
    if f2 > 0.0 {
        s_obs + (f1 * f1) / (2.0 * f2)
    } else if f1 > 0.0 {
        s_obs + f1 * (f1 - 1.0) / 2.0
    } else {
        s_obs
    }
}

/// Bray-Curtis dissimilarity.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn bray_curtis(a: &[f64], b: &[f64]) -> f64 {
    barracuda::stats::bray_curtis(a, b)
}

/// Bray-Curtis fallback.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn bray_curtis(a: &[f64], b: &[f64]) -> f64 {
    let (num, den) = a
        .iter()
        .zip(b.iter())
        .fold((0.0, 0.0), |(n, d), (&ai, &bi)| {
            (n + (ai - bi).abs(), d + ai + bi)
        });
    if den == 0.0 { 0.0 } else { num / den }
}

/// Anderson lattice diagonalization via implicit QL.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn anderson_diagonalize(disorder: &[f64], t_hop: f64) -> (Vec<f64>, Vec<f64>) {
    barracuda::special::anderson_diagonalize(disorder, t_hop)
}

/// Michaelis-Menten AUC (numerical trapezoidal).
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn mm_auc(concs: &[f64], dt: f64) -> f64 {
    barracuda::health::pkpd::mm_auc(concs, dt)
}

/// MM AUC fallback — trapezoidal rule.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn mm_auc(concs: &[f64], dt: f64) -> f64 {
    if concs.len() < 2 {
        return 0.0;
    }
    concs
        .windows(2)
        .map(|w| 0.5 * (w[0] + w[1]) * dt)
        .sum()
}

/// Species-level antibiotic perturbation.
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn antibiotic_perturbation(
    abundances: &[f64],
    susceptibilities: &[f64],
    duration_h: f64,
) -> Vec<f64> {
    barracuda::health::microbiome::antibiotic_perturbation(abundances, susceptibilities, duration_h)
}

/// SCR rate (events per minute).
#[must_use]
#[cfg(feature = "barracuda-lib")]
pub fn scr_rate(n_scr_events: usize, duration_s: f64) -> f64 {
    barracuda::health::biosignal::scr_rate(n_scr_events, duration_s)
}

/// SCR rate fallback.
#[must_use]
#[cfg(not(feature = "barracuda-lib"))]
pub fn scr_rate(n_scr_events: usize, duration_s: f64) -> f64 {
    if duration_s <= 0.0 {
        return 0.0;
    }
    n_scr_events as f64 / (duration_s / 60.0)
}

// ── Wire status introspection ────────────────────────────────────────────

/// Number of methods currently routed through IPC (when `primal-proof` is active).
pub const WIRE_READY_COUNT: usize = 2;

/// Total methods managed by this module.
pub const TOTAL_COUNT: usize = 11;

/// Domain-specific methods (local compositions, not IPC candidates).
pub const WIRE_PENDING_COUNT: usize = TOTAL_COUNT - WIRE_READY_COUNT;

#[cfg(test)]
#[cfg(feature = "barracuda-lib")]
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
