// SPDX-License-Identifier: AGPL-3.0-or-later
//! Push healthSpring scenarios to petalTongue via IPC, or write JSON to disk.
//!
//! When petalTongue is running (discovered via socket), pushes live visualization
//! data via visualization.render. Otherwise falls back to writing scenario JSON files.

use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{
    annotate_population, assessment_to_scenario, full_scenario_json,
    ipc_push::{PetalTonguePushClient, PushError},
    scenarios, HealthScenario,
};
use std::fs;
use std::path::Path;

fn main() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
    fs::create_dir_all(&out).expect("create sandbox/scenarios/");

    let write = |name: &str, json: &str| {
        let path = out.join(name);
        fs::write(&path, json).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        println!("  wrote {} ({} KB)", name, json.len() / 1024);
    };

    println!("=== healthSpring scenario dump ===\n");

    let (pkpd, pkpd_e) = scenarios::pkpd_study();
    let (micro, micro_e) = scenarios::microbiome_study();
    let (bio, bio_e) = scenarios::biosignal_study();
    let (endo, endo_e) = scenarios::endocrine_study();
    let (full, full_e) = scenarios::full_study();

    let mut patient = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    patient.testosterone_ng_dl = Some(450.0);
    patient.on_trt = true;
    patient.trt_months = 12.0;
    patient.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);
    let assessment = assess_patient(&patient);
    let pop = population_montecarlo(&patient, 1000, 42);
    let diagnostic_scenario = annotate_population(
        assessment_to_scenario(&assessment, "Male 55y, TRT 12mo"),
        &pop,
    );
    let diagnostic_json = full_scenario_json(&assessment, &pop, "Male 55y, TRT 12mo");

    match PetalTonguePushClient::discover() {
        Ok(client) => {
            println!("petalTongue found — pushing via IPC\n");
            let push = |session_id: &str, title: &str, scenario: &HealthScenario| {
                if let Err(e) = client.push_render(session_id, title, scenario) {
                    println!("  [WARN] {}: {e}", session_id);
                } else {
                    println!("  pushed {}", session_id);
                }
            };
            push("healthspring-pkpd", &pkpd.name, &pkpd);
            push("healthspring-microbiome", &micro.name, &micro);
            push("healthspring-biosignal", &bio.name, &bio);
            push("healthspring-endocrine", &endo.name, &endo);
            push("healthspring-full-study", &full.name, &full);
            push("healthspring-diagnostic", &diagnostic_scenario.name, &diagnostic_scenario);
            println!("\nAll 6 scenarios pushed to petalTongue");
        }
        Err(PushError::NotFound(_)) => {
            println!("petalTongue not running — writing to disk\n");
            write(
                "healthspring-pkpd.json",
                &scenarios::scenario_with_edges_json(&pkpd, &pkpd_e),
            );
            write(
                "healthspring-microbiome.json",
                &scenarios::scenario_with_edges_json(&micro, &micro_e),
            );
            write(
                "healthspring-biosignal.json",
                &scenarios::scenario_with_edges_json(&bio, &bio_e),
            );
            write(
                "healthspring-endocrine.json",
                &scenarios::scenario_with_edges_json(&endo, &endo_e),
            );
            write(
                "healthspring-full-study.json",
                &scenarios::scenario_with_edges_json(&full, &full_e),
            );
            write("healthspring-diagnostic.json", &diagnostic_json);
            println!("\nAll 6 scenarios written to {}", out.display());
        }
        Err(e) => {
            eprintln!("petalTongue discovery failed: {e}");
            eprintln!("Falling back to file write.\n");
            write(
                "healthspring-pkpd.json",
                &scenarios::scenario_with_edges_json(&pkpd, &pkpd_e),
            );
            write(
                "healthspring-microbiome.json",
                &scenarios::scenario_with_edges_json(&micro, &micro_e),
            );
            write(
                "healthspring-biosignal.json",
                &scenarios::scenario_with_edges_json(&bio, &bio_e),
            );
            write(
                "healthspring-endocrine.json",
                &scenarios::scenario_with_edges_json(&endo, &endo_e),
            );
            write(
                "healthspring-full-study.json",
                &scenarios::scenario_with_edges_json(&full, &full_e),
            );
            write("healthspring-diagnostic.json", &diagnostic_json);
            println!("\nAll 6 scenarios written to {}", out.display());
        }
    }
}
