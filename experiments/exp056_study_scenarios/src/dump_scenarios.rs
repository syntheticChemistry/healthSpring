// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Push healthSpring scenarios to `petalTongue` via IPC, or write JSON to disk.
//!
//! When `petalTongue` is running (discovered via socket), pushes live visualization
//! data via `visualization.render`. Otherwise falls back to writing scenario JSON files.

use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{
    HealthScenario, ScenarioEdge, annotate_population, assessment_to_scenario,
    clinical::{PatientTrtProfile, TrtProtocol, trt_clinical_json, trt_clinical_scenario},
    ipc_push::{PetalTonguePushClient, PushError},
    scenarios,
    scenarios::scenario_with_edges_json,
    scenarios::topology::{
        DispatchStageInfo, TopologyNest, TopologyNode, TopologyTransfer, dispatch_scenario,
        topology_scenario,
    },
};
use std::fs;
use std::path::Path;

fn main() {
    use healthspring_barracuda::validation::OrExit;

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
    fs::create_dir_all(&out).or_exit("create sandbox/scenarios/");

    println!("=== healthSpring scenario dump ===\n");

    let named_scenarios = build_all_scenarios();
    let trt_archetypes = build_trt_archetypes();

    let write_to_disk = |entries: &[(&str, String)], trt: &[(String, PatientTrtProfile)]| {
        for (name, json) in entries {
            let path = out.join(name);
            fs::write(&path, json).or_exit(&format!("write {}", path.display()));
            println!("  wrote {} ({} KB)", name, json.len() / 1024);
        }
        for (sid, profile) in trt {
            let json = trt_clinical_json(profile);
            let path = out.join(format!("{sid}.json"));
            fs::write(&path, &json).or_exit(&format!("write {}", path.display()));
            println!("  wrote {sid}.json ({} KB)", json.len() / 1024);
        }
        println!(
            "\nAll {} scenarios written to {}",
            entries.len() + trt.len(),
            out.display()
        );
    };

    let file_entries: Vec<(&str, String)> = named_scenarios
        .iter()
        .map(|(name, scenario, edges)| (*name, scenario_with_edges_json(scenario, edges)))
        .collect();

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
            for (name, scenario, _) in &named_scenarios {
                let sid = name.trim_end_matches(".json");
                push(sid, &scenario.name, scenario);
            }
            for (sid, profile) in &trt_archetypes {
                let (scn, _) = trt_clinical_scenario(profile);
                if let Err(e) = client.push_render_with_config(sid, &scn.name, &scn, "clinical") {
                    println!("  [WARN] {sid}: {e}");
                } else {
                    println!("  pushed {sid}");
                }
            }
            println!(
                "\nAll {} scenarios pushed to petalTongue",
                named_scenarios.len() + trt_archetypes.len()
            );
            println!("\nAlso writing to disk...");
            write_to_disk(&file_entries, &trt_archetypes);
        }
        Err(PushError::NotFound(_)) => {
            println!("petalTongue not running — writing to disk\n");
            write_to_disk(&file_entries, &trt_archetypes);
        }
        Err(e) => {
            eprintln!("petalTongue discovery failed: {e}");
            eprintln!("Falling back to file write.\n");
            write_to_disk(&file_entries, &trt_archetypes);
        }
    }
}

fn build_all_scenarios() -> Vec<(&'static str, HealthScenario, Vec<ScenarioEdge>)> {
    let (pkpd, pkpd_e) = scenarios::pkpd_study();
    let (micro, micro_e) = scenarios::microbiome_study();
    let (bio, bio_e) = scenarios::biosignal_study();
    let (endo, endo_e) = scenarios::endocrine_study();
    let (nlme, nlme_e) = scenarios::nlme_study();
    let (v16, v16_e) = scenarios::v16_study();
    let (compute, compute_e) = scenarios::compute_pipeline_study();
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

    vec![
        ("healthspring-pkpd.json", pkpd, pkpd_e),
        ("healthspring-microbiome.json", micro, micro_e),
        ("healthspring-biosignal.json", bio, bio_e),
        ("healthspring-endocrine.json", endo, endo_e),
        ("healthspring-nlme.json", nlme, nlme_e),
        ("healthspring-v16.json", v16, v16_e),
        ("healthspring-compute.json", compute, compute_e),
        ("healthspring-full-study.json", full, full_e),
        ("healthspring-diagnostic.json", diagnostic_scenario, vec![]),
        ("healthspring-topology.json", topo_scenario, topo_edges),
        ("healthspring-dispatch.json", dispatch_scn, dispatch_edges),
    ]
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
