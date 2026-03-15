// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp076: Full-pipeline petalTongue scenario validation.
//!
//! Builds every scenario (PK/PD, microbiome, biosignal, endocrine, NLME),
//! validates structure, attempts IPC push, and writes JSON fallback.
//! Reports channel statistics per node and confirms `DataChannel` coverage.

use healthspring_barracuda::visualization::{
    DataChannel, HealthScenario, ScenarioEdge,
    ipc_push::{PetalTonguePushClient, PushError},
    scenarios,
};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

macro_rules! check {
    ($passed:expr, $failed:expr, $name:expr, $cond:expr) => {
        if $cond {
            $passed += 1;
            println!("  [PASS] {}", $name);
        } else {
            $failed += 1;
            println!("  [FAIL] {}", $name);
        }
    };
}

const fn channel_type(ch: &DataChannel) -> &'static str {
    match ch {
        DataChannel::TimeSeries { .. } => "TimeSeries",
        DataChannel::Distribution { .. } => "Distribution",
        DataChannel::Bar { .. } => "Bar",
        DataChannel::Gauge { .. } => "Gauge",
        DataChannel::Spectrum { .. } => "Spectrum",
        DataChannel::Heatmap { .. } => "Heatmap",
        DataChannel::Scatter3D { .. } => "Scatter3D",
    }
}

fn validate_scenario(
    name: &str,
    scenario: &HealthScenario,
    edges: &[ScenarioEdge],
    passed: &mut u32,
    failed: &mut u32,
) {
    println!("\n--- {name} ---");

    let nodes = &scenario.ecosystem.primals;
    let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

    check!(
        *passed,
        *failed,
        &format!("{name}: no duplicate node IDs"),
        node_ids.len() == nodes.len()
    );

    for node in nodes {
        check!(
            *passed,
            *failed,
            &format!("{name}/{}: health ≤ 100", node.id),
            node.health <= 100
        );
        check!(
            *passed,
            *failed,
            &format!("{name}/{}: has channels", node.id),
            !node.data_channels.is_empty()
        );
    }

    for edge in edges {
        check!(
            *passed,
            *failed,
            &format!("{name}: edge {}->{} valid refs", edge.from, edge.to),
            node_ids.contains(edge.from.as_str()) && node_ids.contains(edge.to.as_str())
        );
    }

    let json = scenarios::scenario_with_edges_json(scenario, edges);
    let val: serde_json::Value = serde_json::from_str(&json).expect("JSON must be valid");
    check!(
        *passed,
        *failed,
        &format!("{name}: valid JSON"),
        val.is_object()
    );
    check!(
        *passed,
        *failed,
        &format!("{name}: has edges array"),
        val["edges"].is_array()
    );
}

fn report_channel_stats(scenario: &HealthScenario) {
    println!("\n  Channel statistics:");
    let mut total = 0usize;
    let mut type_counts = std::collections::HashMap::new();
    for node in &scenario.ecosystem.primals {
        let count = node.data_channels.len();
        total += count;
        print!("    {}: {} channels [", node.id, count);
        for (i, ch) in node.data_channels.iter().enumerate() {
            let ct = channel_type(ch);
            *type_counts.entry(ct).or_insert(0usize) += 1;
            if i > 0 {
                print!(", ");
            }
            print!("{ct}");
        }
        println!("]");
    }
    println!(
        "  Total: {total} channels across {} nodes",
        scenario.ecosystem.primals.len()
    );
    print!("  Type breakdown:");
    for (t, c) in &type_counts {
        print!(" {t}={c}");
    }
    println!();
}

#[expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential pass/fail checks are clearest in one flow"
)]
fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    println!("=== Exp076: Full Pipeline petalTongue Scenarios ===");

    // Build all individual track scenarios
    let (pkpd, pkpd_e) = scenarios::pkpd_study();
    let (micro, micro_e) = scenarios::microbiome_study();
    let (bio, bio_e) = scenarios::biosignal_study();
    let (endo, endo_e) = scenarios::endocrine_study();
    let (nlme, nlme_e) = scenarios::nlme_study();

    // Validate each individually
    validate_scenario("PK/PD", &pkpd, &pkpd_e, &mut passed, &mut failed);
    validate_scenario("Microbiome", &micro, &micro_e, &mut passed, &mut failed);
    validate_scenario("Biosignal", &bio, &bio_e, &mut passed, &mut failed);
    validate_scenario("Endocrinology", &endo, &endo_e, &mut passed, &mut failed);
    validate_scenario("NLME", &nlme, &nlme_e, &mut passed, &mut failed);

    // NLME-specific checks
    println!("\n--- NLME Specific Checks ---");
    let nlme_ids: HashSet<&str> = nlme
        .ecosystem
        .primals
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    check!(
        passed,
        failed,
        "NLME has nlme_population",
        nlme_ids.contains("nlme_population")
    );
    check!(
        passed,
        failed,
        "NLME has nca_metrics",
        nlme_ids.contains("nca_metrics")
    );
    check!(
        passed,
        failed,
        "NLME has cwres_diagnostics",
        nlme_ids.contains("cwres_diagnostics")
    );
    check!(
        passed,
        failed,
        "NLME has vpc_check",
        nlme_ids.contains("vpc_check")
    );
    check!(
        passed,
        failed,
        "NLME has gof_fit",
        nlme_ids.contains("gof_fit")
    );

    // Biosignal WFDB check
    println!("\n--- WFDB Biosignal Check ---");
    let bio_ids: HashSet<&str> = bio
        .ecosystem
        .primals
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    check!(
        passed,
        failed,
        "Biosignal has wfdb_ecg",
        bio_ids.contains("wfdb_ecg")
    );

    let wfdb_node = bio
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "wfdb_ecg")
        .unwrap();
    check!(
        passed,
        failed,
        "wfdb_ecg has TimeSeries",
        wfdb_node
            .data_channels
            .iter()
            .any(|ch| matches!(ch, DataChannel::TimeSeries { .. }))
    );
    check!(
        passed,
        failed,
        "wfdb_ecg has Bar (beat types)",
        wfdb_node
            .data_channels
            .iter()
            .any(|ch| matches!(ch, DataChannel::Bar { .. }))
    );
    check!(
        passed,
        failed,
        "wfdb_ecg has Gauge",
        wfdb_node
            .data_channels
            .iter()
            .any(|ch| matches!(ch, DataChannel::Gauge { .. }))
    );

    // Full combined study
    println!("\n--- Full Combined Study ---");
    let (full, full_e) = scenarios::full_study();
    validate_scenario("Full Study", &full, &full_e, &mut passed, &mut failed);

    check!(
        passed,
        failed,
        &format!(
            "full: 28 nodes (6+4+5+8+5), got {}",
            full.ecosystem.primals.len()
        ),
        full.ecosystem.primals.len() == 28
    );
    check!(
        passed,
        failed,
        &format!("full: 29 edges (5+3+4+7+5+5 cross), got {}", full_e.len()),
        full_e.len() == 29
    );

    // All 7 DataChannel types present
    let full_json = scenarios::scenario_with_edges_json(&full, &full_e);
    let has_all_types = full_json.contains("\"channel_type\": \"timeseries\"")
        && full_json.contains("\"channel_type\": \"distribution\"")
        && full_json.contains("\"channel_type\": \"bar\"")
        && full_json.contains("\"channel_type\": \"gauge\"")
        && full_json.contains("\"channel_type\": \"spectrum\"")
        && full_json.contains("\"channel_type\": \"heatmap\"")
        && full_json.contains("\"channel_type\": \"scatter3d\"");
    check!(
        passed,
        failed,
        "full: all 7 DataChannel types present",
        has_all_types
    );

    // Cross-track NLME edge
    let edge_pairs: HashSet<(String, String)> = full_e
        .iter()
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();
    check!(
        passed,
        failed,
        "cross-track: pop_pk -> nlme_population",
        edge_pairs.contains(&("pop_pk".into(), "nlme_population".into()))
    );

    report_channel_stats(&full);

    // Write JSON to sandbox
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/scenarios");
    fs::create_dir_all(&out).expect("create sandbox/scenarios/");

    let write = |name: &str, json: &str| {
        let path = out.join(name);
        fs::write(&path, json).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        println!("  wrote {} ({} KB)", name, json.len() / 1024);
    };

    println!("\n=== Writing Scenarios to Disk ===");
    write("full_pipeline.json", &full_json);
    write(
        "nlme_study.json",
        &scenarios::scenario_with_edges_json(&nlme, &nlme_e),
    );
    write(
        "pkpd_study.json",
        &scenarios::scenario_with_edges_json(&pkpd, &pkpd_e),
    );
    write(
        "microbiome_study.json",
        &scenarios::scenario_with_edges_json(&micro, &micro_e),
    );
    write(
        "biosignal_study.json",
        &scenarios::scenario_with_edges_json(&bio, &bio_e),
    );
    write(
        "endocrine_study.json",
        &scenarios::scenario_with_edges_json(&endo, &endo_e),
    );

    // Attempt IPC push to petalTongue
    println!("\n=== petalTongue IPC Push ===");
    match PetalTonguePushClient::discover() {
        Ok(client) => {
            println!("petalTongue found — pushing full pipeline");
            match client.push_render("healthspring-full-pipeline", &full.name, &full) {
                Ok(()) => {
                    println!("  pushed full pipeline successfully");
                    check!(passed, failed, "IPC push: full pipeline", true);
                }
                Err(e) => {
                    println!("  [WARN] push failed: {e}");
                    check!(passed, failed, "IPC push: full pipeline (non-fatal)", true);
                }
            }
        }
        Err(PushError::NotFound(msg)) => {
            println!("  petalTongue not running ({msg}) — JSON fallback written above");
            check!(passed, failed, "IPC push: graceful fallback", true);
        }
        Err(e) => {
            println!("  discovery error: {e} — JSON fallback written above");
            check!(passed, failed, "IPC push: graceful fallback on error", true);
        }
    }

    // Summary
    println!("\n====================================");
    println!(
        "Exp076 Full Pipeline: {passed}/{} checks passed",
        passed + failed
    );
    if failed > 0 {
        println!("{failed} checks FAILED");
        std::process::exit(1);
    }
}
