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
            id: "diabetes-trt",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp035",
            description: "TRT diabetes biomarker trajectories: weight + HbA1c combined response.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Combined TRT Metabolic Response");

    let months = [0.0, 3.0, 6.0, 12.0, 24.0];
    let mut prev_w = 0.0_f64;
    let mut prev_hba1c = 7.5_f64;

    for &m in &months {
        let w = endocrine::weight_trajectory(m, -5.0, 6.0, 24.0);
        let hba1c = endocrine::hba1c_trajectory(m, 7.5, -1.0, 6.0);

        if m > 0.0 {
            v.check_bool(
                &format!("weight_monotonic_at_month_{m}"),
                w <= prev_w + tolerances::MACHINE_EPSILON,
                &format!("w({m})={w}, w_prev={prev_w}"),
            );
            v.check_bool(
                &format!("hba1c_monotonic_at_month_{m}"),
                hba1c <= prev_hba1c + tolerances::MACHINE_EPSILON,
                &format!("hba1c({m})={hba1c}, hba1c_prev={prev_hba1c}"),
            );
        }
        prev_w = w;
        prev_hba1c = hba1c;
    }

    let biomarker_0 = endocrine::biomarker_trajectory(0.0, 1.0, 0.5, 6.0);
    v.check_abs_or_rel(
        "biomarker_at_t0_equals_baseline",
        biomarker_0,
        1.0,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );
}
