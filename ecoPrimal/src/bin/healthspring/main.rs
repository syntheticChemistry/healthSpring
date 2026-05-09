// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! healthSpring `UniBin` — eukaryotic single binary.
//!
//! Consolidates certification, validation, serve, status, and version into
//! a single binary per the primalSpring `UniBin` architecture pattern.
//!
//! ## Subcommands
//!
//! - `certify`  — Run certification suite (guideStone organelle).
//! - `validate` — Run validation scenarios from the scenario registry.
//! - `serve`    — Start the JSON-RPC 2.0 server.
//! - `status`   — Print discovery and capability status.
//! - `version`  — Print version information.

mod cli;
mod validate;

use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("healthspring=info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Certify { max_tier, bare } => {
            let tier = if bare { 1 } else { max_tier };
            eprintln!("healthSpring UniBin — certify (max tier {tier})\n");
            let v = healthspring_barracuda::certification::certify(tier);
            std::process::exit(v.exit_code());
        }

        cli::Command::Validate {
            tier,
            track,
            scenario,
        } => {
            validate::cmd_validate(tier.as_ref(), track.as_deref(), scenario.as_deref());
        }

        cli::Command::Serve { port } => {
            eprintln!("healthSpring UniBin — serve\n");
            eprintln!(
                "NOTE: Full serve implementation requires server module extraction.\n\
                 For now, use `healthspring_primal serve` directly."
            );
            if let Some(p) = port {
                eprintln!("  requested port: {p}");
            }
            eprintln!("\nServer module extraction is tracked for post-interstadial evolution.");
        }

        cli::Command::Status => {
            cmd_status();
        }

        cli::Command::Version => {
            cmd_version();
        }
    }
}

fn cmd_version() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("healthspring {version}");
    eprintln!("  domain:     {}", healthspring_barracuda::PRIMAL_DOMAIN);
    eprintln!("  primalSpring: v0.9.25 (pinned)");
}

fn cmd_status() {
    eprintln!("healthSpring UniBin — status\n");
    cmd_version();
    eprintln!();

    let ctx = primalspring::composition::CompositionContext::from_live_discovery_with_fallback();
    let caps = ctx.available_capabilities();
    if caps.is_empty() {
        eprintln!("  NUCLEUS: no primals discovered (bare mode)");
    } else {
        eprintln!("  NUCLEUS: {} capabilities discovered", caps.len());
        for cap in &caps {
            eprintln!("    - {cap}");
        }
    }
}
