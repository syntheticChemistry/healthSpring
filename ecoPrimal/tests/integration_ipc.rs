// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests use unwrap/expect for concise assertions"
)]
//! Integration tests for IPC infrastructure.
//!
//! Tests transport abstraction, socket discovery, capability extraction,
//! JSON-RPC envelope round-trips, and dispatch routing without requiring
//! a running primal server.

use healthspring_barracuda::ipc::{error::IpcError, protocol, rpc, socket, transport};

#[test]
fn rpc_success_roundtrip() {
    let id = serde_json::json!(42);
    let result = serde_json::json!({"shannon": 2.1});
    let resp_str = rpc::success(&id, &result);
    let parsed: serde_json::Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 42);
    assert_eq!(parsed["result"]["shannon"], 2.1);
}

#[test]
fn rpc_error_roundtrip() {
    let id = serde_json::json!(1);
    let resp_str = rpc::error(&id, rpc::METHOD_NOT_FOUND, "no such method");
    let parsed: serde_json::Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(parsed["error"]["code"], rpc::METHOD_NOT_FOUND);
    assert!(
        parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("no such method")
    );
}

#[test]
fn extract_result_from_success() {
    let resp = serde_json::json!({"jsonrpc": "2.0", "result": {"ok": true}, "id": 1});
    let result = rpc::extract_rpc_result(&resp);
    assert!(result.is_some());
    assert_eq!(result.unwrap()["ok"], true);
}

#[test]
fn extract_result_from_error_returns_none() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {"code": -32601, "message": "not found"},
        "id": 1
    });
    assert!(rpc::extract_rpc_result(&resp).is_none());
}

#[test]
fn try_send_to_nonexistent_socket_returns_connect_error() {
    let path = std::path::Path::new("/tmp/healthspring_integration_test_nonexistent.sock");
    let result = rpc::try_send(path, "health.liveness", &serde_json::json!({}));
    assert!(matches!(result, Err(IpcError::Connect(_))));
}

#[test]
fn transport_parse_endpoint_unix() {
    let ep = transport::parse_endpoint("unix:///run/biomeos/test.sock");
    assert!(matches!(ep, Ok(transport::Endpoint::Unix(_))));
}

#[test]
fn transport_parse_endpoint_tcp() {
    let ep = transport::parse_endpoint("tcp://127.0.0.1:9090");
    assert!(matches!(ep, Ok(transport::Endpoint::Tcp(_))));
}

#[test]
fn transport_parse_endpoint_path_infers_unix() {
    let ep = transport::parse_endpoint("/tmp/biomeos/healthspring-default.sock");
    assert!(matches!(ep, Ok(transport::Endpoint::Unix(_))));
}

#[test]
fn transport_parse_endpoint_host_port_infers_tcp() {
    let ep = transport::parse_endpoint("localhost:8080");
    assert!(matches!(ep, Ok(transport::Endpoint::Tcp(_))));
}

#[test]
fn transport_connect_nonexistent_unix_fails() {
    let ep = transport::Endpoint::Unix("/tmp/healthspring_integration_transport_test.sock".into());
    assert!(transport::connect(&ep).is_err());
}

#[test]
fn socket_discovery_returns_path_with_primal_name() {
    let path = socket::resolve_bind_path();
    assert!(path.to_string_lossy().contains("healthspring"));
}

#[test]
fn socket_dir_uses_xdg_or_fallback() {
    let dir = socket::resolve_socket_dir();
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.contains("biomeos"),
        "socket dir should contain 'biomeos': {dir_str}"
    );
}

#[test]
fn capability_extraction_all_formats() {
    let format_a = serde_json::json!({
        "science": ["science.pkpd.hill"],
        "infrastructure": ["lifecycle.health"]
    });
    let caps_a = socket::extract_capability_strings(&format_a);
    assert!(caps_a.contains(&"science.pkpd.hill"));
    assert!(caps_a.contains(&"lifecycle.health"));

    let format_b = serde_json::json!({"capabilities": ["compute.dispatch"]});
    let caps_b = socket::extract_capability_strings(&format_b);
    assert!(caps_b.contains(&"compute.dispatch"));

    let format_d = serde_json::json!(["model.infer", "model.load"]);
    let caps_d = socket::extract_capability_strings(&format_d);
    assert!(caps_d.contains(&"model.infer"));
}

#[test]
fn ipc_error_classification() {
    let connect_err = IpcError::Connect(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "refused",
    ));
    assert!(connect_err.is_retriable());
    assert!(connect_err.is_connection_error());

    let timeout_err = IpcError::Timeout(5000);
    assert!(timeout_err.is_retriable());
    assert!(timeout_err.is_timeout_likely());

    let reject_err = IpcError::RpcReject {
        code: -32601,
        message: "method not found".into(),
    };
    assert!(!reject_err.is_retriable());
    assert!(reject_err.is_method_not_found());
}

#[test]
fn dispatch_outcome_classification() {
    use healthspring_barracuda::ipc::error::DispatchOutcome;

    let success = DispatchOutcome::Success(42);
    assert!(!success.should_retry());

    let protocol = DispatchOutcome::<()>::Protocol(IpcError::Timeout(1000));
    assert!(protocol.should_retry());

    let application = DispatchOutcome::<()>::Application(IpcError::RpcReject {
        code: -32601,
        message: "not found".into(),
    });
    assert!(!application.should_retry());
}

#[test]
fn protocol_classify_response_success() {
    let resp = serde_json::json!({"jsonrpc": "2.0", "result": {"data": 1}, "id": 1});
    let outcome = protocol::classify_response(&resp);
    assert!(matches!(outcome, protocol::DispatchOutcome::Ok(_)));
}

#[test]
fn protocol_classify_response_error() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {"code": -32600, "message": "invalid"},
        "id": 1
    });
    let outcome = protocol::classify_response(&resp);
    assert!(matches!(
        outcome,
        protocol::DispatchOutcome::ProtocolError { .. }
    ));
}

#[test]
fn science_dispatch_hill() {
    use healthspring_barracuda::ipc::dispatch::dispatch_science;

    let params = serde_json::json!({
        "e_max": 1.0,
        "ic50": 10.0,
        "hill_n": 1.0,
        "concentration": 10.0
    });
    let result = dispatch_science("science.pkpd.hill_dose_response", &params);
    assert!(result.is_some());
    let val = result.unwrap();
    let response = val["response"].as_f64().unwrap();
    assert!((response - 0.5).abs() < 1e-10, "Hill at EC50 should be 0.5");
}

#[test]
fn science_dispatch_shannon() {
    use healthspring_barracuda::ipc::dispatch::dispatch_science;

    let params = serde_json::json!({"abundances": [0.25, 0.25, 0.25, 0.25]});
    let result = dispatch_science("science.microbiome.shannon_index", &params);
    assert!(result.is_some());
    let h = result.unwrap()["shannon"].as_f64().unwrap();
    assert!(
        (h - 4.0_f64.ln()).abs() < 1e-10,
        "Shannon of uniform-4 should be ln(4)"
    );
}

#[test]
fn science_dispatch_unknown_method_returns_none() {
    use healthspring_barracuda::ipc::dispatch::dispatch_science;

    let result = dispatch_science("science.nonexistent.method", &serde_json::json!({}));
    assert!(result.is_none());
}
