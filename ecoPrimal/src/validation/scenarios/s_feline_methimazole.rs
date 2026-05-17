// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::feline::{self, FELINE_METHIMAZOLE};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "feline-methimazole",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp106",
            description: "Feline methimazole PK: Michaelis-Menten nonlinear elimination + T4 response.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Feline Methimazole PK");

    let (times, concs) = feline::methimazole_simulate(&FELINE_METHIMAZOLE, 2.5, 24.0, 0.01);

    v.check_bool(
        "initial_concentration_positive",
        concs[0] > 0.0,
        &format!("c0={}", concs[0]),
    );
    v.check_bool(
        "concentration_decays",
        concs[concs.len() - 1] < concs[0],
        &format!("c_last={}", concs[concs.len() - 1]),
    );
    v.check_bool(
        "all_concentrations_nonneg",
        concs.iter().all(|&c| c >= 0.0),
        "non-negative",
    );

    let t_half = feline::methimazole_apparent_half_life(&FELINE_METHIMAZOLE, concs[0]);
    v.check_bool(
        "half_life_positive",
        t_half > 0.0,
        &format!("t_half={t_half}"),
    );

    let css = feline::methimazole_css(&FELINE_METHIMAZOLE, 1.0);
    v.check_bool(
        "css_exists_for_low_rate",
        css.is_some(),
        &format!("css={css:?}"),
    );

    let t4 = feline::t4_response(8.0, 2.5, 14.0);
    v.check_bool(
        "t4_decreases_from_baseline",
        t4 < 8.0,
        &format!("t4_14d={t4}, baseline=8.0"),
    );

    let _ = times;
}
