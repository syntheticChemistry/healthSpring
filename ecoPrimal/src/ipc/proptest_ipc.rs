// SPDX-License-Identifier: AGPL-3.0-or-later
//! Property-based tests for IPC protocol parsing.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "proptest strategies require .unwrap() for regex compilation; \
              assertion unwraps in property tests are equivalent to assert_eq"
)]
mod tests {
    use super::super::protocol::{DispatchOutcome, classify_response};
    use super::super::rpc::{extract_rpc_result, extract_rpc_result_owned};
    use super::super::socket::extract_capability_strings;
    use proptest::prelude::*;

    // Test that extract_capability_strings never panics on arbitrary JSON.
    proptest! {
        #[test]
        fn extract_caps_never_panics(json in any::<String>()) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
                let _ = extract_capability_strings(&val);
            }
        }
    }

    /// Strategy for generating JSON values suitable for result/error payloads.
    fn json_value_strategy() -> impl Strategy<Value = serde_json::Value> {
        prop_oneof![
            Just(serde_json::Value::Null),
            any::<i64>().prop_map(serde_json::Value::from),
            any::<u64>().prop_map(|v| serde_json::json!(v)),
            any::<bool>().prop_map(serde_json::Value::from),
            prop::string::string_regex("[a-zA-Z0-9._-]*")
                .unwrap()
                .prop_map(serde_json::Value::String),
            (any::<i64>(), any::<i64>()).prop_map(|(a, b)| serde_json::json!([a, b])),
        ]
    }

    // 1. extract_rpc_result round-trip: error present -> None; result only -> Some; neither -> None.
    proptest! {
        #[test]
        fn extract_rpc_result_error_returns_none(
            code in any::<i64>(),
            msg in prop::string::string_regex("[a-zA-Z0-9 ]*").unwrap()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "error": {"code": code, "message": msg}, "id": 1});
            assert!(extract_rpc_result(&resp).is_none());
        }

        #[test]
        fn extract_rpc_result_result_only_returns_some(
            result_val in json_value_strategy()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": result_val, "id": 1});
            let got = extract_rpc_result(&resp);
            assert!(got.is_some());
            assert_eq!(got.unwrap(), &result_val);
        }

        #[test]
        fn extract_rpc_result_neither_returns_none(
            id in any::<u64>()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "id": id});
            assert!(extract_rpc_result(&resp).is_none());
        }

        #[test]
        fn extract_rpc_result_both_error_and_result_returns_none(
            result_val in json_value_strategy(),
            code in any::<i64>()
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "result": result_val,
                "error": {"code": code, "message": "error wins"},
                "id": 1
            });
            assert!(extract_rpc_result(&resp).is_none());
        }
    }

    // 2. extract_rpc_result_owned parity: same as extract_rpc_result().cloned()
    proptest! {
        #[test]
        fn extract_rpc_result_owned_parity(
            result_val in json_value_strategy()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": result_val, "id": 1});
            let ref_result = extract_rpc_result(&resp).cloned();
            let owned_result = extract_rpc_result_owned(&resp);
            assert_eq!(ref_result, owned_result);
        }

        #[test]
        fn extract_rpc_result_owned_parity_error_case(
            code in any::<i64>()
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": code, "message": "fail"},
                "id": 1
            });
            assert!(extract_rpc_result(&resp).is_none());
            assert!(extract_rpc_result_owned(&resp).is_none());
        }
    }

    // 3. classify_response consistency: Ok -> extract_rpc_result Some; error -> None
    proptest! {
        #[test]
        fn classify_response_ok_implies_extract_some(
            result_val in json_value_strategy()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": result_val, "id": 1});
            let outcome = classify_response(&resp);
            assert!(matches!(outcome, DispatchOutcome::Ok(_)));
            assert!(extract_rpc_result(&resp).is_some());
        }

        #[test]
        fn classify_response_error_implies_extract_none(
            code in any::<i64>(),
            msg in prop::string::string_regex("[a-zA-Z0-9 ]*").unwrap()
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": code, "message": msg},
                "id": 1
            });
            let outcome = classify_response(&resp);
            assert!(!matches!(outcome, DispatchOutcome::Ok(_)));
            assert!(extract_rpc_result(&resp).is_none());
        }

        #[test]
        fn classify_response_missing_both_implies_extract_none(
            id in any::<u64>()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "id": id});
            let outcome = classify_response(&resp);
            assert!(!matches!(outcome, DispatchOutcome::Ok(_)));
            assert!(extract_rpc_result(&resp).is_none());
        }
    }

    /// Strategy for capability-like strings (alphanumeric, dots).
    fn cap_string_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z0-9._]+").unwrap()
    }

    // 4. extract_capability_strings round-trip for all capability formats
    proptest! {
        #[test]
        fn extract_caps_format_a_roundtrip(
            science_caps in prop::collection::vec(cap_string_strategy(), 0..5),
            infra_caps in prop::collection::vec(cap_string_strategy(), 0..5)
        ) {
            let result = serde_json::json!({
                "science": science_caps,
                "infrastructure": infra_caps
            });
            let caps = extract_capability_strings(&result);
            for s in &science_caps {
                assert!(caps.contains(&s.as_str()));
            }
            for s in &infra_caps {
                assert!(caps.contains(&s.as_str()));
            }
        }

        #[test]
        fn extract_caps_format_b_flat_array_roundtrip(
            cap_list in prop::collection::vec(cap_string_strategy(), 0..8)
        ) {
            let result = serde_json::json!({"capabilities": cap_list});
            let caps = extract_capability_strings(&result);
            for s in &cap_list {
                assert!(caps.contains(&s.as_str()));
            }
        }

        #[test]
        fn extract_caps_format_c_nested_roundtrip(
            cap_list in prop::collection::vec(cap_string_strategy(), 0..8)
        ) {
            let result = serde_json::json!({
                "capabilities": {"capabilities": cap_list}
            });
            let caps = extract_capability_strings(&result);
            for s in &cap_list {
                assert!(caps.contains(&s.as_str()));
            }
        }

        #[test]
        fn extract_caps_format_d_raw_array_roundtrip(
            cap_list in prop::collection::vec(cap_string_strategy(), 0..8)
        ) {
            let result = serde_json::json!(cap_list);
            let caps = extract_capability_strings(&result);
            for s in &cap_list {
                assert!(caps.contains(&s.as_str()));
            }
        }

        #[test]
        fn extract_caps_format_e_result_wrapper_array_roundtrip(
            cap_list in prop::collection::vec(cap_string_strategy(), 0..8)
        ) {
            let result = serde_json::json!({"result": cap_list});
            let caps = extract_capability_strings(&result);
            for s in &cap_list {
                assert!(caps.contains(&s.as_str()));
            }
        }

        #[test]
        fn extract_caps_format_e_result_wrapper_object_roundtrip(
            cap_list in prop::collection::vec(cap_string_strategy(), 0..8)
        ) {
            let result = serde_json::json!({"result": {"capabilities": cap_list}});
            let caps = extract_capability_strings(&result);
            for s in &cap_list {
                assert!(caps.contains(&s.as_str()));
            }
        }
    }

    // 5. JSON-RPC 2.0 structure invariants: valid responses have exactly one of result or error
    proptest! {
        #[test]
        fn jsonrpc_valid_success_has_result_only(
            result_val in json_value_strategy(),
            id in any::<u64>()
        ) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": result_val, "id": id});
            let has_result = resp.get("result").is_some();
            let has_error = resp.get("error").is_some();
            assert!(has_result && !has_error);
        }

        #[test]
        fn jsonrpc_valid_error_has_error_only(
            code in any::<i64>(),
            msg in prop::string::string_regex("[a-zA-Z0-9 ]*").unwrap(),
            id in any::<u64>()
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": code, "message": msg},
                "id": id
            });
            let has_result = resp.get("result").is_some();
            let has_error = resp.get("error").is_some();
            assert!(!has_result && has_error);
        }

        #[test]
        fn jsonrpc_valid_responses_via_constructors(
            result_val in json_value_strategy(),
            code in any::<i64>()
        ) {
            use super::super::rpc;
            let id = serde_json::json!(1);
            let success_str = rpc::success(&id, &result_val);
            let success_parsed: serde_json::Value = serde_json::from_str(&success_str).unwrap();
            assert!(success_parsed.get("result").is_some());
            assert!(success_parsed.get("error").is_none());

            let error_str = rpc::error(&id, code, "test error");
            let error_parsed: serde_json::Value = serde_json::from_str(&error_str).unwrap();
            assert!(error_parsed.get("result").is_none());
            assert!(error_parsed.get("error").is_some());
        }
    }

    #[test]
    fn extract_caps_format_a_healthspring() {
        let result = serde_json::json!({
            "science": ["science.health.pkpd", "science.health.microbiome"],
            "infrastructure": ["lifecycle.health"]
        });
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"science.health.pkpd"));
        assert!(caps.contains(&"science.health.microbiome"));
        assert!(caps.contains(&"lifecycle.health"));
    }

    #[test]
    fn extract_caps_format_b_flat_array() {
        let result = serde_json::json!({
            "capabilities": ["compute.dispatch", "ai.nautilus.train"]
        });
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"compute.dispatch"));
        assert!(caps.contains(&"ai.nautilus.train"));
    }

    #[test]
    fn extract_caps_format_c_nested() {
        let result = serde_json::json!({
            "capabilities": {
                "capabilities": ["data.ncbi_fetch", "data.storage.store"]
            }
        });
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"data.ncbi_fetch"));
        assert!(caps.contains(&"data.storage.store"));
    }

    #[test]
    fn extract_caps_format_d_raw_array() {
        let result = serde_json::json!(["cap1", "cap2", "cap3"]);
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"cap1"));
        assert!(caps.contains(&"cap2"));
        assert!(caps.contains(&"cap3"));
    }

    #[test]
    fn extract_caps_format_e_result_wrapper_object() {
        let result = serde_json::json!({
            "result": {"capabilities": ["model.infer", "model.load"]}
        });
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"model.infer"));
        assert!(caps.contains(&"model.load"));
    }

    #[test]
    fn extract_caps_format_e_result_wrapper_array() {
        let result = serde_json::json!({
            "result": ["cap_a", "cap_b"]
        });
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"cap_a"));
        assert!(caps.contains(&"cap_b"));
    }

    #[test]
    fn extract_caps_extracted_always_strings() {
        let result = serde_json::json!({
            "capabilities": ["str1", "str2"]
        });
        let caps = extract_capability_strings(&result);
        for cap in &caps {
            assert!(cap.chars().all(|c| c.is_ascii() || !c.is_control()));
        }
    }

    #[test]
    fn extract_caps_empty_object_returns_empty_vec() {
        let result = serde_json::json!({});
        let caps = extract_capability_strings(&result);
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_caps_empty_array_returns_empty_vec() {
        let result = serde_json::json!([]);
        let caps = extract_capability_strings(&result);
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_caps_non_string_elements_filtered() {
        let result = serde_json::json!({
            "capabilities": ["valid", 42, null, true, "also_valid"]
        });
        let caps = extract_capability_strings(&result);
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&"valid"));
        assert!(caps.contains(&"also_valid"));
    }

    #[test]
    fn extract_caps_result_wrapper_healthspring_format() {
        let result = serde_json::json!({
            "result": {
                "science": ["science.health.pkpd"],
                "infrastructure": []
            }
        });
        let caps = extract_capability_strings(&result);
        assert!(caps.contains(&"science.health.pkpd"));
    }
}
