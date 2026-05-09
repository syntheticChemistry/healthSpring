// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::pkpd::{self, pop_baricitinib};

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "population-pk",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "exp005",
            description: "Population PK Monte Carlo returns one exposure row per patient.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    let n = 37_usize;
    let seed = 2026_u64;
    let times: Vec<f64> = (0..48).map(f64::from).collect();

    let cohort = pkpd::population_pk_monte_carlo(
        n,
        seed,
        pop_baricitinib::CL,
        pop_baricitinib::VD,
        pop_baricitinib::KA,
        pop_baricitinib::DOSE_MG,
        pop_baricitinib::F_BIOAVAIL,
        &times,
    );

    v.check_count("population_mc_row_count", cohort.len(), n);

    let positives = cohort
        .iter()
        .filter(|r| r.cmax > 0.0 && r.auc > 0.0)
        .count();
    v.check_minimum("population_mc_positive_exposures", positives, n);

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");
    v.check_skip(
        "population_pk_live_optional",
        "Tier-1 Monte Carlo — no IPC surface",
    );
}
