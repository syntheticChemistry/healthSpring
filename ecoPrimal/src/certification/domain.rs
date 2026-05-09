// SPDX-License-Identifier: AGPL-3.0-or-later

//! Domain science validation — local analytical checks (Tier 1).

use primalspring::validation::ValidationResult;

use crate::math_dispatch;
use crate::tolerances;

/// Validate domain-specific science locally against analytical baselines.
///
/// Domain functions (Hill, Shannon, Simpson, etc.) are LOCAL compositions
/// of barraCuda primitives — they don't need IPC. The guideStone verifies
/// they produce correct results on this substrate.
pub fn validate_domain_science(v: &mut ValidationResult) {
    // Hill dose-response at IC50
    let hill_ic50 = math_dispatch::hill(10.0, 10.0, 1.0);
    v.check_bool(
        "Hill(x=IC50, n=1) == 0.5",
        (hill_ic50 - 0.5).abs() < tolerances::MACHINE_EPSILON_STRICT,
        &format!("got {hill_ic50}"),
    );

    // Hill monotonicity: hill(2K) > hill(K)
    let hill_2k = math_dispatch::hill(20.0, 10.0, 1.0);
    v.check_bool(
        "Hill monotonic: hill(2K) > hill(K)",
        hill_2k > hill_ic50,
        &format!("{hill_2k} > {hill_ic50}"),
    );

    // Shannon entropy: uniform distribution maximizes entropy
    let uniform_4 = math_dispatch::shannon_from_frequencies(&[0.25, 0.25, 0.25, 0.25]);
    let uniform_2 = math_dispatch::shannon_from_frequencies(&[0.5, 0.5]);
    v.check_bool(
        "Shannon: uniform(4) > uniform(2)",
        uniform_4 > uniform_2,
        &format!("{uniform_4} > {uniform_2}"),
    );

    // Simpson diversity: uniform(4) should give high diversity
    let simpson_uniform = math_dispatch::simpson(&[0.25, 0.25, 0.25, 0.25]);
    v.check_bool(
        "Simpson(uniform_4) > 0.7",
        simpson_uniform > 0.7,
        &format!("got {simpson_uniform}"),
    );

    // Bray-Curtis: identical communities = 0 dissimilarity
    let bc_identical = math_dispatch::bray_curtis(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]);
    v.check_bool(
        "Bray-Curtis(identical) == 0",
        bc_identical.abs() < tolerances::MACHINE_EPSILON,
        &format!("got {bc_identical}"),
    );

    // Bray-Curtis: maximally different > 0
    let bc_different = math_dispatch::bray_curtis(&[10.0, 0.0, 0.0], &[0.0, 0.0, 10.0]);
    v.check_bool(
        "Bray-Curtis(maximally_different) > 0",
        bc_different > 0.0,
        &format!("got {bc_different}"),
    );
}
