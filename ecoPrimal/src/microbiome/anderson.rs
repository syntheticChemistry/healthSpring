// SPDX-License-Identifier: AGPL-3.0-or-later
//! Anderson localization — 1D tight-binding lattice (Exp011).

/// Build a symmetric 1D Anderson Hamiltonian as a flat `L × L` matrix
/// (row-major). Diagonal: on-site energies from `disorder`, off-diagonal
/// nearest-neighbor hopping `t_hop`.
#[must_use]
pub fn anderson_hamiltonian_1d(disorder: &[f64], t_hop: f64) -> Vec<f64> {
    let l = disorder.len();
    let mut h = vec![0.0_f64; l * l];
    for i in 0..l {
        h[i * l + i] = disorder[i];
        if i + 1 < l {
            h[i * l + (i + 1)] = t_hop;
            h[(i + 1) * l + i] = t_hop;
        }
    }
    h
}

/// Diagonalize a 1D Anderson Hamiltonian via the implicit QL algorithm
/// for symmetric tridiagonal matrices.
///
/// Delegates to `barracuda::special::anderson_diagonalize` — the canonical
/// upstream implementation (absorbed from healthSpring V13).
///
/// Returns `(eigenvalues, eigenvectors)` where `eigenvectors` is a flat
/// `L × L` matrix (row-major) with eigenvector `k` in row `k`.
#[must_use]
pub fn anderson_diagonalize(disorder: &[f64], t_hop: f64) -> (Vec<f64>, Vec<f64>) {
    barracuda::special::anderson_diagonalize(disorder, t_hop)
}

/// Inverse participation ratio: `IPR = Σ |ψ_i|⁴`.
///
/// Localized state: `IPR ∼ 1/ξ`. Extended state: `IPR ∼ 1/L`.
#[must_use]
pub fn inverse_participation_ratio(psi: &[f64]) -> f64 {
    psi.iter().map(|&x| x * x * x * x).sum()
}

/// Localization length from IPR: `ξ ≈ 1/IPR`.
#[must_use]
pub fn localization_length_from_ipr(ipr: f64) -> f64 {
    if ipr > 0.0 { 1.0 / ipr } else { f64::INFINITY }
}

/// Mean level-spacing ratio `<r>`.
///
/// For sorted eigenvalues, `r_n = min(s_n, s_{n+1}) / max(s_n, s_{n+1})`.
/// Poisson (localized): `<r> ≈ 0.386`. GOE (extended): `<r> ≈ 0.531`.
#[must_use]
pub fn level_spacing_ratio(eigenvalues: &[f64]) -> f64 {
    if eigenvalues.len() < 3 {
        return 0.0;
    }
    let mut sorted = eigenvalues.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let spacings: Vec<f64> = sorted.windows(2).map(|w| w[1] - w[0]).collect();
    let mut sum = 0.0;
    let mut count = 0usize;
    for w in spacings.windows(2) {
        let mx = w[0].max(w[1]);
        if mx > 0.0 {
            sum += w[0].min(w[1]) / mx;
            count += 1;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "spacing count ≪ 2^52")]
    let result = if count > 0 { sum / count as f64 } else { 0.0 };
    result
}

/// Colonization resistance score: `CR = 1/ξ`.
///
/// Higher `CR` → pathogen more confined → healthier gut.
#[must_use]
pub fn colonization_resistance(xi: f64) -> f64 {
    if xi > 0.0 && xi.is_finite() {
        1.0 / xi
    } else {
        0.0
    }
}
