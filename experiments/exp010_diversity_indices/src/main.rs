// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! healthSpring Exp010 — Microbiome Diversity Indices Validation
//!
//! Cross-validates α-diversity metrics (Shannon, Simpson, Pielou, Chao1)
//! against the Python control baseline (`control/microbiome/exp010_baseline.json`).

use healthspring_barracuda::microbiome::{self, communities};
use healthspring_barracuda::provenance;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;
const W_SCALE: f64 = 10.0;

// Chao1 count data (from Python baseline)
const COUNTS_HEALTHY: [u64; 15] = [250, 200, 150, 120, 100, 80, 50, 30, 10, 5, 3, 2, 1, 1, 1];
const COUNTS_DEPLETED: [u64; 12] = [850, 50, 30, 20, 15, 10, 5, 5, 3, 2, 1, 1];

fn main() {
    let mut h = ValidationHarness::new("Exp010 Diversity Indices");

    let healthy = &communities::HEALTHY_GUT[..];
    let dysbiotic = &communities::DYSBIOTIC_GUT[..];
    let cdiff = &communities::CDIFF_COLONIZED[..];
    let even = &communities::PERFECTLY_EVEN[..];
    let mono = &communities::MONOCULTURE[..];

    let h_even = microbiome::shannon_index(even);
    #[expect(clippy::cast_precision_loss, reason = "species count (10) fits f64")]
    let expected_h = (even.len() as f64).ln();
    h.check_abs(
        "Shannon even = ln(S)",
        h_even,
        expected_h,
        tolerances::DIVERSITY_CROSS_VALIDATE,
    );

    let h_mono = microbiome::shannon_index(mono);
    h.check_abs(
        "Shannon monoculture",
        h_mono,
        0.0,
        tolerances::DIVERSITY_CROSS_VALIDATE,
    );

    let h_healthy = microbiome::shannon_index(healthy);
    let h_dysbiotic = microbiome::shannon_index(dysbiotic);
    let h_cdiff = microbiome::shannon_index(cdiff);
    let ordered =
        h_even > h_healthy && h_healthy > h_cdiff && h_cdiff > h_dysbiotic && h_dysbiotic > h_mono;
    h.check_bool("Shannon ordering", ordered);

    let d_even = microbiome::simpson_index(even);
    #[expect(clippy::cast_precision_loss, reason = "species count (10) fits f64")]
    let expected_d = (even.len() as f64).mul_add(-0.01, 1.0);
    h.check_abs(
        "Simpson even",
        d_even,
        expected_d,
        tolerances::DIVERSITY_CROSS_VALIDATE,
    );

    let d_mono = microbiome::simpson_index(mono);
    h.check_abs(
        "Simpson monoculture",
        d_mono,
        0.0,
        tolerances::DIVERSITY_CROSS_VALIDATE,
    );

    let d_healthy = microbiome::simpson_index(healthy);
    let d_dysbiotic = microbiome::simpson_index(dysbiotic);
    h.check_bool("Simpson ordering", d_healthy > d_dysbiotic);

    let inv_even = microbiome::inverse_simpson(even);
    h.check_abs(
        "Inverse Simpson even",
        inv_even,
        10.0,
        tolerances::DIVERSITY_CROSS_VALIDATE,
    );

    let j_even = microbiome::pielou_evenness(even);
    h.check_abs(
        "Pielou even",
        j_even,
        1.0,
        tolerances::DIVERSITY_CROSS_VALIDATE,
    );

    let j_healthy = microbiome::pielou_evenness(healthy);
    let j_dysbiotic = microbiome::pielou_evenness(dysbiotic);
    let j_cdiff = microbiome::pielou_evenness(cdiff);
    h.check_bool(
        "Pielou ordering",
        j_healthy > j_cdiff && j_cdiff > j_dysbiotic,
    );

    let chao1_h = microbiome::chao1(&COUNTS_HEALTHY);
    let chao1_d = microbiome::chao1(&COUNTS_DEPLETED);
    let s_obs_h = COUNTS_HEALTHY.len();
    let s_obs_d = COUNTS_DEPLETED.len();
    #[expect(clippy::cast_precision_loss, reason = "S_obs small (≤20)")]
    h.check_bool(
        "Chao1 ≥ S_obs",
        chao1_h >= s_obs_h as f64 && chao1_d >= s_obs_d as f64,
    );

    h.check_bool("Chao1 healthy > depleted", chao1_h > chao1_d);

    let w_healthy = microbiome::evenness_to_disorder(j_healthy, W_SCALE);
    let w_dysbiotic = microbiome::evenness_to_disorder(j_dysbiotic, W_SCALE);
    h.check_bool("Anderson W ordering", w_healthy > w_dysbiotic);

    let all_valid = [healthy, dysbiotic, cdiff, even, mono].iter().all(|ab| {
        let sh = microbiome::shannon_index(ab);
        let d = microbiome::simpson_index(ab);
        let j = microbiome::pielou_evenness(ab);
        sh >= -tolerances::DIVERSITY_CROSS_VALIDATE
            && (-tolerances::DIVERSITY_CROSS_VALIDATE..=1.0 + tolerances::DIVERSITY_CROSS_VALIDATE)
                .contains(&d)
            && (ab.len() <= 1
                || (-tolerances::DIVERSITY_CROSS_VALIDATE
                    ..=1.0 + tolerances::DIVERSITY_CROSS_VALIDATE)
                    .contains(&j))
    });
    h.check_bool("All indices valid ranges", all_valid);

    let all_normalized = [healthy, dysbiotic, cdiff, even, mono]
        .iter()
        .all(|ab| (ab.iter().sum::<f64>() - 1.0).abs() < tolerances::ABUNDANCE_NORMALIZATION);
    h.check_bool("Abundance normalization", all_normalized);

    // Cross-validate against exp010_baseline.json
    let baseline_str = include_str!("../../../control/microbiome/exp010_baseline.json");
    if let Some(prov) = provenance::load_provenance(baseline_str) {
        provenance::log_provenance(&prov);
    }
    if let Some(prov) = provenance::load_provenance(baseline_str) {
        provenance::log_provenance(&prov);
    }
    let baseline: serde_json::Value = match serde_json::from_str(baseline_str) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("FATAL: Could not parse baseline: {e}");
            std::process::exit(1);
        }
    };

    let baseline_ok = check_baseline(
        &baseline,
        h_even,
        h_mono,
        h_healthy,
        h_dysbiotic,
        h_cdiff,
        d_even,
        d_mono,
        d_healthy,
        d_dysbiotic,
        inv_even,
        j_even,
        j_healthy,
        j_dysbiotic,
        j_cdiff,
        chao1_h,
        chao1_d,
        w_healthy,
        w_dysbiotic,
    );

    h.check_bool("Baseline JSON cross-validation", baseline_ok);

    h.exit();
}

#[expect(
    clippy::too_many_arguments,
    reason = "validation helper checks all diversity metrics in one call"
)]
fn check_baseline(
    b: &serde_json::Value,
    h_even: f64,
    h_mono: f64,
    h_healthy: f64,
    h_dysbiotic: f64,
    h_cdiff: f64,
    d_even: f64,
    d_mono: f64,
    d_healthy: f64,
    d_dysbiotic: f64,
    inv_even: f64,
    j_even: f64,
    j_healthy: f64,
    j_dysbiotic: f64,
    j_cdiff: f64,
    chao1_h: f64,
    chao1_d: f64,
    w_healthy: f64,
    w_dysbiotic: f64,
) -> bool {
    let f = |key: &str| {
        b.get(key)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN)
    };
    let shannon_all = b.get("shannon_all").and_then(serde_json::Value::as_object);
    let shannon_ok = (f("shannon_even") - h_even).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("shannon_mono") - h_mono).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && shannon_all.is_some_and(|o| {
            (o.get("healthy_gut")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0)
                - h_healthy)
                .abs()
                < tolerances::DIVERSITY_CROSS_VALIDATE
                && (o
                    .get("dysbiotic_gut")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0)
                    - h_dysbiotic)
                    .abs()
                    < tolerances::DIVERSITY_CROSS_VALIDATE
                && (o
                    .get("cdiff_colonized")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0)
                    - h_cdiff)
                    .abs()
                    < tolerances::DIVERSITY_CROSS_VALIDATE
        });
    let simpson_ok = (f("simpson_even") - d_even).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("simpson_mono") - d_mono).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("simpson_healthy") - d_healthy).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("simpson_dysbiotic") - d_dysbiotic).abs() < tolerances::DIVERSITY_CROSS_VALIDATE;
    let inv_ok = (f("inv_simpson_even") - inv_even).abs() < tolerances::DIVERSITY_CROSS_VALIDATE;
    let pielou_ok = (f("pielou_even") - j_even).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("pielou_healthy") - j_healthy).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("pielou_dysbiotic") - j_dysbiotic).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("pielou_cdiff") - j_cdiff).abs() < tolerances::DIVERSITY_CROSS_VALIDATE;
    let chao1_ok = (f("chao1_healthy") - chao1_h).abs() < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("chao1_depleted") - chao1_d).abs() < tolerances::DIVERSITY_CROSS_VALIDATE;
    let anderson_ok = (f("anderson_w_healthy") - w_healthy).abs()
        < tolerances::DIVERSITY_CROSS_VALIDATE
        && (f("anderson_w_dysbiotic") - w_dysbiotic).abs() < tolerances::DIVERSITY_CROSS_VALIDATE;

    shannon_ok && simpson_ok && inv_ok && pielou_ok && chao1_ok && anderson_ok
}
