// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp115 — Composition validation: Proto-nucleate alignment.
//!
//! Validates that healthSpring's actual runtime behavior matches its
//! proto-nucleate graph declaration. Checks socket resolution, discovery
//! helpers, tower atomic structure, and IPC infrastructure readiness
//! without requiring a running primal server.

use healthspring_barracuda::ipc::socket;
use healthspring_barracuda::ipc::dispatch::registered_capabilities;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("Exp115 Composition Proto-Nucleate Alignment");

    // ── Socket resolution follows biomeos convention ────────────────
    let bind_path = socket::resolve_bind_path();
    let bind_str = bind_path.to_string_lossy();
    h.check_bool(
        "Bind path contains primal name",
        bind_str.contains("healthspring"),
    );
    h.check_bool("Bind path ends with .sock", bind_str.ends_with(".sock"));

    let socket_dir = socket::resolve_socket_dir();
    let dir_str = socket_dir.to_string_lossy();
    h.check_bool(
        "Socket dir contains biomeos",
        dir_str.contains("biomeos"),
    );

    // ── Orchestrator socket resolution ──────────────────────────────
    let orch = socket::orchestrator_socket();
    let orch_str = orch.to_string_lossy();
    h.check_bool(
        "Orchestrator socket contains biomeOS",
        orch_str.contains("biomeOS") || orch_str.contains("biomeos"),
    );

    // ── Capability surface matches proto-nucleate expectations ──────
    let caps = registered_capabilities();
    let methods: Vec<&str> = caps.iter().map(|(m, _)| *m).collect();

    // Proto-nucleate declares health domain with these core capabilities
    let proto_expected = [
        "science.pkpd.hill_dose_response",
        "science.microbiome.shannon_index",
        "science.biosignal.pan_tompkins",
        "science.endocrine.testosterone_pk",
        "science.diagnostic.assess_patient",
        "science.clinical.trt_scenario",
        "science.comparative.cross_species_pk",
        "science.discovery.matrix_score",
        "science.toxicology.biphasic_dose_response",
        "science.simulation.mechanistic_fitness",
    ];
    for expected in &proto_expected {
        h.check_bool(
            &format!("Proto-nucleate capability registered: {expected}"),
            methods.contains(expected),
        );
    }

    // ── Discovery helpers return valid paths (without live primals) ──
    // These should return None (no primals running) but not panic
    let compute = socket::discover_compute_primal();
    h.check_bool(
        "Compute discovery returns None without live primals",
        compute.is_none(),
    );
    let data = socket::discover_data_primal();
    h.check_bool(
        "Data discovery returns None without live primals",
        data.is_none(),
    );
    let inference = socket::discover_inference_primal();
    h.check_bool(
        "Inference discovery returns None without live primals",
        inference.is_none(),
    );

    // ── All primals discovery returns empty without live network ─────
    let all_primals = socket::discover_all_primals();
    h.check_bool(
        "All-primals discovery is empty or self-only",
        all_primals.len() <= 1,
    );

    // ── Primal constants ────────────────────────────────────────────
    h.check_bool(
        "PRIMAL_NAME is healthspring",
        healthspring_barracuda::PRIMAL_NAME == "healthspring",
    );
    h.check_bool(
        "PRIMAL_DOMAIN is health",
        healthspring_barracuda::PRIMAL_DOMAIN == "health",
    );

    h.exit();
}
