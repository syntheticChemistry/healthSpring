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
            id: "gut-brain-serotonin",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp080",
            description: "Gut-brain serotonin axis: diversity → tryptophan → 5-HT causal chain.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Diversity drives tryptophan availability");

    let dietary_trp = 200.0;
    let h_healthy = microbiome::shannon_index(&microbiome::communities::HEALTHY_GUT);
    let h_dysbiotic = microbiome::shannon_index(&microbiome::communities::DYSBIOTIC_GUT);

    let trp_healthy = microbiome::tryptophan_availability(dietary_trp, h_healthy);
    let trp_dysbiotic = microbiome::tryptophan_availability(dietary_trp, h_dysbiotic);

    v.check_bool(
        "diversity_drives_trp",
        trp_healthy > trp_dysbiotic,
        &format!("healthy={trp_healthy:.4}, dysbiotic={trp_dysbiotic:.4}"),
    );
    v.check_bool(
        "trp_positive",
        trp_healthy > 0.0 && trp_dysbiotic > 0.0,
        "both positive",
    );

    v.section("Phase 2: Structural — Serotonin production ordering");

    let k_synth = 0.8;
    let scale = 0.1;

    let ser_healthy = microbiome::gut_serotonin_production(trp_healthy, h_healthy, k_synth, scale);
    let ser_dysbiotic =
        microbiome::gut_serotonin_production(trp_dysbiotic, h_dysbiotic, k_synth, scale);

    v.check_bool(
        "5ht_healthy_gt_dysbiotic",
        ser_healthy > ser_dysbiotic,
        &format!("healthy={ser_healthy:.4}, dysbiotic={ser_dysbiotic:.4}"),
    );

    let h_cdiff = microbiome::shannon_index(&microbiome::communities::CDIFF_COLONIZED);
    let ser_cdiff = microbiome::gut_serotonin_production(
        microbiome::tryptophan_availability(dietary_trp, h_cdiff),
        h_cdiff,
        k_synth,
        scale,
    );
    v.check_bool(
        "5ht_ordering_healthy_gt_cdiff_gt_dysbiotic",
        ser_healthy > ser_cdiff && ser_cdiff > ser_dysbiotic,
        &format!("healthy={ser_healthy:.4}, cdiff={ser_cdiff:.4}, dysbiotic={ser_dysbiotic:.4}"),
    );

    v.section("Phase 3: Structural — Monotonicity and boundary");

    let ser_zero_trp = microbiome::gut_serotonin_production(0.0, h_healthy, k_synth, scale);
    v.check_abs_or_rel(
        "zero_trp_zero_5ht",
        ser_zero_trp,
        0.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let low = microbiome::gut_serotonin_production(100.0, 0.5, 1.0, 0.1);
    let mid = microbiome::gut_serotonin_production(100.0, 1.5, 1.0, 0.1);
    let high = microbiome::gut_serotonin_production(100.0, 2.5, 1.0, 0.1);
    v.check_bool(
        "sigmoid_monotone",
        low < mid && mid < high,
        &format!("low={low:.4}, mid={mid:.4}, high={high:.4}"),
    );
}
