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
            id: "trt-outcomes",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp033",
            description: "TRT outcome trajectories: weight, HbA1c, hazard ratio structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Weight Trajectory");

    let w0 = endocrine::weight_trajectory(0.0, -5.0, 6.0, 24.0);
    v.check_abs_or_rel(
        "weight_at_month_0_is_zero",
        w0,
        0.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let w_end = endocrine::weight_trajectory(24.0, -5.0, 6.0, 24.0);
    v.check_bool(
        "weight_loss_at_end",
        w_end < 0.0,
        &format!("delta_weight(24)={w_end}"),
    );

    v.section("Phase 1b: HbA1c Trajectory");

    let hba1c_0 = endocrine::hba1c_trajectory(0.0, 7.5, -1.0, 6.0);
    v.check_abs_or_rel(
        "hba1c_at_month_0_equals_baseline",
        hba1c_0,
        7.5,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let hba1c_12 = endocrine::hba1c_trajectory(12.0, 7.5, -1.0, 6.0);
    v.check_bool(
        "hba1c_decreases_over_time",
        hba1c_12 < hba1c_0,
        &format!("hba1c(12)={hba1c_12}"),
    );

    v.section("Phase 1c: Hazard Ratio");

    let hr_low = endocrine::hazard_ratio_model(200.0, 300.0, 1.5);
    let hr_high = endocrine::hazard_ratio_model(500.0, 300.0, 1.5);
    v.check_bool(
        "low_t_higher_hazard",
        hr_low > hr_high,
        &format!("HR(200)={hr_low}, HR(500)={hr_high}"),
    );
}
