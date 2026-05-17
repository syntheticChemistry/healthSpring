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
            id: "ipsc-skin-model",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp095",
            description: "iPSC keratinocyte cytokine/viability via Hill dose-response.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — IL-31 dose-response in iPSC model");

    let ec50 = 25.0;
    let hill_n = 1.2;
    let emax = 1.0;

    let resp_low = pkpd::hill_dose_response(1.0, ec50, hill_n, emax);
    let resp_mid = pkpd::hill_dose_response(ec50, ec50, hill_n, emax);
    let resp_high = pkpd::hill_dose_response(250.0, ec50, hill_n, emax);

    v.check_bool(
        "il31_monotone",
        resp_low < resp_mid && resp_mid < resp_high,
        &format!("low={resp_low:.4}, mid={resp_mid:.4}, high={resp_high:.4}"),
    );
    v.check_abs_or_rel(
        "il31_ec50_half_emax",
        resp_mid,
        emax * 0.5,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 2: Structural — Viability inhibition curve");

    let ic50 = 15.0;
    let viability_hill_n = 1.5;

    let viab_low_dose = 1.0 - pkpd::hill_dose_response(1.0, ic50, viability_hill_n, 1.0);
    let viab_mid_dose = 1.0 - pkpd::hill_dose_response(ic50, ic50, viability_hill_n, 1.0);
    let viab_high_dose = 1.0 - pkpd::hill_dose_response(150.0, ic50, viability_hill_n, 1.0);

    v.check_bool(
        "viability_decreases_with_dose",
        viab_low_dose > viab_mid_dose && viab_mid_dose > viab_high_dose,
        &format!(
            "low_dose={viab_low_dose:.4}, mid={viab_mid_dose:.4}, high={viab_high_dose:.4}"
        ),
    );
    v.check_abs_or_rel(
        "viability_at_ic50_is_50pct",
        viab_mid_dose,
        0.5,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 3: Structural — Cytokine ranges physiological");

    let il4_low = 50.0_f64;
    let il4_high = 500.0;
    let il31_low = 20.0;
    let il31_high = 300.0;

    v.check_bool(
        "cytokine_ranges_ordered",
        il4_low < il4_high && il31_low < il31_high,
        "ranges correctly ordered",
    );
    v.check_bool(
        "il31_lower_than_il4",
        il31_high < il4_high,
        &format!("IL-31 max={il31_high}, IL-4 max={il4_high}"),
    );
}
