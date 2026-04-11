// SPDX-License-Identifier: AGPL-3.0-or-later

//! Request dispatch and routing — maps JSON-RPC methods to handlers.
//!
//! Handles health probes, provenance trio, cross-primal forwarding,
//! compute offload, and data fetch. Science methods are delegated to
//! the barracuda dispatch layer.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use healthspring_barracuda::ipc::{
    compute_dispatch, data_dispatch, dispatch, inference_dispatch, rpc, shader_dispatch, socket,
};
use tracing::warn;

/// Primal server state shared across connections.
pub struct PrimalState {
    pub start_time: Instant,
    pub requests_served: AtomicU64,
}

/// Dispatches a JSON-RPC request by method name.
///
/// Normalizes legacy prefixed method names (`healthspring.*`, `barracuda.*`)
/// to bare `{domain}.{operation}` per semantic naming standard v2.1.
pub fn dispatch_request(
    method: &str,
    params: &serde_json::Value,
    state: &PrimalState,
) -> serde_json::Value {
    let method = rpc::normalize_method(method);
    match method {
        "lifecycle.health" | "health" | "health.check" | "lifecycle.status" => handle_health(state),
        "health.liveness" => handle_liveness(),
        "health.readiness" => handle_readiness(state),
        "composition.health_health" => handle_composition_health(state),
        _ => dispatch_extended(method, params, state),
    }
}

/// Maps proto-nucleate `health.*` aliases to the canonical `science.*` method.
fn resolve_proto_alias(method: &str) -> Option<&'static str> {
    match method {
        "health.pharmacology" => Some("science.pkpd.hill_dose_response"),
        "health.genomics" => Some("science.microbiome.qs_gene_profile"),
        "health.clinical" => Some("science.diagnostic.assess_patient"),
        "health.de_identify" => Some("science.clinical.patient_parameterize"),
        "health.aggregate" => Some("science.diagnostic.population_montecarlo"),
        _ => None,
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
    if let Some(canonical) = resolve_proto_alias(method) {
        if let Some(result) = dispatch::dispatch_science(canonical, params) {
            return result;
        }
    }

    match method {
        "mcp.tools.list" => handle_mcp_tools_list(),
        "capability.list" | "capabilities.list" | "primal.capabilities" => {
            crate::capabilities::handle_capability_list()
        }
        "provenance.begin" => handle_provenance_begin(params),
        "provenance.record" => handle_provenance_record(params),
        "provenance.complete" => handle_provenance_complete(params),
        "provenance.status" => handle_provenance_status(),
        "primal.forward" => handle_primal_forward(params),
        "primal.discover" => handle_primal_discover(),
        "compute.offload" => handle_compute_offload(params),
        "compute.shader_compile" => handle_shader_compile(params),
        "model.inference_route" | "inference.route" => handle_inference_route(params),
        m if m.starts_with("inference.") => handle_inference_passthrough(m, params),
        "data.fetch" => handle_data_fetch(params),
        _ => {
            warn!(method, "unknown method requested");
            serde_json::json!({"error": "method_not_found", "method": method})
        }
    }
}

fn handle_mcp_tools_list() -> serde_json::Value {
    healthspring_barracuda::ipc::mcp::tool_definitions_json()
}

fn handle_health(state: &PrimalState) -> serde_json::Value {
    let uptime_secs = state.start_time.elapsed().as_secs();
    let requests = state.requests_served.load(Ordering::Relaxed);
    serde_json::json!({
        "healthy": true,
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

/// Composition health per `COMPOSITION_HEALTH_STANDARD.md`.
///
/// Returns the health of the health science pipeline including subsystem
/// status for provenance trio, compute provider, and data provider.
fn handle_composition_health(state: &PrimalState) -> serde_json::Value {
    let provenance = healthspring_barracuda::data::trio_available();
    let compute = socket::discover_compute_primal().is_some();
    let data = socket::discover_data_primal().is_some();

    let science_ok = true;
    let all_healthy = science_ok && provenance && compute && data;

    serde_json::json!({
        "healthy": all_healthy,
        "deploy_graph": "healthspring_health_niche",
        "subsystems": {
            "science_dispatch": if science_ok { "ok" } else { "unavailable" },
            "provenance_trio": if provenance { "ok" } else { "degraded" },
            "compute_provider": if compute { "ok" } else { "degraded" },
            "data_provider": if data { "ok" } else { "degraded" },
        },
        "capabilities_count": crate::capabilities::ALL_CAPABILITIES.len(),
        "science_domains": [
            "pkpd", "microbiome", "biosignal", "endocrine",
            "diagnostic", "clinical", "comparative", "discovery",
            "toxicology", "simulation"
        ],
        "uptime_secs": state.start_time.elapsed().as_secs(),
    })
}

fn handle_liveness() -> serde_json::Value {
    serde_json::json!({"alive": true})
}

fn handle_readiness(state: &PrimalState) -> serde_json::Value {
    let science_ok = true;
    let provenance = healthspring_barracuda::data::trio_available();
    let compute = socket::discover_compute_primal().is_some();
    let data = socket::discover_data_primal().is_some();

    serde_json::json!({
        "ready": science_ok,
        "primal": crate::capabilities::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": crate::capabilities::PRIMAL_DOMAIN,
        "uptime_secs": state.start_time.elapsed().as_secs(),
        "subsystems": {
            "science_dispatch": science_ok,
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

    let target_socket =
        socket::discover_by_capability_public(target).or_else(|| socket::discover_primal(target));
    let Some(target_socket) = target_socket else {
        return serde_json::json!({
            "error": "primal_not_found",
            "target": target,
            "hint": format!("no socket found for '{target}' in socket dir"),
        });
    };

    rpc::resilient_send(&target_socket, method, &inner_params).unwrap_or_else(
        |e| serde_json::json!({"error": "forward_failed", "target": target, "method": method, "detail": e.to_string()}),
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
    let workload = params
        .get("workload")
        .or_else(|| params.get("operation"))
        .and_then(|v| v.as_str())
        .unwrap_or("hill_sweep");
    let inner_params = params
        .get("params")
        .cloned()
        .unwrap_or_else(|| params.clone());

    match compute_dispatch::submit(workload, &inner_params) {
        Ok(handle) => serde_json::json!({"job_id": handle.job_id, "status": "submitted"}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn handle_shader_compile(params: &serde_json::Value) -> serde_json::Value {
    match shader_dispatch::compile(params) {
        Ok(result) => result,
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn handle_inference_route(params: &serde_json::Value) -> serde_json::Value {
    let operation = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("infer");

    match inference_dispatch::route(operation, params) {
        Ok(result) => result,
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn handle_inference_passthrough(method: &str, params: &serde_json::Value) -> serde_json::Value {
    let operation = method.strip_prefix("inference.").unwrap_or("complete");
    match inference_dispatch::route(operation, params) {
        Ok(result) => result,
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn handle_data_fetch(params: &serde_json::Value) -> serde_json::Value {
    let source = params
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("ncbi");
    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");

    match data_dispatch::fetch(source, &serde_json::json!({"query": query})) {
        Ok(result) => result,
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}
