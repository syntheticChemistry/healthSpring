// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests use unwrap/expect for concise assertions"
)]
//! Composition validation integration tests.
//!
//! Validates that the IPC dispatch layer — the composition surface that
//! biomeOS calls via JSON-RPC — faithfully reproduces direct Rust science
//! results. This is Tier 4 validation:
//!
//!   Python baseline → Rust validation → **IPC dispatch parity**
//!
//! Every test calls `dispatch_science(method, params)` and compares the
//! result to the corresponding direct Rust function call. Zero tolerance
//! for deterministic ops (DETERMINISM = 1e-12).

use healthspring_barracuda::ipc::dispatch::{dispatch_science, registered_capabilities};
use healthspring_barracuda::{microbiome, pkpd, tolerances};

fn assert_dispatch_f64(method: &str, params: &serde_json::Value, key: &str, expected: f64) {
    let result =
        dispatch_science(method, params).unwrap_or_else(|| panic!("{method} returned None"));
    let actual = result
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| panic!("{method} missing '{key}' in {result}"));
    assert!(
        (actual - expected).abs() <= tolerances::DETERMINISM,
        "{method}.{key}: got {actual}, want {expected}, diff {}",
        (actual - expected).abs()
    );
}

// ── PK/PD ────────────────────────────────────────────────────────────

#[test]
fn hill_dispatch_matches_direct() {
    for drug in pkpd::ALL_INHIBITORS {
        let direct =
            pkpd::hill_dose_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);
        assert_dispatch_f64(
            "science.pkpd.hill_dose_response",
            &serde_json::json!({
                "concentration": drug.ic50_jak1_nm,
                "ic50": drug.ic50_jak1_nm,
                "hill_n": drug.hill_n,
                "e_max": 1.0,
            }),
            "response",
            direct,
        );
    }
}

#[test]
fn iv_bolus_dispatch_matches_direct() {
    let direct = pkpd::pk_iv_bolus(100.0, 10.0, 6.93, 2.0);
    assert_dispatch_f64(
        "science.pkpd.one_compartment_pk",
        &serde_json::json!({"dose_mg": 100.0, "vd": 10.0, "half_life_hr": 6.93, "t": 2.0}),
        "concentration",
        direct,
    );
}

#[test]
fn auc_dispatch_matches_direct() {
    let times = vec![0.0, 1.0, 2.0, 4.0];
    let concs = vec![10.0, 8.0, 4.0, 1.0];
    let direct = pkpd::auc_trapezoidal(&times, &concs);
    assert_dispatch_f64(
        "science.pkpd.auc_trapezoidal",
        &serde_json::json!({"times": times, "concentrations": concs}),
        "auc",
        direct,
    );
}

#[test]
fn allometric_dispatch_matches_direct() {
    let direct = pkpd::allometric_scale(10.0, 70.0, 70.0, 0.75);
    assert_dispatch_f64(
        "science.pkpd.allometric_scale",
        &serde_json::json!({"param_animal": 10.0, "bw_animal": 70.0, "bw_human": 70.0}),
        "scaled_param",
        direct,
    );
}

#[test]
fn mm_dispatch_matches_direct() {
    let p = pkpd::MichaelisMentenParams {
        vmax: pkpd::PHENYTOIN_PARAMS.vmax,
        km: pkpd::PHENYTOIN_PARAMS.km,
        vd: pkpd::PHENYTOIN_PARAMS.vd,
    };
    let (_, concs) = pkpd::mm_pk_simulate(&p, 25.0, 72.0, 0.1);
    let direct_auc = pkpd::mm_auc(&concs, 0.1);
    assert_dispatch_f64(
        "science.pkpd.michaelis_menten_nonlinear",
        &serde_json::json!({"c0": 25.0, "duration_hr": 72.0, "dt": 0.1}),
        "auc",
        direct_auc,
    );
}

// ── Microbiome ───────────────────────────────────────────────────────

#[test]
fn shannon_dispatch_matches_direct() {
    let abundances = vec![0.4, 0.3, 0.2, 0.1];
    let direct = microbiome::shannon_index(&abundances);
    assert_dispatch_f64(
        "science.microbiome.shannon_index",
        &serde_json::json!({"abundances": abundances}),
        "shannon",
        direct,
    );
}

#[test]
fn simpson_dispatch_matches_direct() {
    let abundances = vec![0.25, 0.25, 0.25, 0.25];
    let direct = microbiome::simpson_index(&abundances);
    assert_dispatch_f64(
        "science.microbiome.simpson_index",
        &serde_json::json!({"abundances": abundances}),
        "simpson",
        direct,
    );
}

#[test]
fn chao1_dispatch_matches_direct() {
    let counts: Vec<u64> = vec![10, 5, 3, 1, 1, 1, 1, 1];
    let direct = microbiome::chao1(&counts);
    assert_dispatch_f64(
        "science.microbiome.chao1",
        &serde_json::json!({"counts": counts}),
        "chao1",
        direct,
    );
}

#[test]
fn colonization_dispatch_matches_direct() {
    let direct = microbiome::colonization_resistance(5.0);
    assert_dispatch_f64(
        "science.microbiome.colonization_resistance",
        &serde_json::json!({"xi": 5.0}),
        "resistance",
        direct,
    );
}

// ── Capability surface ───────────────────────────────────────────────

#[test]
fn all_registered_methods_dispatch() {
    for (method, _domain) in registered_capabilities() {
        let result = dispatch_science(method, &serde_json::json!({}));
        assert!(
            result.is_some(),
            "registered method {method} returned None from dispatch"
        );
    }
}

#[test]
fn all_domains_represented() {
    let caps = registered_capabilities();
    let domains: Vec<&str> = caps.iter().map(|(_, d)| *d).collect();
    for expected in &[
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
    ] {
        assert!(
            domains.contains(expected),
            "domain '{expected}' missing from registry"
        );
    }
}

#[test]
fn dispatch_determinism() {
    let params = serde_json::json!({
        "concentration": 15.0, "ic50": 10.0, "hill_n": 2.0, "e_max": 1.0,
    });
    let r1 = dispatch_science("science.pkpd.hill_dose_response", &params)
        .unwrap()
        .get("response")
        .unwrap()
        .as_f64()
        .unwrap();
    let r2 = dispatch_science("science.pkpd.hill_dose_response", &params)
        .unwrap()
        .get("response")
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        (r1 - r2).abs() <= tolerances::DETERMINISM,
        "dispatch non-determinism: {r1} != {r2}"
    );
}
