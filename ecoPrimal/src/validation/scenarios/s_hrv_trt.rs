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
            id: "hrv-trt-response",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp038",
            description: "HRV response to TRT: SDNN trajectory structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — HRV TRT Response");

    let sdnn_0 = endocrine::hrv_trt_response(80.0, 40.0, 6.0, 0.0);
    v.check_abs_or_rel(
        "hrv_at_month_0_near_baseline",
        sdnn_0,
        80.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let sdnn_12 = endocrine::hrv_trt_response(80.0, 40.0, 6.0, 12.0);
    v.check_bool(
        "sdnn_improves_over_time",
        sdnn_12 >= sdnn_0 - tolerances::MACHINE_EPSILON,
        &format!("sdnn(0)={sdnn_0}, sdnn(12)={sdnn_12}"),
    );
}
