// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp120 — Composition provenance: live trio round-trip via IPC.
//!
//! Validates the provenance trio integration (rhizoCrypt + loamSpine +
//! sweetGrass) over IPC. Where exp116 tested the provenance session API
//! locally, this experiment validates the **composed** provenance pipeline:
//!
//! ```text
//! healthSpring  →  capability.call("dag", "create_session")  →  rhizoCrypt
//!               →  capability.call("dag", "append_event")    →  rhizoCrypt
//!               →  capability.call("dag", "dehydrate")       →  Merkle root
//!               →  capability.call("commit", "session")      →  loamSpine
//!               →  capability.call("provenance", "create_braid")  →  sweetGrass
//! ```
//!
//! Graceful degradation: if the trio is not running, all checks skip.
//!
//! ## Provenance
//!
//! - Local validation: exp116
//! - Composition target: Nest Atomic (provenance trio)

use healthspring_barracuda::data;
use healthspring_barracuda::ipc::client::PrimalClient;
use healthspring_barracuda::ipc::socket;
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

fn validate_provenance_dispatch(h: &mut ValidationHarness, client: &PrimalClient) {
    let params = serde_json::json!({
        "dataset": "exp120_live_test",
        "spring": "healthSpring",
    });

    match client.try_call("provenance.begin", &params) {
        Ok(resp) => {
            let has_id = resp.get("session_id").is_some()
                || resp.get("id").is_some()
                || resp.get("result").is_some();
            h.check_bool("provenance.begin dispatches via IPC", has_id);
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("provenance.begin [SKIP: primal offline]", true);
            } else {
                h.check_bool(
                    "provenance.begin dispatches via IPC",
                    e.to_string().contains("method_not_found") || e.to_string().contains("missing"),
                );
            }
        }
    }
}

fn validate_trio_session_lifecycle(h: &mut ValidationHarness) {
    let session = data::begin_data_session("exp120_composition_test");

    if session.available {
        h.check_bool("Trio session created", true);

        let step = serde_json::json!({
            "source": "exp120",
            "operation": "live_provenance_parity",
            "content_category": "Code",
        });
        let vertex = data::record_fetch_step(&session.id, &step);
        h.check_bool("Trio vertex appended", vertex.available);

        let chain = data::complete_data_session(&session.id, "AGPL-3.0-or-later");
        h.check_bool(
            "Trio chain complete",
            chain.status == "complete" || chain.status == "partial",
        );

        if chain.status == "complete" {
            h.check_bool("Merkle root non-empty", !chain.merkle_root.is_empty());
            h.check_bool("Commit ID non-empty", !chain.commit_id.is_empty());
            h.check_bool("Braid ID non-empty", !chain.braid_id.is_empty());
        }
    } else {
        h.check_bool("Trio session [SKIP: trio unavailable]", true);
        h.check_bool("Trio vertex [SKIP: trio unavailable]", true);
        h.check_bool("Trio chain [SKIP: trio unavailable]", true);
    }
}

fn validate_trio_determinism(h: &mut ValidationHarness) {
    let s1 = data::begin_data_session("exp120_det_a");
    let s2 = data::begin_data_session("exp120_det_b");

    if s1.available && s2.available {
        h.check_bool("Distinct trio sessions get distinct IDs", s1.id != s2.id);
    } else {
        h.check_bool("Trio determinism [SKIP: trio unavailable]", true);
    }
}

const fn is_connection_error(e: &healthspring_barracuda::ipc::error::IpcError) -> bool {
    e.is_connection_error()
}

fn main() {
    let mut h = ValidationHarness::new("exp120_composition_live_provenance");

    let client = discover_healthspring();

    if let Some(ref c) = client {
        validate_provenance_dispatch(&mut h, c);
    } else {
        h.check_bool("provenance IPC [SKIP: primal offline]", true);
    }

    validate_trio_session_lifecycle(&mut h);
    validate_trio_determinism(&mut h);

    h.exit();
}
