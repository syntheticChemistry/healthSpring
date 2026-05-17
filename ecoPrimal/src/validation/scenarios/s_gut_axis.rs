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
            id: "gut-trt-axis",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp037",
            description: "Gut-testosterone axis: Anderson disorder-to-metabolic response.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Gut Metabolic Response");

    let xi_healthy = 5.0;
    let xi_max = 10.0;
    let base = 1.0;
    let response_healthy = endocrine::gut_metabolic_response(xi_healthy, xi_max, base);
    v.check_bool(
        "healthy_response_above_base",
        response_healthy >= base - tolerances::MACHINE_EPSILON,
        &format!("response={response_healthy}"),
    );

    let xi_disordered = 1.0;
    let response_disordered = endocrine::gut_metabolic_response(xi_disordered, xi_max, base);
    v.check_bool(
        "disordered_response_below_healthy",
        response_disordered < response_healthy + tolerances::MACHINE_EPSILON,
        &format!("healthy={response_healthy}, disordered={response_disordered}"),
    );

    v.section("Phase 1b: Disorder-Evenness Mapping");

    let disorder_high_evenness = endocrine::evenness_to_disorder(0.9, 5.0);
    let disorder_low_evenness = endocrine::evenness_to_disorder(0.3, 5.0);
    v.check_bool(
        "high_evenness_lower_disorder",
        disorder_high_evenness < disorder_low_evenness,
        &format!("W(J=0.9)={disorder_high_evenness}, W(J=0.3)={disorder_low_evenness}"),
    );
}
