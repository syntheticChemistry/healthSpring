// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp012 validation: C. diff Colonization Resistance Score
//!
//! Cross-validates microbiome diversity → Anderson → CR pipeline
//! against the Python control (`exp012_cdiff_resistance.py`).

use healthspring_barracuda::microbiome::{self, communities};
use healthspring_barracuda::tolerances;

const W_SCALE: f64 = 10.0;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    let baseline_str = include_str!("../../../control/microbiome/exp012_baseline.json");
    let baseline: serde_json::Value = match serde_json::from_str(baseline_str) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to parse exp012_baseline.json: {e}");
            std::process::exit(1);
        }
    };

    let w_healthy_py = baseline
        .get("scores")
        .and_then(|s| s.get("healthy"))
        .and_then(|h| h.get("disorder_W"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| {
            eprintln!("Missing or invalid scores.healthy.disorder_W in baseline JSON");
            std::process::exit(1);
        });
    let w_dysbiotic_py = baseline
        .get("scores")
        .and_then(|s| s.get("dysbiotic"))
        .and_then(|d| d.get("disorder_W"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| {
            eprintln!("Missing or invalid scores.dysbiotic.disorder_W in baseline JSON");
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
    println!("healthSpring Exp012 — Rust CPU Validation: C. diff Resistance");
    println!("{}", "=".repeat(72));

    let healthy = &communities::HEALTHY_GUT[..];
    let dysbiotic = &communities::DYSBIOTIC_GUT[..];
    let cdiff = &communities::CDIFF_COLONIZED[..];
    let even = &communities::PERFECTLY_EVEN[..];

    let j_h = microbiome::pielou_evenness(healthy);
    let j_d = microbiome::pielou_evenness(dysbiotic);
    let j_c = microbiome::pielou_evenness(cdiff);
    let j_e = microbiome::pielou_evenness(even);

    println!("\n  Pielou: healthy={j_h:.4}  dysbiotic={j_d:.4}  cdiff={j_c:.4}  even={j_e:.4}");

    // Check 1: Pielou ordering
    println!("\n--- Check 1: Pielou ordering ---");
    if j_e > j_h && j_h > j_c && j_c > j_d {
        println!("  [PASS] even > healthy > cdiff > dysbiotic");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 2: All Pielou in [0, 1]
    println!("\n--- Check 2: All Pielou in [0, 1] ---");
    if [j_h, j_d, j_c, j_e]
        .iter()
        .all(|&j| (0.0..=1.0 + tolerances::MACHINE_EPSILON).contains(&j))
    {
        println!("  [PASS]");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 3: W mapping
    println!("\n--- Check 3: W ordering ---");
    let w_h = microbiome::evenness_to_disorder(j_h, W_SCALE);
    let w_d = microbiome::evenness_to_disorder(j_d, W_SCALE);
    let w_c = microbiome::evenness_to_disorder(j_c, W_SCALE);
    if w_h > w_c && w_c > w_d {
        println!("  [PASS] W: healthy({w_h:.2}) > cdiff({w_c:.2}) > dysbiotic({w_d:.2})");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 4: Shannon ordering
    println!("\n--- Check 4: Shannon ordering ---");
    let shannon_healthy = microbiome::shannon_index(healthy);
    let shannon_dysbiotic = microbiome::shannon_index(dysbiotic);
    let shannon_cdiff = microbiome::shannon_index(cdiff);
    if shannon_healthy > shannon_cdiff && shannon_cdiff > shannon_dysbiotic {
        println!(
            "  [PASS] H': healthy({shannon_healthy:.4}) > cdiff({shannon_cdiff:.4}) > dysbiotic({shannon_dysbiotic:.4})"
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 5: Simpson ordering
    println!("\n--- Check 5: Simpson ordering ---");
    let simpson_healthy = microbiome::simpson_index(healthy);
    let simpson_dysbiotic = microbiome::simpson_index(dysbiotic);
    if simpson_healthy > simpson_dysbiotic {
        println!("  [PASS] D: healthy({simpson_healthy:.4}) > dysbiotic({simpson_dysbiotic:.4})");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 6: IPR of uniform vs localized
    println!("\n--- Check 6: IPR physics ---");
    let l: usize = 50;
    #[expect(clippy::cast_precision_loss, reason = "l = 50")]
    let val = 1.0 / (l as f64).sqrt();
    let psi_ext = vec![val; l];
    let mut psi_loc = vec![0.0; l];
    psi_loc[25] = 1.0;
    let ipr_ext = microbiome::inverse_participation_ratio(&psi_ext);
    let ipr_loc = microbiome::inverse_participation_ratio(&psi_loc);
    if ipr_loc > ipr_ext {
        println!("  [PASS] IPR(localized)={ipr_loc:.4} > IPR(extended)={ipr_ext:.6}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 7: CR ordering (from ξ)
    println!("\n--- Check 7: CR from ξ ---");
    let cr_confined = microbiome::colonization_resistance(3.0);
    let cr_extended = microbiome::colonization_resistance(50.0);
    if cr_confined > cr_extended {
        println!("  [PASS] CR(ξ=3)={cr_confined:.4} > CR(ξ=50)={cr_extended:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 8: Anderson Hamiltonian builds correctly
    println!("\n--- Check 8: Hamiltonian structure ---");
    let disorder: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.5).collect();
    let h = microbiome::anderson_hamiltonian_1d(&disorder, 1.0);
    let sym = (0..20).all(|i| {
        (0..20).all(|j| (h[i * 20 + j] - h[j * 20 + i]).abs() < tolerances::ANDERSON_IDENTITY)
    });
    if sym && h.len() == 400 {
        println!("  [PASS] 20×20 symmetric");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 9: W values match Python baseline (from exp012_baseline.json)
    println!("\n--- Check 9: W values match Python ---");
    if (w_h - w_healthy_py).abs() < tolerances::W_CROSS_VALIDATE
        && (w_d - w_dysbiotic_py).abs() < tolerances::W_CROSS_VALIDATE
    {
        println!(
            "  [PASS] W(healthy)={w_h:.2}≈{w_healthy_py}, W(dysbiotic)={w_d:.2}≈{w_dysbiotic_py}"
        );
        passed += 1;
    } else {
        println!("  [FAIL] W_h={w_h:.4}, W_d={w_d:.4}");
        failed += 1;
    }

    // Check 10: Inverse Simpson
    println!("\n--- Check 10: Inverse Simpson ---");
    let inv_h = microbiome::inverse_simpson(healthy);
    let inv_d = microbiome::inverse_simpson(dysbiotic);
    if inv_h > inv_d && inv_h > 1.0 {
        println!("  [PASS] InvSimpson: healthy={inv_h:.2} > dysbiotic={inv_d:.2}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    if failed > 0 {
        std::process::exit(1);
    }
}
