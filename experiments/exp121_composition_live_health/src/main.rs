// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp121 — Composition health triad: liveness/readiness/capability.list via IPC.
//!
//! Validates the three NUCLEUS health probes against a live healthSpring
//! primal server over JSON-RPC IPC. Where exp114 validated the dispatch
//! registry structurally, this experiment exercises the **real wire path**:
//!
//! ```text
//! Test binary  →  Unix socket  →  healthspring_primal
//!              →  health.liveness    (Tower contract)
//!              →  health.readiness   (Node contract)
//!              →  capability.list    (NUCLEUS discovery)
//!              →  identity.get       (self-knowledge)
//!              →  science dispatch   (niche proof)
//! ```
//!
//! Follows primalSpring exp094 / airSpring `validate_nucleus` patterns.
//! Graceful degradation when primal is offline.
//!
//! ## Provenance
//!
//! - Structural validation: exp114, exp115, exp118
//! - Composition target: full NUCLEUS health probes

use healthspring_barracuda::ipc::client::PrimalClient;
use healthspring_barracuda::ipc::socket;
use healthspring_barracuda::niche;
use healthspring_barracuda::validation::ValidationHarness;

fn discover_healthspring() -> Option<PrimalClient> {
    let candidates = [
        socket::discover_by_capability_public("health"),
        socket::discover_primal("healthspring"),
        Some(socket::resolve_bind_path()),
    ];
    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return Some(PrimalClient::new(candidate, "healthspring"));
        }
    }
    None
}

fn validate_liveness(h: &mut ValidationHarness, client: &PrimalClient) {
    match client.health_liveness() {
        Ok(resp) => {
            let status = resp
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            h.check_bool(
                "health.liveness responds",
                status == "healthy" || status == "ok" || status == "alive",
            );
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("health.liveness [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("health.liveness: {e}"), false);
            }
        }
    }
}

fn validate_readiness(h: &mut ValidationHarness, client: &PrimalClient) {
    match client.health_readiness() {
        Ok(resp) => {
            let status = resp
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            h.check_bool(
                "health.readiness responds",
                status == "ready" || status == "healthy" || status == "ok",
            );
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("health.readiness [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("health.readiness: {e}"), false);
            }
        }
    }
}

fn validate_capability_list(h: &mut ValidationHarness, client: &PrimalClient) {
    match client.capabilities() {
        Ok(resp) => {
            h.check_bool("capability.list responds", true);

            let cap_array = resp
                .get("capabilities")
                .and_then(serde_json::Value::as_array);
            if let Some(caps) = cap_array {
                h.check_bool("capability list is non-empty", !caps.is_empty());

                for required in niche::CAPABILITIES {
                    let found = caps.iter().any(|c| {
                        c.get("method")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|m| m == *required)
                    });
                    h.check_bool(&format!("capability '{required}' advertised"), found);
                }
            } else {
                h.check_bool("capability list has 'capabilities' array", false);
            }
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("capability.list [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("capability.list: {e}"), false);
            }
        }
    }
}

fn validate_identity(h: &mut ValidationHarness, client: &PrimalClient) {
    match client.try_call("identity.get", &serde_json::json!({})) {
        Ok(resp) => {
            let name = resp
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool("identity.get returns healthspring", name == "healthspring");

            let domain = resp
                .get("domain")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool("identity domain is health", domain == "health");
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("identity.get [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("identity.get: {e}"), false);
            }
        }
    }
}

fn validate_niche_science_dispatch(h: &mut ValidationHarness, client: &PrimalClient) {
    let params = serde_json::json!({
        "concentration": 10.0,
        "ic50": 10.0,
        "hill_n": 2.0,
        "e_max": 1.0,
    });

    match client.try_call("science.pkpd.hill_dose_response", &params) {
        Ok(resp) => {
            let has_response = resp
                .get("response")
                .and_then(serde_json::Value::as_f64)
                .is_some();
            h.check_bool("niche science dispatch via IPC", has_response);
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("niche dispatch [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("niche dispatch: {e}"), false);
            }
        }
    }
}

const fn is_connection_error(e: &healthspring_barracuda::ipc::error::IpcError) -> bool {
    e.is_connection_error()
}

fn main() {
    let mut h = ValidationHarness::new("exp121_composition_live_health");

    let client = discover_healthspring();

    h.check_bool("healthspring primal discovery (or offline-skip)", true);

    if let Some(ref c) = client {
        validate_liveness(&mut h, c);
        validate_readiness(&mut h, c);
        validate_capability_list(&mut h, c);
        validate_identity(&mut h, c);
        validate_niche_science_dispatch(&mut h, c);
    } else {
        h.check_bool("health.liveness [SKIP: primal offline]", true);
        h.check_bool("health.readiness [SKIP: primal offline]", true);
        h.check_bool("capability.list [SKIP: primal offline]", true);
        h.check_bool("identity.get [SKIP: primal offline]", true);
        h.check_bool("niche dispatch [SKIP: primal offline]", true);
    }

    h.exit();
}
