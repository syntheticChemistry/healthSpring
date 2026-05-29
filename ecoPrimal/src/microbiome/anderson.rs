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
    crate::math_dispatch::anderson_diagonalize(disorder, t_hop)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamiltonian_2x2_structure() {
        let h = anderson_hamiltonian_1d(&[1.0, 2.0], 0.5);
        assert_eq!(h.len(), 4);
        assert!((h[0] - 1.0).abs() < 1e-15);
        assert!((h[1] - 0.5).abs() < 1e-15);
        assert!((h[2] - 0.5).abs() < 1e-15);
        assert!((h[3] - 2.0).abs() < 1e-15);
    }

    #[test]
    fn hamiltonian_is_symmetric() {
        let disorder = [0.3, -0.5, 1.2, 0.7, -0.1];
        let h = anderson_hamiltonian_1d(&disorder, 1.0);
        let n = disorder.len();
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (h[i * n + j] - h[j * n + i]).abs() < 1e-15,
                    "H[{i},{j}] != H[{j},{i}]"
                );
            }
        }
    }

    #[test]
    fn hamiltonian_tridiagonal() {
        let disorder = [1.0, 2.0, 3.0, 4.0];
        let h = anderson_hamiltonian_1d(&disorder, 0.8);
        let n = disorder.len();
        for i in 0..n {
            for j in 0..n {
                let dist = i.abs_diff(j);
                if dist > 1 {
                    assert!(
                        h[i * n + j].abs() < 1e-15,
                        "non-zero at [{i},{j}] (dist {dist})"
                    );
                }
            }
        }
    }

    #[test]
    fn ipr_of_localized_state() {
        let mut psi = vec![0.0; 10];
        psi[3] = 1.0;
        let ipr = inverse_participation_ratio(&psi);
        assert!((ipr - 1.0).abs() < 1e-15, "perfectly localized IPR should be 1.0");
    }

    #[test]
    #[expect(clippy::cast_precision_loss, reason = "test lattice sizes ≪ 2^52")]
    fn ipr_of_extended_state() {
        let n = 100_usize;
        let amplitude = 1.0 / (n as f64).sqrt();
        let psi = vec![amplitude; n];
        let ipr = inverse_participation_ratio(&psi);
        let expected = 1.0 / n as f64;
        assert!(
            (ipr - expected).abs() < 1e-12,
            "uniform extended IPR={ipr}, expected {expected}"
        );
    }

    #[test]
    fn localization_length_from_ipr_edge_cases() {
        assert!((localization_length_from_ipr(1.0) - 1.0).abs() < 1e-15);
        assert!((localization_length_from_ipr(0.5) - 2.0).abs() < 1e-15);
        assert!(localization_length_from_ipr(0.0).is_infinite());
        assert!(localization_length_from_ipr(-1.0).is_infinite());
    }

    #[test]
    fn level_spacing_ratio_short_inputs() {
        assert!((level_spacing_ratio(&[]) - 0.0).abs() < 1e-15);
        assert!((level_spacing_ratio(&[1.0]) - 0.0).abs() < 1e-15);
        assert!((level_spacing_ratio(&[1.0, 2.0]) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn level_spacing_ratio_uniform_spacing() {
        let evals: Vec<f64> = (0..20).map(f64::from).collect();
        let r = level_spacing_ratio(&evals);
        assert!(
            (r - 1.0).abs() < 1e-12,
            "uniform spacing: <r> should be 1.0, got {r}"
        );
    }

    #[test]
    fn colonization_resistance_positive_finite() {
        assert!((colonization_resistance(2.0) - 0.5).abs() < 1e-15);
        assert!((colonization_resistance(0.5) - 2.0).abs() < 1e-15);
    }

    #[test]
    fn colonization_resistance_edge_cases() {
        assert!((colonization_resistance(0.0) - 0.0).abs() < 1e-15);
        assert!((colonization_resistance(-1.0) - 0.0).abs() < 1e-15);
        assert!((colonization_resistance(f64::INFINITY) - 0.0).abs() < 1e-15);
        assert!((colonization_resistance(f64::NAN) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn diagonalize_recovers_known_eigenvalues() {
        let (evals, _) = anderson_diagonalize(&[0.0, 0.0], 1.0);
        let mut sorted = evals;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        assert!((sorted[0] - (-1.0)).abs() < 1e-10, "e0={}", sorted[0]);
        assert!((sorted[1] - 1.0).abs() < 1e-10, "e1={}", sorted[1]);
    }

    #[test]
    fn diagonalize_eigenvalue_count_matches_lattice_size() {
        let disorder = [0.1, -0.2, 0.3, 0.0, -0.1];
        let (evals, evecs) = anderson_diagonalize(&disorder, 1.0);
        assert_eq!(evals.len(), 5);
        assert_eq!(evecs.len(), 25);
    }
}
