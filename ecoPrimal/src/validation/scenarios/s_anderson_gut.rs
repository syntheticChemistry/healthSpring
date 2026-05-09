// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "anderson-gut",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp011",
            description: "Anderson 1-D lattice Hamiltonian symmetry and diagonal disorder wiring.",
        },
        run,
    }
}

fn run(validation: &mut ValidationResult, comp_ctx: &mut CompositionContext) {
    validation.section("Phase 1: Structural");

    let disorder = vec![1.0_f64, -0.5, 0.3, 0.8];
    let dim = disorder.len();
    let ham = microbiome::anderson_hamiltonian_1d(&disorder, 1.0);

    let mut symmetric = true;
    for row in 0..dim {
        for col in 0..dim {
            let h_ij = ham[row * dim + col];
            let h_ji = ham[col * dim + row];
            if (h_ij - h_ji).abs() > tolerances::MACHINE_EPSILON_TIGHT {
                symmetric = false;
            }
        }
    }
    validation.check_bool("anderson_h_symmetric", symmetric, "H_ij = H_ji");

    for site in 0..dim {
        let diag_ok =
            (ham[site * dim + site] - disorder[site]).abs() <= tolerances::MACHINE_EPSILON_TIGHT;
        validation.check_bool(
            &format!("anderson_diagonal_site_{site}"),
            diag_ok,
            "diagonal equals on-site disorder",
        );
    }

    if comp_ctx.available_capabilities().is_empty() {
        return;
    }

    validation.section("Phase 2: Live Composition");
    validation.check_skip(
        "anderson_live_optional",
        "Anderson diagonalize is domain-local unless routed via specialist primal",
    );
}
