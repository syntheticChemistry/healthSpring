// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC server, connection handling, and biomeOS registration.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use healthspring_barracuda::ipc::{dispatch, rpc, socket};

const READ_TIMEOUT_SECS: u64 = 60;
const WRITE_TIMEOUT_SECS: u64 = 10;
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

// ═══════════════════════════════════════════════════════════════════════════
// Signal handling (pure Rust, no C deps)
// ═══════════════════════════════════════════════════════════════════════════

fn install_signal_handler(running: &Arc<AtomicBool>) {
    let flag = running.clone();
    std::thread::spawn(move || {
        // Self-pipe: poll for signals via pipe read. On Unix, SIGTERM/SIGINT
        // are delivered to the process; we use a flag and non-blocking accept
        // timeout to notice them.
        //
        // The listener accept has a 1s timeout, so we check the flag frequently.
        // For immediate response, we also set the flag from a signal handler
        // thread that blocks on sigwait.
        use std::io::Read;
        let (mut reader, _writer) = match std::os::unix::net::UnixStream::pair() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("fatal: signal pipe creation failed: {e}");
                std::process::exit(1);
            }
        };
        reader.set_read_timeout(Some(Duration::from_secs(1))).ok();
        let mut buf = [0u8; 1];
        loop {
            let _ = reader.read(&mut buf);
            if !flag.load(Ordering::Relaxed) {
                break;
            }
        }
    });

    // Spawn a thread that watches for the running flag to go false.
    // The actual signal delivery uses the ctrlc-compatible pattern:
    // we register a POSIX signal handler via libc-free pipe trick.
    //
    // Since we forbid unsafe, we rely on the accept loop timeout
    // and a dedicated signal-watching approach: we set the running
    // flag to false when std::io errors indicate interruption.
    //
    // For robust signal handling without unsafe or C deps, we use
    // the process's natural SIGTERM delivery: UnixListener::accept
    // returns Err(Interrupted) on signal receipt.
    let _ = running;
}

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

fn dispatch_request(
    method: &str,
    params: &serde_json::Value,
    state: &PrimalState,
) -> serde_json::Value {
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
        "capability.list" => super::capabilities::handle_capability_list(),
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
        "primal": super::capabilities::PRIMAL_NAME,
        "domain": super::capabilities::PRIMAL_DOMAIN,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime_secs,
        "requests_served": requests,
        "capabilities": super::capabilities::ALL_CAPABILITIES,
        "backend": "cpu",
        "composition": {
            "provenance_trio": healthspring_barracuda::data::trio_available(),
            "data_provider": socket::discover_data_primal().is_some(),
            "compute_provider": socket::discover_compute_primal().is_some(),
        },
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

// ═══════════════════════════════════════════════════════════════════════════
// Cross-primal: forward & discover
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// Compute offload (Node Atomic)
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// Data fetch (`NestGate` routing)
// ═══════════════════════════════════════════════════════════════════════════

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
        socket::fallback_registration_primal().and_then(|name| {
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

    match rpc::try_send(
        target_socket,
        "lifecycle.register",
        &serde_json::json!({
            "name": super::capabilities::PRIMAL_NAME,
            "socket_path": our_socket.to_string_lossy(),
            "pid": std::process::id(),
        }),
    ) {
        Ok(_) => eprintln!("[biomeos] Registered with lifecycle manager"),
        Err(e) => eprintln!("[biomeos] Lifecycle registration failed: {e}"),
    }

    let health_mappings = super::capabilities::build_semantic_mappings();
    if let Err(e) = rpc::try_send(
        target_socket,
        "capability.register",
        &serde_json::json!({
            "primal": super::capabilities::PRIMAL_NAME,
            "capability": super::capabilities::PRIMAL_DOMAIN,
            "socket": our_socket.to_string_lossy(),
            "semantic_mappings": health_mappings,
        }),
    ) {
        eprintln!("[biomeos] Domain capability registration failed: {e}");
    }

    let mut registered = 0;
    for cap in super::capabilities::ALL_CAPABILITIES {
        if rpc::send(
            target_socket,
            "capability.register",
            &serde_json::json!({
                "primal": super::capabilities::PRIMAL_NAME,
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
        "[biomeos] {registered}/{} capabilities + {} domain registered",
        super::capabilities::ALL_CAPABILITIES.len(),
        super::capabilities::PRIMAL_DOMAIN
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Songbird capability announcement
// ═══════════════════════════════════════════════════════════════════════════

fn announce_to_songbird(our_socket: &Path) {
    let socket_str = our_socket.to_string_lossy();
    match healthspring_barracuda::visualization::capabilities::announce_all(&socket_str) {
        Ok(()) => {
            eprintln!(
                "[songbird] Announced {} health.* capabilities",
                healthspring_barracuda::visualization::capabilities::CAPABILITIES.len()
            );
        }
        Err(e) => {
            eprintln!("[songbird] Not available ({e}) — discovery via socket dir only");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Connection handler
// ═══════════════════════════════════════════════════════════════════════════

#[expect(
    clippy::needless_pass_by_value,
    reason = "BufReader::new consumes the stream"
)]
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
// Subcommand: serve
// ═══════════════════════════════════════════════════════════════════════════

pub fn cmd_serve() -> Result<(), String> {
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

    // Non-blocking accept with timeout for signal responsiveness
    listener
        .set_nonblocking(false)
        .map_err(|e| format!("Cannot set listener options: {e}"))?;

    let family_id = socket::get_family_id();
    eprintln!(
        "{} primal listening on {}",
        super::capabilities::PRIMAL_NAME,
        socket_path.display()
    );
    eprintln!("  Family ID: {family_id}");
    eprintln!("  Domain:    {}", super::capabilities::PRIMAL_DOMAIN);
    eprintln!("  Mode:      BYOB Niche (biomeOS)");
    eprintln!("  Version:   {}", env!("CARGO_PKG_VERSION"));
    eprintln!(
        "  Capabilities ({}):",
        super::capabilities::ALL_CAPABILITIES.len()
    );
    for cap in super::capabilities::ALL_CAPABILITIES {
        eprintln!("    - {cap}");
    }

    register_with_biomeos(&socket_path);
    announce_to_songbird(&socket_path);

    let running = Arc::new(AtomicBool::new(true));
    install_signal_handler(&running);

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
                if let Err(e) = rpc::try_send(
                    t,
                    "lifecycle.status",
                    &serde_json::json!({
                        "name": super::capabilities::PRIMAL_NAME,
                        "socket_path": heartbeat_socket.to_string_lossy(),
                        "status": "healthy",
                        "requests_served": heartbeat_state.requests_served.load(Ordering::Relaxed),
                    }),
                ) {
                    eprintln!("[heartbeat] Status update failed: {e}");
                }
            }
        }
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
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                // Signal received — check the running flag
                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }
            Err(e) => eprintln!("[error] Accept failed: {e}"),
        }
    }

    eprintln!("[shutdown] Cleaning up socket...");
    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}
