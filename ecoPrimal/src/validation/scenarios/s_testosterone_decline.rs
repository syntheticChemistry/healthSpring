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
            id: "testosterone-decline",
            track: Track::Endocrine,
            tier: Tier::Rust,
            source_experiment: "exp032",
            description: "Age-related testosterone decline (BLSA model) structural identities.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Testosterone Decline");

    let t0 = 600.0;
    let rate = 0.01;
    let onset = 30.0;

    let t_at_onset = endocrine::testosterone_decline(t0, rate, onset, onset);
    v.check_abs_or_rel(
        "decline_at_onset_equals_t0",
        t_at_onset,
        t0,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let t_at_50 = endocrine::testosterone_decline(t0, rate, 50.0, onset);
    v.check_bool(
        "decline_at_50_below_baseline",
        t_at_50 < t0,
        &format!("T(50)={t_at_50}"),
    );

    let t_at_80 = endocrine::testosterone_decline(t0, rate, 80.0, onset);
    v.check_bool(
        "monotonic_decline",
        t_at_80 < t_at_50,
        &format!("T(50)={t_at_50}, T(80)={t_at_80}"),
    );

    v.section("Phase 1b: Age at Threshold");

    let threshold = 300.0;
    let age_at = endocrine::age_at_threshold(t0, rate, threshold, onset);
    v.check_bool(
        "threshold_age_after_onset",
        age_at > onset,
        &format!("age_at_threshold={age_at}"),
    );

    let t_at_threshold = endocrine::testosterone_decline(t0, rate, age_at, onset);
    v.check_abs_or_rel(
        "decline_at_threshold_age_matches",
        t_at_threshold,
        threshold,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}
