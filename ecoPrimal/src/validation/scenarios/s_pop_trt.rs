// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::endocrine;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "population-trt",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp036",
            description: "Population TRT Monte Carlo — lognormal parameter generation structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Population TRT Parameters");

    let (mu, sigma) = endocrine::lognormal_params(500.0, 0.3);
    v.check_bool(
        "lognormal_mu_positive",
        mu > 0.0,
        &format!("mu={mu}"),
    );
    v.check_bool(
        "lognormal_sigma_positive",
        sigma > 0.0,
        &format!("sigma={sigma}"),
    );

    let adjusted = endocrine::age_adjusted_t0(600.0, 50.0, 0.01);
    v.check_bool(
        "age_adjusted_t0_less_than_base",
        adjusted < 600.0,
        &format!("adjusted={adjusted}"),
    );

    let adjusted_young = endocrine::age_adjusted_t0(600.0, 20.0, 0.01);
    v.check_abs_or_rel(
        "young_age_minimal_adjustment",
        adjusted_young,
        600.0,
        tolerances::MACHINE_EPSILON * 100.0,
        tolerances::TEST_ASSERTION_LOOSE * 100.0,
    );
}
