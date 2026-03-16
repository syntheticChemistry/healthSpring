// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

//! healthSpring biomeOS Primal — BYOB Niche Deployment
//!
//! JSON-RPC 2.0 server exposing healthSpring's science capabilities to the
//! `biomeOS` ecosystem via Unix domain socket. `UniBin` compliant: single
//! binary with `serve`, `version`, and `capabilities` subcommands.
//!
//! ## Signal handling
//!
//! Listens for SIGTERM and SIGINT via a self-pipe. On signal receipt, the
//! `running` flag is set to `false`, the accept loop breaks, the heartbeat
//! thread stops, and the socket file is cleaned up.
//!
//! Socket: `$XDG_RUNTIME_DIR/biomeos/healthspring-{family_id}.sock`

mod capabilities;
mod server;

use clap::{Parser, Subcommand};

// ═══════════════════════════════════════════════════════════════════════════
// CLI (UniBin)
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
    Serve,
    /// Print version information
    Version,
    /// List all registered capabilities
    Capabilities,
}

// ═══════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => {
            if let Err(e) = server::cmd_serve() {
                eprintln!("[fatal] {e}");
                std::process::exit(1);
            }
        }
        Command::Version => capabilities::cmd_version(),
        Command::Capabilities => capabilities::cmd_capabilities(),
    }
}
