// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    deprecated,
    reason = "registration resolves fallback primal via legacy socket scan until CompositionContext integration"
)]
//! biomeOS registration, Songbird announcement, and heartbeat.
//!
//! Registers the primal with the orchestrator (or fallback), announces
//! capabilities to Songbird, and spawns a heartbeat thread for lifecycle
//! status updates.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use healthspring_barracuda::ipc::{rpc, socket};
use tracing::{info, warn};

const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Registers the primal with the biomeOS orchestrator (or fallback).
///
/// Prefers `primal.announce` (Wave 17 single-call registration) with automatic
/// fallback to `lifecycle.register` + `capability.register` for older biomeOS.
pub fn register_with_biomeos(our_socket: &Path) {
    let biomeos_socket = socket::orchestrator_socket();

    let target = if biomeos_socket.exists() {
        info!(path = %biomeos_socket.display(), "orchestrator found");
        Some(biomeos_socket)
    } else {
        info!(path = %biomeos_socket.display(), "no orchestrator, checking fallback");
        socket::fallback_registration_primal().and_then(|name| {
            let sock = socket::discover_primal(&name);
            if sock.is_some() {
                info!(primal = %name, "fallback primal found");
            }
            sock
        })
    };

    let Some(ref target_socket) = target else {
        info!("running standalone — no orchestrator or fallback");
        return;
    };

    if try_announce(target_socket, our_socket) {
        return;
    }

    register_legacy(target_socket, our_socket);
}

/// Wave 17: single-call registration via `primal.announce`.
fn try_announce(target_socket: &Path, our_socket: &Path) -> bool {
    let methods: Vec<&str> = crate::capabilities::ALL_CAPABILITIES.to_vec();

    let result = rpc::try_send(
        target_socket,
        "primal.announce",
        &serde_json::json!({
            "primal_id": crate::capabilities::PRIMAL_NAME,
            "transport": our_socket.to_string_lossy(),
            "methods": methods,
            "lifecycle": { "state": "running" },
        }),
    );

    match result {
        Ok(_) => {
            info!(
                methods = methods.len(),
                domain = crate::capabilities::PRIMAL_DOMAIN,
                "registered via primal.announce (Wave 17)"
            );
            true
        }
        Err(e) => {
            info!("primal.announce not available ({e}), falling back to legacy registration");
            false
        }
    }
}

/// Pre-Wave 17 registration: `lifecycle.register` + N × `capability.register`.
fn register_legacy(target_socket: &Path, our_socket: &Path) {
    match rpc::try_send(
        target_socket,
        "lifecycle.register",
        &serde_json::json!({
            "name": crate::capabilities::PRIMAL_NAME,
            "socket_path": our_socket.to_string_lossy(),
            "pid": std::process::id(),
        }),
    ) {
        Ok(_) => info!("registered with lifecycle manager"),
        Err(e) => warn!("lifecycle registration failed: {e}"),
    }

    let health_mappings = crate::capabilities::build_semantic_mappings();
    if let Err(e) = rpc::try_send(
        target_socket,
        "capability.register",
        &serde_json::json!({
            "primal": crate::capabilities::PRIMAL_NAME,
            "capability": crate::capabilities::PRIMAL_DOMAIN,
            "socket": our_socket.to_string_lossy(),
            "semantic_mappings": health_mappings,
        }),
    ) {
        warn!("domain capability registration failed: {e}");
    }

    let mut registered = 0;

    for cap in crate::capabilities::LOCAL_CAPABILITIES {
        if rpc::send(
            target_socket,
            "capability.register",
            &serde_json::json!({
                "primal": crate::capabilities::PRIMAL_NAME,
                "capability": cap,
                "socket": our_socket.to_string_lossy(),
                "served_locally": true,
            }),
        )
        .is_some()
        {
            registered += 1;
        }
    }

    for &(cap, provider) in crate::capabilities::ROUTED_CAPABILITIES {
        if rpc::send(
            target_socket,
            "capability.register",
            &serde_json::json!({
                "primal": crate::capabilities::PRIMAL_NAME,
                "capability": cap,
                "socket": our_socket.to_string_lossy(),
                "served_locally": false,
                "canonical_provider": provider,
            }),
        )
        .is_some()
        {
            registered += 1;
        }
    }

    info!(
        registered,
        local = crate::capabilities::LOCAL_CAPABILITIES.len(),
        routed = crate::capabilities::ROUTED_CAPABILITIES.len(),
        total = crate::capabilities::ALL_CAPABILITIES.len(),
        domain = crate::capabilities::PRIMAL_DOMAIN,
        "capabilities registered via legacy pattern (local + routed)"
    );
}

/// Announces health.* capabilities to Songbird for discovery.
pub fn announce_to_songbird(our_socket: &Path) {
    let socket_str = our_socket.to_string_lossy();
    match healthspring_barracuda::visualization::capabilities::announce_all(&socket_str) {
        Ok(()) => {
            info!(
                count = healthspring_barracuda::visualization::capabilities::CAPABILITIES.len(),
                "discovery: announced health.* capabilities"
            );
        }
        Err(e) => {
            info!("discovery service not available ({e}) — socket dir only");
        }
    }
}

/// Spawns the heartbeat thread that sends lifecycle.status to the orchestrator.
pub fn spawn_heartbeat(
    running: Arc<AtomicBool>,
    state: Arc<super::routing::PrimalState>,
    socket_path: std::path::PathBuf,
) {
    let target = {
        let orch = socket::orchestrator_socket();
        if orch.exists() {
            Some(orch)
        } else {
            socket::fallback_registration_primal().and_then(|name| socket::discover_primal(&name))
        }
    };

    std::thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
            if let Some(ref t) = target {
                if let Err(e) = rpc::try_send(
                    t,
                    "lifecycle.status",
                    &serde_json::json!({
                        "name": crate::capabilities::PRIMAL_NAME,
                        "socket_path": socket_path.to_string_lossy(),
                        "status": "healthy",
                        "requests_served": state.requests_served.load(Ordering::Relaxed),
                    }),
                ) {
                    warn!("heartbeat status update failed: {e}");
                }
            }
        }
    });
}
