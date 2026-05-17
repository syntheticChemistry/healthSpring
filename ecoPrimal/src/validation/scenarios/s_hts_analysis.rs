// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::discovery::hts;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "hts-analysis",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp091",
            description: "HTS Z'-factor, SSMD, hit classification structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — HTS Metrics");

    let z_prime = hts::z_prime_factor(0.9, 0.05, 0.1, 0.05);
    v.check_bool(
        "z_prime_excellent_assay",
        z_prime > 0.5,
        &format!("Z'={z_prime}"),
    );

    let ssmd = hts::ssmd(0.9, 0.05, 0.1, 0.05);
    v.check_bool(
        "ssmd_strong_effect",
        ssmd.abs() > 3.0,
        &format!("SSMD={ssmd}"),
    );

    let inhibition = hts::percent_inhibition(0.3, 0.9, 0.1);
    v.check_bool(
        "percent_inhibition_bounded",
        inhibition >= 0.0 && inhibition <= 100.0,
        &format!("inhibition={inhibition}%"),
    );

    let hit_class = hts::classify_ssmd(ssmd.abs());
    v.check_bool(
        "strong_ssmd_classified_as_hit",
        hit_class != hts::HitClass::Inactive,
        &format!("class={hit_class:?}"),
    );

    let z_bad = hts::z_prime_factor(0.5, 0.3, 0.4, 0.3);
    v.check_bool(
        "poor_assay_low_z_prime",
        z_bad < 0.5,
        &format!("Z'_bad={z_bad}"),
    );

    v.check_abs_or_rel(
        "full_inhibition_is_100",
        hts::percent_inhibition(0.1, 0.9, 0.1),
        100.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}
