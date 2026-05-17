// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome::anderson;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "colonization-resistance",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp013",
            description: "Colonization resistance via Anderson localization length (C. diff rCDI).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Colonization Resistance");

    let ordered_disorder = [0.1; 20];
    let (eigs, evecs) = anderson::anderson_diagonalize(&ordered_disorder, 1.0);

    v.check_bool(
        "eigenvalue_count_matches_sites",
        eigs.len() == 20,
        &format!("n_eigenvalues={}", eigs.len()),
    );

    let ipr = anderson::inverse_participation_ratio(&evecs[..20]);
    let xi = anderson::localization_length_from_ipr(ipr);
    v.check_bool(
        "localization_length_positive",
        xi > 0.0,
        &format!("xi={xi}"),
    );

    let cr_ordered = anderson::colonization_resistance(xi);
    v.check_bool(
        "colonization_resistance_bounded",
        (0.0..=1.0).contains(&cr_ordered),
        &format!("CR={cr_ordered}"),
    );

    let high_disorder = [5.0, 0.1, 4.0, 0.2, 3.5, 0.3, 4.5, 0.1, 5.0, 0.2,
                         4.0, 0.3, 3.0, 0.1, 4.5, 0.2, 5.0, 0.3, 3.5, 0.1];
    let (_, evecs_disordered) = anderson::anderson_diagonalize(&high_disorder, 1.0);
    let ipr_dis = anderson::inverse_participation_ratio(&evecs_disordered[..20]);
    let xi_dis = anderson::localization_length_from_ipr(ipr_dis);

    v.check_bool(
        "higher_disorder_shorter_localization",
        xi_dis < xi + tolerances::MACHINE_EPSILON,
        &format!("xi_ordered={xi}, xi_disordered={xi_dis}"),
    );

    let lsr = anderson::level_spacing_ratio(&eigs);
    v.check_bool(
        "level_spacing_ratio_positive",
        lsr > 0.0,
        &format!("lsr={lsr}"),
    );
}
