// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::toxicology::{self, ToxicityModelParams};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "tox-landscape-extended",
            track: Track::Toxicology,
            tier: Tier::Rust,
            source_experiment: "exp098",
            description: "Extended toxicity landscape: multi-tissue dose-scaling structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Dose-Dependent Landscape");

    let model = ToxicityModelParams {
        hill_n: 2.0,
        km: 100.0,
        clearance_threshold: 0.9,
    };

    let low_dose = toxicology::compute_toxicity_landscape(
        10.0, &[50.0, 100.0], &[0.5, 0.3], &[0.8, 0.6], &model,
    );
    let high_dose = toxicology::compute_toxicity_landscape(
        200.0, &[50.0, 100.0], &[0.5, 0.3], &[0.8, 0.6], &model,
    );

    v.check_bool(
        "systemic_burden_increases_with_dose",
        high_dose.systemic_burden >= low_dose.systemic_burden - tolerances::MACHINE_EPSILON,
        &format!("low={}, high={}", low_dose.systemic_burden, high_dose.systemic_burden),
    );

    v.check_bool(
        "ipr_bounded_by_tissue_count",
        low_dose.tox_ipr >= 1.0 && high_dose.tox_ipr >= 1.0,
        &format!("ipr_low={}, ipr_high={}", low_dose.tox_ipr, high_dose.tox_ipr),
    );
}
