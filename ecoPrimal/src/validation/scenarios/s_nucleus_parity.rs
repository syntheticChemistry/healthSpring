// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::{CompositionContext, validate_liveness};
use primalspring::tolerances::CPU_GPU_PARITY_TOL;
use primalspring::validation::ValidationResult;

use crate::composition::validate_parity;
use crate::math_dispatch;
use crate::pkpd;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "nucleus-parity",
            track: Track::Composition,
            tier: Tier::Both,
            source_experiment: "exp123",
            description: "NUCLEUS-style structural anchors plus live tensor parity.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let hill = math_dispatch::hill(12.0, 10.0, 2.0);
    v.check_bool(
        "nucleus_structural_hill_in_unit_interval",
        (0.0..=1.0).contains(&hill),
        &format!("hill={hill}"),
    );

    let c0 = pkpd::pk_iv_bolus(50.0, 25.0, 8.0, 0.0);
    v.check_abs_or_rel(
        "nucleus_structural_iv_c0",
        c0,
        2.0,
        tolerances::MACHINE_EPSILON,
        tolerances::MACHINE_EPSILON,
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");

    let alive = validate_liveness(ctx, v, &["tensor", "security", "storage"]);
    v.check_minimum("nucleus_min_capabilities_alive", alive, 0);

    validate_parity(
        ctx,
        v,
        "nucleus_tensor_mean_probe",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [2.0, 4.0, 6.0]}),
        "result",
        4.0,
        CPU_GPU_PARITY_TOL,
    );
}
