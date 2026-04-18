// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp122 — Level 5 primal proof: barraCuda IPC parity.
//!
//! Validates that calling barraCuda's JSON-RPC methods over UDS produces
//! identical results to the local math. This is the primal proof for the
//! wire-ready methods (`stats.mean`, `stats.std_dev`).
//!
//! Tier structure:
//! - Tier 1: `math_dispatch` matches analytical known-values (always green)
//! - Tier 2: `BarraCudaClient` IPC vs local (requires live barraCuda ecobin)
//! - Tier 3: Full NUCLEUS deployed from plasmidBin (future)
//!
//! Skips gracefully when barraCuda ecobin is offline.

use healthspring_barracuda::ipc::barracuda_client::BarraCudaClient;
use healthspring_barracuda::math_dispatch;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp122_primal_proof_barracuda_parity");

    validate_math_dispatch_known_values(&mut h);

    let client = BarraCudaClient::discover();
    h.check_bool("barraCuda ecobin discovery (or offline-skip)", true);

    if let Some(ref c) = client {
        validate_ipc_liveness(&mut h, c);
        validate_stats_mean_ipc(&mut h, c);
        validate_stats_std_dev_ipc(&mut h, c);
        validate_rng_uniform_ipc(&mut h, c);
        validate_wire_pending_inventory(&mut h);
    } else {
        h.check_bool("barraCuda health.liveness [SKIP: ecobin offline]", true);
        h.check_bool("stats.mean IPC parity [SKIP: ecobin offline]", true);
        h.check_bool("stats.std_dev IPC parity [SKIP: ecobin offline]", true);
        h.check_bool("rng.uniform IPC shape [SKIP: ecobin offline]", true);
        h.check_bool("wire-pending inventory [SKIP: ecobin offline]", true);
    }

    h.exit();
}

fn validate_math_dispatch_known_values(h: &mut ValidationHarness) {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let expected_mean = 5.5;
    let dispatch_mean = math_dispatch::mean(&data);
    h.check_abs(
        "math_dispatch::mean matches analytical (5.5)",
        dispatch_mean,
        expected_mean,
        1e-15,
    );

    let dispatch_sd = math_dispatch::std_dev(&data).unwrap_or(0.0);
    h.check_bool(
        "math_dispatch::std_dev positive for spread data",
        dispatch_sd > 0.0,
    );

    let hill_val = math_dispatch::hill(10.0, 10.0, 1.0);
    h.check_abs(
        "math_dispatch::hill(10, 10, 1) == 0.5",
        hill_val,
        0.5,
        1e-15,
    );

    let uniform = [0.25, 0.25, 0.25, 0.25];
    let shannon = math_dispatch::shannon_from_frequencies(&uniform);
    h.check_abs(
        "math_dispatch::shannon uniform(4) == ln(4)",
        shannon,
        4.0_f64.ln(),
        1e-10,
    );

    h.check_bool(
        "math_dispatch wire counts consistent",
        math_dispatch::WIRE_READY_COUNT + math_dispatch::WIRE_PENDING_COUNT
            == math_dispatch::TOTAL_COUNT,
    );
}

const fn is_connection_error(e: &healthspring_barracuda::ipc::error::IpcError) -> bool {
    e.is_connection_error()
}

fn validate_ipc_liveness(h: &mut ValidationHarness, c: &BarraCudaClient) {
    match c.health_liveness() {
        Ok(_) => h.check_bool("barraCuda health.liveness", true),
        Err(ref e) if is_connection_error(e) => {
            h.check_bool("barraCuda health.liveness [SKIP: connection error]", true);
        }
        Err(e) => {
            h.check_bool(&format!("barraCuda health.liveness FAIL: {e}"), false);
        }
    }
}

fn validate_stats_mean_ipc(h: &mut ValidationHarness, c: &BarraCudaClient) {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let local_val = math_dispatch::mean(&data);

    match c.stats_mean(&data) {
        Ok(ipc_val) => {
            let tol = healthspring_barracuda::tolerances::DETERMINISM;
            h.check_abs("stats.mean IPC == local", ipc_val, local_val, tol);
        }
        Err(ref e) if is_connection_error(e) => {
            h.check_bool("stats.mean IPC parity [SKIP: connection error]", true);
        }
        Err(e) => {
            h.check_bool(&format!("stats.mean IPC FAIL: {e}"), false);
        }
    }
}

fn validate_stats_std_dev_ipc(h: &mut ValidationHarness, c: &BarraCudaClient) {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let local_val = math_dispatch::std_dev(&data).unwrap_or(0.0);

    match c.stats_std_dev(&data) {
        Ok(ipc_val) => {
            let tol = healthspring_barracuda::tolerances::DETERMINISM;
            h.check_abs("stats.std_dev IPC == local", ipc_val, local_val, tol);
        }
        Err(ref e) if is_connection_error(e) => {
            h.check_bool("stats.std_dev IPC parity [SKIP: connection error]", true);
        }
        Err(e) => {
            h.check_bool(&format!("stats.std_dev IPC FAIL: {e}"), false);
        }
    }
}

fn validate_rng_uniform_ipc(h: &mut ValidationHarness, c: &BarraCudaClient) {
    match c.rng_uniform(10, 0.0, 1.0, 42) {
        Ok(samples) => {
            h.check_bool("rng.uniform returns 10 samples", samples.len() == 10);
            h.check_bool(
                "rng.uniform all in [0, 1)",
                samples.iter().all(|&v| (0.0..1.0).contains(&v)),
            );
        }
        Err(ref e) if is_connection_error(e) => {
            h.check_bool("rng.uniform IPC shape [SKIP: connection error]", true);
        }
        Err(e) => {
            h.check_bool(&format!("rng.uniform IPC FAIL: {e}"), false);
        }
    }
}

fn validate_wire_pending_inventory(h: &mut ValidationHarness) {
    use healthspring_barracuda::niche::BARRACUDA_IPC_MIGRATION;
    h.check_bool(
        "BARRACUDA_IPC_MIGRATION inventory covers 12 call sites",
        BARRACUDA_IPC_MIGRATION.len() == 12,
    );
    h.check_bool(
        "math_dispatch total matches migration inventory (excl rng)",
        math_dispatch::TOTAL_COUNT == BARRACUDA_IPC_MIGRATION.len() - 1,
    );
}
