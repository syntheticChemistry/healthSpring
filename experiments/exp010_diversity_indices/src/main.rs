// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! healthSpring Exp010 — Microbiome Diversity Indices Validation
//!
//! Cross-validates α-diversity metrics (Shannon, Simpson, Pielou, Chao1)
//! against the Python control baseline (`control/microbiome/exp010_baseline.json`).

use healthspring_barracuda::microbiome::{self, communities};

const TOL: f64 = 1e-8;
const W_SCALE: f64 = 10.0;

// Chao1 count data (from Python baseline)
const COUNTS_HEALTHY: [u64; 15] = [250, 200, 150, 120, 100, 80, 50, 30, 10, 5, 3, 2, 1, 1, 1];
const COUNTS_DEPLETED: [u64; 12] = [850, 50, 30, 20, 15, 10, 5, 5, 3, 2, 1, 1];

fn main() {
    let mut passed = 0_u32;
    let mut failed = 0_u32;

    let healthy = &communities::HEALTHY_GUT[..];
    let dysbiotic = &communities::DYSBIOTIC_GUT[..];
    let cdiff = &communities::CDIFF_COLONIZED[..];
    let even = &communities::PERFECTLY_EVEN[..];
    let mono = &communities::MONOCULTURE[..];

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp010 — Microbiome Diversity Indices");
    println!("{}", "=".repeat(72));

    // Check 1: Shannon — perfectly even = ln(S)
    println!("\n--- Check 1: Shannon perfectly even = ln(S) ---");
    let h_even = microbiome::shannon_index(even);
    #[expect(clippy::cast_precision_loss, reason = "species count (10) fits f64")]
    let expected_h = (even.len() as f64).ln();
    if (h_even - expected_h).abs() < TOL {
        println!(
            "  [PASS] H' = {h_even:.10} = ln({}) = {expected_h:.10}",
            even.len()
        );
        passed += 1;
    } else {
        println!("  [FAIL] H' = {h_even:.10}");
        failed += 1;
    }

    // Check 2: Shannon — monoculture = 0
    println!("\n--- Check 2: Shannon monoculture = 0 ---");
    let h_mono = microbiome::shannon_index(mono);
    if h_mono.abs() < TOL {
        println!("  [PASS] H' = {h_mono:.10}");
        passed += 1;
    } else {
        println!("  [FAIL] H' = {h_mono:.10}");
        failed += 1;
    }

    // Check 3: Shannon — ordering: even > healthy > cdiff > dysbiotic > mono
    println!("\n--- Check 3: Shannon ordering ---");
    let h_healthy = microbiome::shannon_index(healthy);
    let h_dysbiotic = microbiome::shannon_index(dysbiotic);
    let h_cdiff = microbiome::shannon_index(cdiff);
    let ordered =
        h_even > h_healthy && h_healthy > h_cdiff && h_cdiff > h_dysbiotic && h_dysbiotic > h_mono;
    if ordered {
        println!(
            "  [PASS] even({h_even:.3}) > healthy({h_healthy:.3}) > cdiff({h_cdiff:.3}) > dysbiotic({h_dysbiotic:.3}) > mono({h_mono:.3})"
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 4: Simpson — perfectly even
    println!("\n--- Check 4: Simpson perfectly even ---");
    let d_even = microbiome::simpson_index(even);
    #[expect(clippy::cast_precision_loss, reason = "species count (10) fits f64")]
    let expected_d = (even.len() as f64).mul_add(-0.01, 1.0);
    if (d_even - expected_d).abs() < TOL {
        println!("  [PASS] D = {d_even:.10}");
        passed += 1;
    } else {
        println!("  [FAIL] D = {d_even:.10}");
        failed += 1;
    }

    // Check 5: Simpson — monoculture = 0
    println!("\n--- Check 5: Simpson monoculture = 0 ---");
    let d_mono = microbiome::simpson_index(mono);
    if d_mono.abs() < TOL {
        println!("  [PASS] D = {d_mono:.10}");
        passed += 1;
    } else {
        println!("  [FAIL] D = {d_mono:.10}");
        failed += 1;
    }

    // Check 6: Simpson — healthy > dysbiotic
    println!("\n--- Check 6: Simpson ordering ---");
    let d_healthy = microbiome::simpson_index(healthy);
    let d_dysbiotic = microbiome::simpson_index(dysbiotic);
    if d_healthy > d_dysbiotic {
        println!("  [PASS] healthy {d_healthy:.4} > dysbiotic {d_dysbiotic:.4}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 7: Inverse Simpson — even = S
    println!("\n--- Check 7: Inverse Simpson even = S ---");
    let inv_even = microbiome::inverse_simpson(even);
    if (inv_even - 10.0).abs() < TOL {
        println!("  [PASS] 1/D = {inv_even:.10} = S=10");
        passed += 1;
    } else {
        println!("  [FAIL] 1/D = {inv_even:.10}");
        failed += 1;
    }

    // Check 8: Pielou — even = 1.0
    println!("\n--- Check 8: Pielou evenness even = 1.0 ---");
    let j_even = microbiome::pielou_evenness(even);
    if (j_even - 1.0).abs() < TOL {
        println!("  [PASS] J = {j_even:.10}");
        passed += 1;
    } else {
        println!("  [FAIL] J = {j_even:.10}");
        failed += 1;
    }

    // Check 9: Pielou — ordering
    println!("\n--- Check 9: Pielou ordering ---");
    let j_healthy = microbiome::pielou_evenness(healthy);
    let j_dysbiotic = microbiome::pielou_evenness(dysbiotic);
    let j_cdiff = microbiome::pielou_evenness(cdiff);
    if j_healthy > j_cdiff && j_cdiff > j_dysbiotic {
        println!(
            "  [PASS] healthy({j_healthy:.3}) > cdiff({j_cdiff:.3}) > dysbiotic({j_dysbiotic:.3})"
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 10: Chao1 ≥ S_obs
    println!("\n--- Check 10: Chao1 ≥ observed richness ---");
    let chao1_h = microbiome::chao1(&COUNTS_HEALTHY);
    let chao1_d = microbiome::chao1(&COUNTS_DEPLETED);
    let s_obs_h = COUNTS_HEALTHY.len();
    let s_obs_d = COUNTS_DEPLETED.len();
    #[expect(clippy::cast_precision_loss, reason = "S_obs small (≤20)")]
    if chao1_h >= s_obs_h as f64 && chao1_d >= s_obs_d as f64 {
        println!(
            "  [PASS] Healthy: Chao1={chao1_h:.1} ≥ S_obs={s_obs_h}; Depleted: Chao1={chao1_d:.1} ≥ S_obs={s_obs_d}"
        );
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 11: Chao1 healthy > Chao1 depleted
    println!("\n--- Check 11: Chao1 healthy > depleted ---");
    if chao1_h > chao1_d {
        println!("  [PASS] {chao1_h:.1} > {chao1_d:.1}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 12: Anderson disorder mapping
    println!("\n--- Check 12: Pielou → Anderson disorder W ---");
    let w_healthy = microbiome::evenness_to_disorder(j_healthy, W_SCALE);
    let w_dysbiotic = microbiome::evenness_to_disorder(j_dysbiotic, W_SCALE);
    if w_healthy > w_dysbiotic {
        println!("  [PASS] W(healthy)={w_healthy:.3} > W(dysbiotic)={w_dysbiotic:.3}");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 13: All indices in valid range
    println!("\n--- Check 13: All indices in valid ranges ---");
    let all_valid = [healthy, dysbiotic, cdiff, even, mono].iter().all(|ab| {
        let h = microbiome::shannon_index(ab);
        let d = microbiome::simpson_index(ab);
        let j = microbiome::pielou_evenness(ab);
        h >= -TOL
            && (-TOL..=1.0 + TOL).contains(&d)
            && (ab.len() <= 1 || (-TOL..=1.0 + TOL).contains(&j))
    });
    if all_valid {
        println!("  [PASS] H' ≥ 0, 0 ≤ D ≤ 1, 0 ≤ J ≤ 1 for all communities");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 14: Abundance normalization
    println!("\n--- Check 14: Abundance normalization ---");
    let all_normalized = [healthy, dysbiotic, cdiff, even, mono]
        .iter()
        .all(|ab| (ab.iter().sum::<f64>() - 1.0).abs() < 1e-6);
    if all_normalized {
        println!("  [PASS] All communities sum to 1.0");
        passed += 1;
    } else {
        println!("  [FAIL]");
        failed += 1;
    }

    // Check 15–19: Cross-validate against exp010_baseline.json
    println!("\n--- Check 15: Baseline JSON cross-validation ---");
    let baseline_str = include_str!("../../../control/microbiome/exp010_baseline.json");
    let baseline: serde_json::Value = match serde_json::from_str(baseline_str) {
        Ok(b) => b,
        Err(e) => {
            println!("  [FAIL] Could not parse baseline: {e}");
            failed += 1;
            let total = passed + failed;
            println!("\n{}", "=".repeat(72));
            println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
            println!("{}", "=".repeat(72));
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

    if baseline_ok {
        println!("  [PASS] All values match exp010_baseline.json");
        passed += 1;
    } else {
        println!("  [FAIL] Baseline mismatch");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    std::process::exit(i32::from(failed > 0));
}

#[allow(clippy::too_many_arguments)]
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
    let shannon_ok = (f("shannon_even") - h_even).abs() < TOL
        && (f("shannon_mono") - h_mono).abs() < TOL
        && shannon_all.is_some_and(|o| {
            (o.get("healthy_gut")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0)
                - h_healthy)
                .abs()
                < TOL
                && (o
                    .get("dysbiotic_gut")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0)
                    - h_dysbiotic)
                    .abs()
                    < TOL
                && (o
                    .get("cdiff_colonized")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0)
                    - h_cdiff)
                    .abs()
                    < TOL
        });
    let simpson_ok = (f("simpson_even") - d_even).abs() < TOL
        && (f("simpson_mono") - d_mono).abs() < TOL
        && (f("simpson_healthy") - d_healthy).abs() < TOL
        && (f("simpson_dysbiotic") - d_dysbiotic).abs() < TOL;
    let inv_ok = (f("inv_simpson_even") - inv_even).abs() < TOL;
    let pielou_ok = (f("pielou_even") - j_even).abs() < TOL
        && (f("pielou_healthy") - j_healthy).abs() < TOL
        && (f("pielou_dysbiotic") - j_dysbiotic).abs() < TOL
        && (f("pielou_cdiff") - j_cdiff).abs() < TOL;
    let chao1_ok =
        (f("chao1_healthy") - chao1_h).abs() < TOL && (f("chao1_depleted") - chao1_d).abs() < TOL;
    let anderson_ok = (f("anderson_w_healthy") - w_healthy).abs() < TOL
        && (f("anderson_w_dysbiotic") - w_dysbiotic).abs() < TOL;

    shannon_ok && simpson_ok && inv_ok && pielou_ok && chao1_ok && anderson_ok
}
