// SPDX-License-Identifier: AGPL-3.0-or-later
//! Push healthSpring scenarios to petalTongue via IPC, or write JSON to disk.
//!
//! When petalTongue is running (discovered via socket), pushes live visualization
//! data via visualization.render. Otherwise falls back to writing scenario JSON files.

use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{
    HealthScenario, annotate_population, assessment_to_scenario,
    clinical::{PatientTrtProfile, TrtProtocol, trt_clinical_json, trt_clinical_scenario},
    full_scenario_json,
    ipc_push::{PetalTonguePushClient, PushError},
    scenarios,
    scenarios::topology::{
        DispatchStageInfo, TopologyNest, TopologyNode, TopologyTransfer, dispatch_scenario,
        topology_scenario,
    },
};
use std::fs;
use std::path::Path;

#[expect(
    clippy::too_many_lines,
    reason = "scenario dump orchestrator — linear sequence of scenario builds and writes"
)]
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
    let (nlme, nlme_e) = scenarios::nlme_study();
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

    // Topology scenario
    let topo_nodes = vec![TopologyNode {
        node_id: 0,
        pcie_gen: "Gen4".to_string(),
        nests: vec![
            TopologyNest {
                device_id: 0,
                substrate: "cpu".to_string(),
                label: "CPU (16 cores)".to_string(),
                memory_total_bytes: 64 * 1024 * 1024 * 1024,
                memory_used_bytes: 8 * 1024 * 1024 * 1024,
                utilization_pct: 15.0,
                available: true,
            },
            TopologyNest {
                device_id: 1,
                substrate: "gpu".to_string(),
                label: "GPU (RTX 4090)".to_string(),
                memory_total_bytes: 24 * 1024 * 1024 * 1024,
                memory_used_bytes: 2 * 1024 * 1024 * 1024,
                utilization_pct: 0.0,
                available: true,
            },
            TopologyNest {
                device_id: 2,
                substrate: "npu".to_string(),
                label: "NPU (Akida)".to_string(),
                memory_total_bytes: 256 * 1024 * 1024,
                memory_used_bytes: 0,
                utilization_pct: 0.0,
                available: true,
            },
        ],
    }];
    let topo_transfers = vec![TopologyTransfer {
        src_id: "T0.N0.D2".to_string(),
        dst_id: "T0.N0.D1".to_string(),
        label: "PCIe P2P Gen4 31.5 GB/s".to_string(),
    }];
    let (topo_scenario, topo_edges) = topology_scenario(0, &topo_nodes, &topo_transfers);

    // Dispatch plan scenario
    let dispatch_stages = vec![
        DispatchStageInfo {
            name: "ECG Streaming".to_string(),
            substrate: "npu".to_string(),
            elapsed_us: 50.0,
            output_elements: 360,
        },
        DispatchStageInfo {
            name: "Population PK".to_string(),
            substrate: "gpu".to_string(),
            elapsed_us: 1200.0,
            output_elements: 5000,
        },
        DispatchStageInfo {
            name: "Diagnostic Fusion".to_string(),
            substrate: "cpu".to_string(),
            elapsed_us: 10.0,
            output_elements: 1,
        },
    ];
    let (dispatch_scn, dispatch_edges) =
        dispatch_scenario("Mixed Clinical Dispatch", &dispatch_stages);

    // Clinical TRT archetypes
    let trt_archetypes = build_trt_archetypes();

    match PetalTonguePushClient::discover() {
        Ok(client) => {
            println!("petalTongue found — pushing via IPC\n");
            let push = |session_id: &str, title: &str, scenario: &HealthScenario| {
                if let Err(e) = client.push_render(session_id, title, scenario) {
                    println!("  [WARN] {session_id}: {e}");
                } else {
                    println!("  pushed {session_id}");
                }
            };
            push("healthspring-pkpd", &pkpd.name, &pkpd);
            push("healthspring-microbiome", &micro.name, &micro);
            push("healthspring-biosignal", &bio.name, &bio);
            push("healthspring-endocrine", &endo.name, &endo);
            push("healthspring-nlme", &nlme.name, &nlme);
            push("healthspring-full-study", &full.name, &full);
            push(
                "healthspring-diagnostic",
                &diagnostic_scenario.name,
                &diagnostic_scenario,
            );
            push("healthspring-topology", &topo_scenario.name, &topo_scenario);
            push("healthspring-dispatch", &dispatch_scn.name, &dispatch_scn);
            for (sid, profile) in &trt_archetypes {
                let (scn, _) = trt_clinical_scenario(profile);
                if let Err(e) = client.push_render_with_config(sid, &scn.name, &scn, "clinical") {
                    println!("  [WARN] {sid}: {e}");
                } else {
                    println!("  pushed {sid}");
                }
            }
            println!("\nAll 14 scenarios pushed to petalTongue");
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
                "healthspring-nlme.json",
                &scenarios::scenario_with_edges_json(&nlme, &nlme_e),
            );
            write(
                "healthspring-full-study.json",
                &scenarios::scenario_with_edges_json(&full, &full_e),
            );
            write("healthspring-diagnostic.json", &diagnostic_json);
            write(
                "healthspring-topology.json",
                &scenarios::scenario_with_edges_json(&topo_scenario, &topo_edges),
            );
            write(
                "healthspring-dispatch.json",
                &scenarios::scenario_with_edges_json(&dispatch_scn, &dispatch_edges),
            );
            for (sid, profile) in &trt_archetypes {
                write(&format!("{sid}.json"), &trt_clinical_json(profile));
            }
            println!("\nAll 14 scenarios written to {}", out.display());
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
                "healthspring-nlme.json",
                &scenarios::scenario_with_edges_json(&nlme, &nlme_e),
            );
            write(
                "healthspring-full-study.json",
                &scenarios::scenario_with_edges_json(&full, &full_e),
            );
            write("healthspring-diagnostic.json", &diagnostic_json);
            write(
                "healthspring-topology.json",
                &scenarios::scenario_with_edges_json(&topo_scenario, &topo_edges),
            );
            write(
                "healthspring-dispatch.json",
                &scenarios::scenario_with_edges_json(&dispatch_scn, &dispatch_edges),
            );
            for (sid, profile) in &trt_archetypes {
                write(&format!("{sid}.json"), &trt_clinical_json(profile));
            }
            println!("\nAll 14 scenarios written to {}", out.display());
        }
    }
}

fn build_trt_archetypes() -> Vec<(String, PatientTrtProfile)> {
    let young = {
        let mut p = PatientTrtProfile::new(
            "Young Hypogonadal",
            32.0,
            175.0,
            180.0,
            TrtProtocol::ImWeekly,
        );
        p.sdnn_ms = Some(55.0);
        p.gut_diversity = Some(0.80);
        p
    };
    let obese = {
        let mut p = PatientTrtProfile::new(
            "Obese Diabetic",
            48.0,
            285.0,
            250.0,
            TrtProtocol::ImBiweekly,
        );
        p.hba1c = Some(7.8);
        p.gut_diversity = Some(0.40);
        p.sdnn_ms = Some(28.0);
        p
    };
    let senior = {
        let mut p =
            PatientTrtProfile::new("Senior Sarcopenic", 68.0, 155.0, 220.0, TrtProtocol::Pellet);
        p.sdnn_ms = Some(22.0);
        p.gut_diversity = Some(0.55);
        p
    };
    let athlete = {
        let mut p =
            PatientTrtProfile::new("Former Athlete", 42.0, 210.0, 310.0, TrtProtocol::ImWeekly);
        p.sdnn_ms = Some(72.0);
        p.gut_diversity = Some(0.90);
        p
    };
    let metabolic = {
        let mut p = PatientTrtProfile::new(
            "Metabolic Syndrome",
            55.0,
            240.0,
            195.0,
            TrtProtocol::Pellet,
        );
        p.hba1c = Some(6.9);
        p.gut_diversity = Some(0.35);
        p.sdnn_ms = Some(31.0);
        p
    };

    vec![
        ("healthspring-trt-young".into(), young),
        ("healthspring-trt-obese".into(), obese),
        ("healthspring-trt-senior".into(), senior),
        ("healthspring-trt-athlete".into(), athlete),
        ("healthspring-trt-metabolic".into(), metabolic),
    ]
}
