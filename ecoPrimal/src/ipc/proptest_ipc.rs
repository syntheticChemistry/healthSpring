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

    // ═══════════════════════════════════════════════════════════════════
    // 6. DispatchOutcome ↔ extract_rpc_result consistency
    //    (absorbed from primalSpring Phase 12.1)
    // ═══════════════════════════════════════════════════════════════════

    proptest! {
        /// extract_rpc_result and classify_response agree on success vs failure.
        #[test]
        fn dispatch_outcome_extract_consistency(val in json_value_strategy()) {
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": val, "id": 1});
            let extract_ok = extract_rpc_result(&resp).is_some();
            let classify_ok = matches!(classify_response(&resp), DispatchOutcome::Ok(_));
            prop_assert_eq!(extract_ok, classify_ok);
        }

        /// Error paths: both extract and classify agree it's not success.
        #[test]
        fn dispatch_outcome_error_consistency(
            code in prop_oneof![
                Just(-32700_i64), Just(-32601_i64), Just(-32600_i64),
                Just(-1_i64), Just(0_i64), (-50000_i64..-20000),
            ],
            msg in prop::string::string_regex("[a-zA-Z0-9 ]{0,30}").unwrap(),
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": code, "message": msg},
                "id": 1
            });
            prop_assert!(extract_rpc_result(&resp).is_none());
            prop_assert!(!matches!(classify_response(&resp), DispatchOutcome::Ok(_)));
        }

        /// Protocol errors (-32700..-32600) are classified as ProtocolError.
        #[test]
        fn protocol_error_range_classified_correctly(
            code in -32700_i64..=-32600,
            msg in prop::string::string_regex("[a-zA-Z]{1,20}").unwrap(),
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": code, "message": msg},
                "id": 1
            });
            let outcome = classify_response(&resp);
            prop_assert!(outcome.is_protocol_error());
        }

        /// Application errors (outside protocol range) are ApplicationError.
        #[test]
        fn application_error_range_classified_correctly(
            code in prop_oneof![(-50000_i64..-32701), (-32599_i64..0)],
            msg in prop::string::string_regex("[a-zA-Z]{1,20}").unwrap(),
        ) {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": code, "message": msg},
                "id": 1
            });
            let outcome = classify_response(&resp);
            prop_assert!(!outcome.is_protocol_error());
            let is_app_err = matches!(outcome, DispatchOutcome::ApplicationError { .. });
            prop_assert!(is_app_err);
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // 7. Trio witness wire type fuzzing
    //    (primalSpring audit: fuzz kind, encoding, algorithm, tier, context)
    // ═══════════════════════════════════════════════════════════════════

    fn arb_witness_kind() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("computation"),
            Just("data_fetch"),
            Just("validation"),
            Just("experiment"),
            Just("calibration"),
        ]
    }

    fn arb_encoding() -> impl Strategy<Value = &'static str> {
        prop_oneof![Just("json"), Just("cbor"), Just("msgpack"), Just("raw"),]
    }

    fn arb_algorithm() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("blake3"),
            Just("sha256"),
            Just("sha3-256"),
            Just("xxhash64"),
        ]
    }

    fn arb_tier() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("ephemeral"),
            Just("session"),
            Just("permanent"),
            Just("immutable"),
        ]
    }

    fn arb_context_value() -> impl Strategy<Value = serde_json::Value> {
        prop_oneof![
            Just(serde_json::Value::Null),
            prop::string::string_regex("[a-zA-Z0-9._-]{0,30}")
                .unwrap()
                .prop_map(serde_json::Value::String),
            any::<i64>().prop_map(serde_json::Value::from),
            any::<bool>().prop_map(serde_json::Value::from),
        ]
    }

    proptest! {
        /// Trio witness JSON round-trips without panics. Validates that
        /// arbitrary field combinations for the witness wire type (kind,
        /// encoding, algorithm, tier, context) serialize and deserialize
        /// cleanly through serde_json.
        #[test]
        fn trio_witness_wire_roundtrip(
            kind in arb_witness_kind(),
            encoding in arb_encoding(),
            algorithm in arb_algorithm(),
            tier in arb_tier(),
            session_id in prop::string::string_regex("[a-f0-9]{8}-[a-f0-9]{4}").unwrap(),
            merkle_root in prop::string::string_regex("[a-f0-9]{0,64}").unwrap(),
            context_val in arb_context_value(),
        ) {
            let witness = serde_json::json!({
                "kind": kind,
                "encoding": encoding,
                "algorithm": algorithm,
                "tier": tier,
                "session_id": session_id,
                "merkle_root": merkle_root,
                "context": context_val,
            });
            let serialized = serde_json::to_string(&witness).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
            prop_assert_eq!(parsed["kind"].as_str().unwrap(), &*kind);
            prop_assert_eq!(parsed["encoding"].as_str().unwrap(), &*encoding);
            prop_assert_eq!(parsed["algorithm"].as_str().unwrap(), &*algorithm);
            prop_assert_eq!(parsed["tier"].as_str().unwrap(), &*tier);
        }

        /// DataProvenanceChain-shaped payloads serialize consistently.
        #[test]
        fn provenance_chain_wire_roundtrip(
            status in prop_oneof![Just("complete"), Just("partial"), Just("unavailable")],
            session_id in prop::string::string_regex("[a-f0-9]{0,32}").unwrap(),
            merkle_root in prop::string::string_regex("[a-f0-9]{0,64}").unwrap(),
            commit_id in prop::string::string_regex("[a-f0-9]{0,32}").unwrap(),
            braid_id in prop::string::string_regex("[a-f0-9]{0,32}").unwrap(),
        ) {
            let chain = serde_json::json!({
                "status": status,
                "session_id": session_id,
                "merkle_root": merkle_root,
                "commit_id": commit_id,
                "braid_id": braid_id,
            });
            let serialized = serde_json::to_string(&chain).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
            prop_assert_eq!(parsed["status"].as_str().unwrap(), &*status);
            prop_assert_eq!(parsed["session_id"].as_str().unwrap(), session_id.as_str());
        }

        /// A trio witness embedded in a JSON-RPC response extracts cleanly.
        #[test]
        fn trio_witness_in_rpc_response_extracts(
            kind in arb_witness_kind(),
            tier in arb_tier(),
            algorithm in arb_algorithm(),
        ) {
            let witness = serde_json::json!({
                "witness": {
                    "kind": kind,
                    "tier": tier,
                    "algorithm": algorithm,
                    "session_id": "abc-1234",
                    "merkle_root": "deadbeef",
                },
                "status": "complete",
            });
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": witness, "id": 1});
            let extracted = extract_rpc_result(&resp);
            prop_assert!(extracted.is_some());
            let result = extracted.unwrap();
            prop_assert_eq!(result["witness"]["kind"].as_str().unwrap(), &*kind);
            prop_assert_eq!(result["witness"]["tier"].as_str().unwrap(), &*tier);
        }

        /// Arbitrary JSON as a witness context field never panics in extract.
        #[test]
        fn trio_witness_arbitrary_context_never_panics(
            context in prop::string::string_regex("[\\PC]{0,200}").unwrap(),
        ) {
            let witness = serde_json::json!({
                "kind": "computation",
                "context": context,
            });
            let resp = serde_json::json!({"jsonrpc": "2.0", "result": witness, "id": 1});
            let _ = extract_rpc_result(&resp);
            let _ = classify_response(&resp);
        }
    }
}
