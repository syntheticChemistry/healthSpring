// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::microbiome::{self, clinical};
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "fmt-blend-rcdi",
            track: Track::Microbiome,
            tier: Tier::Rust,
            source_experiment: "exp012",
            description: "FMT blend engraftment + Bray-Curtis distance structural checks.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — FMT Blend");

    let donor = [0.4, 0.3, 0.2, 0.1];
    let recipient = [0.1, 0.1, 0.3, 0.5];

    let blended = clinical::fmt_blend(&donor, &recipient, 1.0);
    v.check_abs_or_rel(
        "full_engraftment_equals_donor",
        blended[0],
        donor[0],
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let blended_zero = clinical::fmt_blend(&donor, &recipient, 0.0);
    v.check_abs_or_rel(
        "zero_engraftment_equals_recipient",
        blended_zero[0],
        recipient[0],
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    v.section("Phase 1b: Bray-Curtis");

    let bc_self = clinical::bray_curtis(&donor, &donor);
    v.check_abs_or_rel(
        "bray_curtis_self_is_zero",
        bc_self,
        0.0,
        tolerances::MACHINE_EPSILON_STRICT,
        tolerances::MACHINE_EPSILON_STRICT,
    );

    let bc = clinical::bray_curtis(&donor, &recipient);
    v.check_bool(
        "bray_curtis_bounded_0_1",
        (0.0..=1.0).contains(&bc),
        &format!("bc={bc}"),
    );

    v.section("Phase 1c: Diversity Recovery");

    let h_donor = microbiome::shannon_index(&donor);
    let h_recipient = microbiome::shannon_index(&recipient);
    let h_blended = microbiome::shannon_index(&clinical::fmt_blend(&donor, &recipient, 0.7));
    v.check_bool(
        "fmt_blend_diversity_between_endpoints",
        h_blended >= h_recipient.min(h_donor) - tolerances::MACHINE_EPSILON,
        &format!("h_donor={h_donor}, h_recipient={h_recipient}, h_blended={h_blended}"),
    );
}
