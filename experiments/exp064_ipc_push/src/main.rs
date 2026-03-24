// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp064: Push visualization data to petalTongue via IPC.
//!
//! Builds a scenario (PK/PD study), discovers petalTongue at runtime, and
//! pushes data via visualization.render. Falls back to writing JSON file
//! if petalTongue is not available.

use healthspring_barracuda::validation::ValidationHarness;
use healthspring_barracuda::visualization::ipc_push::{PetalTonguePushClient, PushError};
use healthspring_barracuda::visualization::scenarios;
use std::fs;
use std::path::Path;

fn main() {
    let mut h = ValidationHarness::new("exp064_ipc_push");

    let (scenario, edges) = scenarios::pkpd_study();
    let json = scenarios::scenario_with_edges_json(&scenario, &edges);

    h.check_exact(
        "pkpd scenario primal count",
        scenario.ecosystem.primals.len() as u64,
        6,
    );
    h.check_exact("pkpd edge count", edges.len() as u64, 5);
    h.check_bool(
        "JSON valid",
        serde_json::from_str::<serde_json::Value>(&json).is_ok(),
    );

    match PetalTonguePushClient::discover() {
        Ok(client) => {
            let session_id = "healthspring-exp064-pkpd";
            match client.push_render(session_id, &scenario.name, &scenario) {
                Ok(()) => {
                    h.check_bool("IPC push succeeded", true);
                }
                Err(e) => {
                    h.check_bool(&format!("IPC push failed: {e}"), false);
                }
            }
        }
        Err(PushError::NotFound(_)) => {
            h.check_bool("petalTongue not found (expected when not running)", true);

            let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
            if fs::create_dir_all(&out).is_err() {
                eprintln!("ERROR: create sandbox/scenarios/");
                std::process::exit(1);
            }
            let path = out.join("healthspring-exp064-pkpd.json");
            if fs::write(&path, &json).is_err() {
                eprintln!("ERROR: write scenario JSON");
                std::process::exit(1);
            }

            h.check_bool("fallback file written", path.exists());
        }
        Err(e) => {
            h.check_bool(&format!("discovery/push error: {e}"), false);
        }
    }

    h.exit();
}
