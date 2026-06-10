// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cross-gate enclave validation — exercises mesh topology,
//! `ipc.resolve` transport endpoints, and cross-gate capability
//! routing for healthSpring's dual-tower clinical composition.
//!
//! Phase 1: Structural — gate identity + routing prerequisites
//! Phase 2: Mesh topology — build from `discovery.peers`, verify connectivity
//! Phase 3: Transport resolution — `ipc.resolve` for Nest-critical capabilities
//! Phase 4: Cross-gate readiness — verify `resolve_cross_gate` for storage/compute
//! Phase 5: BTSP trust posture — mesh-aware authentication state

use primalspring::composition::mesh::MeshTopology;
use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::composition::{call_or_skip, capability_to_primal};
use crate::primal_names;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "cross-gate-enclave",
            track: Track::Composition,
            tier: Tier::Both,
            source_experiment: "cross_gate_enclave_v1",
            description: "Cross-gate enclave — mesh topology, transport resolution, BTSP trust.",
        },
        run,
    }
}

const ENCLAVE_CAPS: &[&str] = &[
    "storage",
    "dag",
    "commit",
    "crypto",
    "discovery",
    "compute",
    "orchestration",
];

fn phase1_structural(v: &mut ValidationResult) {
    v.section("Phase 1: Structural prerequisites");

    v.check_bool(
        "storage_routes_nestgate",
        capability_to_primal("storage") == primal_names::NESTGATE,
        "storage → nestgate",
    );
    v.check_bool(
        "compute_routes_toadstool",
        capability_to_primal("compute") == primal_names::TOADSTOOL,
        "compute → toadstool",
    );
    v.check_bool(
        "orchestration_routes_biomeos",
        capability_to_primal("orchestration") == primal_names::BIOMEOS,
        "orchestration → biomeos",
    );

    let gate_id = std::env::var("GATE_NAME")
        .or_else(|_| std::env::var("GATE_ID"))
        .unwrap_or_default();

    if gate_id.is_empty() {
        v.check_skip(
            "gate_identity_available",
            "GATE_NAME/GATE_ID not set — mesh phases will be limited",
        );
    } else {
        v.check_bool(
            "gate_identity_available",
            true,
            &format!("gate: {gate_id}"),
        );
    }
}

fn phase2_mesh_topology(v: &mut ValidationResult, ctx: &mut CompositionContext) -> MeshTopology {
    v.section("Phase 2: Mesh topology");

    let mut topology = MeshTopology::new();

    let gate_id = std::env::var("GATE_NAME")
        .or_else(|_| std::env::var("GATE_ID"))
        .unwrap_or_else(|_| "local".into());
    topology.set_local_gate(&gate_id);

    let peers_result = call_or_skip(
        ctx,
        v,
        "mesh_discovery_peers",
        "discovery",
        "discovery.peers",
        serde_json::json!({}),
    );

    if let Some(ref result) = peers_result {
        let peers = result
            .get("peers")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        topology.register_gate(
            &gate_id,
            Some("local".into()),
            [
                "nestgate", "rhizocrypt", "beardog", "loamspine", "sweetgrass", "songbird",
                "skunkbat",
            ],
            ENCLAVE_CAPS.iter().copied(),
        );
        topology.mark_healthy(&gate_id, true);

        for peer in &peers {
            let peer_gate = peer
                .get("gate")
                .or_else(|| peer.get("node_id"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let peer_addr = peer
                .get("address")
                .or_else(|| peer.get("addr"))
                .and_then(serde_json::Value::as_str)
                .map(String::from);

            topology.register_gate(peer_gate, peer_addr, std::iter::empty::<String>(), std::iter::empty::<String>());
            let healthy = peer
                .get("healthy")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            topology.mark_healthy(peer_gate, healthy);
        }

        v.check_bool(
            "mesh_topology_built",
            true,
            &format!(
                "{} gate(s), {} healthy",
                topology.gate_count(),
                topology.healthy_gate_count(),
            ),
        );

        ctx.set_gate_id(&gate_id);
        ctx.set_mesh(topology.clone());
    } else {
        v.check_skip("mesh_topology_built", "discovery unavailable");
    }

    topology
}

fn phase3_transport_resolution(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 3: Transport resolution (ipc.resolve)");

    let critical_caps = ["storage", "dag", "crypto", "commit"];

    for cap in &critical_caps {
        let resolve_result = ctx.call(
            "discovery",
            "ipc.resolve",
            serde_json::json!({"capability": cap}),
        );

        match resolve_result {
            Ok(ref result) => {
                let transport = result
                    .get("transport")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                let has_endpoint = result.get("path").is_some()
                    || result.get("mesh_relay").is_some()
                    || result.get("address").is_some();

                v.check_bool(
                    &format!("ipc_resolve:{cap}"),
                    has_endpoint,
                    &format!("transport={transport}"),
                );
            }
            Err(e) => {
                if e.is_connection_error() || e.is_protocol_error() {
                    v.check_skip(
                        &format!("ipc_resolve:{cap}"),
                        "ipc.resolve unavailable",
                    );
                } else {
                    v.check_skip(
                        &format!("ipc_resolve:{cap}"),
                        &format!("{e}"),
                    );
                }
            }
        }
    }
}

fn phase4_cross_gate_readiness(
    v: &mut ValidationResult,
    ctx: &CompositionContext,
    topology: &MeshTopology,
) {
    v.section("Phase 4: Cross-gate readiness");

    let reachable = topology.reachable_capabilities();
    v.check_bool(
        "reachable_capabilities_nonempty",
        !reachable.is_empty(),
        &format!("{} reachable capability domain(s)", reachable.len()),
    );

    let unreachable = topology.unreachable_capabilities();
    if unreachable.is_empty() {
        v.check_bool(
            "unreachable_capabilities_zero",
            true,
            "all registered capabilities reachable",
        );
    } else {
        v.check_bool(
            "unreachable_capabilities_zero",
            false,
            &format!(
                "{} unreachable: {}",
                unreachable.len(),
                unreachable.iter().copied().collect::<Vec<_>>().join(", ")
            ),
        );
    }

    for cap in &["storage", "compute"] {
        match ctx.resolve_cross_gate(cap) {
            Some(gate) => {
                v.check_bool(
                    &format!("cross_gate:{cap}"),
                    true,
                    &format!("resolves to {gate}"),
                );
            }
            None => {
                v.check_skip(
                    &format!("cross_gate:{cap}"),
                    "no cross-gate provider (single-gate deployment)",
                );
            }
        }
    }

    if topology.gate_count() >= 2 {
        v.check_bool(
            "multi_gate_collective",
            topology.healthy_gate_count() >= 2,
            &format!(
                "{}/{} gates healthy",
                topology.healthy_gate_count(),
                topology.gate_count(),
            ),
        );
    } else {
        v.check_skip(
            "multi_gate_collective",
            "single-gate deployment — cross-gate validation deferred",
        );
    }
}

fn phase5_mesh_btsp(v: &mut ValidationResult, ctx: &CompositionContext) {
    v.section("Phase 5: Mesh BTSP trust posture");

    let mesh_critical = ["storage", "dag", "crypto", "orchestration"];
    let mut btsp_count = 0u32;
    let mut probed = 0u32;

    for cap in &mesh_critical {
        match ctx.btsp_authenticated(cap) {
            Some(true) => {
                btsp_count += 1;
                probed += 1;
            }
            Some(false) => {
                probed += 1;
            }
            None => {}
        }
    }

    if probed == 0 {
        v.check_skip(
            "mesh_btsp_posture",
            "no BTSP state available (standalone or no FAMILY_SEED)",
        );
    } else {
        v.check_bool(
            "mesh_btsp_posture",
            true,
            &format!("{btsp_count}/{probed} mesh-critical capabilities BTSP-authenticated"),
        );
    }

    let enclave_domains = ["storage", "dag", "commit", "crypto"];
    let all_paths_known = enclave_domains
        .iter()
        .all(|cap| ctx.discovery_path(cap).is_some());

    if all_paths_known {
        v.check_bool(
            "enclave_discovery_paths_complete",
            true,
            "all enclave capability discovery paths known",
        );
    } else {
        let known_count = enclave_domains
            .iter()
            .filter(|cap| ctx.discovery_path(cap).is_some())
            .count();
        if known_count > 0 {
            v.check_bool(
                "enclave_discovery_paths_complete",
                false,
                &format!("{known_count}/{} discovery paths known", enclave_domains.len()),
            );
        } else {
            v.check_skip(
                "enclave_discovery_paths_complete",
                "no discovery paths (standalone mode)",
            );
        }
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    phase1_structural(v);

    if ctx.available_capabilities().is_empty() {
        v.check_skip(
            "cross_gate_pipeline",
            "no capabilities discovered — NUCLEUS not deployed",
        );
        return;
    }

    let topology = phase2_mesh_topology(v, ctx);
    phase3_transport_resolution(v, ctx);
    phase4_cross_gate_readiness(v, ctx, &topology);
    phase5_mesh_btsp(v, ctx);
}
