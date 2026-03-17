// SPDX-License-Identifier: AGPL-3.0-or-later
//! Property-based tests for IPC protocol parsing.

#[cfg(test)]
mod tests {
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
