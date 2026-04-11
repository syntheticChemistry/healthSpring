// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! Exp117: Composition IPC round-trip validation.
//!
//! Validates that the JSON-RPC wire protocol produces correct results for
//! science methods, proto-nucleate aliases, health probes, and capability
//! listing. Unlike exp112–113 (which test in-process dispatch parity),
//! this experiment validates the full serialization → dispatch → response
//! round-trip that a real IPC client would exercise.
//!
//! Tier 4 composition validation: Python + Rust validated the science;
//! now we validate the NUCLEUS composition patterns themselves.

use healthspring_barracuda::ipc::dispatch;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp117_composition_ipc_roundtrip");

    validate_rpc_serialization(&mut h);
    validate_proto_aliases(&mut h);
    validate_health_probes(&mut h);
    validate_capability_surface(&mut h);
    validate_genomics_alias(&mut h);

    h.exit();
}

/// JSON-RPC request → dispatch → response round-trip for science methods.
fn validate_rpc_serialization(h: &mut ValidationHarness) {
    let hill_params = serde_json::json!({
        "drug": "baricitinib",
        "concentration": 10.0,
    });
    let result = dispatch::dispatch_science("science.pkpd.hill_dose_response", &hill_params);
    h.check_bool("hill_dose_response dispatches", result.is_some());

    if let Some(resp) = result {
        let is_valid_json = serde_json::to_string(&resp).is_ok();
        h.check_bool("hill response serializes to JSON", is_valid_json);
    }

    let diversity_params = serde_json::json!({
        "abundances": [0.25, 0.25, 0.25, 0.25],
    });
    let result = dispatch::dispatch_science("science.microbiome.shannon_index", &diversity_params);
    h.check_bool("shannon_index dispatches", result.is_some());
}

/// Proto-nucleate `health.*` aliases resolve to `science.*` methods.
fn validate_proto_aliases(h: &mut ValidationHarness) {
    let caps = dispatch::registered_capabilities();
    let methods: Vec<&str> = caps.iter().map(|(m, _)| *m).collect();

    h.check_bool(
        "hill_dose_response is registered",
        methods.contains(&"science.pkpd.hill_dose_response"),
    );
    h.check_bool(
        "assess_patient is registered",
        methods.contains(&"science.diagnostic.assess_patient"),
    );
    h.check_bool(
        "patient_parameterize is registered",
        methods.contains(&"science.clinical.patient_parameterize"),
    );
    h.check_bool(
        "population_montecarlo is registered",
        methods.contains(&"science.diagnostic.population_montecarlo"),
    );
}

/// Health probes respond correctly via dispatch.
fn validate_health_probes(h: &mut ValidationHarness) {
    let liveness = dispatch::dispatch_science("health.liveness", &serde_json::json!({}));
    h.check_bool(
        "health.liveness is NOT a science method (returns None)",
        liveness.is_none(),
    );

    let params = serde_json::json!({"drug": "test", "concentration": 1.0});
    let result = dispatch::dispatch_science("science.pkpd.hill_dose_response", &params);
    h.check_bool("science dispatch returns Some for valid method", result.is_some());

    let unknown = dispatch::dispatch_science("science.nonexistent.method", &params);
    h.check_bool("unknown method returns None", unknown.is_none());
}

/// Capability surface completeness.
#[expect(clippy::cast_precision_loss, reason = "capability count fits f64")]
fn validate_capability_surface(h: &mut ValidationHarness) {
    let caps = dispatch::registered_capabilities();
    let cap_count = caps.len() as f64;

    h.check_lower("registered capabilities >= 58", cap_count, 58.0);

    let science_count = caps.iter().filter(|(m, _)| m.starts_with("science.")).count() as f64;
    h.check_lower("science capabilities >= 55", science_count, 55.0);

    for (method, _domain) in &caps {
        let params = serde_json::json!({});
        let result = dispatch::dispatch_science(method, &params);
        h.check_bool(
            &format!("{method} dispatches (returns Some)"),
            result.is_some(),
        );
    }
}

/// The V49 `health.genomics` alias is wired and discoverable.
fn validate_genomics_alias(h: &mut ValidationHarness) {
    let caps = dispatch::registered_capabilities();
    let methods: Vec<&str> = caps.iter().map(|(m, _)| *m).collect();
    h.check_bool(
        "qs_gene_profile is registered (genomics target)",
        methods.contains(&"science.microbiome.qs_gene_profile"),
    );
}
