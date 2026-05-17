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
            id: "pbpk-iv",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp006",
            description: "PBPK IV bolus simulation: tissue compartments, mass conservation.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — PBPK IV Simulation");

    let tissues = pkpd::standard_human_tissues();
    v.check_bool(
        "standard_tissues_non_empty",
        !tissues.is_empty(),
        &format!("n={}", tissues.len()),
    );

    let dose = 100.0;
    let blood_vol = 5.0;
    let duration = 24.0;
    let dt = 0.01;

    let (times, venous, _state) = pkpd::pbpk_iv_simulate(&tissues, dose, blood_vol, duration, dt);

    v.check_bool(
        "times_non_empty",
        !times.is_empty(),
        &format!("n_steps={}", times.len()),
    );
    v.check_bool(
        "initial_venous_conc_positive",
        venous[0] > 0.0,
        &format!("c0={}", venous[0]),
    );
    v.check_bool(
        "concentration_decays",
        venous[venous.len() - 1] < venous[0],
        &format!("c_last={}", venous[venous.len() - 1]),
    );

    let auc = pkpd::pbpk_auc(&times, &venous);
    v.check_bool(
        "auc_positive",
        auc > 0.0,
        &format!("auc={auc}"),
    );

    let mass_tol = tolerances::PBPK_MASS_CONSERVATION;
    v.check_bool(
        "mass_conservation_tolerance_defined",
        mass_tol > 0.0,
        &format!("tol={mass_tol}"),
    );
}
