// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC server, connection handling, and biomeOS registration.
//!
//! Orchestrates signal handling, request routing, connection handling,
//! and lifecycle registration. The main entry point is [`cmd_serve`].

mod connection;
mod registration;
mod routing;
mod signal;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use healthspring_barracuda::ipc::socket;
use tracing::{error, info};

/// Starts the JSON-RPC server: binds to the socket, registers with biomeOS,
/// spawns heartbeat, and accepts connections until shutdown.
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

    let state = Arc::new(routing::PrimalState {
        start_time: Instant::now(),
        requests_served: AtomicU64::new(0),
    });

    let listener = std::os::unix::net::UnixListener::bind(&socket_path)
        .map_err(|e| format!("Cannot bind to {}: {e}", socket_path.display()))?;

    listener
        .set_nonblocking(false)
        .map_err(|e| format!("Cannot set listener options: {e}"))?;

    let family_id = socket::get_family_id();
    info!(
        primal = crate::capabilities::PRIMAL_NAME,
        socket = %socket_path.display(),
        family_id = %family_id,
        domain = crate::capabilities::PRIMAL_DOMAIN,
        version = env!("CARGO_PKG_VERSION"),
        capabilities = crate::capabilities::ALL_CAPABILITIES.len(),
        mode = "BYOB Niche (biomeOS)",
        "primal listening"
    );

    registration::register_with_biomeos(&socket_path);
    registration::announce_to_songbird(&socket_path);

    let running = Arc::new(AtomicBool::new(true));
    signal::install_signal_handler(&running);
    registration::spawn_heartbeat(running.clone(), state.clone(), socket_path.clone());

    info!("accepting connections");
    for stream in listener.incoming() {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || connection::handle_connection(stream, &state));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }
            Err(e) => error!("accept failed: {e}"),
        }
    }

    info!("shutdown — cleaning up socket");
    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}
