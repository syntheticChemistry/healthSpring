// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp078: Antibiotic Perturbation of Gut Microbiome
//!
//! Models Shannon diversity decline during antibiotic exposure and
//! incomplete recovery afterward. Based on Dethlefsen & Relman 2011
//! (ciprofloxacin causes 30-50% diversity decline with partial recovery).
//!
//! Reference: Dethlefsen & Relman 2011, PNAS 108: 4554-4561.

use healthspring_barracuda::microbiome;

macro_rules! check {
    ($p:expr, $f:expr, $name:expr, $cond:expr) => {
        if $cond {
            $p += 1;
            println!("  [PASS] {}", $name);
        } else {
            $f += 1;
            println!("  [FAIL] {}", $name);
        }
    };
}

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp078 — Antibiotic Perturbation Recovery");
    println!("{}", "=".repeat(72));

    let h0 = 2.2;
    let depth = 0.5;
    let k_decline = 0.3;
    let k_recovery = 0.1;
    let treatment_days = 7.0;
    let total_days = 42.0;
    let dt = 0.1;

    let trajectory = microbiome::antibiotic_perturbation(
        h0,
        depth,
        k_decline,
        k_recovery,
        treatment_days,
        total_days,
        dt,
    );

    // Check 1: Starts at baseline
    println!("\n--- Check 1: Starts at baseline H0 ---");
    let h_start = trajectory[0].1;
    check!(
        passed,
        failed,
        format!("H'(0) = {h_start:.4} ≈ {h0}"),
        (h_start - h0).abs() < 0.01
    );

    // Check 2: Declines during treatment
    println!("\n--- Check 2: Decline during treatment ---");
    let h_at_treatment_end = trajectory
        .iter()
        .rfind(|(t, _)| (*t - treatment_days).abs() < dt * 0.6)
        .map_or(h0, |&(_, h)| h);
    check!(
        passed,
        failed,
        format!("H'(7d) = {h_at_treatment_end:.4} < H0 = {h0}"),
        h_at_treatment_end < h0
    );

    // Check 3: Nadir below baseline
    println!("\n--- Check 3: Nadir < H0 ---");
    let nadir = trajectory
        .iter()
        .map(|&(_, h)| h)
        .fold(f64::INFINITY, f64::min);
    check!(
        passed,
        failed,
        format!("nadir = {nadir:.4} < {h0}"),
        nadir < h0
    );

    // Check 4: Nadir occurs during or at end of treatment
    println!("\n--- Check 4: Nadir timing ---");
    let nadir_time = trajectory
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map_or(0.0, |&(t, _)| t);
    check!(
        passed,
        failed,
        format!("nadir at t={nadir_time:.1} ≤ treatment end + 2d"),
        nadir_time <= treatment_days + 2.0
    );

    // Check 5: Recovery after treatment
    println!("\n--- Check 5: Recovery occurs ---");
    let h_final = trajectory.last().unwrap().1;
    check!(
        passed,
        failed,
        format!("H'(42d) = {h_final:.4} > nadir = {nadir:.4}"),
        h_final > nadir
    );

    // Check 6: Incomplete recovery (Dethlefsen finding)
    println!("\n--- Check 6: Incomplete recovery ---");
    check!(
        passed,
        failed,
        format!("H'(42d) = {h_final:.4} < H0 = {h0} (incomplete)"),
        h_final < h0
    );

    // Check 7: Depth matches expected fraction
    println!("\n--- Check 7: Decline depth ---");
    let actual_depth = (h0 - nadir) / h0;
    check!(
        passed,
        failed,
        format!("actual depth = {actual_depth:.3} in [0.3, 0.6]"),
        actual_depth > 0.3 && actual_depth < 0.6
    );

    // Check 8: All H' values positive
    println!("\n--- Check 8: All H' > 0 ---");
    let all_positive = trajectory.iter().all(|&(_, h)| h > 0.0);
    check!(passed, failed, "all Shannon values positive", all_positive);

    // Check 9: Monotone decline during treatment (early phase)
    println!("\n--- Check 9: Decline monotone (first 5 days) ---");
    let early_treatment: Vec<f64> = trajectory
        .iter()
        .filter(|(t, _)| *t <= 5.0)
        .map(|&(_, h)| h)
        .collect();
    let mono_decline = early_treatment.windows(2).all(|w| w[1] <= w[0] + 1e-10);
    check!(
        passed,
        failed,
        "monotone decline first 5 days",
        mono_decline
    );

    // Check 10: Recovery monotone after nadir
    println!("\n--- Check 10: Recovery monotone after day 10 ---");
    let recovery_phase: Vec<f64> = trajectory
        .iter()
        .filter(|(t, _)| *t >= 10.0)
        .map(|&(_, h)| h)
        .collect();
    let mono_recovery = recovery_phase.windows(2).all(|w| w[1] >= w[0] - 1e-10);
    check!(
        passed,
        failed,
        "monotone recovery after day 10",
        mono_recovery
    );

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("Exp078 Antibiotic Perturbation: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
