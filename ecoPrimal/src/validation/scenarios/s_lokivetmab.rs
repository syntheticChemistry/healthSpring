// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::canine;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "lokivetmab-duration",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp103",
            description: "Lokivetmab PK, effective duration, onset structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Lokivetmab PK");

    let dose = 2.0;
    let bw = 10.0;
    let c0 = canine::lokivetmab_pk(dose, bw, 0.0);
    v.check_bool(
        "lokivetmab_c0_positive",
        c0 > 0.0,
        &format!("c0={c0}"),
    );

    let c_late = canine::lokivetmab_pk(dose, bw, 60.0);
    v.check_bool(
        "lokivetmab_decays",
        c_late < c0,
        &format!("c(60d)={c_late}"),
    );

    v.section("Phase 1b: Effective Duration");

    let duration = canine::lokivetmab_effective_duration(dose, bw, 1.0);
    v.check_bool(
        "effective_duration_positive",
        duration > 0.0,
        &format!("duration={duration} days"),
    );

    let duration_high = canine::lokivetmab_effective_duration(4.0, bw, 1.0);
    v.check_bool(
        "higher_dose_longer_duration",
        duration_high >= duration - tolerances::MACHINE_EPSILON,
        &format!("d(2mg/kg)={duration}, d(4mg/kg)={duration_high}"),
    );

    v.section("Phase 1c: Onset");

    let onset = canine::lokivetmab_onset_hr(dose);
    v.check_bool(
        "onset_within_24hr",
        onset > 0.0 && onset < 24.0,
        &format!("onset={onset} hr"),
    );
}
