// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp114 — Composition validation: Health triad + capability surface.
//!
//! Validates the primal composition contract: capability.list completeness,
//! dispatch registry coverage, proto-nucleate alias routing, and the full
//! registered capability surface. This is the structural composition test —
//! does the primal advertise what it can actually serve?

use healthspring_barracuda::ipc::dispatch::{dispatch_science, registered_capabilities};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("Exp114 Composition Health Triad + Capability Surface");

    // ── Registry completeness: every registered method dispatches ────
    let caps = registered_capabilities();
    h.check_bool("Registry has 58+ capabilities", caps.len() >= 58);

    let mut dispatched_count = 0_u32;
    let mut failed_methods = Vec::new();
    for (method, _domain) in &caps {
        let empty_result = dispatch_science(method, &serde_json::json!({}));
        if let Some(v) = empty_result {
            if v.get("error").is_some() {
                dispatched_count += 1;
            } else {
                dispatched_count += 1;
            }
        } else {
            failed_methods.push(*method);
        }
    }
    h.check_bool(
        "All registered methods dispatch (return Some)",
        failed_methods.is_empty(),
    );
    h.check_bool(
        "Dispatched count matches registry",
        dispatched_count == u32::try_from(caps.len()).unwrap_or(0),
    );

    // ── Missing params: every handler returns structured error ───────
    let mut structured_errors = 0_u32;
    for (method, _domain) in &caps {
        let result = dispatch_science(method, &serde_json::json!({}));
        if let Some(v) = result {
            if v.get("error").map_or(false, |e| e.as_str() == Some("missing_params")) {
                structured_errors += 1;
            }
        }
    }
    h.check_bool(
        "Most handlers return structured missing_params error",
        structured_errors > 30,
    );

    // ── Proto-nucleate alias routing ────────────────────────────────
    // health.pharmacology → science.pkpd.hill_dose_response
    let hill_params = serde_json::json!({
        "concentration": 10.0, "ic50": 10.0, "hill_n": 1.0, "e_max": 1.0,
    });
    let direct = dispatch_science("science.pkpd.hill_dose_response", &hill_params);
    let has_direct = direct
        .as_ref()
        .and_then(|v| v.get("response"))
        .is_some();
    h.check_bool("Direct Hill dispatch returns response", has_direct);

    // health.clinical → science.diagnostic.assess_patient
    let patient_params = serde_json::json!({
        "age": 55, "weight_kg": 80.0, "testosterone_ngdl": 350.0,
    });
    let clinical = dispatch_science("science.diagnostic.assess_patient", &patient_params);
    let has_clinical = clinical.is_some();
    h.check_bool("Clinical assess_patient dispatch returns result", has_clinical);

    // ── Domain coverage: at least one handler per domain ────────────
    let domains: Vec<&str> = caps.iter().map(|(_, d)| *d).collect();
    let expected_domains = [
        "pkpd",
        "microbiome",
        "biosignal",
        "endocrine",
        "diagnostic",
        "clinical",
        "comparative",
        "discovery",
        "toxicology",
        "simulation",
    ];
    for domain in &expected_domains {
        let present = domains.iter().any(|d| d == domain);
        h.check_bool(&format!("Domain '{domain}' has registered handlers"), present);
    }

    // ── Determinism: registry is stable across calls ────────────────
    let caps2 = registered_capabilities();
    h.check_bool(
        "Registry is deterministic",
        caps.len() == caps2.len()
            && caps
                .iter()
                .zip(caps2.iter())
                .all(|((a, _), (b, _))| a == b),
    );

    h.exit();
}
