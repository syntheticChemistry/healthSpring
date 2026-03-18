// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::expect_used,
    reason = "Mutex::lock().expect() is idiomatic for poisoned-mutex \
              handling in multi-threaded streaming callbacks"
)]
//! Compute dashboard: wire toadStool `execute_streaming()` → petalTongue
//! `StreamSession` for live pipeline progress visualization.
//!
//! When petalTongue is running, pushes live gauge/bar updates per-stage.
//! Otherwise falls back to stdout + JSON file output suitable for offline
//! rendering.

use healthspring_barracuda::visualization::ipc_push::PushError;
use healthspring_barracuda::visualization::scenarios::scenario_with_edges_json;
use healthspring_barracuda::visualization::scenarios::topology::{
    DispatchStageInfo, TopologyNest, TopologyNode, TopologyTransfer, dispatch_scenario,
    topology_scenario,
};
use healthspring_barracuda::visualization::stream::StreamSession;
use healthspring_forge::Substrate;
use healthspring_toadstool::pipeline::Pipeline;
use healthspring_toadstool::stage::{ReduceKind, Stage, StageOp, TransformKind};
use serde::Serialize;
use std::sync::{Arc, Mutex};

const SESSION_ID: &str = "healthspring-compute-dashboard";

// ── Pipeline construction ────────────────────────────────────────────

fn build_clinical_pipeline() -> Pipeline {
    let mut p = Pipeline::new("Mixed Clinical Pipeline");

    p.add_stage(Stage {
        name: "ECG Fusion (NPU)".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 3 },
    });

    p.add_stage(Stage {
        name: "Hill Dose-Response (GPU)".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 500,
            seed: 42,
        },
    });

    p.add_stage(Stage {
        name: "Hill Transform".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 1.0,
                ec50: 0.5,
                n: 2.0,
            },
        },
    });

    p.add_stage(Stage {
        name: "Population PK (GPU)".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::PopulationPk {
            n_patients: 200,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 123,
        },
    });

    p.add_stage(Stage {
        name: "AUC Trapezoidal".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::AucTrapezoidal { t_max: 24.0 },
    });

    p.add_stage(Stage {
        name: "Diversity + Bray-Curtis".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BrayCurtis {
            communities: vec![
                vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03],
                vec![0.50, 0.20, 0.10, 0.08, 0.06, 0.04, 0.02],
                vec![0.10, 0.10, 0.30, 0.20, 0.15, 0.10, 0.05],
            ],
        },
    });

    p.add_stage(Stage {
        name: "Diagnostic Reduce (CPU)".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Mean,
        },
    });

    p
}

// ── Topology description ─────────────────────────────────────────────

fn build_topology() -> (Vec<TopologyNode>, Vec<TopologyTransfer>) {
    let nodes = vec![TopologyNode {
        node_id: 0,
        pcie_gen: "Gen4".into(),
        nests: vec![
            TopologyNest {
                device_id: 0,
                substrate: "cpu".into(),
                label: "CPU (16 cores)".into(),
                memory_total_bytes: 64 * 1024 * 1024 * 1024,
                memory_used_bytes: 8 * 1024 * 1024 * 1024,
                utilization_pct: 15.0,
                available: true,
            },
            TopologyNest {
                device_id: 1,
                substrate: "gpu".into(),
                label: "GPU (RTX 4090)".into(),
                memory_total_bytes: 24 * 1024 * 1024 * 1024,
                memory_used_bytes: 2 * 1024 * 1024 * 1024,
                utilization_pct: 0.0,
                available: true,
            },
            TopologyNest {
                device_id: 2,
                substrate: "npu".into(),
                label: "NPU (Akida)".into(),
                memory_total_bytes: 256 * 1024 * 1024,
                memory_used_bytes: 0,
                utilization_pct: 0.0,
                available: true,
            },
        ],
    }];
    let transfers = vec![TopologyTransfer {
        src_id: "T0.N0.D2".into(),
        dst_id: "T0.N0.D1".into(),
        label: "PCIe P2P Gen4 31.5 GB/s".into(),
    }];
    (nodes, transfers)
}

// ── Stage results collector ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct StageLog {
    index: usize,
    name: String,
    elapsed_us: f64,
    output_elements: usize,
    success: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DashboardReport {
    pipeline_name: String,
    stages: Vec<StageLog>,
    total_time_us: f64,
    all_success: bool,
    petaltongue_connected: bool,
}

// ── Main ─────────────────────────────────────────────────────────────

#[expect(
    clippy::too_many_lines,
    reason = "dashboard orchestrator — linear setup, streaming execution, and reporting"
)]
fn main() {
    println!("=== exp072: Compute Dashboard — toadStool × petalTongue ===\n");

    let pipeline = build_clinical_pipeline();
    let (topo_nodes, topo_transfers) = build_topology();

    // Build topology scenario for initial render
    let (topo_scenario, topo_edges) = topology_scenario(0, &topo_nodes, &topo_transfers);

    // Try to connect to petalTongue
    let session: Option<Arc<Mutex<StreamSession>>> = match StreamSession::discover(SESSION_ID) {
        Ok(s) => {
            println!("[dashboard] petalTongue found — streaming live\n");
            Some(Arc::new(Mutex::new(s)))
        }
        Err(PushError::NotFound(_)) => {
            println!("[dashboard] petalTongue not running — stdout + JSON mode\n");
            None
        }
        Err(e) => {
            eprintln!("[dashboard] petalTongue discovery error: {e}");
            println!("[dashboard] Falling back to stdout + JSON mode\n");
            None
        }
    };

    // Push initial topology render
    if let Some(ref sess) = session {
        let mut s = sess.lock().expect("lock");
        if let Err(e) =
            s.push_initial_render("Compute Dashboard — NUCLEUS Topology", &topo_scenario)
        {
            eprintln!("[dashboard] topology push failed: {e}");
        } else {
            println!("[dashboard] Pushed NUCLEUS topology scenario");
        }
    }

    // Execute pipeline with streaming callbacks
    let stage_logs: Arc<Mutex<Vec<StageLog>>> = Arc::new(Mutex::new(Vec::new()));
    let logs_ref = Arc::clone(&stage_logs);
    let sess_ref = session.clone();

    #[expect(clippy::cast_precision_loss, reason = "stage index fits f64")]
    let result = pipeline.execute_streaming(|idx, total, stage_result| {
        let log = StageLog {
            index: idx,
            name: stage_result.stage_name.clone(),
            elapsed_us: stage_result.elapsed_us,
            output_elements: stage_result.output_data.len(),
            success: stage_result.success,
        };

        let progress_pct = ((idx + 1) as f64 / total as f64) * 100.0;
        let status = if stage_result.success { "OK" } else { "FAIL" };
        println!(
            "  [{}/{}] {:<30} {:>8.1} μs  {:>6} elements  [{}]  ({:.0}%)",
            idx + 1,
            total,
            stage_result.stage_name,
            stage_result.elapsed_us,
            stage_result.output_data.len(),
            status,
            progress_pct,
        );

        // Push progress gauge to petalTongue
        if let Some(ref sess) = sess_ref {
            let mut s = sess.lock().expect("lock");
            let _ = s.push_gauge("pipeline_progress", progress_pct);
            let _ = s.push_gauge("stage_time_us", stage_result.elapsed_us);
        }

        logs_ref.lock().expect("lock").push(log);
    });

    println!();

    // Build dispatch scenario from actual execution results
    let (dispatch_stages, logs_clone) = {
        let logs = stage_logs.lock().expect("lock");
        let stages: Vec<DispatchStageInfo> = logs
            .iter()
            .map(|l| DispatchStageInfo {
                name: l.name.clone(),
                substrate: infer_substrate(&l.name),
                elapsed_us: l.elapsed_us,
                output_elements: l.output_elements,
            })
            .collect();
        let cloned = logs.clone();
        drop(logs);
        (stages, cloned)
    };

    let (dispatch_scn, dispatch_edges) =
        dispatch_scenario("Mixed Clinical Pipeline — Execution", &dispatch_stages);

    // Push dispatch scenario to petalTongue
    if let Some(ref sess) = session {
        let mut s = sess.lock().expect("lock");
        if let Err(e) = s.push_initial_render("Compute Dashboard — Dispatch Plan", &dispatch_scn)
        {
            eprintln!("[dashboard] dispatch scenario push failed: {e}");
        } else {
            println!("[dashboard] Pushed dispatch plan scenario");
        }
        let stats = s.stats();
        let (frames, errors, cooldowns) = (stats.frames_pushed, stats.errors, stats.cooldowns);
        let avg_latency = stats.avg_push_latency();
        drop(s);
        println!(
            "[dashboard] StreamSession: {frames} frames, {errors} errors, {cooldowns} cooldowns",
        );
        if let Some(avg) = avg_latency {
            println!("[dashboard] Avg push latency: {avg:?}");
        }
    }

    let pt_connected = session.is_some();

    // Build report
    let report = DashboardReport {
        pipeline_name: "Mixed Clinical Pipeline".into(),
        stages: logs_clone,
        total_time_us: result.total_time_us,
        all_success: result.success,
        petaltongue_connected: pt_connected,
    };

    // Write JSON artifacts
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sandbox/dashboard");
    if std::fs::create_dir_all(&out_dir).is_err() {
        eprintln!("FAIL: create sandbox/dashboard/");
        std::process::exit(1);
    }

    let report_json = serde_json::to_string_pretty(&report).unwrap_or_default();
    if std::fs::write(out_dir.join("dashboard_report.json"), &report_json).is_err() {
        eprintln!("FAIL: write report");
        std::process::exit(1);
    }

    let topo_json = scenario_with_edges_json(&topo_scenario, &topo_edges);
    if std::fs::write(out_dir.join("topology.json"), &topo_json).is_err() {
        eprintln!("FAIL: write topology");
        std::process::exit(1);
    }

    let dispatch_json = scenario_with_edges_json(&dispatch_scn, &dispatch_edges);
    if std::fs::write(out_dir.join("dispatch.json"), &dispatch_json).is_err() {
        eprintln!("FAIL: write dispatch");
        std::process::exit(1);
    }

    println!(
        "[dashboard] JSON artifacts written to {}",
        out_dir.display()
    );

    // Validation checks
    println!("\n--- Validation ---");
    let mut passed = 0u32;
    let total = 8u32;

    let check = |name: &str, ok: bool, passed: &mut u32| {
        if ok {
            println!("  [PASS] {name}");
            *passed += 1;
        } else {
            println!("  [FAIL] {name}");
        }
    };

    check("pipeline_success", result.success, &mut passed);
    check(
        "all_stages_ran",
        result.stage_results.len() == 7,
        &mut passed,
    );
    check(
        "total_time_positive",
        result.total_time_us > 0.0,
        &mut passed,
    );
    check(
        "dispatch_scenario_has_nodes",
        dispatch_scn.ecosystem.primals.len() == 8,
        &mut passed,
    );
    check(
        "topology_scenario_has_nests",
        topo_scenario.ecosystem.primals.len() == 3,
        &mut passed,
    );
    check(
        "report_json_valid",
        serde_json::from_str::<serde_json::Value>(&report_json).is_ok(),
        &mut passed,
    );
    check(
        "topo_json_valid",
        serde_json::from_str::<serde_json::Value>(&topo_json).is_ok(),
        &mut passed,
    );
    check(
        "dispatch_json_valid",
        serde_json::from_str::<serde_json::Value>(&dispatch_json).is_ok(),
        &mut passed,
    );

    println!(
        "\nExp072 Compute Dashboard: {passed}/{total} checks passed{}",
        if pt_connected {
            " (live petalTongue)"
        } else {
            " (offline mode)"
        }
    );

    std::process::exit(i32::from(passed != total));
}

fn infer_substrate(stage_name: &str) -> String {
    if stage_name.contains("NPU") || stage_name.contains("Fusion") {
        "npu".into()
    } else if stage_name.contains("GPU") {
        "gpu".into()
    } else {
        "cpu".into()
    }
}
