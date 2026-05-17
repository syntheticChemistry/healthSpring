// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::toxicology;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "hormesis-optimum",
            track: Track::Toxicology,
            tier: Tier::Rust,
            source_experiment: "exp099",
            description: "Hormetic optimum search and biphasic dose-response boundary conditions.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Biphasic Curve Shape");

    let baseline = 1.0;
    let s_max = 0.4;
    let k_stim = 15.0;
    let ic50 = 150.0;
    let hill_n = 1.5;

    let r_zero = toxicology::biphasic_dose_response(0.0, baseline, s_max, k_stim, ic50, hill_n);
    v.check_abs_or_rel(
        "biphasic_zero_dose_baseline",
        r_zero,
        baseline,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let r_stim = toxicology::biphasic_dose_response(k_stim, baseline, s_max, k_stim, ic50, hill_n);
    v.check_bool(
        "stimulatory_zone_above_baseline",
        r_stim > baseline,
        &format!("r_stim={r_stim}"),
    );

    let r_toxic = toxicology::biphasic_dose_response(ic50 * 5.0, baseline, s_max, k_stim, ic50, hill_n);
    v.check_bool(
        "toxic_zone_below_baseline",
        r_toxic < baseline,
        &format!("r_toxic={r_toxic}"),
    );

    v.section("Phase 1b: Optimum Search");

    let (opt_dose, opt_fit) = toxicology::hormetic_optimum(
        baseline, s_max, k_stim, ic50, hill_n, 300.0, 10_000,
    );
    v.check_bool(
        "optimum_dose_in_stimulatory_range",
        opt_dose > 0.0 && opt_dose < ic50,
        &format!("opt_dose={opt_dose}"),
    );
    v.check_bool(
        "optimum_fitness_above_baseline",
        opt_fit > baseline,
        &format!("opt_fit={opt_fit}"),
    );

    let r_at_opt = toxicology::biphasic_dose_response(opt_dose, baseline, s_max, k_stim, ic50, hill_n);
    v.check_abs_or_rel(
        "optimum_matches_biphasic_curve",
        r_at_opt,
        opt_fit,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}
