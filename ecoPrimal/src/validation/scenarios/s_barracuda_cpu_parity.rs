// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::math_dispatch;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "barracuda-cpu-parity",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp040",
            description: "V16 CPU math primitives — Hill, Shannon, Simpson, Chao1, Bray-Curtis vs analytical baselines.",
        },
        run,
    }
}

fn run(vr: &mut ValidationResult, _ctx: &mut CompositionContext) {
    vr.section("Phase 1: Statistical Primitives");

    let data = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    vr.check_abs_or_rel(
        "cpu_mean_arithmetic",
        math_dispatch::mean(&data),
        5.5,
        tolerances::DETERMINISM,
        tolerances::DETERMINISM,
    );

    if let Some(sd) = math_dispatch::std_dev(&data) {
        let expected_sd = 3.027_650_354_097_491_6;
        vr.check_abs_or_rel(
            "cpu_std_dev_sample",
            sd,
            expected_sd,
            1e-10,
            1e-10,
        );
    }

    vr.section("Phase 2: Hill Dose-Response");

    vr.check_abs_or_rel(
        "cpu_hill_at_ic50",
        math_dispatch::hill(10.0, 10.0, 1.0),
        0.5,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    vr.check_abs_or_rel(
        "cpu_hill_at_2x_ic50",
        math_dispatch::hill(20.0, 10.0, 1.0),
        2.0 / 3.0,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let steep = math_dispatch::hill(10.0, 10.0, 4.0);
    vr.check_abs_or_rel(
        "cpu_hill_steep_n4",
        steep,
        0.5,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    vr.section("Phase 3: Diversity Indices");

    let freqs = [0.25_f64, 0.25, 0.25, 0.25];
    let shannon = math_dispatch::shannon_from_frequencies(&freqs);
    let expected_shannon = 4.0_f64.ln();
    vr.check_abs_or_rel(
        "cpu_shannon_uniform_4",
        shannon,
        expected_shannon,
        1e-12,
        1e-12,
    );

    let abundances = [10.0_f64, 10.0, 10.0, 10.0];
    let simpson = math_dispatch::simpson(&abundances);
    vr.check_abs_or_rel(
        "cpu_simpson_uniform_4",
        simpson,
        0.75,
        1e-12,
        1e-12,
    );

    let counts: Vec<u64> = vec![10, 5, 3, 1, 1, 1, 1, 1, 1, 1];
    let chao1 = math_dispatch::chao1_classic(&counts);
    #[expect(clippy::cast_precision_loss, reason = "species count fits f64")]
    let s_obs = counts.iter().filter(|&&ct| ct > 0).count() as f64;
    vr.check_bool(
        "cpu_chao1_ge_observed",
        chao1 >= s_obs,
        &format!("Chao1={chao1} >= S_obs={s_obs}"),
    );

    vr.section("Phase 4: Distance & Similarity");

    let site_a = [1.0_f64, 2.0, 3.0];
    let site_b = [1.0_f64, 2.0, 3.0];
    vr.check_abs_or_rel(
        "cpu_bray_curtis_identical",
        math_dispatch::bray_curtis(&site_a, &site_b),
        0.0,
        tolerances::DETERMINISM,
        tolerances::DETERMINISM,
    );

    let site_c = [1.0_f64, 0.0, 0.0];
    let site_d = [0.0_f64, 0.0, 1.0];
    vr.check_abs_or_rel(
        "cpu_bray_curtis_disjoint",
        math_dispatch::bray_curtis(&site_c, &site_d),
        1.0,
        tolerances::DETERMINISM,
        tolerances::DETERMINISM,
    );
}
