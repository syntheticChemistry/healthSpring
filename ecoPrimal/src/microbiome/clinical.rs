// SPDX-License-Identifier: AGPL-3.0-or-later
//! Clinical gut microbiome models: FMT, antibiotic perturbation, SCFA, gut-brain.
//!
//! Extracted from the microbiome root module for single-responsibility.
//! All functions are re-exported from `microbiome::*`.
//!
//! ## barraCuda delegation
//!
//! - `antibiotic_perturbation_abundances`: Delegates to `barracuda::health::microbiome::antibiotic_perturbation`
//!   (species-level exponential kill).
//! - `antibiotic_perturbation`: Kept local — Shannon diversity time course (Dethlefsen & Relman 2011);
//!   barraCuda has species-level perturbation only.
//! - `scfa_production`, `gut_serotonin_production`, `tryptophan_availability`: Kept local — different
//!   signatures (fiber-first vs params-first, Shannon-based vs `microbiome_factor`) and parameter sets.

// ── FMT Microbiota Transplant ──────────────────────────────────────────

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
///
/// Delegates to `barracuda::stats::bray_curtis`. When `a` and `b` have
/// different lengths, pads the shorter with zeros to preserve API compatibility.
#[must_use]
pub fn bray_curtis(a: &[f64], b: &[f64]) -> f64 {
    if a.len() == b.len() {
        barracuda::stats::bray_curtis(a, b)
    } else {
        let n = a.len().max(b.len());
        let mut a_pad = vec![0.0; n];
        let mut b_pad = vec![0.0; n];
        a_pad[..a.len()].copy_from_slice(a);
        b_pad[..b.len()].copy_from_slice(b);
        barracuda::stats::bray_curtis(&a_pad, &b_pad)
    }
}

// ── Antibiotic Perturbation Model ──────────────────────────────────────

/// Species-level antibiotic perturbation: exponential kill with species-specific susceptibility.
///
/// Delegates to `barracuda::health::microbiome::antibiotic_perturbation`.
/// Returns perturbed abundance vector. For Shannon diversity time course, use
/// `antibiotic_perturbation` (below) instead.
#[must_use]
pub fn antibiotic_perturbation_abundances(
    abundances: &[f64],
    susceptibilities: &[f64],
    duration_h: f64,
) -> Vec<f64> {
    barracuda::health::microbiome::antibiotic_perturbation(abundances, susceptibilities, duration_h)
}

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
            (h0 - h_nadir).mul_add(-(1.0 - (-k_decline * t).exp()), h0)
        } else {
            let t_rec = t - treatment_days;
            let h_at_end = (h0 - h_nadir).mul_add(-(1.0 - (-k_decline * treatment_days).exp()), h0);
            let recovery_target = h0 * depth.mul_add(-0.15, 1.0);
            (recovery_target - h_at_end).mul_add(1.0 - (-k_recovery * t_rec).exp(), h_at_end)
        };
        result.push((t, h));
    }
    result
}

// ── SCFA Production Model ──────────────────────────────────────────────

/// Michaelis-Menten SCFA production parameters per acid.
///
/// Literature: Cummings & Macfarlane 1991, J Appl Bacteriol; den Besten et al. 2013,
/// J Lipid Res (doi:10.1194/jlr.R036012); Topping & Clifton 2001, Physiol Rev.
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
///
/// Michaelis-Menten kinetics: v = Vmax·\[fiber\]/(Km + \[fiber\]).
/// References: Cummings & Macfarlane 1991, J Appl Bacteriol; den Besten et al. 2013,
/// J Lipid Res (doi:10.1194/jlr.R036012); Topping & Clifton 2001, Physiol Rev.
pub const SCFA_HEALTHY_PARAMS: ScfaParams = ScfaParams {
    vmax_acetate: 60.0,
    km_acetate: 8.0,
    vmax_propionate: 20.0,
    km_propionate: 10.0,
    vmax_butyrate: 15.0,
    km_butyrate: 12.0,
};

/// Dysbiotic SCFA params (reduced butyrate producers).
///
/// Reduced butyrate Vmax reflects loss of butyrate-producing taxa.
/// References: den Besten et al. 2013, J Lipid Res (doi:10.1194/jlr.R036012);
/// Topping & Clifton 2001, Physiol Rev.
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

// ── Gut-Brain Serotonin Pathway ────────────────────────────────────────

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
