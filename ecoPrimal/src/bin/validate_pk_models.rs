// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! Thin aggregate validation binary for core PK models (NUCLEUS / CI entrypoint).
//!
//! Wraps the same analytical identities exercised by Exp001, Exp002, Exp005, and
//! Exp077 into a single process exit code.

use healthspring_barracuda::math_dispatch;
use healthspring_barracuda::pkpd::{
    self, PHENYTOIN_PARAMS, auc_trapezoidal, find_cmax_tmax, mm_pk_simulate, oral_tmax,
    pk_iv_bolus, pk_oral_one_compartment, pop_baricitinib, population_pk_cpu,
};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

/// Hill dose-response using [`math_dispatch::hill`] (same algebra as `pkpd::hill_dose_response`).
fn hill_response(concentration: f64, ic50: f64, hill_n: f64, e_max: f64) -> f64 {
    if ic50 <= 0.0 || concentration < 0.0 {
        return 0.0;
    }
    e_max * math_dispatch::hill(concentration, ic50, hill_n)
}

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "small validation grids only")]
            let frac = i as f64 / (n - 1) as f64;
            start + frac * (end - start)
        })
        .collect()
}

fn validate_hill(h: &mut ValidationHarness) {
    let drug = pkpd::ALL_INHIBITORS[0];
    let r_ic50 = hill_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);
    h.check_abs(
        "Hill (dispatch): JAK1 inhibitor at IC50 → 50%",
        r_ic50,
        0.5,
        tolerances::MACHINE_EPSILON,
    );

    let concs: Vec<f64> = (0..50)
        .map(|i| 10.0_f64.powf(-1.0 + 4.0 * f64::from(i) / 49.0))
        .collect();
    let responses: Vec<f64> = concs
        .iter()
        .map(|&c| hill_response(c, drug.ic50_jak1_nm, drug.hill_n, 1.0))
        .collect();
    let monotonic = responses
        .windows(2)
        .all(|w| w[0] <= w[1] + tolerances::MACHINE_EPSILON_STRICT);
    h.check_bool("Hill (dispatch): monotonic sweep", monotonic);

    let conc_sat = drug.ic50_jak1_nm * 100.0;
    let r_sat = hill_response(conc_sat, drug.ic50_jak1_nm, drug.hill_n, 1.0);
    h.check_lower(
        "Hill (dispatch): saturation at 100× IC50",
        r_sat,
        tolerances::HILL_SATURATION_100X,
    );
}

fn validate_one_compartment(h: &mut ValidationHarness) {
    const DOSE_IV: f64 = 500.0;
    const VD_IV: f64 = 50.0;
    const HL_IV: f64 = 6.0;

    const DOSE_ORAL: f64 = 250.0;
    const F_ORAL: f64 = 0.8;
    const VD_ORAL: f64 = 35.0;
    const HL_ORAL: f64 = 4.0;
    const KA_ORAL: f64 = 1.5;

    let times = linspace(0.0, 48.0, 1000);
    let k_e_iv = core::f64::consts::LN_2 / HL_IV;
    let k_e_oral = core::f64::consts::LN_2 / HL_ORAL;
    let c0_iv = DOSE_IV / VD_IV;

    let c_at_0 = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, 0.0);
    h.check_abs(
        "One-compartment IV: C(0) = Dose/Vd",
        c_at_0,
        c0_iv,
        tolerances::MACHINE_EPSILON,
    );

    let c_at_hl = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, HL_IV);
    h.check_abs(
        "One-compartment IV: C at t½ = C0/2",
        c_at_hl,
        c0_iv / 2.0,
        tolerances::HALF_LIFE_POINT,
    );

    let conc_iv: Vec<f64> = times
        .iter()
        .map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t))
        .collect();
    let auc_iv_num = auc_trapezoidal(&times, &conc_iv);
    let auc_iv_ana = DOSE_IV / (VD_IV * k_e_iv);
    h.check_rel(
        "One-compartment IV: AUC vs analytical",
        auc_iv_num,
        auc_iv_ana,
        tolerances::AUC_TRAPEZOIDAL,
    );

    let c_oral_0 = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e_oral, 0.0);
    h.check_abs(
        "One-compartment oral: C(0) = 0",
        c_oral_0.abs(),
        0.0,
        tolerances::MACHINE_EPSILON,
    );

    let c_oral: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e_oral, t))
        .collect();
    let (_, tmax) = find_cmax_tmax(&times, &c_oral);
    let tmax_ana = oral_tmax(KA_ORAL, k_e_oral);
    h.check_abs(
        "One-compartment oral: Tmax vs analytical",
        tmax,
        tmax_ana,
        tolerances::TMAX_NUMERICAL,
    );
}

fn validate_population_pk(h: &mut ValidationHarness) {
    let n_patients: usize = 20;
    let times: Vec<f64> = (0..500).map(|i| 24.0 * f64::from(i) / 499.0).collect();

    let cl_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 20")]
            let fi = i as f64;
            0.2f64.mul_add(fi, 8.0)
        })
        .collect();
    let vd_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 20")]
            let fi = i as f64;
            1.0f64.mul_add(fi, 70.0)
        })
        .collect();
    let ka_vals: Vec<f64> = (0..n_patients)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "i < 20")]
            let fi = i as f64;
            0.05f64.mul_add(fi, 1.0)
        })
        .collect();

    let results = population_pk_cpu(
        n_patients,
        &cl_vals,
        &vd_vals,
        &ka_vals,
        pop_baricitinib::DOSE_MG,
        pop_baricitinib::F_BIOAVAIL,
        &times,
    );

    h.check_exact(
        "Population PK MC: cohort size",
        results.len() as u64,
        n_patients as u64,
    );
    h.check_bool(
        "Population PK MC: all AUC > 0",
        results.iter().all(|r| r.auc > 0.0),
    );

    let first = results.first();
    let last = results.last();
    let ordering_ok = match (first, last) {
        (Some(f), Some(l)) => f.auc > l.auc,
        _ => false,
    };
    h.check_bool("Population PK MC: higher CL → lower AUC", ordering_ok);

    let aucs: Vec<f64> = results.iter().map(|r| r.auc).collect();
    let mean_dispatch = math_dispatch::mean(&aucs);
    #[expect(clippy::cast_precision_loss, reason = "n_patients = 20")]
    let sum_manual: f64 = aucs.iter().sum::<f64>() / (n_patients as f64);
    h.check_abs(
        "Population PK MC: mean via math_dispatch",
        mean_dispatch,
        sum_manual,
        tolerances::MACHINE_EPSILON,
    );
}

fn validate_michaelis_menten(h: &mut ValidationHarness) {
    let params = &PHENYTOIN_PARAMS;

    let (_, concs_short) = mm_pk_simulate(params, 300.0, 1.0, 0.001);
    let c0_expected = 300.0 / params.vd;
    if let Some(&c0) = concs_short.first() {
        h.check_abs(
            "Michaelis-Menten: C0 = dose/Vd",
            c0,
            c0_expected,
            tolerances::MACHINE_EPSILON,
        );
    } else {
        h.check_bool("Michaelis-Menten: non-empty concentration curve", false);
    }

    let (_, concs10) = mm_pk_simulate(params, 300.0, 10.0, 0.001);
    let monotone = concs10
        .windows(2)
        .all(|w| w[1] <= w[0] + tolerances::ANDERSON_IDENTITY);
    h.check_bool("Michaelis-Menten: monotone decline", monotone);

    let t_half_low = pkpd::mm_apparent_half_life(params, 1.0);
    let t_half_mid = pkpd::mm_apparent_half_life(params, 5.0);
    let t_half_high = pkpd::mm_apparent_half_life(params, 20.0);
    h.check_bool(
        "Michaelis-Menten: t½ increases with concentration",
        t_half_low < t_half_mid && t_half_mid < t_half_high,
    );

    let (_, concs_long) = mm_pk_simulate(params, 300.0, 20.0, 0.0001);
    let num_auc = pkpd::mm_auc(&concs_long, 0.0001);
    let anal_auc = pkpd::mm_auc_analytical(params, 300.0);
    h.check_rel(
        "Michaelis-Menten: numerical AUC vs analytical (math_dispatch::mm_auc)",
        num_auc,
        anal_auc,
        tolerances::TEST_ASSERTION_2_PERCENT,
    );
}

fn main() {
    let mut h = ValidationHarness::new("validate_pk_models");

    validate_hill(&mut h);
    validate_one_compartment(&mut h);
    validate_population_pk(&mut h);
    validate_michaelis_menten(&mut h);

    h.exit();
}
