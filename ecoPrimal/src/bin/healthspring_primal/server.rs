// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC server, connection handling, and biomeOS registration.
//!
//! Orchestrates signal handling, request routing, connection handling,
//! and lifecycle registration. Supports both Unix domain socket (primary)
//! and TCP (optional, via `--port`) per Deployment Validation Standard.
//! The main entry point is [`cmd_serve`].

mod connection;
mod registration;
mod routing;
mod signal;

use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use healthspring_barracuda::ipc::socket;
use tracing::{error, info};

/// Server lifecycle errors.
#[derive(Debug)]
pub enum ServerError {
    /// Failed to create socket directory, remove stale socket, or bind.
    Io(std::io::Error),
    /// Failed to configure socket options.
    SocketConfig(std::io::Error),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "server I/O error: {e}"),
            Self::SocketConfig(e) => write!(f, "socket configuration error: {e}"),
        }
    }
}

/// Creates a domain symlink (`health.sock` -> actual socket) for capability
/// discovery per `PRIMAL_IPC_PROTOCOL.md` v3.1.
fn create_domain_symlink(socket_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let parent = socket_path.parent()?;
    let symlink_path = parent.join(format!("{}.sock", crate::capabilities::PRIMAL_DOMAIN));
    if symlink_path.exists() {
        return None;
    }
    let target = socket_path.file_name()?;
    match std::os::unix::fs::symlink(target, &symlink_path) {
        Ok(()) => {
            info!(
                symlink = %symlink_path.display(),
                target = %target.to_string_lossy(),
                "domain symlink created"
            );
            Some(symlink_path)
        }
        Err(e) => {
            tracing::warn!(
                symlink = %symlink_path.display(),
                "domain symlink creation failed: {e}"
            );
            None
        }
    }
}

/// Starts the JSON-RPC server: binds to the Unix socket (and optionally TCP),
/// registers with biomeOS, spawns heartbeat, and accepts connections until
/// shutdown.
pub fn cmd_serve(tcp_port: Option<u16>) -> Result<(), ServerError> {
    let socket_path = socket::resolve_bind_path();

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).map_err(ServerError::Io)?;
    }

    if socket_path.exists() {
        std::fs::remove_file(&socket_path).map_err(ServerError::Io)?;
    }

    let state = Arc::new(routing::PrimalState {
        start_time: Instant::now(),
        requests_served: AtomicU64::new(0),
    });

    let listener = std::os::unix::net::UnixListener::bind(&socket_path).map_err(ServerError::Io)?;

    listener
        .set_nonblocking(false)
        .map_err(ServerError::SocketConfig)?;

    let domain_symlink = create_domain_symlink(&socket_path);

    let family_id = socket::get_family_id();
    info!(
        primal = crate::capabilities::PRIMAL_NAME,
        socket = %socket_path.display(),
        family_id = %family_id,
        domain = crate::capabilities::PRIMAL_DOMAIN,
        version = env!("CARGO_PKG_VERSION"),
        capabilities = crate::capabilities::ALL_CAPABILITIES.len(),
        tcp_port = tcp_port.map_or_else(|| "none".into(), |p| p.to_string()).as_str(),
        mode = "BYOB Niche (biomeOS)",
        "primal listening"
    );

    registration::register_with_biomeos(&socket_path);
    registration::announce_to_songbird(&socket_path);

    let running = Arc::new(AtomicBool::new(true));
    signal::install_signal_handler(&running);
    registration::spawn_heartbeat(running.clone(), state.clone(), socket_path.clone());

    if let Some(port) = tcp_port {
        let tcp_state = state.clone();
        let tcp_running = running.clone();
        std::thread::spawn(move || {
            accept_tcp(port, tcp_state, tcp_running);
        });
    }

    info!("accepting connections");
    for stream in listener.incoming() {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || connection::handle_unix_connection(stream, &state));
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
    if let Some(ref symlink) = domain_symlink {
        let _ = std::fs::remove_file(symlink);
    }
    Ok(())
}

/// TCP accept loop — newline-delimited JSON-RPC per Deployment Validation
/// Standard. Binds to `0.0.0.0:{port}` so plasmidBin and federation peers
/// can reach the primal.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Arc values are moved into this thread-spawned function"
)]
fn accept_tcp(port: u16, state: Arc<routing::PrimalState>, running: Arc<AtomicBool>) {
    let addr = format!("0.0.0.0:{port}");
    let tcp_listener = match TcpListener::bind(&addr) {
        Ok(l) => {
            info!(port, "TCP JSON-RPC listener bound");
            l
        }
        Err(e) => {
            error!(port, "TCP bind failed: {e}");
            return;
        }
    };
    tcp_listener.set_nonblocking(false).ok();

    for stream in tcp_listener.incoming() {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || connection::handle_tcp_connection(stream, &state));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }
            Err(e) => error!("TCP accept failed: {e}"),
        }
    }
}
