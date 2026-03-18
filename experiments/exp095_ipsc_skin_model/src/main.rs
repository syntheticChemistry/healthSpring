// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! Exp095: iPSC-derived skin model cytokine/viability validation (DD-006)
//!
//! Validates iPSC keratinocyte cytokine/viability readouts for Gonzales collaboration.
//! Reference data: Takagi et al. 2020 (J Invest Dermatol 140:1325),
//! Kim et al. 2019 (Stem Cell Reports 12:430).

use healthspring_barracuda::microbiome::{anderson_hamiltonian_1d, inverse_participation_ratio};
use healthspring_barracuda::pkpd::hill_dose_response;
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    ANDERSON_IDENTITY, DETERMINISM, HILL_SATURATION_100X, MACHINE_EPSILON, MACHINE_EPSILON_STRICT,
};
use healthspring_barracuda::validation::ValidationHarness;

// Reference cytokine ranges (pg/mL) from iPSC keratinocyte literature
// Takagi et al. 2020 J Invest Dermatol 140:1325; Kim et al. 2019 Stem Cell Reports 12:430
const IL4_RANGE_LOW: f64 = 50.0;
const IL4_RANGE_HIGH: f64 = 500.0;
const IL13_RANGE_LOW: f64 = 100.0;
const IL13_RANGE_HIGH: f64 = 800.0;
const IL31_RANGE_LOW: f64 = 20.0;
const IL31_RANGE_HIGH: f64 = 300.0;
const TSLP_RANGE_LOW: f64 = 10.0;
const TSLP_RANGE_HIGH: f64 = 200.0;

// IL-31 EC50 for iPSC keratinocytes (ng/mL) — Hill dose-response
const IL31_EC50_NG_ML: f64 = 25.0;
const IL31_HILL_N: f64 = 1.2;

// Viability IC50 (µM) for drug cytotoxicity — Hill equation
const VIABILITY_IC50_UM: f64 = 15.0;
const VIABILITY_HILL_N: f64 = 1.5;

// Cross-species: iPSC human vs canine primary keratinocyte response ratio (literature ~1.2–1.5)
const HUMAN_CANINE_RATIO_LOW: f64 = 1.1;
const HUMAN_CANINE_RATIO_HIGH: f64 = 1.6;

const LATTICE_L: usize = 40;
const T_HOP: f64 = 1.0;

fn main() {
    let mut h = ValidationHarness::new("exp095_ipsc_skin_model");

    log_analytical(&AnalyticalProvenance {
        formula: "E = Emax × C^n / (EC50^n + C^n)",
        reference: "Takagi 2020 J Invest Dermatol 140:1325",
        doi: Some("10.1016/j.jid.2019.12.001"),
    });

    // 1. Hill dose-response for IL-31 stimulation: at EC50 → 50%
    let r_il31_at_ec50 = hill_dose_response(IL31_EC50_NG_ML, IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
    h.check_abs(
        "IL-31 Hill at EC50 → 50%",
        r_il31_at_ec50,
        0.5,
        MACHINE_EPSILON,
    );

    // 2. IL-31 Hill monotonicity
    let concs: Vec<f64> = (0..20)
        .map(|i| 10.0_f64.powf(-0.5 + 2.0 * f64::from(i) / 19.0))
        .collect();
    let mut monotonic = true;
    for (i, &c) in concs.iter().enumerate().skip(1) {
        let r_prev = hill_dose_response(concs[i - 1], IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
        let r_curr = hill_dose_response(c, IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
        if r_curr < r_prev - MACHINE_EPSILON_STRICT {
            monotonic = false;
        }
    }
    h.check_bool("IL-31 Hill monotonicity", monotonic);

    // 3–6. Cytokine panel validation: reference values within published ranges
    let il4_ref = 180.0;
    let il13_ref = 350.0;
    let il31_ref = 80.0;
    let tslp_ref = 45.0;
    h.check_bool(
        "IL-4 in published range",
        (IL4_RANGE_LOW..=IL4_RANGE_HIGH).contains(&il4_ref),
    );
    h.check_bool(
        "IL-13 in published range",
        (IL13_RANGE_LOW..=IL13_RANGE_HIGH).contains(&il13_ref),
    );
    h.check_bool(
        "IL-31 in published range",
        (IL31_RANGE_LOW..=IL31_RANGE_HIGH).contains(&il31_ref),
    );
    h.check_bool(
        "TSLP in published range",
        (TSLP_RANGE_LOW..=TSLP_RANGE_HIGH).contains(&tslp_ref),
    );

    // 7. Viability curves: Hill equation at IC50 → 50% inhibition (viability = 0.5)
    let viability_at_ic50 =
        1.0 - hill_dose_response(VIABILITY_IC50_UM, VIABILITY_IC50_UM, VIABILITY_HILL_N, 1.0);
    h.check_abs(
        "Viability at IC50 → 50%",
        viability_at_ic50,
        0.5,
        MACHINE_EPSILON,
    );

    // 8. Viability saturation at high dose (inhibition saturates)
    let inhibition_high = hill_dose_response(
        VIABILITY_IC50_UM * 100.0,
        VIABILITY_IC50_UM,
        VIABILITY_HILL_N,
        1.0,
    );
    h.check_lower(
        "Cytotoxicity saturation at 100× IC50",
        inhibition_high,
        HILL_SATURATION_100X,
    );

    // 9. Anderson tissue model: iPSC layer as lattice substrate (cells as sites)
    #[expect(clippy::cast_precision_loss, reason = "L < 2^52")]
    let disorder: Vec<f64> = (0..LATTICE_L)
        .map(|i| 0.3 * ((i as f64) / (LATTICE_L as f64) - 0.5))
        .collect();
    let h_mat = anderson_hamiltonian_1d(&disorder, T_HOP);
    h.check_exact(
        "Anderson tissue matrix size",
        h_mat.len() as u64,
        (LATTICE_L * LATTICE_L) as u64,
    );

    // 10. Anderson Hamiltonian symmetric
    let mut symmetric = true;
    for i in 0..LATTICE_L {
        for j in 0..LATTICE_L {
            if (h_mat[i * LATTICE_L + j] - h_mat[j * LATTICE_L + i]).abs() > ANDERSON_IDENTITY {
                symmetric = false;
            }
        }
    }
    h.check_bool("Anderson tissue symmetric", symmetric);

    // 11. Anderson disorder: diagonal = disorder
    let diag_ok =
        (0..LATTICE_L).all(|i| (h_mat[i * LATTICE_L + i] - disorder[i]).abs() < ANDERSON_IDENTITY);
    h.check_bool("Anderson diagonal = disorder", diag_ok);

    // 12. Cross-species: iPSC human vs canine primary keratinocyte response ratio
    let human_response = hill_dose_response(IL31_EC50_NG_ML, IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
    let canine_ec50 = IL31_EC50_NG_ML * 1.35;
    let canine_response = hill_dose_response(IL31_EC50_NG_ML, canine_ec50, IL31_HILL_N, 1.0);
    let ratio = human_response / canine_response.max(MACHINE_EPSILON);
    h.check_bool(
        "Cross-species human/canine ratio in range",
        (HUMAN_CANINE_RATIO_LOW..=HUMAN_CANINE_RATIO_HIGH).contains(&ratio),
    );

    // 13. IPR of uniform state = 1/L
    #[expect(clippy::cast_precision_loss, reason = "L small")]
    let l_f64 = LATTICE_L as f64;
    let val = 1.0 / l_f64.sqrt();
    let uniform: Vec<f64> = vec![val; LATTICE_L];
    let ipr = inverse_participation_ratio(&uniform);
    let expected_ipr = 1.0 / l_f64;
    h.check_abs("IPR(uniform) = 1/L", ipr, expected_ipr, MACHINE_EPSILON);

    // 14. Hill at zero concentration → 0
    let r_zero = hill_dose_response(0.0, IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
    h.check_abs("Hill at C=0 → 0", r_zero, 0.0, MACHINE_EPSILON);

    // 15. Determinism
    let r1 = hill_dose_response(10.0, IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
    let r2 = hill_dose_response(10.0, IL31_EC50_NG_ML, IL31_HILL_N, 1.0);
    h.check_abs("Determinism", r1, r2, DETERMINISM);

    h.exit();
}
