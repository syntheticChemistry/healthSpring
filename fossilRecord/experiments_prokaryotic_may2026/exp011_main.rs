// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp011 validation: Anderson Localization in Gut Lattice
//!
//! Cross-validates `healthspring_barracuda::microbiome` Anderson lattice
//! functions against the Python control (`exp011_anderson_gut_lattice.py`).

use healthspring_barracuda::microbiome;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

const L: usize = 50;
const T_HOP: f64 = 1.0;

fn main() {
    let mut harness = ValidationHarness::new("Exp011 Anderson Gut Lattice");

    let baseline_str = include_str!("../../../control/microbiome/exp011_baseline.json");
    let baseline: serde_json::Value = match serde_json::from_str(baseline_str) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to parse exp011_baseline.json: {e}");
            std::process::exit(1);
        }
    };

    let w_healthy_py = baseline
        .get("pielou_w_healthy")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| {
            eprintln!("Missing or invalid pielou_w_healthy in baseline JSON");
            std::process::exit(1);
        });
    let w_dysbiotic_py = baseline
        .get("pielou_w_dysbiotic")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| {
            eprintln!("Missing or invalid pielou_w_dysbiotic in baseline JSON");
            std::process::exit(1);
        });

    if let Some(prov) = baseline
        .get("_provenance")
        .and_then(serde_json::Value::as_object)
    {
        let date = prov
            .get("date")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("?");
        let git = prov
            .get("git_commit")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("?");
        println!("Baseline provenance: date={date}, git={git}");
    }

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp011 — Rust CPU Validation: Anderson Gut Lattice");
    println!("  L={L}");
    println!("{}", "=".repeat(72));

    // Build Hamiltonian with known disorder
    #[expect(clippy::cast_precision_loss, reason = "i < 50")]
    let disorder: Vec<f64> = (0..L).map(|i| (i as f64 - 25.0) * 0.2).collect();
    let h = microbiome::anderson_hamiltonian_1d(&disorder, T_HOP);

    // Check 1: Matrix size
    harness.check_exact("Matrix size", h.len() as u64, (L * L) as u64);

    // Check 2: Symmetric
    let mut symmetric = true;
    for i in 0..L {
        for j in 0..L {
            if (h[i * L + j] - h[j * L + i]).abs() > tolerances::ANDERSON_IDENTITY {
                symmetric = false;
            }
        }
    }
    harness.check_bool("Symmetric", symmetric);

    // Check 3: Diagonal = disorder
    let diag_ok =
        (0..L).all(|i| (h[i * L + i] - disorder[i]).abs() < tolerances::ANDERSON_IDENTITY);
    harness.check_bool("Diagonal = disorder", diag_ok);

    // Check 4: Off-diagonal hopping
    let hop_ok =
        (0..L - 1).all(|i| (h[i * L + (i + 1)] - T_HOP).abs() < tolerances::ANDERSON_IDENTITY);
    harness.check_bool("Nearest-neighbor hopping", hop_ok);

    // Check 5: No long-range hopping
    let mut no_lr = true;
    for i in 0..L {
        for j in 0..L {
            if i != j && i.abs_diff(j) > 1 && h[i * L + j].abs() > tolerances::ANDERSON_IDENTITY {
                no_lr = false;
            }
        }
    }
    harness.check_bool("No long-range hopping", no_lr);

    // Check 6: IPR of uniform state = 1/L
    #[expect(clippy::cast_precision_loss, reason = "L = 50")]
    let l_f64 = L as f64;
    let val = 1.0 / l_f64.sqrt();
    let uniform: Vec<f64> = vec![val; L];
    let ipr = microbiome::inverse_participation_ratio(&uniform);
    let expected = 1.0 / l_f64;
    harness.check_abs("IPR(uniform)", ipr, expected, tolerances::MACHINE_EPSILON);

    // Check 7: IPR of delta state = 1.0
    let mut delta = vec![0.0; L];
    delta[L / 2] = 1.0;
    let ipr_d = microbiome::inverse_participation_ratio(&delta);
    harness.check_abs("IPR(delta)", ipr_d, 1.0, tolerances::ANDERSON_IDENTITY);

    // Check 8: ξ = 1/IPR
    let xi = microbiome::localization_length_from_ipr(0.25);
    harness.check_abs("ξ(0.25)", xi, 4.0, tolerances::ANDERSON_IDENTITY);

    // Check 9: ξ(0) = inf
    let xi_0 = microbiome::localization_length_from_ipr(0.0);
    harness.check_bool("ξ(0) = ∞", xi_0.is_infinite());

    // Check 10: Level spacing ratio with few values
    let r = microbiome::level_spacing_ratio(&[1.0, 2.0]);
    harness.check_abs("Level spacing r (few)", r, 0.0, tolerances::MACHINE_EPSILON);

    // Check 11: Uniform spacing → r ≈ 1
    let uniform_eigs: Vec<f64> = (0..50).map(f64::from).collect();
    let r_u = microbiome::level_spacing_ratio(&uniform_eigs);
    harness.check_abs(
        "Uniform spacing r",
        r_u,
        1.0,
        tolerances::LEVEL_SPACING_RATIO,
    );

    // Check 12: CR(ξ=2) > CR(ξ=50)
    let cr_short = microbiome::colonization_resistance(2.0);
    let cr_long = microbiome::colonization_resistance(50.0);
    harness.check_bool("CR ordering", cr_short > cr_long);

    // Check 13: Pielou → disorder mapping (values from baseline JSON)
    let w_h = microbiome::evenness_to_disorder(0.863, 10.0);
    let w_d = microbiome::evenness_to_disorder(0.303, 10.0);
    harness.check_bool(
        "Pielou → W",
        w_h > w_d
            && (w_h - w_healthy_py).abs() < tolerances::W_CROSS_VALIDATE
            && (w_d - w_dysbiotic_py).abs() < tolerances::W_CROSS_VALIDATE,
    );

    // Check 14: W=0 lattice is clean (all zeros on diagonal)
    let disorder_zero = vec![0.0; L];
    let h_clean = microbiome::anderson_hamiltonian_1d(&disorder_zero, T_HOP);
    let diag_zero = (0..L).all(|i| h_clean[i * L + i].abs() < tolerances::ANDERSON_IDENTITY);
    harness.check_bool("W=0 clean lattice", diag_zero);

    harness.exit();
}
