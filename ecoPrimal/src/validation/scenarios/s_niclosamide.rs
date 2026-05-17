// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::pkpd;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "niclosamide-delivery",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp096",
            description: "Niclosamide repurposing: PBPK, Hill inhibition, delivery optimization.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Niclosamide Hill inhibition");

    let ic50 = 1.0;
    let hill_n = 1.8;
    let emax = 1.0;

    let resp_sub = pkpd::hill_dose_response(0.1, ic50, hill_n, emax);
    let resp_ic50 = pkpd::hill_dose_response(ic50, ic50, hill_n, emax);
    let resp_supra = pkpd::hill_dose_response(10.0, ic50, hill_n, emax);

    v.check_bool(
        "inhibition_monotone",
        resp_sub < resp_ic50 && resp_ic50 < resp_supra,
        &format!("sub={resp_sub:.4}, ic50={resp_ic50:.4}, supra={resp_supra:.4}"),
    );
    v.check_abs_or_rel(
        "inhibition_at_ic50",
        resp_ic50,
        0.5,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 2: Structural — Oral PK and AUC");

    let dose = 500.0;
    let ka = 0.5;
    let ke = 0.15;
    let f_bio = 0.10;
    let vd = 100.0;

    let times: Vec<f64> = (0..=48).map(|i| f64::from(i) * 0.5).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pkpd::pk_oral_one_compartment(t, dose, ka, ke, f_bio, vd))
        .collect();

    v.check_bool("conc_at_t0_zero", concs[0] < tolerances::MACHINE_EPSILON, "");

    let peak_idx = concs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map_or(0, |(i, _)| i);
    v.check_bool("tmax_after_dosing", peak_idx > 0, &format!("tmax_idx={peak_idx}"));

    let auc = pkpd::auc_trapezoidal(&times, &concs);
    v.check_bool("auc_positive", auc > 0.0, &format!("AUC={auc:.4}"));

    v.section("Phase 3: Structural — Low bioavailability challenge");

    let oral_auc = auc;
    let iv_concs: Vec<f64> = times
        .iter()
        .map(|&t| (dose / vd) * (-ke * t).exp())
        .collect();
    let iv_auc = pkpd::auc_trapezoidal(&times, &iv_concs);

    v.check_bool(
        "oral_auc_much_less_than_iv",
        oral_auc < iv_auc * 0.2,
        &format!("oral={oral_auc:.2}, iv={iv_auc:.2}, ratio={:.3}", oral_auc / iv_auc),
    );
}
