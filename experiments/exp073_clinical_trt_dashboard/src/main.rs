// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Live TRT clinical dashboard: builds a patient-parameterized TRT scenario
//! and streams PK curve updates, population comparison, and outcome gauges
//! to petalTongue via `StreamSession`.
//!
//! Falls back to stdout + JSON when petalTongue is not running.

use healthspring_barracuda::endocrine;
use healthspring_barracuda::visualization::DataChannel;
use healthspring_barracuda::visualization::clinical::{
    PatientTrtProfile, TrtProtocol, trt_clinical_json, trt_clinical_scenario,
};
use healthspring_barracuda::visualization::ipc_push::PushError;
use healthspring_barracuda::visualization::stream::StreamSession;

const SESSION_ID: &str = "healthspring-trt-live";

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "clinical dashboard orchestrator — linear simulation and streaming sequence"
)]
fn main() {
    println!("=== exp073: Clinical TRT Live Dashboard ===\n");

    let mut patient = PatientTrtProfile::new(
        "Live Dashboard Patient",
        52.0,
        230.0,
        210.0,
        TrtProtocol::ImWeekly,
    );
    patient.hba1c = Some(7.4);
    patient.gut_diversity = Some(0.50);
    patient.sdnn_ms = Some(30.0);

    let (scenario, edges) = trt_clinical_scenario(&patient);
    println!(
        "Built TRT scenario: {} ({} nodes, {} edges)",
        scenario.name,
        scenario.ecosystem.primals.len(),
        edges.len(),
    );

    let mut session = match StreamSession::discover(SESSION_ID) {
        Ok(s) => {
            println!("[trt-live] petalTongue found — streaming\n");
            Some(s)
        }
        Err(PushError::NotFound(_)) => {
            println!("[trt-live] petalTongue not running — stdout + JSON mode\n");
            None
        }
        Err(e) => {
            eprintln!("[trt-live] discovery error: {e}");
            None
        }
    };

    if let Some(ref mut sess) = session {
        if let Err(e) =
            sess.push_render_with_domain("TRT Clinical Dashboard", &scenario, "clinical")
        {
            eprintln!("[trt-live] initial render failed: {e}");
        } else {
            println!("[trt-live] Pushed initial clinical scenario");
        }
    }

    // Simulate PK curve evolution: stream weekly trough updates
    let weight_kg = patient.weight_lb * 0.453_592;
    let vd = endocrine::testosterone_cypionate::VD_L * (weight_kg / 70.0);
    let ka = endocrine::testosterone_cypionate::K_A_IM;
    let ke = endocrine::testosterone_cypionate::K_E;
    let dose = endocrine::testosterone_cypionate::DOSE_WEEKLY_MG;
    let f_im = endocrine::testosterone_cypionate::F_IM;

    let n_weeks = 12;
    let mut troughs = Vec::with_capacity(n_weeks);

    println!("Streaming {n_weeks} weeks of PK trough data:\n");
    for week in 1..=n_weeks {
        let t_days = week as f64 * 7.0;
        let conc = endocrine::pk_im_depot(dose, f_im, vd, ka, ke, t_days);
        troughs.push(conc);

        let trough_at_end = endocrine::pk_im_depot(dose, f_im, vd, ka, ke, t_days + 6.5);

        println!("  Week {week:>2}: peak {conc:.2} ng/mL, trough {trough_at_end:.2} ng/mL");

        if let Some(ref mut sess) = session {
            let _ = sess.push_timeseries("pk_curve", &[t_days], &[conc]);
            let _ = sess.push_gauge("steady_trough", trough_at_end);

            // Stream HRV improvement
            let sdnn_base = patient.sdnn_ms.unwrap_or(35.0);
            let sdnn_now = endocrine::hrv_trt_response(sdnn_base, 20.0, 6.0, week as f64);
            let _ = sess.push_gauge("sdnn", sdnn_now);

            // Stream HbA1c improvement via replace (Distribution/Bar not appendable)
            if week % 4 == 0 {
                let hba1c_now = endocrine::hba1c_trajectory(
                    week as f64,
                    patient.hba1c.unwrap_or(7.0),
                    endocrine::diabetes_params::HBA1C_DELTA,
                    endocrine::diabetes_params::TAU_MONTHS,
                );
                let _ = sess.push_gauge("a1c_12mo", hba1c_now);
            }

            // Every 4 weeks, replace the population trough distribution bar chart
            if week % 4 == 0 {
                let bar = DataChannel::Bar {
                    id: "risk_compare".into(),
                    label: format!("Cardiac Risk: Baseline vs Week {week}"),
                    categories: vec!["Baseline".into(), format!("Week {week}")],
                    values: vec![
                        endocrine::cardiac_risk_composite(sdnn_base, 210.0, 1.0),
                        endocrine::cardiac_risk_composite(sdnn_now, conc * 100.0, 1.0),
                    ],
                    unit: "composite score".into(),
                };
                let _ = sess.push_replace_binding("risk_compare", &bar);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    println!();

    if let Some(ref sess) = session {
        let stats = sess.stats();
        println!(
            "[trt-live] StreamSession: {} frames, {} errors",
            stats.frames_pushed, stats.errors,
        );
    }

    // Write JSON artifact
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/clinical");
    std::fs::create_dir_all(&out_dir).expect("create sandbox/clinical/");
    let json = trt_clinical_json(&patient);
    std::fs::write(out_dir.join("trt_live_scenario.json"), &json).expect("write JSON");
    println!("[trt-live] JSON written to {}", out_dir.display());

    // Validation
    println!("\n--- Validation ---");
    let mut passed = 0u32;
    let total = 7u32;

    let check = |name: &str, ok: bool, passed: &mut u32| {
        if ok {
            println!("  [PASS] {name}");
            *passed += 1;
        } else {
            println!("  [FAIL] {name}");
        }
    };

    check(
        "scenario_has_8_nodes",
        scenario.ecosystem.primals.len() == 8,
        &mut passed,
    );
    check("scenario_has_8_edges", edges.len() == 8, &mut passed);
    check(
        "pk_troughs_collected",
        troughs.len() == n_weeks,
        &mut passed,
    );
    check(
        "pk_troughs_positive",
        troughs.iter().all(|&t| t > 0.0),
        &mut passed,
    );
    check(
        "json_valid",
        serde_json::from_str::<serde_json::Value>(&json).is_ok(),
        &mut passed,
    );
    check(
        "scenario_mode_clinical",
        scenario.mode == "clinical",
        &mut passed,
    );
    check(
        "scenario_theme_clinical",
        scenario.ui_config.theme.contains("clinical"),
        &mut passed,
    );

    println!("\nExp073 Clinical TRT Dashboard: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
