// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! healthSpring biomeOS Primal — BYOB Niche Deployment
//!
//! JSON-RPC 2.0 server exposing healthSpring's science capabilities to the
//! `biomeOS` ecosystem via Unix domain socket and optional TCP listener.
//! `UniBin` compliant: single binary with `serve`/`server`, `version`, and
//! `capabilities` subcommands.
//!
//! ## Transport
//!
//! Primary: Unix domain socket at `$XDG_RUNTIME_DIR/biomeos/healthspring-{family_id}.sock`
//! Optional: TCP newline JSON-RPC on `--port <PORT>` (default from `HEALTHSPRING_PORT`)
//!
//! ## Signal handling
//!
//! Listens for SIGTERM and SIGINT via a self-pipe. On signal receipt, the
//! `running` flag is set to `false`, the accept loop breaks, the heartbeat
//! thread stops, and the socket file is cleaned up.

mod capabilities;
mod server;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

// ═══════════════════════════════════════════════════════════════════════════
// CLI (UniBin — DEPLOYMENT_VALIDATION_STANDARD aligned)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Parser)]
#[command(
    name = "healthspring_primal",
    about = "healthSpring biomeOS primal — health science compute via JSON-RPC 2.0",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the JSON-RPC 2.0 server (default)
    Serve {
        /// Optional TCP port for newline JSON-RPC listener
        #[arg(long)]
        port: Option<u16>,
    },
    /// Start the JSON-RPC 2.0 server (alias for `serve`)
    Server {
        /// Optional TCP port for newline JSON-RPC listener
        #[arg(long)]
        port: Option<u16>,
    },
    /// Print version information
    Version,
    /// List all registered capabilities
    Capabilities,
}

// ═══════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("healthspring=info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Serve { port: None }) {
        Command::Serve { port } | Command::Server { port } => {
            let tcp_port = port.or_else(|| {
                std::env::var("HEALTHSPRING_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
            });
            if let Err(e) = server::cmd_serve(tcp_port) {
                tracing::error!("{e}");
                std::process::exit(1);
            }
        }
        Command::Version => capabilities::cmd_version(),
        Command::Capabilities => capabilities::cmd_capabilities(),
    }
}
