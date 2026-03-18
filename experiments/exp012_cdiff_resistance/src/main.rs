// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
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
use healthspring_barracuda::validation::ValidationHarness;

const W_SCALE: f64 = 10.0;

fn main() {
    let mut h = ValidationHarness::new("Exp012 C. diff Resistance");

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

    // Check 1: Pielou ordering
    h.check_bool("Pielou ordering", j_e > j_h && j_h > j_c && j_c > j_d);

    // Check 2: All Pielou in [0, 1]
    h.check_bool(
        "All Pielou in [0, 1]",
        [j_h, j_d, j_c, j_e]
            .iter()
            .all(|&j| (0.0..=1.0 + tolerances::MACHINE_EPSILON).contains(&j)),
    );

    // Check 3: W mapping
    let w_h = microbiome::evenness_to_disorder(j_h, W_SCALE);
    let w_d = microbiome::evenness_to_disorder(j_d, W_SCALE);
    let w_c = microbiome::evenness_to_disorder(j_c, W_SCALE);
    h.check_bool("W ordering", w_h > w_c && w_c > w_d);

    // Check 4: Shannon ordering
    let shannon_healthy = microbiome::shannon_index(healthy);
    let shannon_dysbiotic = microbiome::shannon_index(dysbiotic);
    let shannon_cdiff = microbiome::shannon_index(cdiff);
    h.check_bool(
        "Shannon ordering",
        shannon_healthy > shannon_cdiff && shannon_cdiff > shannon_dysbiotic,
    );

    // Check 5: Simpson ordering
    let simpson_healthy = microbiome::simpson_index(healthy);
    let simpson_dysbiotic = microbiome::simpson_index(dysbiotic);
    h.check_bool("Simpson ordering", simpson_healthy > simpson_dysbiotic);

    // Check 6: IPR of uniform vs localized
    let l: usize = 50;
    #[expect(clippy::cast_precision_loss, reason = "l = 50")]
    let val = 1.0 / (l as f64).sqrt();
    let psi_ext = vec![val; l];
    let mut psi_loc = vec![0.0; l];
    psi_loc[25] = 1.0;
    let ipr_ext = microbiome::inverse_participation_ratio(&psi_ext);
    let ipr_loc = microbiome::inverse_participation_ratio(&psi_loc);
    h.check_bool("IPR physics", ipr_loc > ipr_ext);

    // Check 7: CR ordering (from ξ)
    let cr_confined = microbiome::colonization_resistance(3.0);
    let cr_extended = microbiome::colonization_resistance(50.0);
    h.check_bool("CR from ξ", cr_confined > cr_extended);

    // Check 8: Anderson Hamiltonian builds correctly
    let disorder: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.5).collect();
    let ham = microbiome::anderson_hamiltonian_1d(&disorder, 1.0);
    let sym = (0..20).all(|i| {
        (0..20).all(|j| (ham[i * 20 + j] - ham[j * 20 + i]).abs() < tolerances::ANDERSON_IDENTITY)
    });
    h.check_bool("Hamiltonian structure", sym && ham.len() == 400);

    // Check 9: W values match Python baseline (from exp012_baseline.json)
    h.check_abs("W healthy", w_h, w_healthy_py, tolerances::W_CROSS_VALIDATE);
    h.check_abs(
        "W dysbiotic",
        w_d,
        w_dysbiotic_py,
        tolerances::W_CROSS_VALIDATE,
    );

    // Check 10: Inverse Simpson
    let inv_h = microbiome::inverse_simpson(healthy);
    let inv_d = microbiome::inverse_simpson(dysbiotic);
    h.check_bool("Inverse Simpson", inv_h > inv_d && inv_h > 1.0);

    h.exit();
}
