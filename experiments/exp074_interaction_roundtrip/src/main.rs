// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear protocol format checks"
)]
//! Validates the petalTongue JSON-RPC protocol format for interaction subscribe
//! and capabilities roundtrip.
//!
//! Exercises the protocol builders (render, append, gauge, replace, capabilities,
//! interact.subscribe) and validates request structure, method naming, and params
//! without requiring a live socket.

use healthspring_barracuda::validation::ValidationHarness;
use healthspring_barracuda::visualization::ipc_push::protocol::{
    build_append_params, build_gauge_params, build_render_with_config_params, build_replace_params,
};
use healthspring_barracuda::visualization::{
    Animations, CapReqs, DataChannel, Ecosystem, HealthScenario, NeuralApi, Performance,
    ScenarioNode, SensoryConfig, UiConfig,
};

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

fn build_jsonrpc_request(method: &str, params: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
}

fn run_validation(h: &mut ValidationHarness) {
    let scenario = minimal_scenario();
    let session_id = "exp074-test";

    // 1. visualization.render (push_render_with_domain)
    let render_params =
        build_render_with_config_params(session_id, "Roundtrip Test", &scenario, "health");
    let render_req = build_jsonrpc_request("visualization.render", &render_params);

    h.check_bool(
        "render_request_has_jsonrpc",
        render_req["jsonrpc"].as_str() == Some("2.0"),
    );
    h.check_bool(
        "render_request_has_method",
        render_req["method"].as_str() == Some("visualization.render"),
    );
    h.check_bool("render_request_has_id", render_req["id"].as_i64().is_some());
    h.check_bool(
        "render_params_has_session_id",
        render_params["session_id"].as_str() == Some(session_id),
    );
    h.check_bool(
        "render_params_has_title",
        render_params["title"].as_str() == Some("Roundtrip Test"),
    );
    h.check_bool(
        "render_params_has_domain",
        render_params["domain"].as_str() == Some("health"),
    );
    h.check_bool(
        "render_params_has_bindings",
        render_params.get("bindings").is_some(),
    );
    h.check_bool(
        "render_params_has_thresholds",
        render_params.get("thresholds").is_some(),
    );
    h.check_bool(
        "render_params_has_ui_config",
        render_params.get("ui_config").is_some(),
    );

    // 2. visualization.render.stream append (push_timeseries)
    let append_params = build_append_params(session_id, "pk_curve", &[1.0, 2.0], &[10.0, 20.0]);
    let append_req = build_jsonrpc_request("visualization.render.stream", &append_params);

    h.check_bool(
        "append_request_has_method",
        append_req["method"].as_str() == Some("visualization.render.stream"),
    );
    h.check_bool(
        "append_params_operation_type",
        append_params["operation"]["type"].as_str() == Some("append"),
    );
    h.check_bool(
        "append_params_binding_id",
        append_params["binding_id"].as_str() == Some("pk_curve"),
    );

    // 3. visualization.render.stream gauge (push_gauge)
    let gauge_params = build_gauge_params(session_id, "hr_gauge", 72.0);
    h.check_bool(
        "gauge_params_operation_type",
        gauge_params["operation"]["type"].as_str() == Some("set_value"),
    );
    h.check_bool(
        "gauge_params_value",
        gauge_params["operation"]["value"].as_f64() == Some(72.0),
    );

    // 4. visualization.render.stream replace (push_replace_binding)
    let bar = DataChannel::Bar {
        id: "risk_compare".into(),
        label: "Risk Comparison".into(),
        categories: vec!["Baseline".into(), "Week 4".into()],
        values: vec![0.8, 0.5],
        unit: "composite".into(),
    };
    let replace_params = match build_replace_params(session_id, "risk_compare", &bar) {
        Ok(p) => p,
        Err(e) => {
            h.check_bool("replace_params_build_ok", false);
            eprintln!("replace params build failed: {e}");
            h.exit();
        }
    };
    h.check_bool("replace_params_build_ok", true);
    h.check_bool(
        "replace_params_operation_type",
        replace_params["operation"]["type"].as_str() == Some("replace"),
    );
    h.check_bool(
        "replace_params_has_binding",
        replace_params["operation"].get("binding").is_some(),
    );

    // 5. visualization.capabilities
    let caps_params = serde_json::json!({});
    let caps_req = build_jsonrpc_request("visualization.capabilities", &caps_params);
    h.check_bool(
        "capabilities_request_method",
        caps_req["method"].as_str() == Some("visualization.capabilities"),
    );

    // 6. visualization.interact.subscribe
    let subscribe_params = serde_json::json!({
        "grammar_id": session_id,
        "events": ["select", "focus", "filter"],
        "callback_method": "healthspring.on_interact",
    });
    let subscribe_req =
        build_jsonrpc_request("visualization.interact.subscribe", &subscribe_params);
    h.check_bool(
        "subscribe_request_method",
        subscribe_req["method"].as_str() == Some("visualization.interact.subscribe"),
    );
    h.check_bool(
        "subscribe_params_grammar_id",
        subscribe_params["grammar_id"].as_str() == Some(session_id),
    );
    h.check_bool(
        "subscribe_params_events_array",
        subscribe_params["events"].as_array().map_or(0, Vec::len) == 3,
    );
    h.check_bool(
        "subscribe_params_callback_method",
        subscribe_params["callback_method"].as_str() == Some("healthspring.on_interact"),
    );
}

fn main() {
    let mut h = ValidationHarness::new("Exp074 Interaction Roundtrip");
    run_validation(&mut h);
    h.exit();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    use std::sync::{Arc, Barrier};

    use healthspring_barracuda::visualization::ipc_push::PetalTonguePushClient;
    use healthspring_barracuda::visualization::stream::StreamSession;

    /// Test-only mock petalTongue server for IPC roundtrip validation.
    ///
    /// Responds to `visualization.render`, `visualization.capabilities`, and
    /// `visualization.interact.subscribe` with canned JSON-RPC 2.0 responses.
    fn mock_server(listener: &UnixListener, expected_calls: usize) -> Vec<serde_json::Value> {
        let mut requests = Vec::with_capacity(expected_calls);
        for _ in 0..expected_calls {
            let (mut stream, _) = match listener.accept() {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("FAIL: accept: {e}");
                    continue;
                }
            };
            let mut payload = Vec::new();
            let mut chunk = vec![0u8; 65_536];
            loop {
                let n = match stream.read(&mut chunk) {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("FAIL: read: {e}");
                        break;
                    }
                };
                if n == 0 {
                    break;
                }
                payload.extend_from_slice(&chunk[..n]);
                if n < chunk.len() {
                    break;
                }
            }
            let request: serde_json::Value = match serde_json::from_slice(&payload) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("FAIL: parse request: {e}");
                    continue;
                }
            };

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
            if stream
                .write_all(serde_json::to_vec(&response).unwrap_or_default().as_slice())
                .is_err()
            {
                eprintln!("FAIL: write");
            }
            requests.push(request);
        }
        requests
    }

    #[test]
    fn mock_server_roundtrip_receives_expected_requests() {
        let sock_dir = std::env::temp_dir().join(format!(
            "hs_exp074_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.subsec_nanos())
        ));
        std::fs::create_dir_all(&sock_dir).expect("create socket dir");
        let sock_path = sock_dir.join("mock_pt.sock");
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).expect("bind");

        let n_calls = 9;
        let barrier = Arc::new(Barrier::new(2));

        let b = barrier.clone();
        let handle = std::thread::spawn(move || {
            b.wait();
            mock_server(&listener, n_calls)
        });

        barrier.wait();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let client = PetalTonguePushClient::new(sock_path.clone());
        let mut session = StreamSession::new(client, "exp074-test");

        let scenario = minimal_scenario();
        let _ = session.push_render_with_domain("Roundtrip Test", &scenario, "health");
        let _ = session.push_timeseries("pk_curve", &[1.0, 2.0], &[10.0, 20.0]);
        let _ = session.push_gauge("hr_gauge", 72.0);
        let bar = DataChannel::Bar {
            id: "risk_compare".into(),
            label: "Risk Comparison".into(),
            categories: vec!["Baseline".into(), "Week 4".into()],
            values: vec![0.8, 0.5],
            unit: "composite".into(),
        };
        let _ = session.push_replace_binding("risk_compare", &bar);
        let _ = session.push_hrv_update(55.0, 42.0, 18.0);
        let _ = session.query_capabilities();
        let _ = session
            .subscribe_interactions(&["select", "focus", "filter"], "healthspring.on_interact");

        let requests = handle.join().expect("mock thread");

        std::fs::remove_file(&sock_path).ok();
        std::fs::remove_dir(&sock_dir).ok();

        assert_eq!(requests.len(), n_calls, "expected {n_calls} requests");
        assert_eq!(
            requests[0]["method"], "visualization.render",
            "first request should be render"
        );
        assert_eq!(
            requests[3]["method"], "visualization.render.stream",
            "replace should use render.stream"
        );
        assert_eq!(
            requests[3]["params"]["operation"]["type"], "replace",
            "replace operation type"
        );
        assert_eq!(
            requests[7]["method"], "visualization.capabilities",
            "capabilities method"
        );
        assert_eq!(
            requests[8]["method"], "visualization.interact.subscribe",
            "subscribe method"
        );
    }
}
