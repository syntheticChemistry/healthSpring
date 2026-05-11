// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::composition::capability_to_primal;
use crate::primal_names;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "live-provenance",
            track: Track::Composition,
            tier: Tier::Live,
            source_experiment: "exp120",
            description: "Provenance trio capabilities — NestGate-adjacent health signals.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural");

    v.check_bool(
        "dag_maps_rhizocrypt",
        capability_to_primal("dag") == primal_names::RHIZOCRYPT,
        "dag → rhizoCrypt",
    );
    v.check_bool(
        "commit_maps_loamspine",
        capability_to_primal("commit") == primal_names::LOAMSPINE,
        "commit → loamSpine",
    );
    v.check_bool(
        "braid_maps_sweetgrass",
        capability_to_primal("braid") == primal_names::SWEETGRASS,
        "braid → sweetgrass",
    );

    if ctx.available_capabilities().is_empty() {
        return;
    }

    v.section("Phase 2: Live Composition");

    for (label, cap) in [
        ("sweetgrass_provenance_braid", "braid"),
        ("rhizocrypt_provenance_dag", "dag"),
        ("loamspine_provenance_commit", "commit"),
    ] {
        match ctx.health_check(cap) {
            Ok(alive) => v.check_bool(label, alive, &format!("{cap} health")),
            Err(e) if e.is_connection_error() => {
                v.check_skip(label, &format!("{cap} not reachable: {e}"));
            }
            Err(e) => {
                v.check_bool(label, false, &format!("{cap} error: {e}"));
            }
        }
    }
}
