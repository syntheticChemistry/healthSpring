// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::toxicology::{self, ToxicityModelParams};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "toxicology-landscape",
            track: Track::Toxicology,
            tier: Tier::Rust,
            source_experiment: "exp097",
            description: "Toxicity landscape + biphasic hormesis structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Toxicity Landscape");

    let model = ToxicityModelParams {
        hill_n: 1.5,
        km: 50.0,
        clearance_threshold: 0.8,
    };
    let ic50s = [10.0, 100.0, 500.0, 1000.0];
    let sensitivities = [0.9, 0.3, 0.1, 0.05];
    let repair_caps = [0.2, 0.5, 0.8, 0.95];

    let landscape = toxicology::compute_toxicity_landscape(
        50.0,
        &ic50s,
        &sensitivities,
        &repair_caps,
        &model,
    );

    v.check_bool(
        "tox_landscape_tissue_count",
        landscape.n_tissues == 4,
        &format!("n_tissues={}", landscape.n_tissues),
    );

    v.check_bool(
        "tox_systemic_burden_positive",
        landscape.systemic_burden > 0.0,
        &format!("systemic_burden={}", landscape.systemic_burden),
    );

    #[expect(
        clippy::cast_precision_loss,
        reason = "small tissue count fits f64"
    )]
    let n_tissues_f64 = landscape.n_tissues as f64;
    v.check_bool(
        "tox_ipr_bounded",
        landscape.tox_ipr >= 1.0 && landscape.tox_ipr <= n_tissues_f64,
        &format!("tox_ipr={}", landscape.tox_ipr),
    );

    v.check_bool(
        "tox_clearance_within_threshold",
        landscape.clearance_linear || landscape.max_clearance_utilization <= 1.0,
        &format!(
            "clearance_linear={}, max_util={}",
            landscape.clearance_linear, landscape.max_clearance_utilization
        ),
    );

    v.section("Phase 1b: Structural — Biphasic Hormesis");

    let baseline = 1.0;
    let s_max = 0.5;
    let k_stim = 10.0;
    let ic50 = 100.0;
    let hill_n = 2.0;

    let r_zero = toxicology::biphasic_dose_response(0.0, baseline, s_max, k_stim, ic50, hill_n);
    v.check_bool(
        "hormesis_zero_dose_equals_baseline",
        (r_zero - baseline).abs() < 1e-12,
        &format!("r_zero={r_zero}, baseline={baseline}"),
    );

    let r_low = toxicology::biphasic_dose_response(5.0, baseline, s_max, k_stim, ic50, hill_n);
    v.check_bool(
        "hormesis_low_dose_above_baseline",
        r_low > baseline,
        &format!("r_low={r_low}"),
    );

    let r_high = toxicology::biphasic_dose_response(500.0, baseline, s_max, k_stim, ic50, hill_n);
    v.check_bool(
        "hormesis_high_dose_below_baseline",
        r_high < baseline,
        &format!("r_high={r_high}"),
    );

    let (opt_dose, opt_fitness) =
        toxicology::hormetic_optimum(baseline, s_max, k_stim, ic50, hill_n, 200.0, 10_000);
    v.check_bool(
        "hormetic_optimum_exists",
        opt_dose > 0.0 && opt_fitness > baseline,
        &format!("opt_dose={opt_dose}, opt_fitness={opt_fitness}"),
    );

    v.check_bool(
        "hormetic_optimum_between_endpoints",
        opt_dose > 0.0 && opt_dose < 200.0,
        &format!("opt_dose={opt_dose}"),
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip("toxicology_live_optional", "toxicology models local");
}
