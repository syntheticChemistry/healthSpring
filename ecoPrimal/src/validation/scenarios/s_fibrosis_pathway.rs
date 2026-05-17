// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::discovery::fibrosis;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "fibrosis-pathway",
            track: Track::Discovery,
            tier: Tier::Rust,
            source_experiment: "exp094",
            description: "Rho→MRTF→SRF anti-fibrotic pathway scoring (CCG-1423 vs CCG-203971).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Fibrosis Pathway Scoring");

    let ccg1423 = fibrosis::ccg_1423();
    let ccg203971 = fibrosis::ccg_203971();

    v.check_bool(
        "compounds_have_distinct_ic50_profiles",
        (ccg1423.rho_ic50_um - ccg203971.rho_ic50_um).abs() > tolerances::DIVISION_GUARD,
        &format!("rho_ic50: {} vs {}", ccg1423.rho_ic50_um, ccg203971.rho_ic50_um),
    );

    let fi = fibrosis::fractional_inhibition(10.0, ccg1423.rho_ic50_um);
    v.check_bool(
        "fractional_inhibition_bounded",
        fi >= 0.0 && fi <= 1.0,
        &format!("fi={fi}"),
    );

    let fi_at_ic50 = fibrosis::fractional_inhibition(ccg1423.rho_ic50_um, ccg1423.rho_ic50_um);
    v.check_abs_or_rel(
        "inhibition_at_ic50_is_half",
        fi_at_ic50,
        0.5,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let score = fibrosis::score_anti_fibrotic(&ccg1423, 10.0);
    v.check_bool(
        "anti_fibrotic_score_positive",
        score.anti_fibrotic_score > 0.0,
        &format!("score={}", score.anti_fibrotic_score),
    );

    v.check_bool(
        "pathway_components_bounded",
        score.rho_inhibition >= 0.0
            && score.rho_inhibition <= 1.0
            && score.mrtf_block >= 0.0
            && score.mrtf_block <= 1.0
            && score.srf_reduction >= 0.0
            && score.srf_reduction <= 1.0,
        &format!(
            "rho={}, mrtf={}, srf={}",
            score.rho_inhibition, score.mrtf_block, score.srf_reduction
        ),
    );
}
