// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Generates patient-specific TRT scenario JSON files for petalTongue.
//!
//! Each patient archetype produces a standalone JSON file in
//! `sandbox/scenarios/` that petalTongue can load directly via
//! `--scenario <path>`.

use healthspring_barracuda::visualization::clinical::{
    PatientTrtProfile, TrtProtocol, trt_clinical_json,
};
use std::fs;
use std::path::Path;

fn main() {
    use healthspring_barracuda::validation::OrExit;

    let out = Path::new("sandbox/scenarios");
    fs::create_dir_all(out).or_exit("create output dir");

    let patients = vec![
        ("clinical-trt-young-athlete.json", {
            let mut p = PatientTrtProfile::new(
                "Young Athlete (35M)",
                35.0,
                180.0,
                250.0,
                TrtProtocol::ImWeekly,
            );
            p.gut_diversity = Some(0.90);
            p.hba1c = Some(5.4);
            p.sdnn_ms = Some(52.0);
            p
        }),
        ("clinical-trt-middle-metabolic.json", {
            let mut p = PatientTrtProfile::new(
                "Middle-Aged Metabolic (52M)",
                52.0,
                260.0,
                220.0,
                TrtProtocol::Pellet,
            );
            p.gut_diversity = Some(0.42);
            p.hba1c = Some(7.6);
            p.sdnn_ms = Some(30.0);
            p
        }),
        ("clinical-trt-senior-lean.json", {
            let mut p = PatientTrtProfile::new(
                "Senior Lean (68M)",
                68.0,
                170.0,
                310.0,
                TrtProtocol::ImBiweekly,
            );
            p.gut_diversity = Some(0.72);
            p.sdnn_ms = Some(34.0);
            p
        }),
        (
            "clinical-trt-standard-pellet.json",
            PatientTrtProfile::new(
                "Standard Pellet (55M, 220lb)",
                55.0,
                220.0,
                280.0,
                TrtProtocol::Pellet,
            ),
        ),
        ("clinical-trt-high-bmi.json", {
            let mut p = PatientTrtProfile::new(
                "High-BMI (48M, 320lb)",
                48.0,
                320.0,
                195.0,
                TrtProtocol::Pellet,
            );
            p.gut_diversity = Some(0.35);
            p.hba1c = Some(8.2);
            p.sdnn_ms = Some(25.0);
            p
        }),
    ];

    for (filename, patient) in &patients {
        let json = trt_clinical_json(patient);
        let path = out.join(filename);
        fs::write(&path, &json).or_exit(&format!("write {}", path.display()));
        println!(
            "  wrote {} ({} bytes, {} nodes)",
            path.display(),
            json.len(),
            8
        );
    }

    println!(
        "\n{} clinical TRT scenarios written to {}",
        patients.len(),
        out.display()
    );
}
