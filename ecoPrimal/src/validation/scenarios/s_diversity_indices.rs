// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "diversity-indices",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp010",
            description: "α-diversity structural checks (Shannon, Simpson, Chao1).",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let uniform_4 = [0.25_f64, 0.25, 0.25, 0.25];
    let h = microbiome::shannon_index(&uniform_4);
    v.check_abs_or_rel(
        "shannon_uniform_four_taxa",
        h,
        4.0_f64.ln(),
        tolerances::MACHINE_EPSILON,
        tolerances::MACHINE_EPSILON,
    );

    let simp = microbiome::simpson_index(&uniform_4);
    v.check_bool(
        "simpson_uniform_four_gt_point_seven",
        simp > 0.7,
        &format!("Simpson(D)={simp}"),
    );

    let counts = [12_u64, 8, 5, 3, 2, 1, 1];
    let chao = microbiome::chao1(&counts);
    #[expect(clippy::cast_precision_loss, reason = "small OTU count")]
    let s_obs = counts.len() as f64;
    v.check_bool(
        "chao1_geq_sobs",
        chao + tolerances::MACHINE_EPSILON >= s_obs,
        &format!("Chao1={chao}, S_obs={s_obs}"),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "diversity_live_optional",
        "diversity stats local / barracuda-lib — optional IPC in math_dispatch",
    );
}
