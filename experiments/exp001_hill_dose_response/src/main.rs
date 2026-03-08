// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! healthSpring Exp001 — Rust validation binary
//!
//! Cross-validates the Rust Hill dose-response implementation against the
//! Python control baseline (`control/pkpd/exp001_baseline.json`).

use healthspring_barracuda::pkpd::{self, ALL_INHIBITORS, compute_ec_values, hill_dose_response};

fn main() {
    let mut passed = 0_u32;
    let mut failed = 0_u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp001 [Rust]: Hill Dose-Response — Human JAK Inhibitors");
    println!("{}", "=".repeat(72));

    // Check 1–4: Hill at IC50 = 0.5 for each drug
    for drug in ALL_INHIBITORS {
        print!("\n--- Check: {} at IC50 → 50% --- ", drug.name);
        let r = hill_dose_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);
        if (r - 0.5).abs() < 1e-10 {
            println!("[PASS] response={r:.10}");
            passed += 1;
        } else {
            println!("[FAIL] response={r:.10}");
            failed += 1;
        }
    }

    // Check 5–8: Monotonicity
    let concs: Vec<f64> = (0..100)
        .map(|i| 10.0_f64.powf(-1.0 + 5.0 * f64::from(i) / 99.0))
        .collect();
    for drug in ALL_INHIBITORS {
        print!("\n--- Check: {} monotonicity --- ", drug.name);
        let responses = pkpd::hill_sweep(drug.ic50_jak1_nm, drug.hill_n, 1.0, &concs);
        let monotonic = responses.windows(2).all(|w| w[0] <= w[1] + 1e-15);
        if monotonic {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }
    }

    // Check 9: Potency ordering at 10 nM
    print!("\n--- Check: Potency ordering at 10 nM --- ");
    let r_bari = hill_dose_response(10.0, ALL_INHIBITORS[0].ic50_jak1_nm, 1.0, 1.0);
    let r_upa = hill_dose_response(10.0, ALL_INHIBITORS[1].ic50_jak1_nm, 1.0, 1.0);
    let r_ocla = hill_dose_response(10.0, ALL_INHIBITORS[3].ic50_jak1_nm, 1.0, 1.0);
    let r_abro = hill_dose_response(10.0, ALL_INHIBITORS[2].ic50_jak1_nm, 1.0, 1.0);
    if r_bari > r_upa && r_upa > r_ocla && r_ocla > r_abro {
        println!(
            "[PASS] bari({r_bari:.3}) > upa({r_upa:.3}) > ocla({r_ocla:.3}) > abro({r_abro:.3})"
        );
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 10–13: EC values ordered
    for drug in ALL_INHIBITORS {
        print!("\n--- Check: {} EC values --- ", drug.name);
        let ec = compute_ec_values(drug.ic50_jak1_nm, drug.hill_n);
        if ec.ec10 < ec.ec50 && ec.ec50 < ec.ec90 {
            println!(
                "[PASS] EC10={:.2} < EC50={:.2} < EC90={:.2}",
                ec.ec10, ec.ec50, ec.ec90
            );
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }
    }

    // Check 14: Cooperativity below IC50
    print!("\n--- Check: Hill n=2 steeper below IC50 --- ");
    let r_n1 = hill_dose_response(5.0, 10.0, 1.0, 1.0);
    let r_n2 = hill_dose_response(5.0, 10.0, 2.0, 1.0);
    if r_n2 < r_n1 {
        println!("[PASS] n=2 ({r_n2:.4}) < n=1 ({r_n1:.4})");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 15: Cooperativity above IC50
    print!("\n--- Check: Hill n=2 higher above IC50 --- ");
    let r_n1a = hill_dose_response(20.0, 10.0, 1.0, 1.0);
    let r_n2a = hill_dose_response(20.0, 10.0, 2.0, 1.0);
    if r_n2a > r_n1a {
        println!("[PASS] n=2 ({r_n2a:.4}) > n=1 ({r_n1a:.4})");
        passed += 1;
    } else {
        println!("[FAIL]");
        failed += 1;
    }

    // Check 16–19: Saturation at 100x IC50
    for drug in ALL_INHIBITORS {
        print!("\n--- Check: {} saturation at 100x --- ", drug.name);
        let conc = drug.ic50_jak1_nm * 100.0;
        let r = hill_dose_response(conc, drug.ic50_jak1_nm, drug.hill_n, 1.0);
        if r > 0.99 {
            println!("[PASS] {r:.6} at {conc:.0} nM");
            passed += 1;
        } else {
            println!("[FAIL] {r:.6}");
            failed += 1;
        }
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));

    std::process::exit(i32::from(failed > 0));
}
