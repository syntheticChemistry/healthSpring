// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp064: Push visualization data to petalTongue via IPC.
//!
//! Builds a scenario (PK/PD study), discovers petalTongue at runtime, and
//! pushes data via visualization.render. Falls back to writing JSON file
//! if petalTongue is not available.

use healthspring_barracuda::visualization::ipc_push::{PetalTonguePushClient, PushError};
use healthspring_barracuda::visualization::scenarios;
use std::fs;
use std::path::Path;

fn main() {
    let mut checks = 0;
    let mut pass = 0;

    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            checks += 1;
            if $cond {
                pass += 1;
                println!("  [PASS] {}", $name);
            } else {
                println!("  [FAIL] {}", $name);
            }
        }};
    }

    println!("\n=== Exp064: IPC Push to petalTongue ===\n");

    // Build scenario
    let (scenario, edges) = scenarios::pkpd_study();
    let json = scenarios::scenario_with_edges_json(&scenario, &edges);

    check!("pkpd scenario built", scenario.ecosystem.primals.len() == 6);
    check!("pkpd has edges", edges.len() == 5);
    check!("JSON valid", serde_json::from_str::<serde_json::Value>(&json).is_ok());

    // Try IPC push
    match PetalTonguePushClient::discover() {
        Ok(client) => {
            let session_id = "healthspring-exp064-pkpd";
            match client.push_render(session_id, &scenario.name, &scenario) {
                Ok(()) => {
                    check!("IPC push succeeded", true);
                    println!("  Pushed to petalTongue session '{}'", session_id);
                }
                Err(e) => {
                    check!(
                        &format!("IPC push failed: {e}"),
                        false
                    );
                }
            }
        }
        Err(PushError::NotFound(_)) => {
            check!("petalTongue not found (expected when not running)", true);
            println!("  petalTongue not running — falling back to file write");

            let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
            fs::create_dir_all(&out).expect("create sandbox/scenarios/");
            let path = out.join("healthspring-exp064-pkpd.json");
            fs::write(&path, &json).expect("write scenario JSON");
            println!("  wrote {} ({} KB)", path.display(), json.len() / 1024);

            check!("fallback file written", path.exists());
        }
        Err(e) => {
            check!(&format!("discovery/push error: {e}"), false);
        }
    }

    println!("\n====================================");
    println!("Exp064 IPC Push: {pass}/{checks} checks passed");
    assert_eq!(pass, checks, "some checks failed");
}
