// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp088: Unified dashboard — generates, validates, and optionally pushes
//! all healthSpring `petalTongue` scenarios (original tracks + V16 + compute).

use healthspring_barracuda::visualization::{
    DataChannel, HealthScenario, ScenarioEdge, ipc_push::PetalTonguePushClient, scenarios,
    scenarios::scenario_with_edges_json,
};
use std::fs;
use std::path::Path;

struct Tally {
    checks: u32,
    pass: u32,
}

impl Tally {
    fn check(&mut self, name: &str, cond: bool) {
        self.checks += 1;
        if cond {
            self.pass += 1;
            println!("  [PASS] {name}");
        } else {
            println!("  [FAIL] {name}");
        }
    }
}

fn main() {
    let mut t = Tally { checks: 0, pass: 0 };

    println!("=== Exp088: Unified Dashboard ===\n");

    let tracks = build_and_validate_tracks(&mut t);
    let (v16, v16_edges) = build_and_validate_v16(&mut t);
    let (compute, compute_edges) = build_and_validate_compute(&mut t);
    let (full, full_edges) = build_and_validate_full_study(&mut t);

    let all_scenarios = build_scenario_list(
        &tracks,
        &v16,
        &v16_edges,
        &compute,
        &compute_edges,
        &full,
        &full_edges,
    );
    validate_json_roundtrip(&mut t, &all_scenarios);
    validate_channel_coverage(&mut t, &full);
    push_or_dump(&all_scenarios);

    println!(
        "\n=== Exp088 Result: {}/{} checks passed ===",
        t.pass, t.checks
    );
    if t.pass != t.checks {
        eprintln!("ERROR: some checks failed");
        std::process::exit(1);
    }
}

fn build_and_validate_tracks(
    t: &mut Tally,
) -> Vec<(&'static str, HealthScenario, Vec<ScenarioEdge>)> {
    println!("─── Track Scenarios ───");
    #[expect(
        clippy::type_complexity,
        reason = "one-shot tuple list for validation loop"
    )]
    let raw: Vec<(&str, usize, (HealthScenario, Vec<ScenarioEdge>))> = vec![
        ("pkpd", 6, scenarios::pkpd_study()),
        ("microbiome", 4, scenarios::microbiome_study()),
        ("biosignal", 5, scenarios::biosignal_study()),
        ("endocrine", 8, scenarios::endocrine_study()),
        ("nlme", 5, scenarios::nlme_study()),
    ];

    let mut tracks = Vec::new();
    for (name, expected, (scenario, edges)) in raw {
        t.check(
            &format!("{name}: node count = {expected}"),
            scenario.ecosystem.primals.len() == expected,
        );
        validate_scenario(t, name, &scenario, &edges);
        tracks.push((name, scenario, edges));
    }
    tracks
}

fn build_and_validate_v16(t: &mut Tally) -> (HealthScenario, Vec<ScenarioEdge>) {
    println!("\n─── V16 Primitives ───");
    let (v16, v16_edges) = scenarios::v16_study();
    t.check("v16: node count = 6", v16.ecosystem.primals.len() == 6);
    t.check("v16: edge count = 5", v16_edges.len() == 5);
    validate_scenario(t, "v16", &v16, &v16_edges);

    let v16_ids: Vec<&str> = v16
        .ecosystem
        .primals
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    for expected_id in &[
        "mm_nonlinear_pk",
        "abx_perturbation",
        "scfa_prod",
        "gut_serotonin",
        "eda_stress",
        "arrhythmia_classify",
    ] {
        t.check(
            &format!("v16: has node {expected_id}"),
            v16_ids.contains(expected_id),
        );
    }
    (v16, v16_edges)
}

fn build_and_validate_compute(t: &mut Tally) -> (HealthScenario, Vec<ScenarioEdge>) {
    println!("\n─── Compute Pipeline ───");
    let (gpu_scaling, gpu_edges) = scenarios::gpu_scaling_study();
    t.check(
        "gpu_scaling: node count = 1",
        gpu_scaling.ecosystem.primals.len() == 1,
    );
    validate_scenario(t, "gpu_scaling", &gpu_scaling, &gpu_edges);

    let (v16_topo, topo_edges) = scenarios::v16_topology_study();
    t.check(
        "v16_topology: node count = 3",
        v16_topo.ecosystem.primals.len() == 3,
    );
    validate_scenario(t, "v16_topology", &v16_topo, &topo_edges);

    let (v16_dispatch, dispatch_edges) = scenarios::v16_dispatch_study();
    t.check(
        "v16_dispatch: node count = 6",
        v16_dispatch.ecosystem.primals.len() == 6,
    );
    validate_scenario(t, "v16_dispatch", &v16_dispatch, &dispatch_edges);

    let (compute, compute_edges) = scenarios::compute_pipeline_study();
    t.check(
        "compute_pipeline: node count = 10",
        compute.ecosystem.primals.len() == 10,
    );
    validate_scenario(t, "compute_pipeline", &compute, &compute_edges);
    (compute, compute_edges)
}

fn build_and_validate_full_study(t: &mut Tally) -> (HealthScenario, Vec<ScenarioEdge>) {
    println!("\n─── Full Study (V19) ───");
    let (full, full_edges) = scenarios::full_study();
    t.check(
        "full_study: node count = 34",
        full.ecosystem.primals.len() == 34,
    );
    t.check("full_study: edge count = 38", full_edges.len() == 38);
    validate_scenario(t, "full_study", &full, &full_edges);

    let full_ids: std::collections::HashSet<&str> = full
        .ecosystem
        .primals
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    t.check("full_study: all IDs unique", full_ids.len() == 34);
    (full, full_edges)
}

fn build_scenario_list<'a>(
    tracks: &'a [(&'a str, HealthScenario, Vec<ScenarioEdge>)],
    v16: &'a HealthScenario,
    v16_edges: &'a [ScenarioEdge],
    compute: &'a HealthScenario,
    compute_edges: &'a [ScenarioEdge],
    full: &'a HealthScenario,
    full_edges: &'a [ScenarioEdge],
) -> Vec<(&'a str, &'a HealthScenario, &'a [ScenarioEdge])> {
    let mut v: Vec<(&str, &HealthScenario, &[ScenarioEdge])> = tracks
        .iter()
        .map(|(name, s, e)| (*name, s, e.as_slice()))
        .collect();
    v.push(("v16", v16, v16_edges));
    v.push(("compute_pipeline", compute, compute_edges));
    v.push(("full_study", full, full_edges));
    v
}

fn validate_json_roundtrip(t: &mut Tally, all: &[(&str, &HealthScenario, &[ScenarioEdge])]) {
    println!("\n─── JSON Round-Trip ───");
    for (name, scenario, edges) in all {
        let json = scenario_with_edges_json(scenario, edges);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        t.check(&format!("{name}: JSON valid"), parsed.is_ok());
        if let Ok(val) = parsed {
            t.check(
                &format!("{name}: JSON has edges array"),
                val["edges"].is_array(),
            );
        }
    }
}

fn validate_channel_coverage(t: &mut Tally, full: &HealthScenario) {
    println!("\n─── Channel Type Coverage ───");
    let all_channels: Vec<&DataChannel> = full
        .ecosystem
        .primals
        .iter()
        .flat_map(|n| &n.data_channels)
        .collect();

    let has = |pred: fn(&DataChannel) -> bool| all_channels.iter().any(|ch| pred(ch));
    t.check(
        "full_study has TimeSeries",
        has(|ch| matches!(ch, DataChannel::TimeSeries { .. })),
    );
    t.check(
        "full_study has Bar",
        has(|ch| matches!(ch, DataChannel::Bar { .. })),
    );
    t.check(
        "full_study has Gauge",
        has(|ch| matches!(ch, DataChannel::Gauge { .. })),
    );
    t.check(
        "full_study has Distribution",
        has(|ch| matches!(ch, DataChannel::Distribution { .. })),
    );
    t.check(
        "full_study has Spectrum",
        has(|ch| matches!(ch, DataChannel::Spectrum { .. })),
    );
    t.check(
        "full_study has Heatmap",
        has(|ch| matches!(ch, DataChannel::Heatmap { .. })),
    );
    t.check(
        "full_study has Scatter3D",
        has(|ch| matches!(ch, DataChannel::Scatter3D { .. })),
    );
}

fn push_or_dump(all: &[(&str, &HealthScenario, &[ScenarioEdge])]) {
    println!("\n─── Output ───");
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
    if fs::create_dir_all(&out).is_err() {
        eprintln!("ERROR: create sandbox/scenarios/");
        std::process::exit(1);
    }

    let write = |name: &str, json: &str| {
        let path = out.join(name);
        if fs::write(&path, json).is_err() {
            eprintln!("ERROR: write {}", path.display());
            std::process::exit(1);
        }
        println!("  wrote {} ({} KB)", name, json.len() / 1024);
    };

    let write_list: Vec<(String, String)> = all
        .iter()
        .map(|(name, scenario, edges)| {
            (
                format!("healthspring-{name}.json"),
                scenario_with_edges_json(scenario, edges),
            )
        })
        .collect();

    if let Ok(client) = PetalTonguePushClient::discover() {
        println!("petalTongue found — pushing via IPC\n");
        for (name, scenario, _) in all {
            let session_id = format!("healthspring-{name}");
            match client.push_render(&session_id, &scenario.name, scenario) {
                Ok(()) => println!("  pushed {session_id}"),
                Err(e) => println!("  [WARN] {session_id}: {e}"),
            }
        }
        println!("\nAlso writing JSON to disk for offline viewing.");
    } else {
        println!("petalTongue not running — writing to disk\n");
    }

    for (filename, json) in &write_list {
        write(filename, json);
    }

    let scenario_dir = out.display();
    println!("\n─── Quick Start Guide ───");
    println!("View any scenario with petalTongue:");
    println!("  petaltongue ui --scenario {scenario_dir}/healthspring-full_study.json");
    println!("  petaltongue tui --scenario {scenario_dir}/healthspring-v16.json");
    println!("  petaltongue web --scenario {scenario_dir}/healthspring-compute_pipeline.json");
    println!(
        "  petaltongue headless --scenario {scenario_dir}/healthspring-full_study.json --output report.svg"
    );
}

fn validate_scenario(t: &mut Tally, name: &str, scenario: &HealthScenario, edges: &[ScenarioEdge]) {
    let node_ids: std::collections::HashSet<&str> = scenario
        .ecosystem
        .primals
        .iter()
        .map(|n| n.id.as_str())
        .collect();

    t.check(
        &format!("{name}: no duplicate node IDs"),
        node_ids.len() == scenario.ecosystem.primals.len(),
    );

    for node in &scenario.ecosystem.primals {
        t.check(
            &format!("{name}/{}: health ≤ 100", node.id),
            node.health <= 100,
        );
        t.check(
            &format!("{name}/{}: has channels", node.id),
            !node.data_channels.is_empty(),
        );
    }

    for edge in edges {
        t.check(
            &format!("{name}: edge {}->{} valid", edge.from, edge.to),
            node_ids.contains(edge.from.as_str()) && node_ids.contains(edge.to.as_str()),
        );
    }
}
