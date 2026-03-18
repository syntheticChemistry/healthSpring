// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! healthSpring Exp031 — Testosterone Pellet Depot PK (Rust validation)

use healthspring_barracuda::endocrine::{self, pellet_params as pp, testosterone_cypionate as tc};
use healthspring_barracuda::tolerances::{
    AUC_TRUNCATED, LOGNORMAL_MEAN, LOGNORMAL_RECOVERY, MACHINE_EPSILON, MACHINE_EPSILON_TIGHT,
    WASHOUT_HALF_LIFE,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp031 Testosterone Pellet PK");

    let c_ss = pp::RELEASE_RATE / (tc::VD_L * tc::K_E);

    // --- Check 1: C(0) = 0 ---
    let c0 = endocrine::pellet_concentration(
        0.0,
        pp::RELEASE_RATE,
        tc::K_E,
        tc::VD_L,
        pp::DURATION_DAYS,
    );
    h.check_abs("C(0) = 0", c0, 0.0, MACHINE_EPSILON);

    // --- Check 2: Approaches steady-state by 5 half-lives ---
    let c_5hl = endocrine::pellet_concentration(
        5.0 * tc::T_HALF_DAYS,
        pp::RELEASE_RATE,
        tc::K_E,
        tc::VD_L,
        pp::DURATION_DAYS,
    );
    let ratio = c_5hl / c_ss;
    h.check_lower("C(5t½)/C_ss > 0.95", ratio, 0.95);

    // --- Check 3: Stable plateau CV < 5% ---
    let plateau: Vec<f64> = (0..800)
        .filter_map(|i: i32| {
            let t = 60.0 + 80.0 * f64::from(i) / 799.0;
            if t <= 140.0 {
                Some(endocrine::pellet_concentration(
                    t,
                    pp::RELEASE_RATE,
                    tc::K_E,
                    tc::VD_L,
                    pp::DURATION_DAYS,
                ))
            } else {
                None
            }
        })
        .collect();
    #[expect(
        clippy::cast_precision_loss,
        reason = "plateau.len() < 1000 fits in f64"
    )]
    let mean_p: f64 = plateau.iter().sum::<f64>() / plateau.len() as f64;
    #[expect(
        clippy::cast_precision_loss,
        reason = "plateau.len() < 1000 fits in f64"
    )]
    let var_p: f64 =
        plateau.iter().map(|c| (c - mean_p).powi(2)).sum::<f64>() / plateau.len() as f64;
    let cv = var_p.sqrt() / mean_p;
    h.check_upper("Plateau CV < 5%", cv, LOGNORMAL_RECOVERY);

    // --- Check 4: Plateau > 0 ---
    h.check_lower("Mean plateau > 0", mean_p, 0.0);

    // --- Check 5: Washout begins after duration ---
    let c_end = endocrine::pellet_concentration(
        pp::DURATION_DAYS,
        pp::RELEASE_RATE,
        tc::K_E,
        tc::VD_L,
        pp::DURATION_DAYS,
    );
    let c_post = endocrine::pellet_concentration(
        180.0,
        pp::RELEASE_RATE,
        tc::K_E,
        tc::VD_L,
        pp::DURATION_DAYS,
    );
    h.check_upper("C(6mo) < 50% C(end)", c_post, c_end * 0.50);

    // --- Check 6: Washout t½ ≈ 8 days ---
    let half_c = c_end / 2.0;
    let mut t_half_obs = None;
    for i in 0..1000_i32 {
        let t = pp::DURATION_DAYS + 30.0 * f64::from(i) / 999.0;
        let c = endocrine::pellet_concentration(
            t,
            pp::RELEASE_RATE,
            tc::K_E,
            tc::VD_L,
            pp::DURATION_DAYS,
        );
        if c <= half_c {
            t_half_obs = Some(t - pp::DURATION_DAYS);
            break;
        }
    }
    if let Some(th) = t_half_obs {
        h.check_rel("Washout t½", th, tc::T_HALF_DAYS, WASHOUT_HALF_LIFE);
    } else {
        h.check_bool("Washout t½ crossing found", false);
    }

    // --- Check 7: Pellet fluctuation < 10% ---
    let p_max = plateau.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let p_min = plateau.iter().copied().fold(f64::INFINITY, f64::min);
    let fluct = if p_min > 0.0 {
        (p_max - p_min) / p_min
    } else {
        f64::INFINITY
    };
    h.check_upper("Pellet fluctuation < 10%", fluct, 0.10);

    // --- Check 8: AUC proportional to dose ---
    let n = 3000usize;
    #[expect(clippy::cast_precision_loss, reason = "n < 10000 fits in f64")]
    let dt = 180.0 / (n - 1) as f64;
    let mut auc = 0.0;
    let mut c_prev = 0.0_f64;
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "loop index and n fit in f64")]
        let t = 180.0 * (i as f64) / ((n - 1) as f64);
        let c = endocrine::pellet_concentration(
            t,
            pp::RELEASE_RATE,
            tc::K_E,
            tc::VD_L,
            pp::DURATION_DAYS,
        );
        if i > 0 {
            auc = (0.5 * dt).mul_add(c_prev + c, auc);
        }
        c_prev = c;
    }
    let auc_ana = pp::DOSE_MG / (tc::VD_L * tc::K_E);
    h.check_rel("AUC ≈ D/(Vd*ke)", auc, auc_ana, AUC_TRUNCATED);

    // --- Check 9: Dose-weight scaling ---
    let dose_150 = 10.0 * 150.0;
    let rr_150 = dose_150 / pp::DURATION_DAYS;
    let c_150: Vec<f64> = (0..800)
        .filter_map(|i: i32| {
            let t = 60.0 + 80.0 * f64::from(i) / 799.0;
            if t <= 140.0 {
                Some(endocrine::pellet_concentration(
                    t,
                    rr_150,
                    tc::K_E,
                    tc::VD_L,
                    pp::DURATION_DAYS,
                ))
            } else {
                None
            }
        })
        .collect();
    #[expect(clippy::cast_precision_loss, reason = "c_150.len() < 1000 fits in f64")]
    let mean_150 = c_150.iter().sum::<f64>() / c_150.len() as f64;
    let ratio_c = if mean_150 > 0.0 {
        mean_p / mean_150
    } else {
        0.0
    };
    let ratio_d = pp::DOSE_MG / dose_150;
    h.check_rel("Dose scaling ratio", ratio_c, ratio_d, LOGNORMAL_MEAN);

    // --- Check 10: All non-negative ---
    let all_nn = (0..3000_i32).all(|i| {
        let t = 180.0 * f64::from(i) / 2999.0;
        endocrine::pellet_concentration(t, pp::RELEASE_RATE, tc::K_E, tc::VD_L, pp::DURATION_DAYS)
            >= -MACHINE_EPSILON_TIGHT
    });
    h.check_bool("All non-negative", all_nn);

    h.exit();
}
