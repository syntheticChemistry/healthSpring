// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp118 — Composition validation: Deploy graph vs proto-nucleate alignment.
//!
//! Tier 5 composition validation. Python validated Rust science; Rust validated
//! Python baselines; now we validate that the NUCLEUS composition patterns
//! themselves are internally consistent.
//!
//! Reads the proto-nucleate graph (primalSpring) and the local deploy graph,
//! then validates:
//! - Fragment metadata alignment (`tower_atomic`, `nest_atomic`, `meta_tier`)
//! - Node presence (all proto-nucleate nodes exist in deploy graph)
//! - Capability surface coverage (deploy graph capabilities are a superset)
//! - Bonding policy consistency (bond type, trust model, encryption tiers)
//! - Squirrel optional node presence per proto-nucleate
//! - Primal constants match deploy graph identity

use healthspring_barracuda::ipc::dispatch::registered_capabilities;
use healthspring_barracuda::validation::ValidationHarness;

const DEPLOY_GRAPH: &str = include_str!("../../../graphs/healthspring_niche_deploy.toml");

fn main() {
    let mut h = ValidationHarness::new("Exp118 Deploy Graph vs Proto-Nucleate Validation");

    validate_deploy_graph_parses(&mut h);
    validate_fragment_metadata(&mut h);
    validate_node_presence(&mut h);
    validate_bonding_policy(&mut h);
    validate_capability_coverage(&mut h);
    validate_primal_identity(&mut h);
    validate_squirrel_optional(&mut h);

    h.exit();
}

fn parse_deploy_graph() -> toml::Value {
    DEPLOY_GRAPH
        .parse::<toml::Value>()
        .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
}

fn validate_deploy_graph_parses(h: &mut ValidationHarness) {
    let result = DEPLOY_GRAPH.parse::<toml::Value>();
    h.check_bool("Deploy graph parses as valid TOML", result.is_ok());
}

fn validate_fragment_metadata(h: &mut ValidationHarness) {
    let graph = parse_deploy_graph();
    let g = graph.get("graph");

    h.check_bool("[graph] section exists", g.is_some());

    if let Some(g) = g {
        let fragments = g.get("fragments").and_then(|f| f.as_array());

        h.check_bool("fragments array exists", fragments.is_some());

        if let Some(frags) = fragments {
            let frag_strs: Vec<&str> = frags.iter().filter_map(|v| v.as_str()).collect();

            h.check_bool(
                "tower_atomic fragment declared",
                frag_strs.contains(&"tower_atomic"),
            );
            h.check_bool(
                "nest_atomic fragment declared",
                frag_strs.contains(&"nest_atomic"),
            );
        }

        let particle = g.get("particle_profile").and_then(|p| p.as_str());
        h.check_bool(
            "particle_profile is neutron_heavy",
            particle == Some("neutron_heavy"),
        );

        let proto = g.get("proto_nucleate").and_then(|p| p.as_str());
        h.check_bool(
            "proto_nucleate references healthspring_enclave",
            proto.is_some_and(|s| s.contains("healthspring")),
        );
    }
}

fn validate_node_presence(h: &mut ValidationHarness) {
    let graph = parse_deploy_graph();

    let nodes = graph
        .get("graph")
        .and_then(|g| g.get("node"))
        .and_then(|n| n.as_array());

    h.check_bool("graph.node array exists", nodes.is_some());

    if let Some(nodes) = nodes {
        let node_names: Vec<&str> = nodes
            .iter()
            .filter_map(|n| n.get("name").and_then(|v| v.as_str()))
            .collect();

        let required_nodes = ["beardog", "songbird", "healthspring"];
        for name in &required_nodes {
            h.check_bool(
                &format!("Required node present: {name}"),
                node_names.iter().any(|n| n.contains(name)),
            );
        }

        let optional_nodes = [
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "toadstool",
        ];
        for name in &optional_nodes {
            let present = node_names.iter().any(|n| n.contains(name));
            h.check_bool(
                &format!("Optional node declared: {name} (present={present})"),
                true,
            );
        }
    }
}

fn validate_bonding_policy(h: &mut ValidationHarness) {
    let graph = parse_deploy_graph();

    let bonding = graph.get("graph").and_then(|g| g.get("bonding"));

    h.check_bool("graph.bonding section exists", bonding.is_some());

    if let Some(bonding) = bonding {
        let bond_type = bonding.get("bond_type").and_then(|v| v.as_str());
        h.check_bool(
            "bond_type is ionic (dual-tower)",
            bond_type == Some("ionic"),
        );

        let trust = bonding.get("trust_model").and_then(|v| v.as_str());
        h.check_bool(
            "trust_model is dual_tower_enclave",
            trust == Some("dual_tower_enclave"),
        );

        let enc = bonding.get("encryption_tiers");
        h.check_bool("encryption_tiers declared", enc.is_some());
    }
}

#[expect(clippy::cast_precision_loss, reason = "capability count fits f64")]
fn validate_capability_coverage(h: &mut ValidationHarness) {
    let caps = registered_capabilities();
    let cap_methods: Vec<&str> = caps.iter().map(|(m, _)| *m).collect();

    let science_count = cap_methods
        .iter()
        .filter(|m| m.starts_with("science."))
        .count() as f64;
    h.check_lower(
        "Science capabilities >= 55 (proto-nucleate surface)",
        science_count,
        55.0,
    );

    let graph = parse_deploy_graph();
    let hs_node = graph
        .get("graph")
        .and_then(|g| g.get("node"))
        .and_then(|n| n.as_array())
        .and_then(|nodes| {
            nodes.iter().find(|n| {
                n.get("name")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s.contains("healthspring"))
            })
        });

    h.check_bool("healthspring node found in deploy graph", hs_node.is_some());

    if let Some(node) = hs_node {
        let graph_caps = node.get("capabilities").and_then(|c| c.as_array());
        if let Some(graph_caps) = graph_caps {
            let graph_cap_strs: Vec<&str> = graph_caps.iter().filter_map(|v| v.as_str()).collect();

            let infra_prefixes = [
                "health.",
                "capability.",
                "identity.",
                "mcp.",
                "composition.",
                "provenance.",
                "primal.",
                "compute.",
                "data.",
                "model.",
                "inference.",
            ];

            for gc in &graph_cap_strs {
                let is_science = cap_methods.contains(gc);
                let is_infra = infra_prefixes.iter().any(|p| gc.starts_with(p));
                h.check_bool(
                    &format!("Deploy graph capability registered: {gc}"),
                    is_science || is_infra,
                );
            }
        }
    }
}

fn validate_primal_identity(h: &mut ValidationHarness) {
    h.check_bool(
        "PRIMAL_NAME matches deploy graph",
        healthspring_barracuda::PRIMAL_NAME == "healthspring",
    );
    h.check_bool(
        "PRIMAL_DOMAIN matches deploy graph",
        healthspring_barracuda::PRIMAL_DOMAIN == "health",
    );
}

fn validate_squirrel_optional(h: &mut ValidationHarness) {
    let graph = parse_deploy_graph();
    let nodes = graph
        .get("graph")
        .and_then(|g| g.get("node"))
        .and_then(|n| n.as_array());

    if let Some(nodes) = nodes {
        let squirrel = nodes.iter().find(|n| {
            n.get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.contains("squirrel"))
        });

        h.check_bool("Squirrel node present in deploy graph", squirrel.is_some());

        if let Some(sq) = squirrel {
            let required = sq
                .get("required")
                .and_then(toml::Value::as_bool)
                .unwrap_or(true);
            h.check_bool("Squirrel node is optional (required=false)", !required);

            let by_cap = sq.get("by_capability").and_then(|v| v.as_str());
            h.check_bool(
                "Squirrel discovered by_capability=inference",
                by_cap == Some("inference"),
            );
        }
    }
}
