// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome;
use crate::qs::{self, QsGeneMatrix};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "qs-augmented-anderson",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp107",
            description: "QS-augmented Anderson disorder improves colonization resistance prediction.",
        },
        run,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "lattice size L < 2^52 — lossless"
)]
fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — QS gene density and profile");

    let matrix = test_matrix();
    let abundances = &microbiome::communities::HEALTHY_GUT;

    let density = qs::qs_gene_density(abundances, &matrix, qs::QsFamily::LuxIR);
    v.check_bool(
        "luxir_density_positive",
        density > 0.0,
        &format!("density={density:.6}"),
    );

    let profile = qs::qs_profile(abundances, &matrix);
    v.check_bool(
        "total_qs_density_positive",
        profile.total_qs_density > 0.0,
        &format!("total_qs_density={:.6}", profile.total_qs_density),
    );
    v.check_bool(
        "signaling_diversity_non_negative",
        profile.signaling_diversity >= 0.0,
        &format!("signaling_diversity={:.6}", profile.signaling_diversity),
    );

    v.section("Phase 2: Structural — Effective disorder modulation");

    let pielou = microbiome::pielou_evenness(abundances);
    let alpha = 0.7;
    let w_scale = 10.0;

    let w_eff = qs::effective_disorder(pielou, &profile, alpha, w_scale);
    let w_structural = pielou * w_scale;

    v.check_bool(
        "qs_modulates_disorder",
        (w_eff - w_structural).abs() > tolerances::MACHINE_EPSILON,
        &format!("w_eff={w_eff:.4}, w_structural={w_structural:.4}"),
    );

    v.section("Phase 3: Structural — Anderson lattice with QS disorder");

    let l = 50;
    let t_hop = 1.0;
    let disorder: Vec<f64> = (0..l)
        .map(|i| w_eff * (i as f64 / l as f64 - 0.5))
        .collect();

    let h_mat = microbiome::anderson_hamiltonian_1d(&disorder, t_hop);
    v.check_bool(
        "hamiltonian_flat_size",
        h_mat.len() == l * l,
        &format!("len={}, expected={}", h_mat.len(), l * l),
    );

    let mut delta = vec![0.0; l];
    delta[l / 2] = 1.0;
    let ipr = microbiome::inverse_participation_ratio(&delta);
    v.check_abs_or_rel(
        "ipr_delta_is_one",
        ipr,
        1.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}

fn test_matrix() -> QsGeneMatrix {
    QsGeneMatrix {
        species: vec![
            "Bacteroides".into(),
            "Clostridioides".into(),
            "Faecalibacterium".into(),
            "Roseburia".into(),
            "Escherichia".into(),
        ],
        families: vec!["LuxIR".into(), "LuxS".into(), "Agr".into()],
        presence: vec![
            vec![true, true, false],
            vec![false, true, true],
            vec![true, false, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    }
}
