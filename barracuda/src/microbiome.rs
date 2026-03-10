// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gut microbiome analytics for human health.
//!
//! Extends wetSpring Track 1 (16S, metagenomics) and Paper 01/06
//! (Anderson localization in microbial communities) to clinical
//! gut microbiome analysis.
//!
//! ## Cross-spring provenance
//!
//! - **Diversity stats** (Shannon, Simpson, Pielou, Chao1, Bray-Curtis):
//!   Originated here, absorbed into `barracuda::stats`. Local implementations
//!   retained as validated reference; cross-validation tests confirm parity.
//! - **Anderson localization**: Delegates to `barracuda::special::anderson_diagonalize`
//!   (absorbed from healthSpring V13). The QL eigensolver was originally
//!   in this module; now canonical in barraCuda.
//! - **wetSpring bio shaders**: `diversity_fusion_f64.wgsl` (Shannon+Simpson+Pielou
//!   in one GPU dispatch) originated from healthSpring and was absorbed into
//!   `barracuda::shaders::bio::diversity_fusion_f64.wgsl`, shared with wetSpring.
//!
//! ## Tier 1 (CPU)
//!
//! - [`shannon_index`]: Shannon diversity `H'`
//! - [`simpson_index`]: Simpson diversity `D`
//! - [`inverse_simpson`]: Inverse Simpson `1/D`
//! - [`pielou_evenness`]: Pielou `J`
//! - [`chao1`]: Chao1 richness estimator
//! - [`evenness_to_disorder`]: Pielou `J` → Anderson disorder `W`
//!
//! ## Anderson Connection
//!
//! Pielou evenness maps directly to Anderson disorder strength `W`:
//! - High evenness (diverse, healthy gut) → high `W` → extended states
//!   → colonization resistance (signals propagate, ecosystem responds)
//! - Low evenness (dysbiotic) → low `W` → localized states
//!   → colonization vulnerability (*C. diff* exploits silent niches)

/// Shannon diversity index: `H' = -Σ p_i · ln(p_i)`.
///
/// `H' = 0` for monoculture, `H' = ln(S)` for perfectly even community.
///
/// ```
/// use healthspring_barracuda::microbiome::shannon_index;
///
/// let uniform = vec![0.25, 0.25, 0.25, 0.25];
/// let h = shannon_index(&uniform);
/// assert!((h - 4.0_f64.ln()).abs() < 1e-10);
/// ```
#[must_use]
pub fn shannon_index(abundances: &[f64]) -> f64 {
    abundances
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.ln())
        .sum()
}

/// Simpson diversity: `D = 1 - Σ p_i²`.
///
/// `D = 0` for monoculture, approaches `1 - 1/S` for even community.
#[must_use]
pub fn simpson_index(abundances: &[f64]) -> f64 {
    1.0 - abundances.iter().map(|&p| p * p).sum::<f64>()
}

/// Inverse Simpson: `1 / Σ p_i²`.
///
/// Equals `S` for perfectly even community.
#[must_use]
pub fn inverse_simpson(abundances: &[f64]) -> f64 {
    let d: f64 = abundances.iter().map(|&p| p * p).sum();
    if d > 0.0 { 1.0 / d } else { 0.0 }
}

/// Pielou evenness: `J = H' / ln(S)`.
///
/// `J = 1.0` for perfectly even, `J → 0` for dominated.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "species count (small usize) → f64 for log"
)]
pub fn pielou_evenness(abundances: &[f64]) -> f64 {
    let s = abundances.len();
    if s <= 1 {
        return 0.0;
    }
    let h = shannon_index(abundances);
    let h_max = (s as f64).ln();
    if h_max == 0.0 { 0.0 } else { h / h_max }
}

/// Chao1 richness estimator: `S_obs + f1² / (2·f2)`.
///
/// `f1` = singletons, `f2` = doubletons. When `f2 = 0`, uses the
/// bias-corrected form `S_obs + f1·(f1-1)/2`.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "count values (small u64) → f64 for arithmetic"
)]
pub fn chao1(counts: &[u64]) -> f64 {
    let s_obs = counts.len() as f64;
    let f1 = counts.iter().filter(|&&c| c == 1).count() as f64;
    let f2 = counts.iter().filter(|&&c| c == 2).count() as f64;
    if f2 == 0.0 {
        if f1 > 1.0 {
            s_obs + f1 * (f1 - 1.0) / 2.0
        } else {
            s_obs
        }
    } else {
        s_obs + f1 * f1 / (2.0 * f2)
    }
}

/// Map Pielou evenness to Anderson disorder `W`.
///
/// `W = J * scale`. Higher diversity → higher disorder → extended
/// states → colonization resistance.
#[must_use]
pub fn evenness_to_disorder(evenness: f64, w_scale: f64) -> f64 {
    evenness * w_scale
}

// ═══════════════════════════════════════════════════════════════════════
// Anderson Localization — 1D Tight-Binding Lattice (Exp011)
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// FMT Microbiota Transplant Modeling (Exp013)
// ═══════════════════════════════════════════════════════════════════════

/// Simulate post-FMT community as weighted blend of donor and recipient.
///
/// `blended_i = (1 - engraftment) * recipient_i + engraftment * donor_i`
/// then re-normalized so abundances sum to 1.0.
#[must_use]
pub fn fmt_blend(donor: &[f64], recipient: &[f64], engraftment: f64) -> Vec<f64> {
    let n = donor.len().max(recipient.len());
    let mut blended = vec![0.0; n];
    for i in 0..n {
        let d = if i < donor.len() { donor[i] } else { 0.0 };
        let r = if i < recipient.len() {
            recipient[i]
        } else {
            0.0
        };
        blended[i] = (1.0 - engraftment).mul_add(r, engraftment * d);
    }
    let total: f64 = blended.iter().sum();
    if total > 0.0 {
        for v in &mut blended {
            *v /= total;
        }
    }
    blended
}

/// Bray-Curtis dissimilarity between two communities.
///
/// `BC = 1 - 2*Σ min(a_i, b_i) / (Σ a_i + Σ b_i)`
/// BC = 0 means identical, BC = 1 means completely different.
#[must_use]
pub fn bray_curtis(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().max(b.len());
    let mut sum_min = 0.0;
    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    for i in 0..n {
        let ai = if i < a.len() { a[i] } else { 0.0 };
        let bi = if i < b.len() { b[i] } else { 0.0 };
        sum_min += ai.min(bi);
        sum_a += ai;
        sum_b += bi;
    }
    let denom = sum_a + sum_b;
    if denom > 0.0 {
        1.0 - 2.0 * sum_min / denom
    } else {
        0.0
    }
}

// ── Antibiotic Perturbation Model ──────────────────────────────────────

/// Simulate Shannon diversity time course under antibiotic perturbation
/// and recovery. Returns `(time, shannon)` pairs.
///
/// - `h0`: baseline Shannon H'
/// - `depth`: fractional decline at nadir (0–1)
/// - `k_decline`: decline rate constant (per day)
/// - `k_recovery`: recovery rate constant (per day)
/// - `treatment_days`: duration of antibiotic exposure
/// - `total_days`: total simulation time
/// - `dt`: time step
///
/// Reference: Dethlefsen & Relman 2011 (Nature) — ciprofloxacin causes
/// 30-50% diversity decline with incomplete recovery.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "n_steps fits f64")]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "total_days / dt is small positive"
)]
pub fn antibiotic_perturbation(
    h0: f64,
    depth: f64,
    k_decline: f64,
    k_recovery: f64,
    treatment_days: f64,
    total_days: f64,
    dt: f64,
) -> Vec<(f64, f64)> {
    let n_steps = (total_days / dt) as usize;
    let mut result = Vec::with_capacity(n_steps + 1);
    let h_nadir = h0 * (1.0 - depth);

    for i in 0..=n_steps {
        let t = i as f64 * dt;
        let h = if t <= treatment_days {
            h0 - (h0 - h_nadir) * (1.0 - (-k_decline * t).exp())
        } else {
            let t_rec = t - treatment_days;
            let h_at_end = h0 - (h0 - h_nadir) * (1.0 - (-k_decline * treatment_days).exp());
            let recovery_target = h0 * (1.0 - depth * 0.15);
            h_at_end + (recovery_target - h_at_end) * (1.0 - (-k_recovery * t_rec).exp())
        };
        result.push((t, h));
    }
    result
}

// ── SCFA Production Model ─────────────────────────────────────────────

/// Michaelis-Menten SCFA production parameters per acid.
#[derive(Debug, Clone)]
pub struct ScfaParams {
    /// Acetate Vmax (mmol/L/day)
    pub vmax_acetate: f64,
    /// Acetate Km (g fiber/L)
    pub km_acetate: f64,
    /// Propionate Vmax
    pub vmax_propionate: f64,
    /// Propionate Km
    pub km_propionate: f64,
    /// Butyrate Vmax
    pub vmax_butyrate: f64,
    /// Butyrate Km
    pub km_butyrate: f64,
}

/// Reference SCFA parameters for healthy gut (Cummings 1987 ratios: 60:20:15).
pub const SCFA_HEALTHY_PARAMS: ScfaParams = ScfaParams {
    vmax_acetate: 60.0,
    km_acetate: 8.0,
    vmax_propionate: 20.0,
    km_propionate: 10.0,
    vmax_butyrate: 15.0,
    km_butyrate: 12.0,
};

/// Dysbiotic SCFA params (reduced butyrate producers).
pub const SCFA_DYSBIOTIC_PARAMS: ScfaParams = ScfaParams {
    vmax_acetate: 55.0,
    km_acetate: 8.0,
    vmax_propionate: 18.0,
    km_propionate: 10.0,
    vmax_butyrate: 5.0,
    km_butyrate: 15.0,
};

/// Michaelis-Menten SCFA production from fiber substrate.
///
/// Returns `(acetate, propionate, butyrate)` in mmol/L.
/// Reference: den Besten et al. 2013, Cummings 1987.
#[must_use]
pub fn scfa_production(fiber_g_per_l: f64, params: &ScfaParams) -> (f64, f64, f64) {
    let mm = |vmax: f64, km: f64| vmax * fiber_g_per_l / (km + fiber_g_per_l);
    (
        mm(params.vmax_acetate, params.km_acetate),
        mm(params.vmax_propionate, params.km_propionate),
        mm(params.vmax_butyrate, params.km_butyrate),
    )
}

// ── Gut-Brain Serotonin Pathway ───────────────────────────────────────

/// Gut serotonin production rate as a function of tryptophan availability
/// and microbiome diversity.
///
/// ~90% of body serotonin is gut-derived. Diverse microbiome increases
/// tryptophan availability for enterochromaffin cells.
///
/// `rate = k_synth · trp · diversity_factor(H')`
///
/// where `diversity_factor = sigmoid((H' - H_ref) / scale)`.
///
/// Reference: Yano et al. 2015 (Cell), Clarke et al. 2013.
#[must_use]
pub fn gut_serotonin_production(
    tryptophan_umol_l: f64,
    shannon_h: f64,
    k_synth: f64,
    scale: f64,
) -> f64 {
    let h_ref = 1.5;
    let diversity_factor = 1.0 / (1.0 + (-(shannon_h - h_ref) / scale).exp());
    k_synth * tryptophan_umol_l * diversity_factor
}

/// Tryptophan availability from dietary intake modulated by microbiome.
///
/// Healthy microbiome: ~80% of dietary tryptophan available.
/// Dysbiotic: ~40% (more diverted to indole pathway).
#[must_use]
pub fn tryptophan_availability(dietary_trp_umol_l: f64, shannon_h: f64) -> f64 {
    let availability_fraction = 0.4 + 0.4 / (1.0 + (-3.0 * (shannon_h - 1.5)).exp());
    dietary_trp_umol_l * availability_fraction
}

/// Synthetic gut microbiome community profiles for testing.
pub mod communities {
    pub const HEALTHY_GUT: [f64; 10] = [0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01];
    pub const DYSBIOTIC_GUT: [f64; 10] = [
        0.85, 0.05, 0.03, 0.02, 0.02, 0.01, 0.01, 0.005, 0.003, 0.002,
    ];
    pub const CDIFF_COLONIZED: [f64; 10] = [
        0.60, 0.15, 0.10, 0.05, 0.04, 0.03, 0.02, 0.005, 0.003, 0.002,
    ];
    pub const PERFECTLY_EVEN: [f64; 10] = [0.1; 10];
    pub const MONOCULTURE: [f64; 1] = [1.0];
}

#[cfg(test)]
mod tests {
    use super::*;
    use communities::*;

    const TOL: f64 = 1e-10;

    #[test]
    fn shannon_perfectly_even() {
        let h = shannon_index(&PERFECTLY_EVEN);
        let expected = (10.0_f64).ln();
        assert!((h - expected).abs() < TOL, "H' = ln(10)");
    }

    #[test]
    fn shannon_monoculture_zero() {
        let h = shannon_index(&MONOCULTURE);
        assert!(h.abs() < TOL, "H' = 0 for monoculture");
    }

    #[test]
    fn shannon_ordering() {
        let h_even = shannon_index(&PERFECTLY_EVEN);
        let h_healthy = shannon_index(&HEALTHY_GUT);
        let h_cdiff = shannon_index(&CDIFF_COLONIZED);
        let h_dysbiotic = shannon_index(&DYSBIOTIC_GUT);
        let h_mono = shannon_index(&MONOCULTURE);
        assert!(h_even > h_healthy, "even > healthy");
        assert!(h_healthy > h_cdiff, "healthy > cdiff");
        assert!(h_cdiff > h_dysbiotic, "cdiff > dysbiotic");
        assert!(h_dysbiotic > h_mono, "dysbiotic > mono");
    }

    #[test]
    fn simpson_perfectly_even() {
        let d = simpson_index(&PERFECTLY_EVEN);
        let expected = 10.0f64.mul_add(-0.01, 1.0);
        assert!((d - expected).abs() < TOL);
    }

    #[test]
    fn simpson_monoculture_zero() {
        let d = simpson_index(&MONOCULTURE);
        assert!(d.abs() < TOL);
    }

    #[test]
    fn simpson_healthy_gt_dysbiotic() {
        let d_h = simpson_index(&HEALTHY_GUT);
        let d_d = simpson_index(&DYSBIOTIC_GUT);
        assert!(d_h > d_d);
    }

    #[test]
    fn inverse_simpson_even_equals_s() {
        let inv = inverse_simpson(&PERFECTLY_EVEN);
        assert!((inv - 10.0).abs() < TOL);
    }

    #[test]
    fn pielou_even_is_one() {
        let j = pielou_evenness(&PERFECTLY_EVEN);
        assert!((j - 1.0).abs() < TOL);
    }

    #[test]
    fn pielou_ordering() {
        let j_h = pielou_evenness(&HEALTHY_GUT);
        let j_c = pielou_evenness(&CDIFF_COLONIZED);
        let j_d = pielou_evenness(&DYSBIOTIC_GUT);
        assert!(j_h > j_c, "healthy > cdiff");
        assert!(j_c > j_d, "cdiff > dysbiotic");
    }

    #[test]
    fn chao1_geq_sobs() {
        let counts_h: Vec<u64> = vec![250, 200, 150, 120, 100, 80, 50, 30, 10, 5, 3, 2, 1, 1, 1];
        let counts_d: Vec<u64> = vec![850, 50, 30, 20, 15, 10, 5, 5, 3, 2, 1, 1];
        let c_h = chao1(&counts_h);
        let c_d = chao1(&counts_d);
        #[expect(clippy::cast_precision_loss, reason = "species count fits f64")]
        {
            assert!(c_h >= counts_h.len() as f64, "Chao1 ≥ S_obs");
            assert!(c_d >= counts_d.len() as f64, "Chao1 ≥ S_obs");
        }
        assert!(c_h > c_d, "healthy Chao1 > depleted");
    }

    #[test]
    fn anderson_disorder_mapping() {
        let w_h = evenness_to_disorder(pielou_evenness(&HEALTHY_GUT), 10.0);
        let w_d = evenness_to_disorder(pielou_evenness(&DYSBIOTIC_GUT), 10.0);
        assert!(w_h > w_d, "more diverse → more disorder → extended states");
    }

    #[test]
    fn all_indices_valid_ranges() {
        for ab in [
            &HEALTHY_GUT[..],
            &DYSBIOTIC_GUT[..],
            &CDIFF_COLONIZED[..],
            &PERFECTLY_EVEN[..],
        ] {
            let h = shannon_index(ab);
            let d = simpson_index(ab);
            let j = pielou_evenness(ab);
            assert!(h >= -TOL, "H' ≥ 0");
            assert!((-TOL..=1.0 + TOL).contains(&d), "0 ≤ D ≤ 1");
            assert!((-TOL..=1.0 + TOL).contains(&j), "0 ≤ J ≤ 1");
        }
    }

    // Anderson lattice tests (Exp011)

    #[test]
    fn anderson_hamiltonian_symmetric() {
        let disorder = vec![1.0, -0.5, 0.3, 0.8];
        let l = disorder.len();
        let h = anderson_hamiltonian_1d(&disorder, 1.0);
        for i in 0..l {
            for j in 0..l {
                assert!((h[i * l + j] - h[j * l + i]).abs() < TOL, "H symmetric");
            }
        }
    }

    #[test]
    fn anderson_hamiltonian_diagonal() {
        let disorder = vec![2.0, -1.0, 3.5];
        let l = disorder.len();
        let h = anderson_hamiltonian_1d(&disorder, 1.0);
        for i in 0..l {
            assert!(
                (h[i * l + i] - disorder[i]).abs() < TOL,
                "diagonal = disorder"
            );
        }
    }

    #[test]
    fn anderson_hamiltonian_hopping() {
        let disorder = vec![0.0; 4];
        let l = 4;
        let h = anderson_hamiltonian_1d(&disorder, 2.5);
        assert!((h[1] - 2.5).abs() < TOL);
        assert!((h[l] - 2.5).abs() < TOL);
        assert!(h[2].abs() < TOL, "no long-range hopping");
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "test lattice size fits f64 mantissa"
    )]
    fn ipr_uniform_state() {
        let l = 100;
        let val = 1.0 / (l as f64).sqrt();
        let psi = vec![val; l];
        let ipr = inverse_participation_ratio(&psi);
        let expected = 1.0 / l as f64;
        assert!((ipr - expected).abs() < 1e-8, "extended state IPR = 1/L");
    }

    #[test]
    fn ipr_localized_state() {
        let l = 100;
        let mut psi = vec![0.0; l];
        psi[50] = 1.0;
        let ipr = inverse_participation_ratio(&psi);
        assert!((ipr - 1.0).abs() < TOL, "perfectly localized IPR = 1.0");
    }

    #[test]
    fn localization_length_inverse() {
        let xi = localization_length_from_ipr(0.25);
        assert!((xi - 4.0).abs() < TOL);
    }

    #[test]
    fn localization_length_zero_ipr() {
        let xi = localization_length_from_ipr(0.0);
        assert!(xi.is_infinite());
    }

    #[test]
    fn level_spacing_ratio_few_values() {
        assert!((level_spacing_ratio(&[1.0, 2.0])).abs() < f64::EPSILON);
    }

    #[test]
    fn level_spacing_ratio_uniform() {
        let eigs: Vec<f64> = (0..100).map(f64::from).collect();
        let r = level_spacing_ratio(&eigs);
        assert!((r - 1.0).abs() < 0.01, "uniform spacing → r=1.0, got {r}");
    }

    #[test]
    fn colonization_resistance_basic() {
        let cr_confined = colonization_resistance(2.0);
        let cr_extended = colonization_resistance(50.0);
        assert!(cr_confined > cr_extended, "shorter ξ → higher CR");
    }

    #[test]
    fn diversity_indices_deterministic() {
        let community = &HEALTHY_GUT[..];
        let h1 = shannon_index(community);
        let h2 = shannon_index(community);
        assert_eq!(h1.to_bits(), h2.to_bits(), "Shannon must be bit-identical");

        let s1 = simpson_index(community);
        let s2 = simpson_index(community);
        assert_eq!(s1.to_bits(), s2.to_bits(), "Simpson must be bit-identical");
    }

    // FMT tests (Exp013)

    #[test]
    fn bray_curtis_identical() {
        let bc = bray_curtis(&HEALTHY_GUT, &HEALTHY_GUT);
        assert!(bc.abs() < TOL, "identical communities → BC=0");
    }

    #[test]
    fn bray_curtis_range() {
        let bc = bray_curtis(&HEALTHY_GUT, &DYSBIOTIC_GUT);
        assert!((0.0..=1.0).contains(&bc), "BC in [0,1]");
        assert!(bc > 0.0, "different communities → BC > 0");
    }

    #[test]
    fn fmt_blend_pure_donor() {
        let blended = fmt_blend(&HEALTHY_GUT, &DYSBIOTIC_GUT, 1.0);
        for (a, b) in blended.iter().zip(HEALTHY_GUT.iter()) {
            assert!((a - b).abs() < TOL, "100% engraftment = donor");
        }
    }

    #[test]
    fn fmt_blend_zero_engraftment() {
        let blended = fmt_blend(&HEALTHY_GUT, &DYSBIOTIC_GUT, 0.0);
        for (a, b) in blended.iter().zip(DYSBIOTIC_GUT.iter()) {
            assert!((a - b).abs() < TOL, "0% engraftment = recipient");
        }
    }

    #[test]
    fn fmt_improves_diversity() {
        let post_fmt = fmt_blend(&HEALTHY_GUT, &DYSBIOTIC_GUT, 0.7);
        let h_pre = shannon_index(&DYSBIOTIC_GUT);
        let h_post = shannon_index(&post_fmt);
        assert!(h_post > h_pre, "FMT should improve diversity");
    }

    // Cross-validation: local implementations vs upstream barracuda::stats

    #[test]
    fn cross_validate_shannon_vs_upstream() {
        for ab in [
            &HEALTHY_GUT[..],
            &DYSBIOTIC_GUT[..],
            &CDIFF_COLONIZED[..],
            &PERFECTLY_EVEN[..],
        ] {
            let local = shannon_index(ab);
            let upstream = barracuda::stats::shannon_from_frequencies(ab);
            assert!(
                (local - upstream).abs() < 1e-10,
                "Shannon mismatch: local={local}, upstream={upstream}"
            );
        }
    }

    #[test]
    fn cross_validate_bray_curtis_vs_upstream() {
        let local = bray_curtis(&HEALTHY_GUT, &DYSBIOTIC_GUT);
        let upstream = barracuda::stats::bray_curtis(&HEALTHY_GUT, &DYSBIOTIC_GUT);
        assert!(
            (local - upstream).abs() < 1e-10,
            "Bray-Curtis mismatch: local={local}, upstream={upstream}"
        );
    }

    #[test]
    fn antibiotic_perturbation_decline() {
        let result = super::antibiotic_perturbation(2.2, 0.5, 0.3, 0.1, 7.0, 21.0, 0.1);
        assert!(result.len() > 1);
        let nadir = result.iter().map(|&(_, h)| h).fold(f64::INFINITY, f64::min);
        assert!(nadir < 2.2, "Shannon must decline during antibiotics");
    }

    #[test]
    fn antibiotic_perturbation_recovery() {
        let result = super::antibiotic_perturbation(2.2, 0.5, 0.3, 0.1, 7.0, 42.0, 0.1);
        let h_final = result.last().unwrap().1;
        let nadir = result.iter().map(|&(_, h)| h).fold(f64::INFINITY, f64::min);
        assert!(h_final > nadir, "should recover after antibiotics end");
    }

    #[test]
    fn scfa_ratios_normal() {
        let (a, p, b) = super::scfa_production(20.0, &super::SCFA_HEALTHY_PARAMS);
        let total = a + p + b;
        let acetate_frac = a / total;
        assert!(
            acetate_frac > 0.50 && acetate_frac < 0.75,
            "acetate fraction should be ~60%: {acetate_frac}"
        );
    }

    #[test]
    fn scfa_saturates() {
        let (a1, _, _) = super::scfa_production(10.0, &super::SCFA_HEALTHY_PARAMS);
        let (a2, _, _) = super::scfa_production(100.0, &super::SCFA_HEALTHY_PARAMS);
        assert!(a2 / a1 < 10.0, "SCFA should saturate (Michaelis-Menten)");
    }

    #[test]
    fn gut_brain_serotonin_diversity_link() {
        let s_high = super::gut_serotonin_production(200.0, 2.2, 0.8, 0.1);
        let s_low = super::gut_serotonin_production(200.0, 0.8, 0.8, 0.1);
        assert!(
            s_high > s_low,
            "higher diversity → more serotonin: {s_high} vs {s_low}"
        );
    }

    #[test]
    fn cross_validate_anderson_vs_upstream() {
        let disorder = vec![1.0, -0.5, 0.3, 0.8, -0.2];
        let (eigs_local, vecs_local) = anderson_diagonalize(&disorder, 1.0);
        let (eigs_upstream, vecs_upstream) =
            barracuda::special::anderson_diagonalize(&disorder, 1.0);
        for (a, b) in eigs_local.iter().zip(eigs_upstream.iter()) {
            assert!(
                (a - b).abs() < 1e-12,
                "Eigenvalue mismatch: local={a}, upstream={b}"
            );
        }
        for (a, b) in vecs_local.iter().zip(vecs_upstream.iter()) {
            assert!(
                (a - b).abs() < 1e-12,
                "Eigenvector mismatch: local={a}, upstream={b}"
            );
        }
    }
}
