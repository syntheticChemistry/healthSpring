// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "parity test assertions"
)]

//! Parity tests: library vs IPC vs `CompositionContext`.
//!
//! These tests validate that the same computations produce identical results
//! via direct library call, `BarraCudaClient` IPC, and `CompositionContext`.
//! They run in Tier 1 (structural) mode — no live primals required.

use healthspring_barracuda::math_dispatch;

#[test]
fn hill_structural_parity() {
    let result = math_dispatch::hill(10.0, 10.0, 1.0);
    assert!(
        (result - 0.5).abs() < 1e-15,
        "Hill(IC50, n=1) == 0.5, got {result}"
    );
}

#[test]
fn mean_structural_parity() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let result = math_dispatch::mean(&data);
    assert!(
        (result - 5.5).abs() < 1e-15,
        "mean([1..10]) == 5.5, got {result}"
    );
}

#[test]
fn std_dev_structural_parity() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let sd = math_dispatch::std_dev(&data).expect("valid std_dev");
    assert!(sd > 0.0, "std_dev positive, got {sd}");
    let expected = 3.027_650_354_097_492; // N-1 sample std dev
    assert!(
        (sd - expected).abs() < 1e-10,
        "std_dev parity: got {sd}, expected {expected}"
    );
}

#[test]
fn shannon_structural_parity() {
    let freqs = [0.25, 0.25, 0.25, 0.25];
    let result = math_dispatch::shannon_from_frequencies(&freqs);
    let expected = 4.0_f64.ln();
    assert!(
        (result - expected).abs() < 1e-14,
        "Shannon(uniform_4) == ln(4), got {result}"
    );
}

#[test]
fn simpson_structural_parity() {
    let freqs = [0.25, 0.25, 0.25, 0.25];
    let result = math_dispatch::simpson(&freqs);
    assert!(result > 0.7, "Simpson(uniform_4) > 0.7, got {result}");
}

#[test]
fn bray_curtis_structural_parity() {
    let identical = math_dispatch::bray_curtis(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]);
    assert!(
        identical.abs() < 1e-15,
        "Bray-Curtis(identical) == 0, got {identical}"
    );

    let different = math_dispatch::bray_curtis(&[10.0, 0.0, 0.0], &[0.0, 0.0, 10.0]);
    assert!(
        different > 0.0,
        "Bray-Curtis(different) > 0, got {different}"
    );
}
