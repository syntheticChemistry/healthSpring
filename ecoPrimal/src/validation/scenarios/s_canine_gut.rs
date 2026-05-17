// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome::anderson;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "canine-gut-anderson",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp105",
            description: "Canine gut Anderson localization applied to species-specific microbiome.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Canine Gut Anderson Model");

    let canine_disorder = [1.5, 2.0, 0.5, 1.0, 3.0, 0.8, 1.2, 2.5, 0.3, 1.8];
    let (eigs, evecs) = anderson::anderson_diagonalize(&canine_disorder, 1.0);

    v.check_bool(
        "eigenvalue_count_matches",
        eigs.len() == canine_disorder.len(),
        &format!("n={}", eigs.len()),
    );

    let ipr = anderson::inverse_participation_ratio(&evecs[..canine_disorder.len()]);
    let xi = anderson::localization_length_from_ipr(ipr);
    v.check_bool(
        "canine_localization_positive",
        xi > 0.0,
        &format!("xi={xi}"),
    );

    let cr = anderson::colonization_resistance(xi);
    v.check_bool(
        "canine_colonization_resistance_bounded",
        (0.0..=1.0).contains(&cr),
        &format!("CR={cr}"),
    );

    let healthy_canine = [0.5; 10];
    let (_, evecs_h) = anderson::anderson_diagonalize(&healthy_canine, 1.0);
    let ipr_h = anderson::inverse_participation_ratio(&evecs_h[..10]);
    let xi_h = anderson::localization_length_from_ipr(ipr_h);
    let cr_h = anderson::colonization_resistance(xi_h);
    v.check_bool(
        "healthy_canine_higher_cr",
        cr_h >= cr,
        &format!("cr_disordered={cr}, cr_healthy={cr_h}"),
    );
}
