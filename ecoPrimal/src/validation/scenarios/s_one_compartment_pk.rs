// SPDX-License-Identifier: AGPL-3.0-or-later

use core::f64::consts::LN_2;

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
            id: "one-compartment-pk",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp002",
            description: "One-compartment IV bolus C0 and half-life decay (pk_iv_bolus).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let dose = 100.0_f64;
    let vd = 10.0_f64;
    let k_e = 0.1_f64;
    let half_life_hr = LN_2 / k_e;

    let c0 = pkpd::pk_iv_bolus(dose, vd, half_life_hr, 0.0);
    let expected_c0 = dose / vd;
    v.check_abs_or_rel(
        "iv_bolus_c0_dose_over_vd",
        c0,
        expected_c0,
        tolerances::MACHINE_EPSILON,
        tolerances::MACHINE_EPSILON,
    );

    let c_half = pkpd::pk_iv_bolus(dose, vd, half_life_hr, half_life_hr);
    v.check_abs_or_rel(
        "iv_bolus_one_half_life",
        c_half,
        expected_c0 * 0.5,
        tolerances::MACHINE_EPSILON,
        tolerances::MACHINE_EPSILON,
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "one_compartment_live_optional",
        "use exp119 / composition client for science.pkpd.one_compartment_pk IPC",
    );
}
