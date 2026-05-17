// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::ppg;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "ppg-spo2",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp022",
            description: "PPG SpO2 estimation: Beer-Lambert R-value to saturation mapping.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — PPG SpO2");

    let synth = ppg::generate_synthetic_ppg(500.0, 5.0, 72.0, 97.0, 42);

    v.check_bool(
        "red_channel_populated",
        !synth.red.is_empty(),
        &format!("n={}", synth.red.len()),
    );
    v.check_bool(
        "ir_channel_populated",
        !synth.ir.is_empty(),
        &format!("n={}", synth.ir.len()),
    );

    let (ac_red, dc_red) = ppg::ppg_extract_ac_dc(&synth.red);
    let (ac_ir, dc_ir) = ppg::ppg_extract_ac_dc(&synth.ir);

    v.check_bool(
        "dc_red_positive",
        dc_red > 0.0,
        &format!("dc_red={dc_red}"),
    );
    v.check_bool(
        "dc_ir_positive",
        dc_ir > 0.0,
        &format!("dc_ir={dc_ir}"),
    );

    let r = ppg::ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
    v.check_bool(
        "r_value_positive",
        r > 0.0,
        &format!("r={r}"),
    );

    let spo2 = ppg::spo2_from_r(r);
    v.check_bool(
        "spo2_in_physiological_range",
        spo2 > 80.0 && spo2 <= 100.0,
        &format!("spo2={spo2}"),
    );
}
