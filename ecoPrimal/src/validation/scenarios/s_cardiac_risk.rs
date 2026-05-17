// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::endocrine;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "cardiac-risk-trt",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp034",
            description: "Cardiac risk composite from SDNN/testosterone structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Cardiac Risk Composite");

    let risk_low_sdnn = endocrine::cardiac_risk_composite(50.0, 300.0, 0.1);
    let risk_high_sdnn = endocrine::cardiac_risk_composite(150.0, 300.0, 0.1);
    v.check_bool(
        "low_sdnn_higher_risk",
        risk_low_sdnn > risk_high_sdnn,
        &format!("risk(50ms)={risk_low_sdnn}, risk(150ms)={risk_high_sdnn}"),
    );

    let risk_low_t = endocrine::cardiac_risk_composite(100.0, 200.0, 0.1);
    let risk_high_t = endocrine::cardiac_risk_composite(100.0, 500.0, 0.1);
    v.check_bool(
        "low_testosterone_higher_risk",
        risk_low_t > risk_high_t,
        &format!("risk(T=200)={risk_low_t}, risk(T=500)={risk_high_t}"),
    );

    let risk_base = endocrine::cardiac_risk_composite(100.0, 300.0, 0.0);
    v.check_bool(
        "zero_baseline_risk_bounded",
        risk_base >= 0.0,
        &format!("risk={risk_base}"),
    );
}
