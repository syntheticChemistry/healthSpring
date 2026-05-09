// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::{CompositionContext, validate_parity_flex};
use primalspring::tolerances::CPU_GPU_PARITY_TOL;
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
            id: "barracuda-parity",
            track: Track::Composition,
            tier: Tier::Live,
            source_experiment: "exp122",
            description: "barraCuda tensor stats parity — IPC vs local math_dispatch baselines.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let data = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let mean_local = math_dispatch::mean(&data);
    v.check_abs_or_rel(
        "math_dispatch_mean_closed_form",
        mean_local,
        5.5,
        tolerances::DETERMINISM,
        tolerances::DETERMINISM,
    );

    let hill_local = math_dispatch::hill(10.0, 10.0, 1.0);
    v.check_abs_or_rel(
        "math_dispatch_hill_midpoint",
        hill_local,
        0.5,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");

    validate_parity_flex(
        ctx,
        v,
        "barracuda_stats_mean_ipc",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": data}),
        &["result", "mean"],
        mean_local,
        CPU_GPU_PARITY_TOL,
    );

    let spread = [2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let sd_local = math_dispatch::std_dev(&spread).unwrap_or(f64::NAN);
    if sd_local.is_finite() {
        validate_parity_flex(
            ctx,
            v,
            "barracuda_stats_std_dev_ipc",
            "tensor",
            "stats.std_dev",
            serde_json::json!({"data": spread}),
            &["std_dev", "result"],
            sd_local,
            CPU_GPU_PARITY_TOL,
        );
    } else {
        v.check_skip(
            "barracuda_stats_std_dev_ipc",
            "local std_dev unavailable (IPC-only build)",
        );
    }
}
