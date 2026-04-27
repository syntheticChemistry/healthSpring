// SPDX-License-Identifier: AGPL-3.0-or-later
//! Push visualization data to petalTongue via JSON-RPC IPC.
//!
//! Springs discover petalTongue at runtime and push `DataChannel` payloads
//! without compile-time coupling. Uses the `visualization.render` and
//! `visualization.render.stream` JSON-RPC methods.

mod client;
pub mod protocol;

pub use super::types::DataChannel;
pub use client::PetalTonguePushClient;

/// Result type for push operations
pub type PushResult<T> = Result<T, PushError>;

/// Error type for push operations
#[derive(Debug)]
pub enum PushError {
    /// petalTongue socket not found
    NotFound(String),
    /// Connection failed
    ConnectionFailed(std::io::Error),
    /// JSON serialization error
    SerializationError(String),
    /// RPC error response from petalTongue.
    RpcError {
        /// JSON-RPC or application error code.
        code: i64,
        /// Error message from the peer.
        message: String,
    },
}

impl std::fmt::Display for PushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "visualization primal not found: {msg}"),
            Self::ConnectionFailed(e) => write!(f, "connection failed: {e}"),
            Self::SerializationError(e) => write!(f, "serialization error: {e}"),
            Self::RpcError { code, message } => write!(f, "RPC error {code}: {message}"),
        }
    }
}

impl std::error::Error for PushError {}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::tolerances;
    use std::path::PathBuf;

    use super::super::types::{
        Animations, CapReqs, ClinicalRange, ClinicalStatus, DataChannel as DataChannelType,
        Ecosystem, HealthScenario, NeuralApi, NodeStatus, NodeType, Performance, ScenarioNode,
        SensoryConfig, UiConfig,
    };

    fn minimal_scenario() -> HealthScenario {
        let primal = ScenarioNode {
            id: "node-1".into(),
            name: "Test Node".into(),
            node_type: NodeType::Compute,
            family: "test".into(),
            status: NodeStatus::Healthy,
            health: 100,
            confidence: 0,
            position: None,
            capabilities: vec![],
            data_channels: vec![DataChannelType::Gauge {
                id: "gauge-1".into(),
                label: "Test Gauge".into(),
                value: 42.0,
                min: 0.0,
                max: 100.0,
                unit: "unit".into(),
                normal_range: [0.0, 50.0],
                warning_range: [50.0, 80.0],
            }],
            clinical_ranges: vec![ClinicalRange {
                label: "normal".into(),
                min: 0.0,
                max: 50.0,
                status: ClinicalStatus::Normal,
            }],
        };
        HealthScenario {
            name: "test".into(),
            description: "test scenario".into(),
            version: "1.0".into(),
            mode: "live".into(),
            sensory_config: SensoryConfig {
                required_capabilities: CapReqs {
                    outputs: vec![],
                    inputs: vec![],
                },
                optional_capabilities: CapReqs {
                    outputs: vec![],
                    inputs: vec![],
                },
                complexity_hint: "test".into(),
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
                primals: vec![primal],
            },
            neural_api: NeuralApi { enabled: false },
            edges: vec![],
        }
    }

    #[test]
    fn push_error_display_not_found() {
        let e = PushError::NotFound("no socket".into());
        let s = format!("{e}");
        assert!(s.contains("visualization primal not found"));
        assert!(s.contains("no socket"));
    }

    #[test]
    fn push_error_display_connection_failed() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "connection refused");
        let e = PushError::ConnectionFailed(io_err);
        let s = format!("{e}");
        assert!(s.contains("connection failed"));
    }

    #[test]
    fn push_error_display_serialization_error() {
        let e = PushError::SerializationError("invalid json".into());
        let s = format!("{e}");
        assert!(s.contains("serialization error"));
        assert!(s.contains("invalid json"));
    }

    #[test]
    fn push_error_display_rpc_error() {
        let e = PushError::RpcError {
            code: -32600,
            message: "invalid request".into(),
        };
        let s = format!("{e}");
        assert!(s.contains("RPC error"));
        assert!(s.contains("-32600"));
        assert!(s.contains("invalid request"));
    }

    #[test]
    fn push_error_impl_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<PushError>();
    }

    #[test]
    fn client_new_stores_path() {
        let path = PathBuf::from("/tmp/test-socket.sock");
        let client = PetalTonguePushClient::new(path.clone());
        assert_eq!(client.socket_path(), &path);
    }

    #[test]
    fn build_render_params_structure() {
        let scenario = minimal_scenario();
        let params = protocol::build_render_params("sess-123", "My Title", &scenario);

        assert_eq!(
            params.get("session_id").and_then(|v| v.as_str()),
            Some("sess-123")
        );
        assert_eq!(
            params.get("title").and_then(|v| v.as_str()),
            Some("My Title")
        );
        assert_eq!(
            params.get("domain").and_then(|v| v.as_str()),
            Some("health")
        );
        assert!(params.get("bindings").is_some());
        assert!(params.get("thresholds").is_some());

        let bindings = params.get("bindings").unwrap().as_array().unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(
            bindings[0].get("channel_type").and_then(|v| v.as_str()),
            Some("gauge")
        );
    }

    #[test]
    fn build_append_params_structure() {
        let params = protocol::build_append_params(
            "sess-456",
            "binding-1",
            &[1.0, 2.0, 3.0],
            &[10.0, 20.0, 30.0],
        );

        assert_eq!(
            params.get("session_id").and_then(|v| v.as_str()),
            Some("sess-456")
        );
        assert_eq!(
            params.get("binding_id").and_then(|v| v.as_str()),
            Some("binding-1")
        );
        let op = params.get("operation").unwrap();
        assert_eq!(op.get("type").and_then(|v| v.as_str()), Some("append"));
        let xs = op.get("x_values").and_then(|v| v.as_array()).unwrap();
        let ys = op.get("y_values").and_then(|v| v.as_array()).unwrap();
        assert_eq!(xs.len(), 3);
        assert_eq!(ys.len(), 3);
        assert!((xs[0].as_f64().unwrap() - 1.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert!((ys[1].as_f64().unwrap() - 20.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn build_gauge_params_structure() {
        let params = protocol::build_gauge_params("sess-789", "gauge-binding", 73.5);

        assert_eq!(
            params.get("session_id").and_then(|v| v.as_str()),
            Some("sess-789")
        );
        assert_eq!(
            params.get("binding_id").and_then(|v| v.as_str()),
            Some("gauge-binding")
        );
        let op = params.get("operation").unwrap();
        assert_eq!(op.get("type").and_then(|v| v.as_str()), Some("set_value"));
        assert_eq!(
            op.get("value").and_then(serde_json::Value::as_f64),
            Some(73.5)
        );
    }

    fn mock_petaltongue_response(listener: &std::os::unix::net::UnixListener) -> serde_json::Value {
        use std::io::{Read, Write};
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = vec![0u8; 8192];
        let n = stream.read(&mut buf).expect("read");
        let request: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse request");
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": "ok",
            "id": 1,
        });
        stream
            .write_all(serde_json::to_vec(&response).unwrap().as_slice())
            .expect("write response");
        request
    }

    fn mock_petaltongue_error(listener: &std::os::unix::net::UnixListener) {
        use std::io::{Read, Write};
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).expect("read");
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32600, "message": "test error" },
            "id": 1,
        });
        stream
            .write_all(serde_json::to_vec(&response).unwrap().as_slice())
            .expect("write response");
    }

    fn socket_test_setup(name: &str) -> (PathBuf, std::os::unix::net::UnixListener) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("hs_ipc_{name}_{}_{}", std::process::id(), seq));
        std::fs::create_dir_all(&dir).ok();
        let sock_path = dir.join(format!("{name}.sock"));
        let _ = std::fs::remove_file(&sock_path);
        let listener = std::os::unix::net::UnixListener::bind(&sock_path).expect("bind");
        (sock_path, listener)
    }

    fn socket_test_cleanup(sock_path: &std::path::Path) {
        std::fs::remove_file(sock_path).ok();
        if let Some(parent) = sock_path.parent() {
            std::fs::remove_dir(parent).ok();
        }
    }

    /// Run a socket test: spawn mock, send, verify. The `UnixListener` is
    /// already bound before threads, so the kernel backlog queues the client
    /// `connect()` until the mock calls `accept()` — no barrier needed.
    fn run_socket_test<F, R>(name: &str, setup_client: F) -> (serde_json::Value, R)
    where
        F: FnOnce(&PetalTonguePushClient) -> R,
    {
        let (sock_path, listener) = socket_test_setup(name);
        let client = PetalTonguePushClient::new(sock_path.clone());
        let handle = std::thread::spawn(move || mock_petaltongue_response(&listener));
        let result = setup_client(&client);
        let request = handle.join().expect("mock thread");
        socket_test_cleanup(&sock_path);
        (request, result)
    }

    #[test]
    fn push_render_sends_valid_jsonrpc() {
        let scenario = minimal_scenario();
        let (request, result) = run_socket_test("render", |c| {
            c.push_render("sess-1", "Test Render", &scenario)
        });

        assert!(result.is_ok());
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "visualization.render");
        assert_eq!(request["params"]["session_id"], "sess-1");
        assert_eq!(request["params"]["title"], "Test Render");
    }

    #[test]
    fn push_append_sends_valid_jsonrpc() {
        let (request, result) = run_socket_test("append", |c| {
            c.push_append("sess-2", "bind-1", &[1.0, 2.0], &[10.0, 20.0])
        });

        assert!(result.is_ok());
        assert_eq!(request["method"], "visualization.render.stream");
        assert_eq!(request["params"]["binding_id"], "bind-1");
        assert_eq!(request["params"]["operation"]["type"], "append");
    }

    #[test]
    fn push_gauge_update_sends_valid_jsonrpc() {
        let (request, result) =
            run_socket_test("gauge", |c| c.push_gauge_update("sess-3", "gauge-1", 42.5));

        assert!(result.is_ok());
        assert_eq!(request["method"], "visualization.render.stream");
        assert_eq!(request["params"]["operation"]["type"], "set_value");
        assert_eq!(request["params"]["operation"]["value"], 42.5);
    }

    #[test]
    fn push_returns_rpc_error_on_error_response() {
        let (sock_path, listener) = socket_test_setup("rpc_err");
        let client = PetalTonguePushClient::new(sock_path.clone());
        let handle = std::thread::spawn(move || mock_petaltongue_error(&listener));
        let result = client.push_gauge_update("sess-err", "gauge-1", 1.0);
        handle.join().expect("mock thread");
        socket_test_cleanup(&sock_path);

        assert!(result.is_err());
        if let Err(PushError::RpcError { code, message }) = result {
            assert_eq!(code, -32600);
            assert_eq!(message, "test error");
        } else {
            panic!("expected RpcError");
        }
    }

    #[test]
    fn push_connection_failed_on_missing_socket() {
        let client = PetalTonguePushClient::new(PathBuf::from("/tmp/nonexistent_hs_test.sock"));
        let result = client.push_gauge_update("s", "b", 1.0);
        assert!(result.is_err());
        assert!(
            matches!(result, Err(PushError::ConnectionFailed(_))),
            "expected ConnectionFailed"
        );
    }

    #[test]
    fn discover_returns_not_found_when_no_socket_exists() {
        let result = PetalTonguePushClient::discover();
        if result.is_ok() {
            return;
        }
        assert!(
            matches!(result, Err(PushError::NotFound(_))),
            "expected NotFound"
        );
    }

    #[test]
    fn build_replace_params_structure() {
        let binding = DataChannelType::Bar {
            id: "bar-1".into(),
            label: "Test Bar".into(),
            categories: vec!["A".into(), "B".into()],
            values: vec![10.0, 20.0],
            unit: "kg".into(),
        };
        let params = protocol::build_replace_params("sess-rep", "bar-1", &binding).unwrap();

        assert_eq!(params["session_id"], "sess-rep");
        assert_eq!(params["binding_id"], "bar-1");
        let op = &params["operation"];
        assert_eq!(op["type"], "replace");
        assert!(op.get("binding").is_some());
        assert_eq!(op["binding"]["channel_type"], "bar");
        assert_eq!(op["binding"]["id"], "bar-1");
    }

    #[test]
    fn build_replace_params_heatmap() {
        let binding = DataChannelType::Heatmap {
            id: "hm-1".into(),
            label: "Test Heatmap".into(),
            x_labels: vec!["A".into(), "B".into()],
            y_labels: vec!["X".into(), "Y".into()],
            values: vec![1.0, 2.0, 3.0, 4.0],
            unit: "BC".into(),
        };
        let params = protocol::build_replace_params("sess-hm", "hm-1", &binding).unwrap();
        assert_eq!(params["operation"]["binding"]["channel_type"], "heatmap");
    }

    #[test]
    fn build_render_with_config_params_structure() {
        let scenario = minimal_scenario();
        let params = protocol::build_render_with_config_params(
            "sess-cfg",
            "Clinical Test",
            &scenario,
            "clinical",
        );

        assert_eq!(params["session_id"], "sess-cfg");
        assert_eq!(params["title"], "Clinical Test");
        assert_eq!(params["domain"], "clinical");
        assert!(params.get("ui_config").is_some());
        assert_eq!(params["ui_config"]["theme"], "dark");
    }

    #[test]
    fn push_replace_sends_valid_jsonrpc() {
        let binding = DataChannelType::Bar {
            id: "bar-1".into(),
            label: "Test".into(),
            categories: vec!["A".into()],
            values: vec![5.0],
            unit: "u".into(),
        };
        let (request, result) =
            run_socket_test("replace", |c| c.push_replace("sess-rep", "bar-1", &binding));

        assert!(result.is_ok());
        assert_eq!(request["method"], "visualization.render.stream");
        assert_eq!(request["params"]["operation"]["type"], "replace");
        assert!(request["params"]["operation"]["binding"].is_object());
    }

    #[test]
    fn push_render_with_config_sends_valid_jsonrpc() {
        let scenario = minimal_scenario();
        let (request, result) = run_socket_test("render_cfg", |c| {
            c.push_render_with_config("sess-cfg", "Clinical", &scenario, "clinical")
        });

        assert!(result.is_ok());
        assert_eq!(request["method"], "visualization.render");
        assert_eq!(request["params"]["domain"], "clinical");
        assert!(request["params"]["ui_config"].is_object());
    }

    #[test]
    fn query_capabilities_sends_valid_jsonrpc() {
        let (request, result) = run_socket_test("caps", PetalTonguePushClient::query_capabilities);

        assert!(result.is_ok());
        assert_eq!(request["method"], "visualization.capabilities");
    }

    #[test]
    fn subscribe_interactions_sends_valid_jsonrpc() {
        let (request, result) = run_socket_test("interact", |c| {
            c.subscribe_interactions("sess-int", &["select", "focus"], "healthspring.on_interact")
        });

        assert!(result.is_ok());
        assert_eq!(request["method"], "visualization.interact.subscribe");
        assert_eq!(request["params"]["grammar_id"], "sess-int");
        let events = request["params"]["events"].as_array().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            request["params"]["callback_method"],
            "healthspring.on_interact"
        );
    }
}
