// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome::clinical;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "scfa-serotonin",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp079",
            description: "SCFA production and gut-brain serotonin axis structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — SCFA Production");

    let params = &clinical::SCFA_HEALTHY_PARAMS;
    let (acetate, propionate, butyrate) = clinical::scfa_production(10.0, params);

    v.check_bool(
        "acetate_positive",
        acetate > 0.0,
        &format!("acetate={acetate}"),
    );
    v.check_bool(
        "propionate_positive",
        propionate > 0.0,
        &format!("propionate={propionate}"),
    );
    v.check_bool(
        "butyrate_positive",
        butyrate > 0.0,
        &format!("butyrate={butyrate}"),
    );
    v.check_bool(
        "acetate_dominant",
        acetate > propionate && acetate > butyrate,
        &format!("acetate={acetate}, propionate={propionate}, butyrate={butyrate}"),
    );

    let (a0, _, _) = clinical::scfa_production(0.0, params);
    v.check_abs_or_rel(
        "zero_fiber_zero_scfa",
        a0,
        0.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 1b: Gut-Brain Serotonin");

    let trp_available = clinical::tryptophan_availability(100.0, 3.0);
    v.check_bool(
        "tryptophan_positive",
        trp_available > 0.0,
        &format!("trp={trp_available}"),
    );

    let serotonin = clinical::gut_serotonin_production(trp_available, 3.0, 0.1, 1.0);
    v.check_bool(
        "serotonin_positive",
        serotonin > 0.0,
        &format!("serotonin={serotonin}"),
    );
}
