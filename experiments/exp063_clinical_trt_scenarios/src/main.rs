// SPDX-License-Identifier: AGPL-3.0-or-later
//! healthSpring Exp063 — Clinical TRT Scenario Validation
//!
//! Validates that patient-parameterized TRT scenarios produce correct
//! structure, meaningful data ranges, and protocol-appropriate results
//! for multiple patient archetypes.

use healthspring_barracuda::visualization::clinical::{
    PatientTrtProfile, TrtProtocol, trt_clinical_scenario,
};
use healthspring_barracuda::visualization::DataChannel;

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp063: Clinical TRT Scenarios (Rust)");
    println!("{}", "=".repeat(72));

    // Patient archetypes
    let patients = vec![
        {
            let mut p = PatientTrtProfile::new("Young Low-T", 35.0, 180.0, 250.0, TrtProtocol::ImWeekly);
            p.gut_diversity = Some(0.85);
            p.hba1c = Some(5.8);
            p.sdnn_ms = Some(45.0);
            p
        },
        {
            let mut p = PatientTrtProfile::new("Middle-Aged Obese", 52.0, 280.0, 220.0, TrtProtocol::Pellet);
            p.gut_diversity = Some(0.40);
            p.hba1c = Some(7.8);
            p.sdnn_ms = Some(28.0);
            p
        },
        {
            let mut p = PatientTrtProfile::new("Senior Lean", 68.0, 170.0, 310.0, TrtProtocol::ImBiweekly);
            p.gut_diversity = Some(0.70);
            p.sdnn_ms = Some(32.0);
            p
        },
        PatientTrtProfile::new("Minimal Input", 45.0, 200.0, 280.0, TrtProtocol::Pellet),
    ];

    for patient in &patients {
        println!("\n--- Patient: {} ---", patient.name);
        let (scenario, edges) = trt_clinical_scenario(patient);

        // Check 1: 8 nodes
        print!("  8 nodes: ");
        if scenario.ecosystem.primals.len() == 8 {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL] got {}", scenario.ecosystem.primals.len());
            failed += 1;
        }

        // Check 2: 8 edges
        print!("  8 edges: ");
        if edges.len() == 8 {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL] got {}", edges.len());
            failed += 1;
        }

        // Check 3: every node has data channels
        print!("  all nodes have data: ");
        let all_have_data = scenario
            .ecosystem
            .primals
            .iter()
            .all(|n| !n.data_channels.is_empty());
        if all_have_data {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }

        // Check 4: every node has clinical ranges
        print!("  all nodes have ranges: ");
        let all_have_ranges = scenario
            .ecosystem
            .primals
            .iter()
            .all(|n| !n.clinical_ranges.is_empty());
        if all_have_ranges {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }

        // Check 5: scenario name contains patient name
        print!("  name personalized: ");
        if scenario.name.contains(&patient.name) {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL] name = {}", scenario.name);
            failed += 1;
        }

        // Check 6: assessment node baseline T matches patient
        print!("  baseline T correct: ");
        let assess = scenario.ecosystem.primals.iter().find(|n| n.id == "assessment").unwrap();
        let baseline_gauge = assess.data_channels.iter().find_map(|ch| {
            if let DataChannel::Gauge { id, value, .. } = ch {
                if id == "baseline_t" { return Some(*value); }
            }
            None
        });
        if let Some(val) = baseline_gauge {
            if (val - patient.baseline_t_ng_dl).abs() < 0.01 {
                println!("[PASS] {val:.0} ng/dL");
                passed += 1;
            } else {
                println!("[FAIL] got {val}, expected {}", patient.baseline_t_ng_dl);
                failed += 1;
            }
        } else {
            println!("[FAIL] no baseline_t gauge");
            failed += 1;
        }

        // Check 7: protocol node name matches protocol type
        print!("  protocol matches: ");
        let prot = scenario.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();
        let matches = match patient.protocol {
            TrtProtocol::ImWeekly => prot.name.contains("Weekly"),
            TrtProtocol::ImBiweekly => prot.name.contains("Biweekly"),
            TrtProtocol::Pellet => prot.name.contains("Pellet"),
        };
        if matches {
            println!("[PASS] {}", prot.name);
            passed += 1;
        } else {
            println!("[FAIL] {}", prot.name);
            failed += 1;
        }

        // Check 8: population distribution has 100 values
        print!("  population n=100: ");
        let pop = scenario.ecosystem.primals.iter().find(|n| n.id == "population").unwrap();
        let has_100 = pop.data_channels.iter().any(|ch| {
            if let DataChannel::Distribution { values, .. } = ch {
                values.len() == 100
            } else {
                false
            }
        });
        if has_100 {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }

        // Check 9: JSON roundtrip
        print!("  JSON valid: ");
        let json = healthspring_barracuda::visualization::clinical::trt_clinical_json(patient);
        match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(v) => {
                if v["ecosystem"]["primals"].is_array() && v["edges"].is_array() {
                    println!("[PASS] ({} bytes)", json.len());
                    passed += 1;
                } else {
                    println!("[FAIL] missing structure");
                    failed += 1;
                }
            }
            Err(e) => {
                println!("[FAIL] {e}");
                failed += 1;
            }
        }

        // Check 10: all timeseries have non-empty x/y of equal length
        print!("  timeseries valid: ");
        let ts_ok = scenario.ecosystem.primals.iter().all(|n| {
            n.data_channels.iter().all(|ch| {
                if let DataChannel::TimeSeries { x_values, y_values, .. } = ch {
                    !x_values.is_empty() && x_values.len() == y_values.len()
                } else {
                    true
                }
            })
        });
        if ts_ok {
            println!("[PASS]");
            passed += 1;
        } else {
            println!("[FAIL]");
            failed += 1;
        }
    }

    // Cross-patient checks
    println!("\n--- Cross-Patient Validation ---");

    // Check: heavier patient gets larger pellet dose
    print!("  pellet dose scales: ");
    let heavy = PatientTrtProfile::new("Heavy", 50.0, 300.0, 280.0, TrtProtocol::Pellet);
    let light = PatientTrtProfile::new("Light", 50.0, 150.0, 280.0, TrtProtocol::Pellet);
    let (sh, _) = trt_clinical_scenario(&heavy);
    let (sl, _) = trt_clinical_scenario(&light);
    let ph = sh.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();
    let pl = sl.ecosystem.primals.iter().find(|n| n.id == "protocol").unwrap();
    if ph.name.contains("3000") && pl.name.contains("1500") {
        println!("[PASS] 300lb→3000mg, 150lb→1500mg");
        passed += 1;
    } else {
        println!("[FAIL] heavy={}, light={}", ph.name, pl.name);
        failed += 1;
    }

    // Check: low baseline T → critical assessment
    print!("  low T flags critical: ");
    let low_t = PatientTrtProfile::new("LowT", 60.0, 200.0, 180.0, TrtProtocol::ImWeekly);
    let (s_low, _) = trt_clinical_scenario(&low_t);
    let a_low = s_low.ecosystem.primals.iter().find(|n| n.id == "assessment").unwrap();
    if a_low.status == "critical" && a_low.health <= 50 {
        println!("[PASS]");
        passed += 1;
    } else {
        println!("[FAIL] status={}, health={}", a_low.status, a_low.health);
        failed += 1;
    }

    // Check: high vs low gut diversity → different response predictions
    print!("  gut diversity modulates: ");
    let high_gut = {
        let mut p = PatientTrtProfile::new("HG", 50.0, 200.0, 280.0, TrtProtocol::Pellet);
        p.gut_diversity = Some(0.95);
        p
    };
    let low_gut = {
        let mut p = PatientTrtProfile::new("LG", 50.0, 200.0, 280.0, TrtProtocol::Pellet);
        p.gut_diversity = Some(0.20);
        p
    };
    let (s_hg, _) = trt_clinical_scenario(&high_gut);
    let (s_lg, _) = trt_clinical_scenario(&low_gut);
    let gh = s_hg.ecosystem.primals.iter().find(|n| n.id == "gut_health").unwrap();
    let gl = s_lg.ecosystem.primals.iter().find(|n| n.id == "gut_health").unwrap();
    let get_resp = |n: &healthspring_barracuda::visualization::ScenarioNode| -> f64 {
        n.data_channels.iter().find_map(|ch| {
            if let DataChannel::Gauge { id, value, .. } = ch {
                if id == "patient_response" { return Some(*value); }
            }
            None
        }).unwrap_or(0.0)
    };
    let r_h = get_resp(gh);
    let r_l = get_resp(gl);
    if r_h > r_l {
        println!("[PASS] high={r_h:.1}kg > low={r_l:.1}kg");
        passed += 1;
    } else {
        println!("[FAIL] high={r_h:.1}, low={r_l:.1}");
        failed += 1;
    }

    let total = passed + failed;
    println!("\n{}", "=".repeat(72));
    println!("TOTAL: {passed}/{total} PASS, {failed}/{total} FAIL");
    println!("{}", "=".repeat(72));
    std::process::exit(i32::from(failed > 0));
}
