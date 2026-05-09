// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::{CanineIl31Treatment, canine_jak_ic50_panel, il31_serum_kinetics};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "canine-il31",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp100",
            description: "Canine IL-31 kinetics and JAK1 selectivity structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let baseline = 44.5_f64;
    let t = 48.0_f64;
    let untreated = il31_serum_kinetics(baseline, t, CanineIl31Treatment::Untreated);
    let lokiv = il31_serum_kinetics(baseline, t, CanineIl31Treatment::Lokivetmab);
    v.check_bool(
        "lokivetmab_lowers_il31_vs_untreated",
        lokiv < untreated - tolerances::MACHINE_EPSILON,
        &format!("untreated={untreated}, lokiv={lokiv}"),
    );

    let panel = canine_jak_ic50_panel();
    let sel = panel.jak1_selectivity();
    v.check_bool(
        "oclacitinib_jak1_selectivity_gt_one",
        sel > 1.0,
        &format!("selectivity={sel}"),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip("canine_il31_live_optional", "comparative models local");
}
