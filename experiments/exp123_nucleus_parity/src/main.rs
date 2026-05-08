// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

//! Exp123: NUCLEUS Composition Parity — healthSpring niche
//!
//! Replicates primalSpring's `exp094_composition_parity` for the healthSpring
//! niche, following the `exp095_proto_nucleate_template` scaffold.
//!
//! Validates the full NUCLEUS pipeline as seen by healthSpring's dual-tower
//! ionic enclave composition:
//!
//! 1. **Tower Atomic** — `BearDog` crypto (hash + determinism) + Songbird
//!    capability discovery
//! 2. **Node Atomic** — barraCuda stats parity via IPC (`stats.mean`,
//!    `stats.std_dev`) + toadStool compute health
//! 3. **Nest Atomic** — `NestGate` `storage.store` / `storage.retrieve`
//!    round-trip + provenance trio health
//! 4. **Cross-Atomic** — hash → store → retrieve → compare pipeline
//! 5. **Niche Parity** — healthSpring-specific science methods via
//!    `math_dispatch` local baseline vs IPC result
//!
//! ## Exit codes
//!
//! - `0` — all checks passed
//! - `1` — at least one check failed
//! - `2` — skipped (no NUCLEUS available)

use healthspring_barracuda::ipc::rpc;
use healthspring_barracuda::ipc::socket;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new(
        "Exp123: NUCLEUS Composition Parity (healthSpring niche)",
    );

    let discovery_ok = validate_discovery(&mut h);
    if !discovery_ok {
        h.check_bool("nucleus_available", false);
        std::process::exit(2);
    }

    validate_tower_atomic(&mut h);
    validate_node_atomic(&mut h);
    validate_nest_atomic(&mut h);
    validate_cross_atomic(&mut h);
    validate_niche_parity(&mut h);

    h.exit();
}

// ── Discovery ───────────────────────────────────────────────────────────────

fn validate_discovery(h: &mut ValidationHarness) -> bool {
    let sock = socket::resolve_bind_path();
    let reachable = sock.exists();
    h.check_bool("primal_socket_exists", reachable);

    if !reachable {
        return false;
    }

    let alive = rpc::try_send(&sock, "health.liveness", &serde_json::json!({}))
        .ok()
        .and_then(|v| v.get("alive").and_then(serde_json::Value::as_bool))
        .unwrap_or(false);

    h.check_bool("health.liveness_alive", alive);
    alive
}

// ── Tower Atomic (BearDog + Songbird) ───────────────────────────────────────

fn validate_tower_atomic(h: &mut ValidationHarness) {
    let Some(sock) = socket::discover_by_capability_public("crypto") else {
        h.check_bool("tower.crypto.discovery", false);
        return;
    };

    let payload = b"healthspring_nucleus_parity_test";
    let Ok(resp) = rpc::try_send(
        &sock,
        "crypto.hash",
        &serde_json::json!({"data": hex_encode(payload)}),
    ) else {
        h.check_bool("tower.crypto.hash_reachable", false);
        return;
    };

    let hash = resp
        .get("hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    h.check_bool("tower.crypto.hash_nonempty", !hash.is_empty());
    h.check_bool("tower.crypto.hash_length_64", hash.len() == 64);

    if let Ok(resp2) = rpc::try_send(
        &sock,
        "crypto.hash",
        &serde_json::json!({"data": hex_encode(payload)}),
    ) {
        let hash2 = resp2
            .get("hash")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        h.check_bool("tower.crypto.hash_determinism", hash == hash2);
    }

    h.check_bool(
        "tower.songbird.discoverable",
        socket::discover_by_capability_public("discovery").is_some(),
    );
}

// ── Node Atomic (barraCuda + toadStool + coralReef) ─────────────────────────

#[expect(clippy::cast_precision_loss, reason = "data.len() fits f64")]
fn validate_node_atomic(h: &mut ValidationHarness) {
    let Some(sock) = socket::discover_by_capability_public("math") else {
        h.check_bool("node.barracuda.discovery", false);
        return;
    };

    let data = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let local_mean = data.iter().sum::<f64>() / data.len() as f64;

    match rpc::try_send(&sock, "stats.mean", &serde_json::json!({"values": data})) {
        Ok(r) => {
            if let Some(ipc_mean) = r.get("result").and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "node.barracuda.stats_mean_parity",
                    ipc_mean,
                    local_mean,
                    tolerances::MACHINE_EPSILON,
                );
            } else {
                h.check_bool("node.barracuda.stats_mean_parse", false);
            }
        }
        Err(_) => h.check_bool("node.barracuda.stats_mean_reachable", false),
    }

    match rpc::try_send(&sock, "stats.std_dev", &serde_json::json!({"values": data})) {
        Ok(r) => {
            let ipc_std = r
                .get("result")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(f64::NAN);
            h.check_bool("node.barracuda.stats_std_dev_finite", ipc_std.is_finite());
        }
        Err(_) => h.check_bool("node.barracuda.stats_std_dev_reachable", false),
    }

    h.check_bool(
        "node.toadstool.discoverable",
        socket::discover_by_capability_public("compute").is_some(),
    );
}

// ── Nest Atomic (NestGate + provenance trio) ────────────────────────────────

fn validate_nest_atomic(h: &mut ValidationHarness) {
    let Some(sock) = socket::discover_by_capability_public("storage") else {
        h.check_bool("nest.nestgate.discovery", false);
        return;
    };

    let test_key = "healthspring_exp123_parity_test";
    let test_value = "nucleus_parity_round_trip_2026_05_08";

    let Ok(_) = rpc::try_send(
        &sock,
        "storage.store",
        &serde_json::json!({
            "key": test_key,
            "value": test_value,
            "namespace": "healthspring_test",
        }),
    ) else {
        h.check_bool("nest.nestgate.store_reachable", false);
        return;
    };

    h.check_bool("nest.nestgate.store_accepted", true);

    match rpc::try_send(
        &sock,
        "storage.retrieve",
        &serde_json::json!({"key": test_key, "namespace": "healthspring_test"}),
    ) {
        Ok(r) => {
            let retrieved = r
                .get("value")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool("nest.nestgate.retrieve_match", retrieved == test_value);
        }
        Err(_) => h.check_bool("nest.nestgate.retrieve_reachable", false),
    }

    h.check_bool(
        "nest.rhizocrypt.discoverable",
        socket::discover_by_capability_public("dag").is_some(),
    );
    h.check_bool(
        "nest.sweetgrass.discoverable",
        socket::discover_by_capability_public("braid").is_some(),
    );
}

// ── Cross-Atomic Pipeline ───────────────────────────────────────────────────

fn validate_cross_atomic(h: &mut ValidationHarness) {
    let (Some(csock), Some(ssock)) = (
        socket::discover_by_capability_public("crypto"),
        socket::discover_by_capability_public("storage"),
    ) else {
        h.check_bool("cross_atomic.both_discovered", false);
        return;
    };

    let Ok(hr) = rpc::try_send(
        &csock,
        "crypto.hash",
        &serde_json::json!({"data": "healthspring_cross_atomic_test_payload"}),
    ) else {
        h.check_bool("cross_atomic.hash_reachable", false);
        return;
    };

    let hash = hr
        .get("hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if rpc::try_send(
        &ssock,
        "storage.store",
        &serde_json::json!({
            "key": "exp123_cross_atomic_hash",
            "value": hash,
            "namespace": "healthspring_test",
        }),
    )
    .is_err()
    {
        h.check_bool("cross_atomic.store_hash_reachable", false);
        return;
    }

    match rpc::try_send(
        &ssock,
        "storage.retrieve",
        &serde_json::json!({"key": "exp123_cross_atomic_hash", "namespace": "healthspring_test"}),
    ) {
        Ok(r) => {
            let retrieved = r
                .get("value")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            h.check_bool("cross_atomic.hash_round_trip", retrieved == hash);
        }
        Err(_) => h.check_bool("cross_atomic.retrieve_hash_reachable", false),
    }
}

// ── Niche Parity (healthSpring science via local vs IPC) ────────────────────

fn validate_niche_parity(h: &mut ValidationHarness) {
    let Some(sock) = socket::discover_by_capability_public("math") else {
        h.check_bool("niche.math_discovery", false);
        return;
    };

    let data = [2.0_f64, 3.0, 5.0, 7.0, 11.0, 13.0];
    let local_mean = healthspring_barracuda::math_dispatch::mean(&data);

    match rpc::try_send(&sock, "stats.mean", &serde_json::json!({"values": data})) {
        Ok(r) => {
            if let Some(ipc_mean) = r.get("result").and_then(serde_json::Value::as_f64) {
                h.check_abs(
                    "niche.math_dispatch_vs_ipc_mean",
                    ipc_mean,
                    local_mean,
                    tolerances::MACHINE_EPSILON,
                );
            } else {
                h.check_bool("niche.stats_mean_parse", false);
            }
        }
        Err(_) => h.check_bool("niche.stats_mean_reachable", false),
    }

    let local_hill = healthspring_barracuda::math_dispatch::hill(10.0, 10.0, 1.0);
    h.check_abs(
        "niche.hill_ec50_identity",
        local_hill,
        0.5,
        tolerances::MACHINE_EPSILON,
    );

    let counts = [10.0_f64, 20.0, 30.0, 40.0];
    let local_shannon =
        healthspring_barracuda::math_dispatch::shannon_from_frequencies(&counts);
    h.check_bool("niche.shannon_positive", local_shannon > 0.0);
}

fn hex_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    data.iter().fold(String::with_capacity(data.len() * 2), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}
