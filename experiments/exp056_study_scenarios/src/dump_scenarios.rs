// SPDX-License-Identifier: AGPL-3.0-or-later
//! Write all healthSpring petalTongue scenario JSON files to `sandbox/scenarios/`.

use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{full_scenario_json, scenarios};
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
    write(
        "healthspring-pkpd.json",
        &scenarios::scenario_with_edges_json(&pkpd, &pkpd_e),
    );

    let (micro, micro_e) = scenarios::microbiome_study();
    write(
        "healthspring-microbiome.json",
        &scenarios::scenario_with_edges_json(&micro, &micro_e),
    );

    let (bio, bio_e) = scenarios::biosignal_study();
    write(
        "healthspring-biosignal.json",
        &scenarios::scenario_with_edges_json(&bio, &bio_e),
    );

    let (endo, endo_e) = scenarios::endocrine_study();
    write(
        "healthspring-endocrine.json",
        &scenarios::scenario_with_edges_json(&endo, &endo_e),
    );

    let (full, full_e) = scenarios::full_study();
    write(
        "healthspring-full-study.json",
        &scenarios::scenario_with_edges_json(&full, &full_e),
    );

    let mut patient = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    patient.testosterone_ng_dl = Some(450.0);
    patient.on_trt = true;
    patient.trt_months = 12.0;
    patient.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);
    let assessment = assess_patient(&patient);
    let pop = population_montecarlo(&patient, 1000, 42);
    write(
        "healthspring-diagnostic.json",
        &full_scenario_json(&assessment, &pop, "Male 55y, TRT 12mo"),
    );

    println!("\nAll 6 scenarios written to {}", out.display());
}
