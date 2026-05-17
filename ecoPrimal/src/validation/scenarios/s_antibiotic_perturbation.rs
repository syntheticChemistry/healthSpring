// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome::clinical::{self, AntibioticSimConfig};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "antibiotic-perturbation",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp078",
            description: "Antibiotic perturbation dynamics — diversity crash and recovery trajectory.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Antibiotic Perturbation");

    let cfg = AntibioticSimConfig {
        h0: 3.5,
        depth: 0.4,
        k_decline: 0.3,
        k_recovery: 0.05,
        treatment_days: 7.0,
        total_days: 100.0,
        dt: 0.1,
    };

    let trajectory = clinical::antibiotic_perturbation(&cfg);

    v.check_bool(
        "trajectory_non_empty",
        !trajectory.is_empty(),
        &format!("n_points={}", trajectory.len()),
    );

    let (_, h_start) = trajectory[0];
    let min_h = trajectory.iter().map(|(_, h)| *h).fold(f64::INFINITY, f64::min);
    v.check_bool(
        "diversity_crashes_below_initial",
        min_h < h_start,
        &format!("h_start={h_start}, min_h={min_h}"),
    );

    let (_, h_end) = trajectory[trajectory.len() - 1];
    v.check_bool(
        "diversity_recovers_toward_initial",
        h_end > min_h,
        &format!("h_end={h_end}, min_h={min_h}"),
    );
}
