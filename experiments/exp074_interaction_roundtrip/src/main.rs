// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Validates the petalTongue interaction subscribe + capabilities roundtrip.
//!
//! Spins up a mock petalTongue Unix socket server, then exercises the full
//! IPC flow: discover → render → append → replace → gauge →
//! capabilities query → interaction subscription.

use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Barrier};

use healthspring_barracuda::visualization::ipc_push::PetalTonguePushClient;
use healthspring_barracuda::visualization::stream::StreamSession;
use healthspring_barracuda::visualization::{
    Animations, CapReqs, DataChannel, Ecosystem, HealthScenario, NeuralApi, Performance,
    ScenarioNode, SensoryConfig, UiConfig,
};

fn mock_server(listener: &UnixListener, expected_calls: usize) -> Vec<serde_json::Value> {
    let mut requests = Vec::with_capacity(expected_calls);
    for _ in 0..expected_calls {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut payload = Vec::new();
        let mut chunk = vec![0u8; 65_536];
        loop {
            let n = stream.read(&mut chunk).expect("read");
            if n == 0 {
                break;
            }
            payload.extend_from_slice(&chunk[..n]);
            if n < chunk.len() {
                break;
            }
        }
        let request: serde_json::Value = serde_json::from_slice(&payload).expect("parse request");

        let method = request["method"].as_str().unwrap_or("");
        let response = match method {
            "visualization.capabilities" => serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "supported_channels": ["timeseries", "distribution", "bar", "gauge", "spectrum", "heatmap", "scatter3d"],
                    "max_bindings": 128,
                    "streaming": true,
                    "interaction": true,
                },
                "id": request["id"],
            }),
            "visualization.interact.subscribe" => serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "subscription_id": "sub-001",
                    "events": request["params"]["events"],
                },
                "id": request["id"],
            }),
            _ => serde_json::json!({
                "jsonrpc": "2.0",
                "result": "ok",
                "id": request["id"],
            }),
        };
        stream
            .write_all(serde_json::to_vec(&response).unwrap().as_slice())
            .expect("write");
        requests.push(request);
    }
    requests
}

#[expect(
    clippy::too_many_lines,
    reason = "interaction roundtrip orchestrator — linear sequence of mock server + client calls"
)]
fn main() {
    println!("=== exp074: Interaction Roundtrip ===\n");

    let sock_dir = std::env::temp_dir().join(format!(
        "hs_exp074_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .subsec_nanos()
    ));
    std::fs::create_dir_all(&sock_dir).expect("create socket dir");
    let sock_path = sock_dir.join("mock_pt.sock");
    let _ = std::fs::remove_file(&sock_path);

    let listener = UnixListener::bind(&sock_path).expect("bind");
    println!("[mock] Listening on {}", sock_path.display());

    let n_calls = 9;
    let barrier = Arc::new(Barrier::new(2));

    let b = barrier.clone();
    let handle = std::thread::spawn(move || {
        b.wait();
        mock_server(&listener, n_calls)
    });

    barrier.wait();

    // Give the mock a moment to start accepting
    std::thread::sleep(std::time::Duration::from_millis(10));

    let client = PetalTonguePushClient::new(sock_path.clone());
    let mut session = StreamSession::new(client, "exp074-test");

    // 1. Push initial render (uses push_render_with_domain)
    let scenario = minimal_scenario();
    let r1 = session.push_render_with_domain("Roundtrip Test", &scenario, "health");
    println!("[client] push_render_with_domain: {}", status(&r1));

    // 2. Push timeseries append
    let r2 = session.push_timeseries("pk_curve", &[1.0, 2.0], &[10.0, 20.0]);
    println!("[client] push_timeseries: {}", status(&r2));

    // 3. Push gauge
    let r3 = session.push_gauge("hr_gauge", 72.0);
    println!("[client] push_gauge: {}", status(&r3));

    // 4. Push replace (Bar channel)
    let bar = DataChannel::Bar {
        id: "risk_compare".into(),
        label: "Risk Comparison".into(),
        categories: vec!["Baseline".into(), "Week 4".into()],
        values: vec![0.8, 0.5],
        unit: "composite".into(),
    };
    let r4 = session.push_replace_binding("risk_compare", &bar);
    println!("[client] push_replace_binding: {}", status(&r4));

    // 5. Push HRV update (3 gauge pushes)
    let r5 = session.push_hrv_update(55.0, 42.0, 18.0);
    println!("[client] push_hrv_update: {}", status(&r5));

    // 6. Query capabilities
    let r6 = session.query_capabilities();
    println!("[client] query_capabilities: {}", status_val(&r6));

    // 7. Subscribe interactions
    let r7 =
        session.subscribe_interactions(&["select", "focus", "filter"], "healthspring.on_interact");
    println!("[client] subscribe_interactions: {}", status_val(&r7));

    let requests = handle.join().expect("mock thread");
    println!("\n[mock] Received {} requests\n", requests.len());

    // Cleanup
    std::fs::remove_file(&sock_path).ok();
    std::fs::remove_dir(&sock_dir).ok();

    // Validation
    println!("--- Validation ---");
    let mut passed = 0u32;
    let total = 12u32;

    let check = |name: &str, ok: bool, passed: &mut u32| {
        if ok {
            println!("  [PASS] {name}");
            *passed += 1;
        } else {
            println!("  [FAIL] {name}");
        }
    };

    check("render_ok", r1.is_ok(), &mut passed);
    check("timeseries_ok", r2.is_ok(), &mut passed);
    check("gauge_ok", r3.is_ok(), &mut passed);
    check("replace_ok", r4.is_ok(), &mut passed);
    check("hrv_ok", r5.is_ok(), &mut passed);
    check("capabilities_ok", r6.is_ok(), &mut passed);
    check("subscribe_ok", r7.is_ok(), &mut passed);
    check("total_requests", requests.len() == n_calls, &mut passed);

    // Validate request methods
    check(
        "render_method",
        requests[0]["method"] == "visualization.render",
        &mut passed,
    );
    check(
        "replace_method",
        requests[3]["method"] == "visualization.render.stream"
            && requests[3]["params"]["operation"]["type"] == "replace",
        &mut passed,
    );
    check(
        "caps_method",
        requests[7]["method"] == "visualization.capabilities",
        &mut passed,
    );
    check(
        "subscribe_method",
        requests[8]["method"] == "visualization.interact.subscribe",
        &mut passed,
    );

    // Validate capabilities response
    if let Ok(caps) = &r6 {
        let channels = caps["result"]["supported_channels"]
            .as_array()
            .map_or(0, Vec::len);
        println!("  [INFO] petalTongue supports {channels} channel types");
    }

    // Validate subscribe response
    if let Ok(sub) = &r7 {
        let sub_id = sub["result"]["subscription_id"].as_str().unwrap_or("none");
        println!("  [INFO] subscription_id = {sub_id}");
    }

    // Session stats
    let stats = session.stats();
    println!(
        "\n[stats] frames={}, errors={}, cooldowns={}",
        stats.frames_pushed, stats.errors, stats.cooldowns,
    );

    println!("\nExp074 Interaction Roundtrip: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}

const fn status<T>(r: &Result<T, impl std::fmt::Display>) -> &'static str {
    if r.is_ok() { "ok" } else { "FAIL" }
}

fn status_val(r: &Result<serde_json::Value, impl std::fmt::Display>) -> String {
    match r {
        Ok(v) => format!("ok ({})", v.to_string().len()),
        Err(e) => format!("FAIL: {e}"),
    }
}

fn minimal_scenario() -> HealthScenario {
    HealthScenario {
        name: "exp074-roundtrip".into(),
        description: "Minimal scenario for interaction roundtrip validation".into(),
        version: "1.0".into(),
        mode: "test".into(),
        sensory_config: SensoryConfig {
            required_capabilities: CapReqs {
                outputs: vec!["visual".into()],
                inputs: vec![],
            },
            optional_capabilities: CapReqs {
                outputs: vec![],
                inputs: vec![],
            },
            complexity_hint: "minimal".into(),
        },
        ui_config: UiConfig {
            theme: "dark".into(),
            animations: Animations {
                enabled: false,
                breathing_nodes: false,
                connection_pulses: false,
                smooth_transitions: false,
                celebration_effects: false,
            },
            performance: Performance {
                target_fps: 60,
                vsync: true,
                hardware_acceleration: false,
            },
            show_panels: None,
            awakening_enabled: None,
            initial_zoom: None,
        },
        ecosystem: Ecosystem {
            primals: vec![ScenarioNode {
                id: "test-node".into(),
                name: "Test".into(),
                node_type: "compute".into(),
                family: "test".into(),
                status: "ok".into(),
                health: 100,
                confidence: 95,
                position: None,
                capabilities: vec![],
                data_channels: vec![DataChannel::Gauge {
                    id: "test_gauge".into(),
                    label: "Test".into(),
                    value: 50.0,
                    min: 0.0,
                    max: 100.0,
                    unit: "%".into(),
                    normal_range: [0.0, 80.0],
                    warning_range: [80.0, 95.0],
                }],
                clinical_ranges: vec![],
            }],
        },
        neural_api: NeuralApi { enabled: false },
        edges: vec![],
    }
}
