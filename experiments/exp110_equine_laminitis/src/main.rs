// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp110: Equine laminitis inflammatory cascade (CM-008)
//!
//! Validates hoof lamellae as Anderson lattice, inflammatory cytokine gradient,
//! phenylbutazone Michaelis-Menten PK, and cross-species allometric comparison.
//! References: Pollitt 2004 Clin Tech Equine Pract 3:34,
//! Lees et al. 2004 JVPT 27:397.

use healthspring_barracuda::comparative::species_params::{
    allometric_clearance, allometric_half_life, allometric_volume,
};
use healthspring_barracuda::microbiome::{
    anderson_hamiltonian_1d, inverse_participation_ratio, localization_length_from_ipr,
};
use healthspring_barracuda::pkpd::{MichaelisMentenParams, hill_dose_response, mm_pk_simulate};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{ANDERSON_IDENTITY, DETERMINISM, MACHINE_EPSILON};
use healthspring_barracuda::validation::ValidationHarness;

// Phenylbutazone in horses — Lees et al. 2004 JVPT 27:397 (capacity-limited metabolism)
const PHENYLBUTAZONE_PARAMS: MichaelisMentenParams = MichaelisMentenParams {
    vmax: 800.0,
    km: 25.0,
    vd: 45.0,
};

// Hoof lamellae lattice
const L: usize = 100;
const T_HOP: f64 = 1.0;

// Inflammatory cascade: LPS → TNF-α (Hill kinetics)
const TNF_EC50: f64 = 10.0;
const TNF_HILL_N: f64 = 1.5;

fn main() {
    let mut h = ValidationHarness::new("exp110_equine_laminitis");

    log_analytical(&AnalyticalProvenance {
        formula: "dC/dt = -Vmax×C/(Km+C)",
        reference: "Lees 2004 JVPT 27:397",
        doi: None,
    });

    // 1. Hoof lamellae as lattice substrate (L=100 sites)
    #[expect(clippy::cast_precision_loss, reason = "L < 2^52")]
    let disorder: Vec<f64> = (0..L)
        .map(|i| {
            let x = i as f64 / L as f64;
            2.0 * (x - 0.5) * (x - 0.5)
        })
        .collect();
    let h_mat = anderson_hamiltonian_1d(&disorder, T_HOP);
    h.check_exact("Lamellae lattice size", h_mat.len() as u64, (L * L) as u64);

    // 2. Anderson disorder from inflammatory cytokine gradient (TNF-α, IL-1β, IL-6 spatial)
    let disorder_max = disorder.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let disorder_min = disorder.iter().copied().fold(f64::INFINITY, f64::min);
    h.check_bool(
        "Cytokine gradient creates spatial disorder",
        disorder_max > disorder_min,
    );

    // 3. Hamiltonian symmetric
    let mut symmetric = true;
    for i in 0..L {
        for j in 0..L {
            if (h_mat[i * L + j] - h_mat[j * L + i]).abs() > ANDERSON_IDENTITY {
                symmetric = false;
            }
        }
    }
    h.check_bool("Anderson symmetric", symmetric);

    // 4. Phenylbutazone Michaelis-Menten: monotonic decline
    let (_, concs) = mm_pk_simulate(&PHENYLBUTAZONE_PARAMS, 500.0, 24.0, 0.01);
    let monotone = concs.windows(2).all(|w| w[1] <= w[0] + ANDERSON_IDENTITY);
    h.check_bool("Phenylbutazone monotonic decline", monotone);

    // 5. Localization length ξ: laminar failure threshold (short ξ → failure risk)
    let ipr = 0.05;
    let xi = localization_length_from_ipr(ipr);
    h.check_abs("ξ = 1/IPR", xi, 20.0, MACHINE_EPSILON);

    // 6. ξ predicts failure: shorter ξ → more localized → higher failure risk
    let xi_short = localization_length_from_ipr(0.2);
    let xi_long = localization_length_from_ipr(0.02);
    h.check_bool("Shorter ξ → more localized", xi_short < xi_long);

    // 7. Inflammatory cascade: LPS → TNF-α (Hill kinetics)
    let r_at_ec50 = hill_dose_response(TNF_EC50, TNF_EC50, TNF_HILL_N, 1.0);
    h.check_abs("TNF-α Hill at EC50 → 50%", r_at_ec50, 0.5, MACHINE_EPSILON);

    // 8. Hill monotonicity for cytokine amplification
    let r_low = hill_dose_response(5.0, TNF_EC50, TNF_HILL_N, 1.0);
    let r_high = hill_dose_response(20.0, TNF_EC50, TNF_HILL_N, 1.0);
    h.check_bool("TNF-α Hill monotonic", r_high > r_low);

    // 9. Cross-species: equine vs canine allometric CL
    let equine_bw = 500.0;
    let canine_bw = 15.0;
    let cl_ref = 1.0;
    let cl_equine = allometric_clearance(cl_ref, canine_bw, equine_bw);
    let cl_canine = allometric_clearance(cl_ref, canine_bw, canine_bw);
    h.check_bool(
        "Equine CL per kg < canine (allometric)",
        cl_equine / equine_bw < cl_canine / canine_bw,
    );

    // 10. Allometric volume: 2× weight → 2× Vd
    let vd_2x = allometric_volume(10.0, 70.0, 140.0);
    h.check_abs("Allometric Vd 2× weight", vd_2x, 20.0, MACHINE_EPSILON);

    // 11. Allometric half-life identity
    let t_half = allometric_half_life(10.0, 2.0);
    let expected = core::f64::consts::LN_2 * 10.0 / 2.0;
    h.check_abs("t½ = ln(2)×Vd/CL", t_half, expected, MACHINE_EPSILON);

    // 12. IPR of delta state = 1.0
    let mut delta = vec![0.0; L];
    delta[L / 2] = 1.0;
    let ipr_d = inverse_participation_ratio(&delta);
    h.check_abs("IPR(delta) = 1.0", ipr_d, 1.0, ANDERSON_IDENTITY);

    // 13. Diagonal = disorder
    let diag_ok = (0..L).all(|i| (h_mat[i * L + i] - disorder[i]).abs() < ANDERSON_IDENTITY);
    h.check_bool("Diagonal = disorder", diag_ok);

    // 14. Phenylbutazone C(0) = dose/Vd
    let c0 = 500.0 / PHENYLBUTAZONE_PARAMS.vd;
    h.check_abs(
        "Phenylbutazone C(0) = dose/Vd",
        concs[0],
        c0,
        MACHINE_EPSILON,
    );

    // 15. Determinism
    let (_, c1) = mm_pk_simulate(&PHENYLBUTAZONE_PARAMS, 500.0, 12.0, 0.01);
    let (_, c2) = mm_pk_simulate(&PHENYLBUTAZONE_PARAMS, 500.0, 12.0, 0.01);
    let last1 = c1.last().copied().unwrap_or(0.0);
    let last2 = c2.last().copied().unwrap_or(0.0);
    h.check_abs("Determinism", last1, last2, DETERMINISM);

    h.exit();
}
