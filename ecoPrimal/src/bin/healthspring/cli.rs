// SPDX-License-Identifier: AGPL-3.0-or-later

//! CLI definition for the healthSpring `UniBin`.

use clap::{Parser, Subcommand};

use super::validate::TierFilter;

#[derive(Parser)]
#[command(
    name = "healthspring_unibin",
    about = "healthSpring UniBin — eukaryotic single binary for certification, validation, and serving",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run certification suite (guideStone organelle).
    Certify {
        /// Maximum certification tier (1=bare, 2=IPC, 3=full NUCLEUS).
        #[arg(long, default_value_t = 3)]
        max_tier: u8,

        /// Run only the bare (Tier 1) structural checks, no IPC.
        #[arg(long)]
        bare: bool,
    },

    /// Run validation scenarios from the scenario registry.
    Validate {
        /// Filter by tier: rust, live, or both.
        #[arg(long)]
        tier: Option<TierFilter>,

        /// Filter by track (e.g., pkpd, microbiome, biosignal).
        #[arg(long)]
        track: Option<String>,

        /// Run a specific scenario by ID.
        #[arg(long)]
        scenario: Option<String>,
    },

    /// Start the JSON-RPC 2.0 server.
    Serve {
        /// Optional TCP port for newline JSON-RPC listener.
        #[arg(long)]
        port: Option<u16>,
    },

    /// Print discovery and capability status.
    Status,

    /// Print version information.
    Version,
}
