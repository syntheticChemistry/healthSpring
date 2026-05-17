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
            id: "two-compartment-pk",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp003",
            description: "Two-compartment IV PK structural identities (macro-constants, mass conservation).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Two-Compartment IV");

    let k10 = 0.1;
    let k12 = 0.05;
    let k21 = 0.08;
    let (alpha, beta) = pkpd::micro_to_macro(k10, k12, k21);

    v.check_bool(
        "alpha_gt_beta",
        alpha > beta,
        &format!("alpha={alpha}, beta={beta}"),
    );

    v.check_abs_or_rel(
        "macro_sum_equals_micro_sum",
        alpha + beta,
        k10 + k12 + k21,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let dose = 100.0;
    let vd = 50.0;
    let c0 = dose / vd;
    let (a, b) = pkpd::two_compartment_ab(c0, alpha, beta, k21);

    v.check_abs_or_rel(
        "ab_sum_equals_c0",
        a + b,
        c0,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let c_0 = pkpd::pk_two_compartment_iv(dose, vd, alpha, beta, k21, 0.0);
    v.check_abs_or_rel(
        "concentration_at_t0_equals_c0",
        c_0,
        c0,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let c_late = pkpd::pk_two_compartment_iv(dose, vd, alpha, beta, k21, 100.0);
    v.check_bool(
        "concentration_decays_toward_zero",
        c_late < c0 * 0.01,
        &format!("c(100)={c_late}"),
    );
}
