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
            id: "real-16s-anderson",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp108",
            description: "Real 16S community profiles through Anderson localization pipeline.",
        },
        run,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "lattice size L < 2^52 — lossless"
)]
fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — HMP reference community diversity");

    let healthy_stool: Vec<f64> = vec![
        0.25, 0.03, 0.15, 0.12, 0.08, 0.04, 0.01, 0.06, 0.10, 0.16,
    ];
    let ibd_stool: Vec<f64> = vec![
        0.05, 0.01, 0.03, 0.02, 0.40, 0.20, 0.15, 0.02, 0.04, 0.08,
    ];

    let h_healthy = microbiome::shannon_index(&healthy_stool);
    let h_ibd = microbiome::shannon_index(&ibd_stool);

    v.check_bool(
        "healthy_higher_diversity",
        h_healthy > h_ibd,
        &format!("healthy={h_healthy:.4}, ibd={h_ibd:.4}"),
    );

    v.section("Phase 2: Structural — Anderson lattice from real profiles");

    let l = 50;
    let t_hop = 1.0;
    let w_scale = 10.0;

    let pielou_h = microbiome::pielou_evenness(&healthy_stool);
    let pielou_ibd = microbiome::pielou_evenness(&ibd_stool);

    let w_healthy = w_scale * (1.0 - pielou_h);
    let w_ibd = w_scale * (1.0 - pielou_ibd);

    v.check_bool(
        "ibd_higher_disorder",
        w_ibd > w_healthy,
        &format!("w_healthy={w_healthy:.4}, w_ibd={w_ibd:.4}"),
    );

    let disorder_h: Vec<f64> = (0..l)
        .map(|i| w_healthy * (i as f64 / l as f64 - 0.5))
        .collect();
    let h_mat_healthy = microbiome::anderson_hamiltonian_1d(&disorder_h, t_hop);
    v.check_bool(
        "healthy_hamiltonian_size",
        h_mat_healthy.len() == l * l,
        &format!("len={}", h_mat_healthy.len()),
    );

    let disorder_ibd: Vec<f64> = (0..l)
        .map(|i| w_ibd * (i as f64 / l as f64 - 0.5))
        .collect();
    let h_mat_ibd = microbiome::anderson_hamiltonian_1d(&disorder_ibd, t_hop);
    v.check_bool(
        "ibd_hamiltonian_size",
        h_mat_ibd.len() == l * l,
        &format!("len={}", h_mat_ibd.len()),
    );

    v.section("Phase 3: Structural — IPR and localization length");

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

    let loc = microbiome::localization_length_from_ipr(0.05);
    v.check_abs_or_rel(
        "localization_length_identity",
        loc,
        20.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 4: Structural — QS augmentation of real profiles");

    let matrix = hmp_qs_matrix();
    let profile = qs::qs_profile(&healthy_stool, &matrix);
    v.check_bool(
        "qs_total_density_positive",
        profile.total_qs_density > 0.0,
        &format!("total_qs={:.6}", profile.total_qs_density),
    );

    let w_eff = qs::effective_disorder(pielou_h, &profile, 0.7, w_scale);
    v.check_bool(
        "qs_modulates_disorder",
        (w_eff - w_healthy).abs() > tolerances::MACHINE_EPSILON,
        &format!("w_eff={w_eff:.4}, w_plain={w_healthy:.4}"),
    );
}

fn hmp_qs_matrix() -> QsGeneMatrix {
    QsGeneMatrix {
        species: vec![
            "Bacteroides".into(),
            "Clostridioides".into(),
            "Faecalibacterium".into(),
            "Roseburia".into(),
            "Escherichia".into(),
            "Enterococcus".into(),
            "Pseudomonas".into(),
            "Akkermansia".into(),
            "Bifidobacterium".into(),
            "Vibrio".into(),
        ],
        families: vec!["LuxIR".into(), "LuxS".into(), "Agr".into()],
        presence: vec![
            vec![true, true, false],
            vec![false, true, true],
            vec![true, false, false],
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, true],
            vec![true, false, true],
            vec![false, false, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    }
}
