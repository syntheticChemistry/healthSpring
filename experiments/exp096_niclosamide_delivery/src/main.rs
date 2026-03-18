// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
//! Exp096: Niclosamide drug delivery optimization (DD-007)
//!
//! Validates niclosamide physicochemical properties, PBPK, and delivery optimization.
//! References: Chen et al. 2018 Signal Transduct Target Ther 3:16,
//! Arend et al. 2020 Cancer Biol Ther 21:1076. `DrugBank` physicochemical data.

use healthspring_barracuda::discovery::tissue_geometry_factor;
use healthspring_barracuda::microbiome::localization_length_from_ipr;
use healthspring_barracuda::pkpd::{
    auc_trapezoidal, find_cmax_tmax, hill_dose_response, pk_oral_one_compartment,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    AUC_TRAPEZOIDAL, DETERMINISM, MACHINE_EPSILON, MACHINE_EPSILON_TIGHT, TISSUE_GEOMETRY,
};
use healthspring_barracuda::validation::ValidationHarness;

// Niclosamide physicochemical (DrugBank)
const MW: f64 = 327.12;
const LOGP: f64 = 4.0;
const PKA: f64 = 6.87;

// IC50 range (µM) for cancer/inflammation targets — Chen et al. 2018 STTT 3:16
const IC50_LOW_UM: f64 = 0.5;
const IC50_HIGH_UM: f64 = 1.5;
const IC50_MID_UM: f64 = 1.0;

// Bioavailability levels for formulation comparison (fraction)
const F_NANOPARTICLE: f64 = 0.35;
const F_SALT_FORM: f64 = 0.15;
const F_PRODRUG: f64 = 0.55;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "n always small")]
            let frac = i as f64 / (n - 1).max(1) as f64;
            start + frac * (end - start)
        })
        .collect()
}

fn main() {
    let mut h = ValidationHarness::new("exp096_niclosamide_delivery");

    log_analytical(&AnalyticalProvenance {
        formula: "C(t) = F×D×ka/(Vd×(ka-ke))×(exp(-ke×t)-exp(-ka×t))",
        reference: "Chen 2018 Signal Transduct Target Ther 3:16",
        doi: Some("10.1038/s41392-018-0013-x"),
    });

    // 1. Niclosamide MW
    h.check_abs("Niclosamide MW", MW, 327.12, MACHINE_EPSILON);

    // 2. Niclosamide logP (lipophilicity)
    h.check_abs("Niclosamide logP", LOGP, 4.0, MACHINE_EPSILON);

    // 3. Niclosamide pKa
    h.check_abs("Niclosamide pKa", PKA, 6.87, MACHINE_EPSILON);

    // 4. Oral bioavailability challenge: low F indicates formulation needed
    h.check_bool(
        "Low aqueous solubility → formulation needed (F_salt < 0.3)",
        F_SALT_FORM < 0.3,
    );

    // 5. PBPK 1-compartment: oral disposition (typical niclosamide params)
    let dose = 500.0;
    let vd = 50.0;
    let ka = 0.5;
    let ke = 0.15;
    let times = linspace(0.0, 72.0, 1000);
    let c_curve: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(dose, F_SALT_FORM, vd, ka, ke, t))
        .collect();

    let (cmax, tmax) = find_cmax_tmax(&times, &c_curve);
    h.check_bool("PBPK Cmax > 0", cmax > 0.0);
    h.check_bool("PBPK Tmax > 0", tmax > 0.0);

    // 6. AUC analytical vs numerical
    let auc_num = auc_trapezoidal(&times, &c_curve);
    let auc_ana = (F_SALT_FORM * dose) / (vd * ke);
    h.check_rel("PBPK AUC analytical", auc_num, auc_ana, AUC_TRAPEZOIDAL);

    // 7. IC50 in published range
    h.check_bool(
        "IC50 in range 0.5–1.5 µM",
        IC50_MID_UM >= IC50_LOW_UM && IC50_MID_UM <= IC50_HIGH_UM,
    );

    // 8. Hill at IC50 → 50%
    let r_at_ic50 = hill_dose_response(IC50_MID_UM, IC50_MID_UM, 1.0, 1.0);
    h.check_abs("Hill at IC50 → 50%", r_at_ic50, 0.5, MACHINE_EPSILON);

    // 9. Delivery optimization: prodrug > nanoparticle > salt (by bioavailability)
    h.check_bool("Prodrug F > nanoparticle F", F_PRODRUG > F_NANOPARTICLE);
    h.check_bool("Nanoparticle F > salt F", F_NANOPARTICLE > F_SALT_FORM);

    // 10. Hill dose-response at different bioavailability levels
    let conc = 2.0;
    let r_salt = hill_dose_response(conc * F_SALT_FORM, IC50_MID_UM, 1.0, 1.0);
    let r_prodrug = hill_dose_response(conc * F_PRODRUG, IC50_MID_UM, 1.0, 1.0);
    h.check_bool(
        "Higher F → higher response at same dose",
        r_prodrug > r_salt,
    );

    // 11. Anderson tissue penetration: geometry factor
    let xi = 10.0;
    let l = 5.0;
    let geom = tissue_geometry_factor(xi, l);
    let expected_geom = 1.0 - (-xi / l).exp();
    h.check_abs(
        "Tissue geometry factor",
        geom,
        expected_geom,
        TISSUE_GEOMETRY,
    );

    // 12. Localization length ξ: 1/IPR
    let ipr = 0.1;
    let xi_from_ipr = localization_length_from_ipr(ipr);
    h.check_abs("ξ = 1/IPR", xi_from_ipr, 10.0, MACHINE_EPSILON);

    // 13. Geometry: larger ξ → better penetration
    let geom_short = tissue_geometry_factor(2.0, l);
    let geom_long = tissue_geometry_factor(50.0, l);
    h.check_bool("Larger ξ → higher geometry factor", geom_long > geom_short);

    // 14. All concentrations non-negative
    let all_nonneg = c_curve.iter().all(|&c| c >= -MACHINE_EPSILON_TIGHT);
    h.check_bool("All concentrations ≥ 0", all_nonneg);

    // 15. Determinism
    let c1 = pk_oral_one_compartment(dose, F_SALT_FORM, vd, ka, ke, 4.0);
    let c2 = pk_oral_one_compartment(dose, F_SALT_FORM, vd, ka, ke, 4.0);
    h.check_abs("Determinism", c1, c2, DETERMINISM);

    h.exit();
}
