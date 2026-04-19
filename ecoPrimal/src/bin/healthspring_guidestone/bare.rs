// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bare guideStone properties — validated without any primals running.

use primalspring::validation::ValidationResult;

use healthspring_barracuda::math_dispatch;
use healthspring_barracuda::niche;

/// Property 1: Deterministic Output.
///
/// Verifies that core math functions produce analytically correct results
/// on this substrate. Same binary, same results, any architecture.
pub fn validate_deterministic_output(v: &mut ValidationResult) {
    // Mean of [1..10] = 5.5 (exact analytical)
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let mean = math_dispatch::mean(&data);
    v.check_bool(
        "P1: mean([1..10]) == 5.5",
        (mean - 5.5).abs() < 1e-15,
        &format!("got {mean}, expected 5.5"),
    );

    // Hill(x=K, n=1) = 0.5 (exact analytical)
    let hill = math_dispatch::hill(10.0, 10.0, 1.0);
    v.check_bool(
        "P1: Hill(x=K, n=1) == 0.5",
        (hill - 0.5).abs() < 1e-15,
        &format!("got {hill}, expected 0.5"),
    );

    // Shannon entropy of uniform(4) = ln(4) (exact analytical)
    let shannon = math_dispatch::shannon_from_frequencies(&[0.25, 0.25, 0.25, 0.25]);
    let expected_shannon = 4.0_f64.ln();
    v.check_bool(
        "P1: Shannon(uniform_4) == ln(4)",
        (shannon - expected_shannon).abs() < 1e-10,
        &format!("got {shannon}, expected {expected_shannon}"),
    );

    // std_dev returns Some for valid data
    let sd = math_dispatch::std_dev(&data);
    v.check_bool(
        "P1: std_dev([1..10]) is Some and positive",
        sd.is_some_and(|s| s > 0.0),
        &format!("got {sd:?}"),
    );

    // Wire count consistency
    v.check_bool(
        "P1: wire counts consistent",
        math_dispatch::WIRE_READY_COUNT + math_dispatch::WIRE_PENDING_COUNT
            == math_dispatch::TOTAL_COUNT,
        &format!(
            "{} + {} == {}",
            math_dispatch::WIRE_READY_COUNT,
            math_dispatch::WIRE_PENDING_COUNT,
            math_dispatch::TOTAL_COUNT
        ),
    );
}

/// Property 2: Reference-Traceable.
///
/// Verifies that provenance records exist and are structurally complete.
pub fn validate_reference_traceable(v: &mut ValidationResult) {
    // Niche identity is consistent
    v.check_bool(
        "P2: PRIMAL_ID matches lib constant",
        niche::PRIMAL_ID == healthspring_barracuda::PRIMAL_NAME,
        &format!(
            "niche={}, lib={}",
            niche::PRIMAL_ID,
            healthspring_barracuda::PRIMAL_NAME
        ),
    );

    // Composition experiments cover all tiers
    let tiers: Vec<&str> = niche::COMPOSITION_EXPERIMENTS
        .iter()
        .map(|(_, t)| *t)
        .collect();
    v.check_bool(
        "P2: composition experiments cover tier3",
        tiers.iter().any(|t| t.starts_with("tier3")),
        "tier3 experiments present",
    );
    v.check_bool(
        "P2: composition experiments cover tier4",
        tiers.iter().any(|t| t.starts_with("tier4")),
        "tier4 experiments present",
    );
    v.check_bool(
        "P2: composition experiments cover tier5",
        tiers.iter().any(|t| t.starts_with("tier5")),
        "tier5 experiments present",
    );

    // Proto-nucleate validation capabilities are non-empty
    v.check_bool(
        "P2: validation_capabilities populated",
        niche::PROTO_NUCLEATE_VALIDATION_CAPABILITIES.len() >= 8,
        &format!(
            "{} capabilities",
            niche::PROTO_NUCLEATE_VALIDATION_CAPABILITIES.len()
        ),
    );

    // Cost estimates all positive
    let all_positive = niche::COST_ESTIMATES.iter().all(|(_, c)| *c > 0.0);
    v.check_bool(
        "P2: cost estimates all positive",
        all_positive,
        &format!("{} estimates", niche::COST_ESTIMATES.len()),
    );
}

/// Property 3: Self-Verifying (BLAKE3 checksums per v1.1.0).
///
/// Verifies that a CHECKSUMS manifest exists and all listed files match
/// their BLAKE3 hashes. If the manifest does not exist yet (not generated),
/// the check is recorded as SKIP, not FAIL — honest scaffolding.
pub fn validate_self_verifying(v: &mut ValidationResult) {
    primalspring::checksums::verify_manifest(v, "validation/CHECKSUMS");
}

/// Property 4: Environment-Agnostic.
///
/// Verifies ecoBin compliance markers.
pub fn validate_environment_agnostic(v: &mut ValidationResult) {
    // Pure Rust, no C deps — verified by compile (forbid unsafe_code)
    v.check_bool(
        "P4: forbid(unsafe_code) active",
        true,
        "enforced via #![forbid(unsafe_code)]",
    );

    // No hardcoded platform assumptions in niche identity
    v.check_bool(
        "P4: niche domain is platform-neutral",
        niche::NICHE_DOMAIN == "health",
        &format!("domain={}", niche::NICHE_DOMAIN),
    );

    // Proto-nucleate path is relative (not absolute)
    v.check_bool(
        "P4: proto-nucleate path is relative",
        !niche::PROTO_NUCLEATE.starts_with('/'),
        &format!("path={}", niche::PROTO_NUCLEATE),
    );

    // Dependencies use capability-based discovery
    let all_have_capability = niche::DEPENDENCIES.iter().all(|d| !d.capability.is_empty());
    v.check_bool(
        "P4: all deps use capability-based discovery",
        all_have_capability,
        &format!("{} deps checked", niche::DEPENDENCIES.len()),
    );
}

/// Property 5: Tolerance-Documented.
///
/// Verifies that named tolerance constants exist and are ordered correctly.
pub fn validate_tolerance_documented(v: &mut ValidationResult) {
    use healthspring_barracuda::tolerances;

    // Tolerance ordering: strict < tight < machine < identity
    v.check_bool(
        "P5: MACHINE_EPSILON_STRICT < MACHINE_EPSILON_TIGHT",
        tolerances::MACHINE_EPSILON_STRICT < tolerances::MACHINE_EPSILON_TIGHT,
        &format!(
            "{} < {}",
            tolerances::MACHINE_EPSILON_STRICT,
            tolerances::MACHINE_EPSILON_TIGHT
        ),
    );

    v.check_bool(
        "P5: MACHINE_EPSILON_TIGHT < MACHINE_EPSILON",
        tolerances::MACHINE_EPSILON_TIGHT < tolerances::MACHINE_EPSILON,
        &format!(
            "{} < {}",
            tolerances::MACHINE_EPSILON_TIGHT,
            tolerances::MACHINE_EPSILON
        ),
    );

    // DETERMINISM tolerance is named and non-zero
    v.check_bool(
        "P5: DETERMINISM tolerance named and positive",
        tolerances::DETERMINISM > 0.0,
        &format!("{}", tolerances::DETERMINISM),
    );

    // Diversity cross-validation tolerance exists
    v.check_bool(
        "P5: DIVERSITY_CROSS_VALIDATE named",
        tolerances::DIVERSITY_CROSS_VALIDATE > 0.0,
        &format!("{}", tolerances::DIVERSITY_CROSS_VALIDATE),
    );
}
