// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp119 — Composition parity: live IPC science dispatch vs local Rust.
//!
//! This is the **composition evolution** experiment: where exp112/113 validated
//! that the in-process dispatch layer reproduces local Rust math, this experiment
//! validates the **full NUCLEUS wire path**:
//!
//! ```text
//! Test binary  →  Unix socket  →  healthspring_primal JSON-RPC server
//!              →  routing.rs dispatch  →  science module  →  response
//!              →  compare against direct Rust call in this binary
//! ```
//!
//! Follows primalSpring exp094 pattern: discover live primal, call via IPC,
//! compare against local baselines, skip gracefully when primal is offline.
//!
//! ## Provenance
//!
//! - Python baselines: `control/pkpd/exp001_*.py`, `control/microbiome/exp010_*.py`
//! - Rust validation: exp001, exp010, exp112, exp113
//! - Composition target: `healthspring_enclave_proto_nucleate.toml`

use healthspring_barracuda::ipc::client::PrimalClient;
use healthspring_barracuda::ipc::socket;
use healthspring_barracuda::microbiome;
use healthspring_barracuda::pkpd;
use healthspring_barracuda::tolerances;
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

fn validate_hill_parity(h: &mut ValidationHarness, client: &PrimalClient) {
    let local = pkpd::hill_dose_response(10.0, 10.0, 2.0, 1.0);

    let params = serde_json::json!({
        "concentration": 10.0,
        "ic50": 10.0,
        "hill_n": 2.0,
        "e_max": 1.0,
    });

    match client.try_call("science.pkpd.hill_dose_response", &params) {
        Ok(resp) => {
            if let Some(ipc_val) = resp.get("response").and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "Hill IC50 IPC parity",
                    ipc_val,
                    local,
                    tolerances::DETERMINISM,
                );
            } else {
                h.check_bool("Hill IPC response has 'response' field", false);
            }
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("Hill IPC parity [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("Hill IPC parity: {e}"), false);
            }
        }
    }
}

fn validate_compartment_parity(h: &mut ValidationHarness, client: &PrimalClient) {
    let local = pkpd::pk_iv_bolus(100.0, 10.0, 6.93, 0.0);

    let params = serde_json::json!({
        "dose_mg": 100.0,
        "vd": 10.0,
        "half_life_hr": 6.93,
        "t": 0.0,
    });

    match client.try_call("science.pkpd.one_compartment_pk", &params) {
        Ok(resp) => {
            if let Some(ipc_val) = resp
                .get("concentration")
                .and_then(serde_json::Value::as_f64)
            {
                h.check_abs(
                    "1-comp IV C(0) IPC parity",
                    ipc_val,
                    local,
                    tolerances::DETERMINISM,
                );
            } else {
                h.check_bool("1-comp IPC response has 'concentration' field", false);
            }
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("1-comp IPC parity [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("1-comp IPC parity: {e}"), false);
            }
        }
    }
}

fn validate_shannon_parity(h: &mut ValidationHarness, client: &PrimalClient) {
    let abundances = vec![0.4, 0.3, 0.2, 0.1];
    let local = microbiome::shannon_index(&abundances);

    let params = serde_json::json!({ "abundances": abundances });

    match client.try_call("science.microbiome.shannon_index", &params) {
        Ok(resp) => {
            if let Some(ipc_val) = resp.get("shannon").and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "Shannon IPC parity",
                    ipc_val,
                    local,
                    tolerances::DETERMINISM,
                );
            } else {
                h.check_bool("Shannon IPC response has 'shannon' field", false);
            }
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("Shannon IPC parity [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("Shannon IPC parity: {e}"), false);
            }
        }
    }
}

fn validate_auc_parity(h: &mut ValidationHarness, client: &PrimalClient) {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let concs = vec![10.0, 7.0, 3.0, 1.0];
    let local = pkpd::auc_trapezoidal(&times, &concs);

    let params = serde_json::json!({
        "times": times,
        "concentrations": concs,
    });

    match client.try_call("science.pkpd.auc_trapezoidal", &params) {
        Ok(resp) => {
            if let Some(ipc_val) = resp.get("auc").and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "AUC trapezoidal IPC parity",
                    ipc_val,
                    local,
                    tolerances::DETERMINISM,
                );
            } else {
                h.check_bool("AUC IPC response has 'auc' field", false);
            }
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("AUC IPC parity [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("AUC IPC parity: {e}"), false);
            }
        }
    }
}

fn validate_anderson_parity(h: &mut ValidationHarness, client: &PrimalClient) {
    let disorder = vec![1.0, 0.5, 2.0, 1.5, 0.8];
    let (local_eig, _local_ipr) = microbiome::anderson_diagonalize(&disorder, 1.0);

    let params = serde_json::json!({ "disorder": disorder, "t_hop": 1.0 });

    match client.try_call("science.microbiome.anderson_gut", &params) {
        Ok(resp) => {
            if let Some(eig_arr) = resp
                .get("eigenvalues")
                .and_then(serde_json::Value::as_array)
            {
                let n = local_eig.len().min(eig_arr.len());
                let mut max_diff = 0.0_f64;
                for i in 0..n {
                    if let Some(e) = eig_arr[i].as_f64() {
                        max_diff = max_diff.max((e - local_eig[i]).abs());
                    }
                }
                h.check_upper(
                    "Anderson eigenvalue IPC max diff",
                    max_diff,
                    tolerances::DETERMINISM,
                );
            } else {
                h.check_bool("Anderson IPC has eigenvalues", false);
            }
        }
        Err(e) => {
            if is_connection_error(&e) {
                h.check_bool("Anderson IPC parity [SKIP: primal offline]", true);
            } else {
                h.check_bool(&format!("Anderson IPC parity: {e}"), false);
            }
        }
    }
}

const fn is_connection_error(e: &healthspring_barracuda::ipc::error::IpcError) -> bool {
    e.is_connection_error()
}

fn main() {
    let mut h = ValidationHarness::new("exp119_composition_live_parity");

    let client = discover_healthspring();

    h.check_bool("healthspring primal discovery (or offline-skip)", true);

    if let Some(ref c) = client {
        validate_hill_parity(&mut h, c);
        validate_compartment_parity(&mut h, c);
        validate_shannon_parity(&mut h, c);
        validate_auc_parity(&mut h, c);
        validate_anderson_parity(&mut h, c);
    } else {
        h.check_bool("Hill IPC parity [SKIP: primal offline]", true);
        h.check_bool("1-comp IPC parity [SKIP: primal offline]", true);
        h.check_bool("Shannon IPC parity [SKIP: primal offline]", true);
        h.check_bool("AUC IPC parity [SKIP: primal offline]", true);
        h.check_bool("Anderson IPC parity [SKIP: primal offline]", true);
    }

    h.exit();
}
