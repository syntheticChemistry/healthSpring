// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::canine::{self, CanineIl31Treatment};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "pruritus-vas",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp102",
            description: "Canine pruritus VAS time course: oclacitinib vs lokivetmab onset kinetics.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Pruritus Time Course");

    let (times, vas) = canine::pruritus_time_course(
        44.5,
        CanineIl31Treatment::Oclacitinib,
        168.0,
        7,
    );

    v.check_bool(
        "time_course_correct_length",
        times.len() == 7 && vas.len() == 7,
        &format!("t_len={}, vas_len={}", times.len(), vas.len()),
    );

    let vas_first = vas[0];
    let vas_last = vas[vas.len() - 1];
    v.check_bool(
        "vas_decreases_under_treatment",
        vas_last < vas_first,
        &format!("vas_first={vas_first}, vas_last={vas_last}"),
    );

    let untreated_vas = canine::pruritus_vas_response(44.5);
    v.check_bool(
        "untreated_vas_positive",
        untreated_vas > 0.0,
        &format!("untreated_vas={untreated_vas}"),
    );
}
