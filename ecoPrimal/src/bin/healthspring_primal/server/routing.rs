// SPDX-License-Identifier: AGPL-3.0-or-later

//! Request dispatch and routing — maps JSON-RPC methods to handlers.
//!
//! Handles health probes, provenance trio, cross-primal forwarding,
//! compute offload, and data fetch. Science methods are delegated to
//! the barracuda dispatch layer.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use healthspring_barracuda::ipc::{dispatch, rpc, socket};
use tracing::warn;

/// Primal server state shared across connections.
pub struct PrimalState {
    pub start_time: Instant,
    pub requests_served: AtomicU64,
}

/// Dispatches a JSON-RPC request by method name.
pub fn dispatch_request(
    method: &str,
    params: &serde_json::Value,
    state: &PrimalState,
) -> serde_json::Value {
    match method {
        "lifecycle.health" | "health" | "health.check" | "lifecycle.status" => handle_health(state),
        "health.liveness" => handle_liveness(),
        "health.readiness" => handle_readiness(state),
        _ => dispatch_extended(method, params, state),
    }
}

fn dispatch_extended(
    method: &str,
    params: &serde_json::Value,
    _state: &PrimalState,
) -> serde_json::Value {
    if let Some(result) = dispatch::dispatch_science(method, params) {
        return result;
    }

    match method {
        "capability.list" => crate::capabilities::handle_capability_list(),
        "provenance.begin" => handle_provenance_begin(params),
        "provenance.record" => handle_provenance_record(params),
        "provenance.complete" => handle_provenance_complete(params),
        "provenance.status" => handle_provenance_status(),
        "primal.forward" => handle_primal_forward(params),
        "primal.discover" => handle_primal_discover(),
        "compute.offload" => handle_compute_offload(params),
        "data.fetch" => handle_data_fetch(params),
        _ => {
            warn!(method, "unknown method requested");
            serde_json::json!({"error": "method_not_found", "method": method})
        }
    }
}

fn handle_health(state: &PrimalState) -> serde_json::Value {
    let uptime_secs = state.start_time.elapsed().as_secs();
    let requests = state.requests_served.load(Ordering::Relaxed);
    serde_json::json!({
        "status": "healthy",
        "primal": crate::capabilities::PRIMAL_NAME,
        "domain": crate::capabilities::PRIMAL_DOMAIN,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime_secs,
        "requests_served": requests,
        "capabilities": crate::capabilities::ALL_CAPABILITIES,
        "backend": "cpu",
        "composition": {
            "provenance_trio": healthspring_barracuda::data::trio_available(),
            "data_provider": socket::discover_data_primal().is_some(),
            "compute_provider": socket::discover_compute_primal().is_some(),
        },
    })
}

fn handle_liveness() -> serde_json::Value {
    serde_json::json!({"alive": true})
}

fn handle_readiness(state: &PrimalState) -> serde_json::Value {
    let provenance = healthspring_barracuda::data::trio_available();
    let compute = socket::discover_compute_primal().is_some();
    let data = socket::discover_data_primal().is_some();
    let ready = true;

    serde_json::json!({
        "ready": ready,
        "uptime_secs": state.start_time.elapsed().as_secs(),
        "subsystems": {
            "science_dispatch": true,
            "provenance_trio": provenance,
            "compute_provider": compute,
            "data_provider": data,
        },
    })
}

fn handle_provenance_begin(params: &serde_json::Value) -> serde_json::Value {
    let experiment = params
        .get("experiment")
        .or_else(|| params.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed");

    let result = healthspring_barracuda::data::begin_data_session(experiment);
    serde_json::json!({
        "session_id": result.id,
        "available": result.available,
        "data": result.data,
    })
}

fn handle_provenance_record(params: &serde_json::Value) -> serde_json::Value {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let step = params
        .get("step")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let result = healthspring_barracuda::data::record_fetch_step(session_id, &step);
    serde_json::json!({"recorded": result.available, "session_id": session_id})
}

fn handle_provenance_complete(params: &serde_json::Value) -> serde_json::Value {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let license = params
        .get("license")
        .and_then(|v| v.as_str())
        .unwrap_or("AGPL-3.0-or-later");

    let chain = healthspring_barracuda::data::complete_data_session(session_id, license);
    serde_json::json!({
        "status": chain.status,
        "merkle_root": chain.merkle_root,
        "commit_id": chain.commit_id,
        "braid_id": chain.braid_id,
    })
}

fn handle_provenance_status() -> serde_json::Value {
    serde_json::json!({
        "available": healthspring_barracuda::data::trio_available(),
        "trio": {
            "rhizocrypt": "dag.* via capability.call",
            "loamspine": "commit.* via capability.call",
            "sweetgrass": "provenance.* via capability.call",
        },
        "degradation": "domain logic succeeds without provenance",
    })
}

fn handle_primal_forward(params: &serde_json::Value) -> serde_json::Value {
    let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("");
    let method = params.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let inner_params = params
        .get("params")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let Some(target_socket) = socket::discover_primal(target) else {
        return serde_json::json!({
            "error": "primal_not_found",
            "target": target,
            "hint": format!("no socket found for '{target}' in socket dir"),
        });
    };

    rpc::send(&target_socket, method, &inner_params).unwrap_or_else(
        || serde_json::json!({"error": "forward_failed", "target": target, "method": method}),
    )
}

fn handle_primal_discover() -> serde_json::Value {
    let primals = socket::discover_all_primals();
    serde_json::json!({
        "socket_dir": socket::resolve_socket_dir().to_string_lossy(),
        "primals": primals,
        "count": primals.len(),
    })
}

fn handle_compute_offload(params: &serde_json::Value) -> serde_json::Value {
    let operation = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let Some(compute_socket) = socket::discover_compute_primal() else {
        return serde_json::json!({
            "error": "compute_primal_not_found",
            "hint": "start Node Atomic (toadStool) to enable GPU offload",
            "env_override": "HEALTHSPRING_COMPUTE_PRIMAL",
        });
    };

    rpc::send(&compute_socket, &format!("compute.{operation}"), params).unwrap_or_else(
        || serde_json::json!({"error": "compute_offload_failed", "operation": operation}),
    )
}

fn handle_data_fetch(params: &serde_json::Value) -> serde_json::Value {
    let source = params
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("ncbi");
    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");

    let Some(data_socket) = socket::discover_data_primal() else {
        return serde_json::json!({
            "error": "data_primal_not_found",
            "hint": "start Nest Atomic (NestGate) for data routing",
            "env_override": "HEALTHSPRING_DATA_PRIMAL",
        });
    };

    let method = format!("data.{source}_fetch");
    rpc::send(&data_socket, &method, &serde_json::json!({"query": query}))
        .unwrap_or_else(|| serde_json::json!({"error": "data_fetch_failed", "source": source}))
}
