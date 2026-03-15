// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

//! healthSpring biomeOS Primal — BYOB Niche Deployment
//!
//! JSON-RPC 2.0 server exposing healthSpring's science capabilities to the
//! `biomeOS` ecosystem via Unix domain socket. Replaces experiment binaries
//! with a single discoverable service.
//!
//! ## Capability domains
//!
//! **PK/PD**: Hill dose-response, compartmental PK, PBPK, population Monte Carlo,
//!   allometric scaling, NCA, NLME (FOCE/SAEM), Michaelis-Menten nonlinear
//!
//! **Microbiome**: Shannon, Simpson, Pielou, Chao1, Anderson gut lattice,
//!   colonization resistance, FMT blending, Bray-Curtis, SCFA production
//!
//! **Biosignal**: Pan-Tompkins QRS, HRV metrics, PPG `SpO2`, EDA stress,
//!   arrhythmia classification, multi-channel fusion
//!
//! **Endocrine**: Testosterone PK (IM depot/pellet), TRT outcomes, HRV-TRT
//!   response, cardiac risk composite
//!
//! **Diagnostic**: Integrated 4-track patient assessment, composite risk,
//!   population Monte Carlo
//!
//! ## `biomeOS` integration
//!
//! On startup, probes for a `biomeOS` orchestrator socket and registers
//! capabilities via `lifecycle.register` + `capability.register`.
//! Sends heartbeats every 30 s. Cleans up socket on SIGTERM.
//!
//! Socket: `$XDG_RUNTIME_DIR/biomeos/healthspring-{family_id}.sock`

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use healthspring_barracuda::ipc::{dispatch, rpc, socket};

const PRIMAL_NAME: &str = "healthspring";
const PRIMAL_DOMAIN: &str = "health";
const READ_TIMEOUT_SECS: u64 = 60;
const WRITE_TIMEOUT_SECS: u64 = 10;
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Every capability this primal advertises to `biomeOS`.
const ALL_CAPABILITIES: &[&str] = &[
    // ── PK/PD ────────────────────────────────────────────────────────
    "science.pkpd.hill_dose_response",
    "science.pkpd.one_compartment_pk",
    "science.pkpd.two_compartment_pk",
    "science.pkpd.pbpk_simulate",
    "science.pkpd.population_pk",
    "science.pkpd.allometric_scale",
    "science.pkpd.auc_trapezoidal",
    "science.pkpd.nlme_foce",
    "science.pkpd.nlme_saem",
    "science.pkpd.nca_analysis",
    "science.pkpd.cwres_diagnostics",
    "science.pkpd.vpc_simulate",
    "science.pkpd.gof_compute",
    "science.pkpd.michaelis_menten_nonlinear",
    // ── Microbiome ───────────────────────────────────────────────────
    "science.microbiome.shannon_index",
    "science.microbiome.simpson_index",
    "science.microbiome.pielou_evenness",
    "science.microbiome.chao1",
    "science.microbiome.anderson_gut",
    "science.microbiome.colonization_resistance",
    "science.microbiome.fmt_blend",
    "science.microbiome.bray_curtis",
    "science.microbiome.antibiotic_perturbation",
    "science.microbiome.scfa_production",
    "science.microbiome.gut_brain_serotonin",
    "science.microbiome.qs_gene_profile",
    "science.microbiome.qs_effective_disorder",
    // ── Biosignal ────────────────────────────────────────────────────
    "science.biosignal.pan_tompkins",
    "science.biosignal.hrv_metrics",
    "science.biosignal.ppg_spo2",
    "science.biosignal.eda_analysis",
    "science.biosignal.eda_stress_detection",
    "science.biosignal.arrhythmia_classification",
    "science.biosignal.fuse_channels",
    "science.biosignal.wfdb_decode",
    // ── Endocrine ────────────────────────────────────────────────────
    "science.endocrine.testosterone_pk",
    "science.endocrine.trt_outcomes",
    "science.endocrine.population_trt",
    "science.endocrine.hrv_trt_response",
    "science.endocrine.cardiac_risk",
    // ── Diagnostic ───────────────────────────────────────────────────
    "science.diagnostic.assess_patient",
    "science.diagnostic.population_montecarlo",
    "science.diagnostic.composite_risk",
    // ── Clinical ─────────────────────────────────────────────────────
    "science.clinical.trt_scenario",
    "science.clinical.patient_parameterize",
    "science.clinical.risk_annotate",
    // ── Provenance trio (`biomeOS` composition) ──────────────────────
    "provenance.begin",
    "provenance.record",
    "provenance.complete",
    "provenance.status",
    // ── Cross-primal ─────────────────────────────────────────────────
    "primal.forward",
    "primal.discover",
    // ── Niche deployment (`biomeOS` graph composition) ───────────────
    "capability.list",
    // ── Compute offload (Node Atomic) ────────────────────────────────
    "compute.offload",
    // ── Data (`NestGate` routing) ────────────────────────────────────
    "data.fetch",
];

// ═══════════════════════════════════════════════════════════════════════════
// Primal state
// ═══════════════════════════════════════════════════════════════════════════

struct PrimalState {
    start_time: Instant,
    requests_served: AtomicU64,
}

// ═══════════════════════════════════════════════════════════════════════════
// Request dispatch
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_request(method: &str, params: &serde_json::Value, state: &PrimalState) -> serde_json::Value {
    if method == "lifecycle.health" || method == "health" || method == "health.check" {
        return handle_health(state);
    }

    if method == "lifecycle.status" {
        return handle_health(state);
    }

    if let Some(result) = dispatch::dispatch_science(method, params) {
        return result;
    }

    match method {
        "capability.list" => handle_capability_list(),
        "provenance.begin" => handle_provenance_begin(params),
        "provenance.record" => handle_provenance_record(params),
        "provenance.complete" => handle_provenance_complete(params),
        "provenance.status" => handle_provenance_status(),
        "primal.forward" => handle_primal_forward(params),
        "primal.discover" => handle_primal_discover(),
        "compute.offload" => handle_compute_offload(params),
        "data.fetch" => handle_data_fetch(params),
        _ => serde_json::json!({"error": "method_not_found", "method": method}),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Health / lifecycle
// ═══════════════════════════════════════════════════════════════════════════

fn handle_health(state: &PrimalState) -> serde_json::Value {
    let uptime_secs = state.start_time.elapsed().as_secs();
    let requests = state.requests_served.load(Ordering::Relaxed);
    serde_json::json!({
        "status": "healthy",
        "primal": PRIMAL_NAME,
        "domain": PRIMAL_DOMAIN,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime_secs,
        "requests_served": requests,
        "capabilities": ALL_CAPABILITIES,
        "backend": "cpu",
        "composition": {
            "provenance_trio": healthspring_barracuda::data::trio_available(),
            "nestgate": socket::discover_data_primal().is_some(),
            "toadstool": socket::discover_compute_primal().is_some(),
        },
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Capability listing (`biomeOS` niche composition)
// ═══════════════════════════════════════════════════════════════════════════

fn handle_capability_list() -> serde_json::Value {
    let science: Vec<&str> = ALL_CAPABILITIES
        .iter()
        .filter(|c| c.starts_with("science."))
        .copied()
        .collect();
    let infra: Vec<&str> = ALL_CAPABILITIES
        .iter()
        .filter(|c| {
            c.starts_with("primal.")
                || c.starts_with("compute.")
                || c.starts_with("data.")
                || c.starts_with("capability.")
                || c.starts_with("provenance.")
        })
        .copied()
        .collect();

    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": PRIMAL_DOMAIN,
        "total": ALL_CAPABILITIES.len(),
        "science": science,
        "infrastructure": infra,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Provenance trio handlers
// ═══════════════════════════════════════════════════════════════════════════

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
    let session_id = params.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
    let step = params.get("step").cloned().unwrap_or_else(|| serde_json::json!({}));

    healthspring_barracuda::data::record_fetch_step(session_id, &step);
    serde_json::json!({"recorded": true, "session_id": session_id})
}

fn handle_provenance_complete(params: &serde_json::Value) -> serde_json::Value {
    let session_id = params.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
    let license = params
        .get("license")
        .and_then(|v| v.as_str())
        .unwrap_or("AGPL-3.0-only");

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

// ═══════════════════════════════════════════════════════════════════════════
// Cross-primal: forward & discover
// ═══════════════════════════════════════════════════════════════════════════

fn handle_primal_forward(params: &serde_json::Value) -> serde_json::Value {
    let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("");
    let method = params.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let inner_params = params.get("params").cloned().unwrap_or_else(|| serde_json::json!({}));

    let Some(target_socket) = socket::discover_primal(target) else {
        return serde_json::json!({
            "error": "primal_not_found",
            "target": target,
            "hint": format!("no socket found for '{target}' in socket dir"),
        });
    };

    rpc::send(&target_socket, method, &inner_params).unwrap_or_else(|| {
        serde_json::json!({"error": "forward_failed", "target": target, "method": method})
    })
}

fn handle_primal_discover() -> serde_json::Value {
    let primals = socket::discover_all_primals();
    serde_json::json!({
        "socket_dir": socket::resolve_socket_dir().to_string_lossy(),
        "primals": primals,
        "count": primals.len(),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Compute offload (Node Atomic)
// ═══════════════════════════════════════════════════════════════════════════

fn handle_compute_offload(params: &serde_json::Value) -> serde_json::Value {
    let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("");

    let Some(compute_socket) = socket::discover_compute_primal() else {
        return serde_json::json!({
            "error": "compute_primal_not_found",
            "hint": "start Node Atomic (toadStool) to enable GPU offload",
            "env_override": "HEALTHSPRING_COMPUTE_PRIMAL",
        });
    };

    rpc::send(&compute_socket, &format!("compute.{operation}"), params)
        .unwrap_or_else(|| serde_json::json!({"error": "compute_offload_failed", "operation": operation}))
}

// ═══════════════════════════════════════════════════════════════════════════
// Data fetch (`NestGate` routing)
// ═══════════════════════════════════════════════════════════════════════════

fn handle_data_fetch(params: &serde_json::Value) -> serde_json::Value {
    let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("ncbi");
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

// ═══════════════════════════════════════════════════════════════════════════
// biomeOS registration
// ═══════════════════════════════════════════════════════════════════════════

fn register_with_biomeos(our_socket: &Path) {
    let biomeos_socket = socket::orchestrator_socket();

    let target = if biomeos_socket.exists() {
        eprintln!(
            "[biomeos] Orchestrator found at {}",
            biomeos_socket.display()
        );
        Some(biomeos_socket)
    } else {
        eprintln!(
            "[biomeos] No orchestrator at {}, checking fallback...",
            biomeos_socket.display()
        );
        socket::fallback_registration_primal()
            .and_then(|name| {
                let sock = socket::discover_primal(&name);
                if sock.is_some() {
                    eprintln!("[biomeos] Fallback primal '{name}' found");
                }
                sock
            })
    };

    let Some(ref target_socket) = target else {
        eprintln!("[biomeos] Running standalone — no orchestrator or fallback");
        return;
    };

    let _ = rpc::send(
        target_socket,
        "lifecycle.register",
        &serde_json::json!({
            "name": PRIMAL_NAME,
            "socket_path": our_socket.to_string_lossy(),
            "pid": std::process::id(),
        }),
    );
    eprintln!("[biomeos] Registered with lifecycle manager");

    let health_mappings = build_semantic_mappings();
    let _ = rpc::send(
        target_socket,
        "capability.register",
        &serde_json::json!({
            "primal": PRIMAL_NAME,
            "capability": PRIMAL_DOMAIN,
            "socket": our_socket.to_string_lossy(),
            "semantic_mappings": health_mappings,
        }),
    );

    let mut registered = 0;
    for cap in ALL_CAPABILITIES {
        if rpc::send(
            target_socket,
            "capability.register",
            &serde_json::json!({
                "primal": PRIMAL_NAME,
                "capability": cap,
                "socket": our_socket.to_string_lossy(),
            }),
        )
        .is_some()
        {
            registered += 1;
        }
    }

    eprintln!(
        "[biomeos] {registered}/{} capabilities + {PRIMAL_DOMAIN} domain registered",
        ALL_CAPABILITIES.len()
    );
}

fn build_semantic_mappings() -> serde_json::Value {
    serde_json::json!({
        "hill_dose_response":       "science.pkpd.hill_dose_response",
        "one_compartment_pk":       "science.pkpd.one_compartment_pk",
        "two_compartment_pk":       "science.pkpd.two_compartment_pk",
        "pbpk_simulate":            "science.pkpd.pbpk_simulate",
        "population_pk":            "science.pkpd.population_pk",
        "nca_analysis":             "science.pkpd.nca_analysis",
        "shannon_index":            "science.microbiome.shannon_index",
        "simpson_index":            "science.microbiome.simpson_index",
        "anderson_gut":             "science.microbiome.anderson_gut",
        "colonization_resistance":  "science.microbiome.colonization_resistance",
        "pan_tompkins":             "science.biosignal.pan_tompkins",
        "hrv_metrics":              "science.biosignal.hrv_metrics",
        "ppg_spo2":                 "science.biosignal.ppg_spo2",
        "testosterone_pk":          "science.endocrine.testosterone_pk",
        "trt_outcomes":             "science.endocrine.trt_outcomes",
        "assess_patient":           "science.diagnostic.assess_patient",
        "composite_risk":           "science.diagnostic.composite_risk",
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Connection handler
// ═══════════════════════════════════════════════════════════════════════════

#[expect(clippy::needless_pass_by_value, reason = "BufReader::new consumes the stream")]
fn handle_connection(stream: UnixStream, state: &PrimalState) {
    stream
        .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT_SECS)))
        .ok();

    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line_result in reader.lines() {
        let Ok(line) = line_result else { break };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = rpc::error(
                    &serde_json::Value::Null,
                    rpc::PARSE_ERROR,
                    &format!("Parse error: {e}"),
                );
                let _ = writeln!(writer, "{resp}");
                let _ = writer.flush();
                continue;
            }
        };

        let id = parsed.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = parsed.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = parsed
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        state.requests_served.fetch_add(1, Ordering::Relaxed);
        let result = dispatch_request(method, &params, state);
        let response = rpc::success(&id, &result);

        let _ = writeln!(writer, "{response}");
        let _ = writer.flush();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════

fn run() -> Result<(), String> {
    let socket_path = socket::resolve_bind_path();

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create socket directory {}: {e}", parent.display()))?;
    }

    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .map_err(|e| format!("Cannot remove stale socket {}: {e}", socket_path.display()))?;
    }

    let state = Arc::new(PrimalState {
        start_time: Instant::now(),
        requests_served: AtomicU64::new(0),
    });

    let listener = UnixListener::bind(&socket_path)
        .map_err(|e| format!("Cannot bind to {}: {e}", socket_path.display()))?;

    let family_id = socket::get_family_id();
    eprintln!("{PRIMAL_NAME} primal listening on {}", socket_path.display());
    eprintln!("  Family ID: {family_id}");
    eprintln!("  Domain:    {PRIMAL_DOMAIN}");
    eprintln!("  Mode:      BYOB Niche (biomeOS)");
    eprintln!("  Version:   {}", env!("CARGO_PKG_VERSION"));
    eprintln!("  Capabilities ({}):", ALL_CAPABILITIES.len());
    for cap in ALL_CAPABILITIES {
        eprintln!("    - {cap}");
    }

    register_with_biomeos(&socket_path);

    let running = Arc::new(AtomicBool::new(true));

    // Heartbeat thread
    let heartbeat_running = running.clone();
    let heartbeat_state = state.clone();
    let heartbeat_socket = socket_path.clone();
    std::thread::spawn(move || {
        let target = {
            let orch = socket::orchestrator_socket();
            if orch.exists() {
                Some(orch)
            } else {
                socket::fallback_registration_primal()
                    .and_then(|name| socket::discover_primal(&name))
            }
        };

        while heartbeat_running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
            if let Some(ref t) = target {
                let _ = rpc::send(
                    t,
                    "lifecycle.status",
                    &serde_json::json!({
                        "name": PRIMAL_NAME,
                        "socket_path": heartbeat_socket.to_string_lossy(),
                        "status": "healthy",
                        "requests_served": heartbeat_state.requests_served.load(Ordering::Relaxed),
                    }),
                );
            }
        }
    });

    // Socket cleanup on shutdown (via the running flag)
    let cleanup_running = running.clone();
    let cleanup_socket = socket_path.clone();
    std::thread::spawn(move || {
        while cleanup_running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_secs(1));
        }
        let _ = std::fs::remove_file(&cleanup_socket);
    });

    eprintln!("[ready] Accepting connections...");
    for stream in listener.incoming() {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || handle_connection(stream, &state));
            }
            Err(e) => eprintln!("[error] Accept failed: {e}"),
        }
    }

    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[fatal] {e}");
        std::process::exit(1);
    }
}
